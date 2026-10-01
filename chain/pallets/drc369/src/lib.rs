//! DRC-369's semantics over `pallet-nfts` (ADR-025), in the shape ADR-047 decides.
//!
//! # What this pallet is, at M4.1
//!
//! `pallet-nfts` is the ownership ledger: it holds collections, items, owners,
//! approvals, transfers and deposits, and it keeps the index of what each account
//! holds. This pallet adds what makes an item a DRC-369 asset, and at M4.1 only
//! that much of it:
//!
//! - **the content reference** of every asset, exactly as ADR-047 decision 3
//!   defines it: a hash algorithm tag, a 32-byte root and a size, 41 bytes. The
//!   root is the BLAKE3-256 hash of the asset's manifest, which lives off chain
//!   (decision 4). The chain never computes it; it stores it and compares it;
//! - **the commit a mint pins**, by hash and never by branch (decisions 8 and 9);
//! - **revision** until a **one-way switch** makes an asset permanent
//!   (decision 10, and the owner's answer to decision 13 row 11);
//! - **one singles collection per creator**, created by the creator's first mint
//!   (the owner's answer to decision 13 row 12);
//! - **remix provenance** (M4.2, ADR-061): a mint may name the asset it was
//!   derived from. The field is immutable, and the remix depth is checked **at
//!   mint** and refused past [`Config::MaxRemixDepth`], so no settlement path
//!   ever walks the graph (decision 11).
//!
//! - **nesting** (M4.2 and M4.5, requirement R-2): an owner places one of their
//!   assets inside another of their assets, and takes it out again. See below.
//!
//! Royalties live beside this pallet, in `pallet-drc369-royalties`, which reads
//! `derived_from` to pay a remix's upstream creator. State and XP and physics
//! are later M4 items and are not started here. Neither is sponsorship (M4.4):
//! the minter pays every deposit.
//!
//! # Nesting, and why a cycle cannot be made (requirement R-2)
//!
//! `pallet-nfts` has no item-owns-item relation (inventory F-D3), so the
//! relation lives here: [`ParentOf`] says which asset an asset is inside, and
//! [`ChildCount`] how many an asset holds. Four rules, each a refusal:
//!
//! - **Only the owner of both.** `nest` needs the signer to hold the child *and*
//!   the parent. The custom chain checked the child alone until 14 September
//!   2026, so a stranger could nest under an owner's asset and block its burn.
//! - **No cycles.** `nest` walks from the parent towards the root and refuses if
//!   it meets the child: an asset inside itself, or A inside B inside A, at any
//!   length. The walk is at most [`Config::MaxNestingDepth`] steps, because
//!   nothing is ever nested deeper than that, so the check has a fixed cost.
//! - **A bounded tree.** A nest deeper than [`Config::MaxNestingDepth`] is
//!   refused, and so is one into a parent already holding
//!   [`Config::MaxChildren`]. An asset that holds others cannot itself be
//!   nested: a tree is built from the root down, which is what keeps the depth
//!   bound exact without walking a subtree to measure it.
//! - **Held in place.** While an asset is nested, or holds a nested asset, it is
//!   locked in `pallet-nfts` (ADR-025): this pallet is the runtime's `Locker`,
//!   so `pallet-nfts` itself refuses to transfer or burn it, whatever call or
//!   batch the attempt arrives in, and a sale cannot hand it over. Nothing here
//!   moves a tree, so nothing has to walk one inside a transfer's weight.
//!
//! `unnest` reverses `nest` exactly, and may take out an asset that still holds
//! others: the depth of what it holds is never stored, only walked.
//!
//! # A mint must be authorised (requirement 7)
//!
//! A mint is signed, and it mints **to the signer, into the signer's own
//! collection**. The call has no recipient and no collection parameter, so there
//! is nothing to point at somebody else's account. That is the defect the custom
//! chain had and fixed on 14 September 2026, carried as a requirement by the
//! migration inventory, and here it cannot be written at all.
//!
//! # What this pallet relies on the runtime for
//!
//! `pallet-nfts`'s own calls can create collections and mint bare items, and a
//! collection's admin can hand its roles away. None of that would carry a content
//! reference. The runtime's base call filter lets through only `pallet-nfts`'s
//! transfer and approval calls, so this pallet's `mint` is the only way an item
//! comes into existence. That rule lives in the runtime, next to the filter, and
//! is tested there.
//!
//! # Weights
//!
//! **Placeholders, not benchmarks.** See [`weights`]: every weight here is built
//! from `pallet-nfts`'s own reference weights plus this pallet's storage accesses,
//! and is debt owed to roadmap item M7.2.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use polkadot_sdk::*;
use scale_info::TypeInfo;
use sp_core::H256;

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

/// A collection's identifier. `u32`, as on Asset Hub (ADR-047, decision 13 row 5).
pub type CollectionId = u32;

/// An item's identifier within its collection. `u32` (ADR-047, decision 13 row 5).
pub type ItemId = u32;

/// The algorithm that produced a fingerprint (ADR-047, decision 2).
///
/// A tag, so that a second algorithm is a runtime upgrade rather than a breaking
/// change to a frozen format. The indices are part of the wire format and are
/// pinned explicitly. A byte that is none of them does not decode, so a call
/// carrying one is refused before it is dispatched.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum HashAlgo {
    /// BLAKE3 with a 256-bit output. **The only algorithm a mint accepts today**
    /// (decision 1).
    #[codec(index = 0)]
    Blake3_256,
    /// SHA-256. Named so the tag can carry it later; refused today.
    #[codec(index = 1)]
    Sha2_256,
    /// BLAKE2b with a 256-bit output. Named so the tag can carry it later;
    /// refused today.
    #[codec(index = 2)]
    Blake2_256,
}

/// What an asset is: the fingerprint of its manifest (ADR-047, decisions 3 and 4).
///
/// **41 bytes, constant, forever:** one byte of algorithm, 32 of root, eight of
/// size. `root` hashes the manifest, never a file and never a location, and
/// `size` is the length of the bytes that hash to `root`, so a fetcher can check
/// what it was sent before it hashes it.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub struct ContentRef {
    pub algo: HashAlgo,
    pub root: H256,
    pub size: u64,
}

/// The source commit an asset's content was taken from (ADR-047, decision 8).
///
/// A Qontrol project is a git repository on disk, so its commit ids are git
/// object ids: SHA-1 today, SHA-256 in a repository created that way. Never
/// BLAKE3, and never a branch name (decision 9). Tagged for the same reason as
/// [`HashAlgo`].
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum CommitId {
    #[codec(index = 0)]
    Sha1([u8; 20]),
    #[codec(index = 1)]
    Sha256([u8; 32]),
}

/// The per-asset record: the part of ADR-047 decision 7 that M4.1 needs.
///
/// `pallet-nfts` keeps who owns it. This keeps what it is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Asset {
    /// The reference it was minted with. Never written again.
    pub origin: ContentRef,
    /// The reference it carries now. Equal to `origin` until it is revised.
    pub current: ContentRef,
    /// The commit `current` was taken from, if it came from a repository.
    pub commit: Option<CommitId>,
    /// Whether it may still be revised. One-way: `true` may become `false`,
    /// and nothing makes it `true` again.
    pub revisable: bool,
    /// The asset this one was derived from, if its minter declared one. Never
    /// written after mint (ADR-047 decision 7, ADR-061).
    pub derived_from: Option<(CollectionId, ItemId)>,
    /// How many remixes deep it is: `0` for an original, the parent's depth
    /// plus one for a remix. Bounded at mint by [`Config::MaxRemixDepth`].
    pub remix_depth: u8,
}

/// A creator's singles collection, and the next item id in it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct SinglesCollection {
    pub collection: CollectionId,
    pub next_item: ItemId,
}

/// One asset an account holds, as the runtime API returns it.
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
pub struct OwnedAsset {
    pub collection: CollectionId,
    pub item: ItemId,
    pub asset: Asset,
    /// The item's `pallet-nfts` metadata, which a mint sets to the asset's name.
    pub name: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::tokens::nonfungibles_v2::{Create, Mutate},
    };
    use frame_system::pallet_prelude::*;
    use pallet_nfts::{
        CollectionConfig, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
    };

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Tightly coupled to `pallet-nfts`, whose identifiers ADR-047 fixes at
    /// `u32`/`u32`. `RuntimeEvent` is inherited from `frame_system::Config`.
    #[pallet::config]
    pub trait Config:
        polkadot_sdk::frame_system::Config
        + pallet_nfts::Config<CollectionId = CollectionId, ItemId = ItemId>
    {
        /// Placeholders until M7.2 benchmarks them. See [`crate::weights`].
        type WeightInfo: WeightInfo;

        /// How many remixes deep an asset may be (ADR-047 decision 13 row 7:
        /// `16`, an engineering bound and part of the wire format).
        #[pallet::constant]
        type MaxRemixDepth: Get<u8>;

        /// How deep an asset may be nested: a free-standing asset is at depth
        /// `0`, an asset inside it at `1`, and nothing is deeper than this
        /// (ADR-047 decision 13 row 7: `8`, an engineering bound and part of the
        /// wire format).
        ///
        /// **Why there is a bound at all:** refusing a cycle (R-2) means walking
        /// from the parent to its root, one storage read a step. Without a bound
        /// an owner could build a chain as long as they liked and make every
        /// later `nest` under it as expensive as they liked, at a weight that
        /// has to be charged before the walk is made. With it the walk is at
        /// most this many reads, and the weight charges exactly that many.
        #[pallet::constant]
        type MaxNestingDepth: Get<u8>;

        /// How many assets one asset may hold directly (ADR-047 decision 13 row
        /// 7: `64`, an engineering bound and part of the wire format). It bounds
        /// what anything that later enumerates a parent's children has to read.
        #[pallet::constant]
        type MaxChildren: Get<u32>;
    }

    /// Each creator's singles collection. Written by the creator's first mint,
    /// and never by anything else.
    #[pallet::storage]
    pub type Singles<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, SinglesCollection, OptionQuery>;

    /// What each asset is. Who holds it is `pallet-nfts`'s `Item` and `Account`.
    #[pallet::storage]
    pub type Assets<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CollectionId,
        Blake2_128Concat,
        ItemId,
        Asset,
        OptionQuery,
    >;

    /// How many remixes name each asset as their source. Written only by a
    /// remix's mint. `pallet-drc369-royalties` reads it: once a work has been
    /// remixed, its remix share may not rise (ADR-062).
    #[pallet::storage]
    pub type RemixCount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CollectionId,
        Blake2_128Concat,
        ItemId,
        u32,
        ValueQuery,
    >;

    /// The asset each nested asset is inside. Absent for a free-standing asset.
    /// Written by `nest`, removed by `unnest`, and by nothing else.
    #[pallet::storage]
    pub type ParentOf<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CollectionId,
        Blake2_128Concat,
        ItemId,
        (CollectionId, ItemId),
        OptionQuery,
    >;

    /// How many assets each asset holds directly. Never above
    /// [`Config::MaxChildren`]. Which assets they are is in the `Nested` and
    /// `Unnested` events, for the indexer (ADR-028); the chain keeps no list.
    #[pallet::storage]
    pub type ChildCount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CollectionId,
        Blake2_128Concat,
        ItemId,
        u32,
        ValueQuery,
    >;

    /// Every event carries what an indexer needs, because it can recover nothing
    /// an event leaves out (ADR-047 decision 12, ADR-028).
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A creator's first mint created their singles collection.
        SinglesCollectionCreated {
            creator: T::AccountId,
            collection: CollectionId,
        },
        /// An asset was minted.
        Minted {
            collection: CollectionId,
            item: ItemId,
            owner: T::AccountId,
            by: T::AccountId,
            origin: ContentRef,
            commit: Option<CommitId>,
            revisable: bool,
            /// The asset it was derived from, if it is a remix.
            derived_from: Option<(CollectionId, ItemId)>,
        },
        /// An asset now carries a new content reference.
        Revised {
            collection: CollectionId,
            item: ItemId,
            from: ContentRef,
            to: ContentRef,
            commit: Option<CommitId>,
            by: T::AccountId,
        },
        /// An asset was made permanent: it can never be revised again. `content`
        /// is the reference it is now fixed at. Named `Locked` in ADR-047
        /// decision 12.
        Locked {
            collection: CollectionId,
            item: ItemId,
            content: ContentRef,
            by: T::AccountId,
        },
        /// `child` was placed inside `parent`. `depth` is how deep `child` now
        /// is: `1` inside a free-standing asset.
        Nested {
            parent: (CollectionId, ItemId),
            child: (CollectionId, ItemId),
            by: T::AccountId,
            depth: u8,
        },
        /// `child` was taken out of `parent`, and stands on its own again.
        Unnested {
            parent: (CollectionId, ItemId),
            child: (CollectionId, ItemId),
            by: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The reference is tagged with an algorithm a mint does not accept.
        /// Only BLAKE3-256 is accepted today (ADR-047, decision 1).
        UnsupportedAlgorithm,
        /// The reference says its manifest is zero bytes long, which no manifest
        /// is.
        EmptyContent,
        /// No DRC-369 asset has that collection and item.
        UnknownAsset,
        /// Only the asset's owner may do that.
        NotOwner,
        /// The asset was made permanent and can never be revised.
        Permanent,
        /// The asset is already permanent.
        AlreadyPermanent,
        /// The revision names the content and commit the asset already carries.
        Unchanged,
        /// The creator's singles collection has used every item id.
        NoItemIdsLeft,
        /// The asset a remix names as its source is not a DRC-369 asset.
        UnknownSource,
        /// The remix would be deeper than `MaxRemixDepth` allows.
        RemixTooDeep,
        /// The asset to nest into is not a DRC-369 asset.
        UnknownParent,
        /// The parent is the asset itself, or is nested somewhere inside it
        /// (requirement R-2).
        NestingCycle,
        /// The asset would be nested deeper than `MaxNestingDepth` allows.
        NestedTooDeep,
        /// The asset is already inside another. Take it out first.
        AlreadyNested,
        /// The asset is not inside another.
        NotNested,
        /// The asset holds other assets, and only an asset holding none may be
        /// nested. Take them out first.
        HoldsAssets,
        /// The parent already holds `MaxChildren` assets.
        TooManyChildren,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mint an asset to the signer, in the signer's singles collection.
        ///
        /// The signer's first mint creates that collection, and pays its deposit.
        /// `name` becomes the item's `pallet-nfts` metadata, where wallets and
        /// explorers already look for it.
        ///
        /// `derived_from` names the asset this one remixes, if any. Naming one is
        /// open to anyone — it obliges the remix, not its source — and cannot be
        /// changed afterwards. Its depth is checked here, so nothing later has to
        /// walk the remix graph.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::mint())]
        pub fn mint(
            origin: OriginFor<T>,
            content: ContentRef,
            commit: Option<CommitId>,
            name: BoundedVec<u8, <T as pallet_nfts::Config>::StringLimit>,
            revisable: bool,
            derived_from: Option<(CollectionId, ItemId)>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::accept(&content)?;
            // Checked before the new item exists, so a remix cannot name itself.
            let remix_depth = Self::remix_depth_under(derived_from)?;

            let (collection, item) = Self::next_single(&who)?;

            let item_config = ItemConfig {
                settings: ItemSettings::all_enabled(),
            };
            // The minter pays the item deposit (`false`: not the collection owner,
            // though today they are the same account).
            <pallet_nfts::Pallet<T> as Mutate<T::AccountId, ItemConfig>>::mint_into(
                &collection,
                &item,
                &who,
                &item_config,
                false,
            )?;
            if !name.is_empty() {
                <pallet_nfts::Pallet<T> as Mutate<T::AccountId, ItemConfig>>::set_item_metadata(
                    Some(&who),
                    &collection,
                    &item,
                    &name,
                )?;
            }

            if let Some((source_collection, source_item)) = derived_from {
                RemixCount::<T>::mutate(source_collection, source_item, |count| {
                    *count = count.saturating_add(1)
                });
            }
            Assets::<T>::insert(
                collection,
                item,
                Asset {
                    origin: content,
                    current: content,
                    commit,
                    revisable,
                    derived_from,
                    remix_depth,
                },
            );

            Self::deposit_event(Event::Minted {
                collection,
                item,
                owner: who.clone(),
                by: who,
                origin: content,
                commit,
                revisable,
                derived_from,
            });
            Ok(())
        }

        /// Carry a new version: the owner points the asset at new content.
        ///
        /// Refused once the asset is permanent. `origin` is untouched.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::revise())]
        pub fn revise(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            content: ContentRef,
            commit: Option<CommitId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::accept(&content)?;

            Assets::<T>::try_mutate(collection, item, |maybe| -> DispatchResult {
                let asset = maybe.as_mut().ok_or(Error::<T>::UnknownAsset)?;
                Self::ensure_owner(&who, collection, item)?;
                ensure!(asset.revisable, Error::<T>::Permanent);
                ensure!(
                    asset.current != content || asset.commit != commit,
                    Error::<T>::Unchanged
                );

                let from = asset.current;
                asset.current = content;
                asset.commit = commit;

                Self::deposit_event(Event::Revised {
                    collection,
                    item,
                    from,
                    to: content,
                    commit,
                    by: who,
                });
                Ok(())
            })
        }

        /// Make an asset permanent. **One-way:** there is no call that undoes it.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::make_permanent())]
        pub fn make_permanent(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Assets::<T>::try_mutate(collection, item, |maybe| -> DispatchResult {
                let asset = maybe.as_mut().ok_or(Error::<T>::UnknownAsset)?;
                Self::ensure_owner(&who, collection, item)?;
                ensure!(asset.revisable, Error::<T>::AlreadyPermanent);

                asset.revisable = false;

                Self::deposit_event(Event::Locked {
                    collection,
                    item,
                    content: asset.current,
                    by: who,
                });
                Ok(())
            })
        }

        /// Place an asset inside another asset. The signer must hold both.
        ///
        /// Refused if it would make a cycle (R-2), nest deeper than
        /// [`Config::MaxNestingDepth`], or fill the parent past
        /// [`Config::MaxChildren`]; if the asset is already nested; and if it
        /// holds assets itself. From here until `unnest`, neither asset can be
        /// transferred or sold (see [`Pallet::is_held_in_place`]).
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::nest(T::MaxNestingDepth::get().into()))]
        pub fn nest(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            parent: (CollectionId, ItemId),
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let child = (collection, item);

            ensure!(
                Assets::<T>::contains_key(collection, item),
                Error::<T>::UnknownAsset
            );
            ensure!(
                Assets::<T>::contains_key(parent.0, parent.1),
                Error::<T>::UnknownParent
            );
            // Both, not the child alone: nesting under an asset changes what its
            // owner can do with it, so only its owner may (requirement 7).
            Self::ensure_owner(&who, collection, item)?;
            Self::ensure_owner(&who, parent.0, parent.1)?;
            ensure!(
                !ParentOf::<T>::contains_key(collection, item),
                Error::<T>::AlreadyNested
            );

            let depth = Self::depth_inside(parent, child)?;

            // Checked after the walk, so a cycle is reported as one. What the
            // child holds would end up deeper than `depth`, and nothing here
            // measures how much deeper, so it is refused instead.
            ensure!(
                ChildCount::<T>::get(collection, item) == 0,
                Error::<T>::HoldsAssets
            );
            let held = ChildCount::<T>::get(parent.0, parent.1);
            ensure!(held < T::MaxChildren::get(), Error::<T>::TooManyChildren);

            ParentOf::<T>::insert(collection, item, parent);
            ChildCount::<T>::insert(parent.0, parent.1, held.saturating_add(1));

            Self::deposit_event(Event::Nested {
                parent,
                child,
                by: who,
                depth,
            });
            Ok(())
        }

        /// Take a nested asset out of its parent. Only its owner, who is the
        /// parent's owner too: neither has moved since the nest.
        ///
        /// What the asset itself holds stays inside it.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::unnest())]
        pub fn unnest(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let parent = ParentOf::<T>::get(collection, item).ok_or(Error::<T>::NotNested)?;
            Self::ensure_owner(&who, collection, item)?;

            ParentOf::<T>::remove(collection, item);
            ChildCount::<T>::mutate_exists(parent.0, parent.1, |count| {
                // Dropped at zero, so an asset holding nothing leaves no entry.
                *count = count
                    .and_then(|held| held.checked_sub(1))
                    .filter(|left| *left > 0);
            });

            Self::deposit_event(Event::Unnested {
                parent,
                child: (collection, item),
                by: who,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// How deep `child` would be inside `parent`, refusing a cycle and a
        /// depth past the bound (requirement R-2).
        ///
        /// Walks from `parent` to its root. `parent` at the root gives `1`. The
        /// walk reads [`ParentOf`] at most [`Config::MaxNestingDepth`] times:
        /// each step raises the depth, and the depth is refused as soon as it
        /// passes the bound. Meeting `child` on the way means `parent` is
        /// `child` or is inside it.
        fn depth_inside(
            parent: (CollectionId, ItemId),
            child: (CollectionId, ItemId),
        ) -> Result<u8, DispatchError> {
            let bound = T::MaxNestingDepth::get();
            let mut depth: u8 = 1;
            let mut cursor = parent;
            loop {
                ensure!(cursor != child, Error::<T>::NestingCycle);
                ensure!(depth <= bound, Error::<T>::NestedTooDeep);
                match ParentOf::<T>::get(cursor.0, cursor.1) {
                    Some(above) => {
                        depth = depth.checked_add(1).ok_or(Error::<T>::NestedTooDeep)?;
                        cursor = above;
                    }
                    None => return Ok(depth),
                }
            }
        }

        /// Whether an asset is nested or holds a nested asset, and so cannot be
        /// transferred, sold or burned. Two reads, whatever the tree looks like.
        pub fn is_held_in_place(collection: CollectionId, item: ItemId) -> bool {
            ParentOf::<T>::contains_key(collection, item)
                || ChildCount::<T>::get(collection, item) > 0
        }

        /// The asset this one is inside, if it is nested.
        pub fn parent_of(collection: CollectionId, item: ItemId) -> Option<(CollectionId, ItemId)> {
            ParentOf::<T>::get(collection, item)
        }

        /// How many assets this one holds directly.
        pub fn children_of(collection: CollectionId, item: ItemId) -> u32 {
            ChildCount::<T>::get(collection, item)
        }

        /// A reference a mint or a revision will carry.
        fn accept(content: &ContentRef) -> DispatchResult {
            ensure!(
                content.algo == HashAlgo::Blake3_256,
                Error::<T>::UnsupportedAlgorithm
            );
            ensure!(content.size > 0, Error::<T>::EmptyContent);
            Ok(())
        }

        /// The depth a remix of `source` would have: `0` for no source, and
        /// the source's depth plus one otherwise, refused past the bound.
        fn remix_depth_under(source: Option<(CollectionId, ItemId)>) -> Result<u8, DispatchError> {
            let Some((collection, item)) = source else {
                return Ok(0);
            };
            let parent = Assets::<T>::get(collection, item).ok_or(Error::<T>::UnknownSource)?;
            let depth = parent
                .remix_depth
                .checked_add(1)
                .ok_or(Error::<T>::RemixTooDeep)?;
            ensure!(depth <= T::MaxRemixDepth::get(), Error::<T>::RemixTooDeep);
            Ok(depth)
        }

        fn ensure_owner(
            who: &T::AccountId,
            collection: CollectionId,
            item: ItemId,
        ) -> DispatchResult {
            ensure!(
                pallet_nfts::Pallet::<T>::owner(collection, item).as_ref() == Some(who),
                Error::<T>::NotOwner
            );
            Ok(())
        }

        /// The creator's singles collection and the next item id in it, creating
        /// the collection if this is the creator's first mint.
        fn next_single(creator: &T::AccountId) -> Result<(CollectionId, ItemId), DispatchError> {
            let mut singles = match Singles::<T>::get(creator) {
                Some(singles) => singles,
                None => {
                    // The creator owns it, pays its deposit and administers it.
                    let collection = <pallet_nfts::Pallet<T> as Create<
                        T::AccountId,
                        pallet_nfts::CollectionConfigFor<T>,
                    >>::create_collection(
                        creator, creator, &Self::singles_config()
                    )?;
                    Self::deposit_event(Event::SinglesCollectionCreated {
                        creator: creator.clone(),
                        collection,
                    });
                    SinglesCollection {
                        collection,
                        next_item: 0,
                    }
                }
            };

            let item = singles.next_item;
            singles.next_item = item.checked_add(1).ok_or(Error::<T>::NoItemIdsLeft)?;
            Singles::<T>::insert(creator, singles);
            Ok((singles.collection, item))
        }

        /// Deposits required, items transferable, no supply cap, and only the
        /// issuer may mint — which, behind the runtime's call filter, means only
        /// this pallet.
        fn singles_config() -> pallet_nfts::CollectionConfigFor<T> {
            CollectionConfig {
                settings: CollectionSettings::all_enabled(),
                max_supply: None,
                mint_settings: MintSettings::default(),
            }
        }

        /// Every DRC-369 asset `owner` holds, read from `pallet-nfts`'s owner
        /// index. An item that index lists without a DRC-369 record is not a
        /// DRC-369 asset and is left out.
        pub fn assets_of(owner: &T::AccountId) -> Vec<OwnedAsset> {
            pallet_nfts::Account::<T>::iter_key_prefix((owner.clone(),))
                .filter_map(|(collection, item)| {
                    let asset = Assets::<T>::get(collection, item)?;
                    let name = pallet_nfts::ItemMetadataOf::<T>::get(collection, item)
                        .map(|metadata| metadata.data.into_inner())
                        .unwrap_or_default();
                    Some(OwnedAsset {
                        collection,
                        item,
                        asset,
                        name,
                    })
                })
                .collect()
        }

        /// How many remixes name this asset as their source.
        pub fn remixes_of(collection: CollectionId, item: ItemId) -> u32 {
            RemixCount::<T>::get(collection, item)
        }

        /// One asset's record, if it is a DRC-369 asset.
        pub fn asset(collection: CollectionId, item: ItemId) -> Option<Asset> {
            Assets::<T>::get(collection, item)
        }
    }
}

/// `pallet-nfts` asks this before it transfers or burns an item (ADR-025: "a
/// nested child is locked in `pallet-nfts`"). Mounted as the runtime's
/// `pallet_nfts::Config::Locker`, it holds a nested asset and the asset holding
/// it in place by whatever route the move arrives: a bare call, a batch, an
/// approved account, or a sale.
impl<T: Config> frame_support::traits::Locker<CollectionId, ItemId> for Pallet<T> {
    fn is_locked(collection: CollectionId, item: ItemId) -> bool {
        Self::is_held_in_place(collection, item)
    }
}

/// The runtime API a client reads DRC-369 through, besides raw storage.
pub mod runtime_api {
    use super::*;

    sp_api::decl_runtime_apis! {
        /// DRC-369 assets, read from chain state.
        pub trait Drc369Api<AccountId> where AccountId: codec::Codec {
            /// Every DRC-369 asset `owner` holds.
            fn assets_of(owner: AccountId) -> Vec<OwnedAsset>;
            /// One asset's record, if `(collection, item)` is a DRC-369 asset.
            fn asset(collection: CollectionId, item: ItemId) -> Option<Asset>;
        }
    }
}
