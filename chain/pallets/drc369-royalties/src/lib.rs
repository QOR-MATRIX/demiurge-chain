//! DRC-369 royalties, and remix royalties, settled in CGT (M4.2, ADR-061).
//!
//! # What a royalty can bind to
//!
//! A plain transfer names no price, so nothing can be taken from it — on this
//! chain or any other (ADR-047's constraint table, inventory F-D5). A royalty
//! binds only to a sale the chain itself settles. So this pallet holds both
//! halves, and nothing else:
//!
//! - **the terms** an asset's creator sets, and may change while they still
//!   hold it (ADR-062): up to
//!   [`Config::MaxRoyaltyRecipients`] recipients, each with a share of every sale
//!   (a `Permill`), and a **remix share** — what a sale of any asset derived from
//!   this one owes this one's recipients;
//! - **the settled sale**: an owner lists an asset at a price in CGT, and a buyer
//!   pays it. One transaction pays the remix share upstream, then the royalties,
//!   then the seller, and hands the asset over. All of it happens, or none of it.
//!
//! # The arithmetic, in one place
//!
//! [`split`] is the whole of it, a pure function, and every CGT amount this
//! pallet moves comes out of it. For a price `p`:
//!
//! 1. **Upstream.** If the asset was derived from a source that has terms, the
//!    pool is the source's remix share of `p`, rounded down. It is divided
//!    between the source's recipients in proportion to their shares, each part
//!    rounded down.
//! 2. **Royalties.** Each of the asset's own recipients receives their share of
//!    what the upstream payments left, rounded down. Shares apply to the
//!    remainder, not to `p`, so the two never add up to more than the price,
//!    whatever order the terms were set in.
//! 3. **The seller** receives everything else, rounding included.
//!
//! The parts always sum to exactly `p`. Every fraction uses the SDK's own helpers
//! — `Permill::mul_floor` and `multiply_by_rational_with_rounding` — never a
//! hand-written `a * b / c` (AGENTS.md §5, ADR-035), and the test
//! `the_largest_intermediate_cannot_overflow` pins that `p = u128::MAX` is safe.
//!
//! # What a buyer agrees to
//!
//! A buyer agrees to a price and to a work. `max_price` holds the price: a
//! listing repriced after they looked cannot take more. Nothing held the work:
//! `pallet-drc369`'s `revise` may change a listed asset's content reference, so
//! a revision landing between the buyer's confirmation and the block sold them
//! something they had not looked at. [`Pallet::buy_exact`] closes that. It is
//! `buy` with one more argument, the content reference the buyer expects, and it
//! refuses the sale whole if the asset carries any other. `buy` keeps its
//! encoding and its behaviour, so a client that still sends it is not broken; it
//! is also not protected.
//!
//! # Asking before paying
//!
//! [`Pallet::sale_preview`], served as the runtime API
//! [`runtime_api::Drc369RoyaltiesApi`], answers what a sale of an asset at a
//! price would pay, part by part, and whether it could settle. The parts come
//! from [`split`] through the same function `buy` uses, so a client has no
//! reason to carry a copy of the arithmetic. It moves nothing.
//!
//! # One level of remix, by design
//!
//! A sale pays its direct source only, never the source's source. Paying the
//! whole ancestry would mean walking the remix graph inside a sale's weight,
//! which ADR-047 decision 11 forbids. A creator who wants their work's remixes'
//! remixes to pay them is paid by the remix in between, whose own remix share
//! they cannot set — see ADR-061 for why that is accepted.
//!
//! # What this pallet does not do
//!
//! It takes no platform share (U-15 is open, and no treasury exists), charges no
//! fee (OPEN-4) and creates no CGT. A listing and a set of terms are bounded,
//! one of each per asset, and carry no deposit of their own: an asset's own
//! deposits already bound how many can exist (ADR-061, U-14).
//!
//! # Weights
//!
//! **Placeholders, not benchmarks** — debt owed to M7.2. See [`weights`].

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use polkadot_sdk::*;
use scale_info::TypeInfo;
use sp_arithmetic::{
    helpers_128bit::multiply_by_rational_with_rounding, per_things::Rounding, PerThing, Permill,
};

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

pub use pallet_drc369::{CollectionId, ContentRef, ItemId};

/// An amount of CGT, in Sparks (AGENTS.md §5): the runtime's balance type.
pub type Balance = u128;

/// How one sale's price is divided. Every amount this pallet moves comes from
/// here, and the three parts always sum to the price.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Split<AccountId> {
    /// What the source's recipients receive, if the asset is a remix.
    pub remix: Vec<(AccountId, Balance)>,
    /// What the asset's own recipients receive.
    pub royalties: Vec<(AccountId, Balance)>,
    /// What the seller receives: everything else.
    pub seller: Balance,
}

/// Divide `price` between a remix's source, the asset's own recipients and the
/// seller. See the module documentation for the rule.
///
/// `upstream` is the source's remix share and the source's recipients. Shares of
/// zero never occur in stored terms (`set_terms` refuses them); if every share
/// in `upstream` were zero, nothing would be paid upstream.
pub fn split<AccountId: Clone>(
    price: Balance,
    upstream: Option<(Permill, &[(AccountId, Permill)])>,
    own: &[(AccountId, Permill)],
) -> Split<AccountId> {
    let mut remix = Vec::new();
    let mut paid_upstream: Balance = 0;
    if let Some((share, recipients)) = upstream {
        let pool = share.mul_floor(price);
        let weights: u128 = recipients
            .iter()
            .map(|(_, part)| u128::from(part.deconstruct()))
            .sum();
        if weights > 0 {
            for (who, part) in recipients {
                // `part <= weights`, so the result is at most `pool` and the
                // helper cannot report an overflow.
                let amount = multiply_by_rational_with_rounding(
                    pool,
                    u128::from(part.deconstruct()),
                    weights,
                    Rounding::Down,
                )
                .unwrap_or(0);
                paid_upstream = paid_upstream.saturating_add(amount);
                remix.push((who.clone(), amount));
            }
        }
    }

    // `paid_upstream <= pool <= price`.
    let rest = price.saturating_sub(paid_upstream);
    let mut paid_own: Balance = 0;
    let royalties = own
        .iter()
        .map(|(who, share)| {
            let amount = share.mul_floor(rest);
            paid_own = paid_own.saturating_add(amount);
            (who.clone(), amount)
        })
        .collect();

    // Stored shares sum to at most one whole, so `paid_own <= rest`.
    Split {
        remix,
        royalties,
        seller: rest.saturating_sub(paid_own),
    }
}

/// What a sale of one asset at one price would pay, and whether it could
/// settle: the answer [`Pallet::sale_preview`] gives. Nothing is moved to
/// produce it.
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
pub struct SalePreview<AccountId> {
    /// The asset this one was derived from, if it is a remix.
    pub source: Option<(CollectionId, ItemId)>,
    /// What each of the source's recipients would receive, in the order paid.
    pub remix: Vec<(AccountId, Balance)>,
    /// What each of the asset's own recipients would receive, in the order paid.
    pub royalties: Vec<(AccountId, Balance)>,
    /// Who holds the asset, and so who would sell it.
    pub seller: AccountId,
    /// What the seller would receive: everything else, rounding included.
    /// `remix`, `royalties` and this sum to the price.
    pub seller_receives: Balance,
    /// Why the sale could not settle, as the error the transaction itself
    /// would report; `None` if nothing found stops it. See
    /// [`Pallet::sale_preview`] for what is checked with and without a buyer.
    pub refusal: Option<sp_runtime::DispatchError>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        storage::{with_transaction, TransactionOutcome},
        traits::{
            fungible::{Inspect, Mutate},
            tokens::{nonfungibles_v2::Transfer, DepositConsequence, Preservation, Provenance},
            Locker,
        },
        CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound,
    };
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Tightly coupled to `pallet-drc369`, whose records say what an asset is and
    /// what it was derived from, and through it to `pallet-nfts`, which says who
    /// holds it. `RuntimeEvent` is inherited from `frame_system::Config`.
    #[pallet::config]
    pub trait Config: polkadot_sdk::frame_system::Config + pallet_drc369::Config {
        /// CGT. A sale is paid in it and nothing else.
        type Currency: Mutate<Self::AccountId, Balance = Balance>;

        /// How many recipients one asset's terms may name. `8` (ADR-047 decision
        /// 13 row 7, confirmed by the owner in ADR-057). Part of the wire format.
        #[pallet::constant]
        type MaxRoyaltyRecipients: Get<u32>;

        /// Placeholders until M7.2 benchmarks them. See [`crate::weights`].
        type WeightInfo: WeightInfo;
    }

    /// An asset's royalty terms: who is paid from each sale, and what a sale of
    /// a remix of it owes them.
    #[derive(
        CloneNoBound,
        PartialEqNoBound,
        EqNoBound,
        DebugNoBound,
        Encode,
        Decode,
        DecodeWithMemTracking,
        MaxEncodedLen,
        TypeInfo,
    )]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound())]
    pub struct Terms<T: Config> {
        /// Each recipient and their share of every sale of this asset. No
        /// account appears twice, no share is zero, and the shares sum to at
        /// most one whole.
        pub recipients: BoundedVec<(T::AccountId, Permill), T::MaxRoyaltyRecipients>,
        /// The share of every sale of a remix of this asset that is paid to
        /// `recipients`, divided in proportion to their shares. Zero when there
        /// are no recipients.
        pub remix: Permill,
    }

    /// An asset offered for sale at a fixed price in CGT.
    #[derive(
        Clone, PartialEq, Eq, Debug, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo,
    )]
    pub struct Listing<AccountId> {
        /// Who listed it. A sale goes ahead only while they still hold it.
        pub seller: AccountId,
        /// The price, in Sparks.
        pub price: Balance,
    }

    /// Each asset's royalty terms. Written by the asset's creator, and changed
    /// only while they still hold it, so no buyer ever holds an asset whose
    /// terms change under them (ADR-062).
    #[pallet::storage]
    pub type RoyaltyTerms<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CollectionId,
        Blake2_128Concat,
        ItemId,
        Terms<T>,
        OptionQuery,
    >;

    /// Assets offered for sale. At most one listing per asset.
    #[pallet::storage]
    pub type Listings<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CollectionId,
        Blake2_128Concat,
        ItemId,
        Listing<T::AccountId>,
        OptionQuery,
    >;

    /// Every event carries what an indexer needs, because it can recover nothing
    /// an event leaves out (ADR-047 decision 12, ADR-028).
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An asset's creator set its royalty terms, or changed them.
        TermsSet {
            collection: CollectionId,
            item: ItemId,
            recipients: Vec<(T::AccountId, Permill)>,
            remix: Permill,
            by: T::AccountId,
        },
        /// An asset was offered for sale, or its price changed.
        Listed {
            collection: CollectionId,
            item: ItemId,
            seller: T::AccountId,
            price: Balance,
        },
        /// A listing was withdrawn.
        Unlisted {
            collection: CollectionId,
            item: ItemId,
            by: T::AccountId,
        },
        /// An asset was sold and settled on chain. `remix` is what the source's
        /// recipients received and `royalties` what the asset's own recipients
        /// received; `seller_received` is the rest. Together they are `price`.
        Sold {
            collection: CollectionId,
            item: ItemId,
            from: T::AccountId,
            to: T::AccountId,
            price: Balance,
            source: Option<(CollectionId, ItemId)>,
            remix: Vec<(T::AccountId, Balance)>,
            royalties: Vec<(T::AccountId, Balance)>,
            seller_received: Balance,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No DRC-369 asset has that collection and item.
        UnknownAsset,
        /// Only the asset's holder may do that.
        NotOwner,
        /// Only the asset's creator, while they still hold it, may set its terms.
        NotCreator,
        /// The asset has been remixed, so its remix share may be lowered but not
        /// raised: the remixes were made on the share it had (ADR-062).
        RemixShareLocked,
        /// The shares add up to more than the whole price.
        SharesExceedWhole,
        /// A share of zero names a recipient who would receive nothing.
        ZeroShare,
        /// The same account is named twice.
        DuplicateRecipient,
        /// A remix share needs at least one recipient to pay it to.
        RemixShareWithoutRecipients,
        /// A price of nothing is a gift, which is a transfer, not a sale.
        ZeroPrice,
        /// The asset is not listed.
        NotListed,
        /// Whoever listed the asset no longer holds it; the listing is void.
        ListingStale,
        /// The seller cannot buy their own listing.
        OwnListing,
        /// The price is above what the buyer agreed to pay.
        PriceAboveLimit,
        /// A recipient cannot receive their part — most often because it is below
        /// the existential deposit and their account does not exist yet. The
        /// whole sale is refused rather than paying anyone else their part.
        PaymentCannotBeReceived,
        /// The asset no longer carries the content the buyer agreed to buy: it
        /// was revised after they looked. Nothing moves.
        ContentChanged,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set an asset's royalty terms, or change them.
        ///
        /// Only the asset's creator — the owner of the collection it was minted
        /// into — and only while they hold it, so a creator can correct a
        /// mistake but never change the terms of an asset someone else holds.
        /// `remix` is the share of every sale of a remix of this asset owed to
        /// `recipients`; once the asset has been remixed it may be lowered, never
        /// raised (ADR-062).
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::set_terms())]
        pub fn set_terms(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            recipients: BoundedVec<(T::AccountId, Permill), T::MaxRoyaltyRecipients>,
            remix: Permill,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_asset(collection, item)?;
            ensure!(
                pallet_nfts::Pallet::<T>::collection_owner(collection).as_ref() == Some(&who)
                    && Self::holder(collection, item).as_ref() == Some(&who),
                Error::<T>::NotCreator
            );
            if pallet_drc369::Pallet::<T>::remixes_of(collection, item) > 0 {
                let current = RoyaltyTerms::<T>::get(collection, item)
                    .map(|terms| terms.remix)
                    .unwrap_or_else(Permill::zero);
                ensure!(remix <= current, Error::<T>::RemixShareLocked);
            }

            let mut total: u32 = 0;
            for (index, (account, share)) in recipients.iter().enumerate() {
                ensure!(!share.is_zero(), Error::<T>::ZeroShare);
                ensure!(
                    !recipients[..index]
                        .iter()
                        .any(|(other, _)| other == account),
                    Error::<T>::DuplicateRecipient
                );
                // At most eight parts of a million each: no overflow.
                total = total.saturating_add(share.deconstruct());
            }
            ensure!(total <= Permill::ACCURACY, Error::<T>::SharesExceedWhole);
            ensure!(
                remix.is_zero() || !recipients.is_empty(),
                Error::<T>::RemixShareWithoutRecipients
            );

            RoyaltyTerms::<T>::insert(
                collection,
                item,
                Terms::<T> {
                    recipients: recipients.clone(),
                    remix,
                },
            );
            Self::deposit_event(Event::TermsSet {
                collection,
                item,
                recipients: recipients.into_inner(),
                remix,
                by: who,
            });
            Ok(())
        }

        /// Offer an asset for sale at `price`, or change the price it is offered
        /// at. Only its holder.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::list())]
        pub fn list(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            price: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_asset(collection, item)?;
            ensure!(
                Self::holder(collection, item).as_ref() == Some(&who),
                Error::<T>::NotOwner
            );
            ensure!(price > 0, Error::<T>::ZeroPrice);

            Listings::<T>::insert(
                collection,
                item,
                Listing {
                    seller: who.clone(),
                    price,
                },
            );
            Self::deposit_event(Event::Listed {
                collection,
                item,
                seller: who,
                price,
            });
            Ok(())
        }

        /// Withdraw a listing. Its seller or the asset's holder may; so may
        /// anyone once the listing is void because the seller no longer holds the
        /// asset, which clears state nobody can use.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::unlist())]
        pub fn unlist(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let listing = Listings::<T>::get(collection, item).ok_or(Error::<T>::NotListed)?;
            let holder = Self::holder(collection, item);
            let void = holder.as_ref() != Some(&listing.seller);
            ensure!(
                void || who == listing.seller || holder.as_ref() == Some(&who),
                Error::<T>::NotOwner
            );

            Listings::<T>::remove(collection, item);
            Self::deposit_event(Event::Unlisted {
                collection,
                item,
                by: who,
            });
            Ok(())
        }

        /// Buy a listed asset. `max_price` is the most the buyer agrees to pay,
        /// so a price raised after they looked cannot be taken from them.
        ///
        /// Pays the remix share upstream, then the royalties, then the seller,
        /// then hands the asset over, in one transaction: if any part fails,
        /// nothing moves.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::buy())]
        pub fn buy(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            max_price: Balance,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            Self::do_buy(buyer, collection, item, max_price, None)
        }

        /// Buy a listed asset, and only if it is still the work the buyer looked
        /// at. `buy` with one more condition: `content` is the content reference
        /// the buyer expects the asset to carry, and if it carries any other —
        /// its owner revised it after the buyer looked — the sale is refused
        /// whole with [`Error::ContentChanged`] and nothing moves.
        ///
        /// A separate call, not a new argument to `buy`, so that `buy` keeps its
        /// encoding and a client that sends it keeps working.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::buy_exact())]
        pub fn buy_exact(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            max_price: Balance,
            content: ContentRef,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            Self::do_buy(buyer, collection, item, max_price, Some(content))
        }
    }

    impl<T: Config> Pallet<T> {
        fn ensure_asset(
            collection: CollectionId,
            item: ItemId,
        ) -> Result<pallet_drc369::Asset, DispatchError> {
            pallet_drc369::Pallet::<T>::asset(collection, item)
                .ok_or_else(|| Error::<T>::UnknownAsset.into())
        }

        fn holder(collection: CollectionId, item: ItemId) -> Option<T::AccountId> {
            pallet_nfts::Pallet::<T>::owner(collection, item)
        }

        /// `buy` and `buy_exact`: what must hold before a sale is settled.
        /// `expected` is the content the buyer agreed to, if they named it.
        fn do_buy(
            buyer: T::AccountId,
            collection: CollectionId,
            item: ItemId,
            max_price: Balance,
            expected: Option<ContentRef>,
        ) -> DispatchResult {
            let listing = Listings::<T>::get(collection, item).ok_or(Error::<T>::NotListed)?;
            let asset = Self::ensure_asset(collection, item)?;
            ensure!(
                Self::holder(collection, item).as_ref() == Some(&listing.seller),
                Error::<T>::ListingStale
            );
            ensure!(buyer != listing.seller, Error::<T>::OwnListing);
            ensure!(listing.price <= max_price, Error::<T>::PriceAboveLimit);
            if let Some(content) = expected {
                // `current`, not `origin`: what the asset is now is what is sold.
                ensure!(asset.current == content, Error::<T>::ContentChanged);
            }

            Self::settle(
                &buyer,
                &listing.seller,
                collection,
                item,
                &asset,
                listing.price,
            )
        }

        /// How a sale of this asset at `price` is divided: [`split`], given the
        /// source's terms and the asset's own. The one place they are read for
        /// a sale, so a preview and a sale cannot disagree.
        fn parts(
            collection: CollectionId,
            item: ItemId,
            asset: &pallet_drc369::Asset,
            price: Balance,
        ) -> Split<T::AccountId> {
            let source_terms = asset
                .derived_from
                .and_then(|(c, i)| RoyaltyTerms::<T>::get(c, i));
            let own_terms = RoyaltyTerms::<T>::get(collection, item);
            split(
                price,
                source_terms
                    .as_ref()
                    .map(|terms| (terms.remix, &terms.recipients[..])),
                own_terms
                    .as_ref()
                    .map(|terms| &terms.recipients[..])
                    .unwrap_or(&[]),
            )
        }

        /// Settle an agreed sale: pay the remix share upstream, then the
        /// royalties, then the seller, hand the asset over and clear its
        /// listing. Any part failing fails all of it.
        fn settle(
            buyer: &T::AccountId,
            seller: &T::AccountId,
            collection: CollectionId,
            item: ItemId,
            asset: &pallet_drc369::Asset,
            price: Balance,
        ) -> DispatchResult {
            let split = Self::parts(collection, item, asset, price);

            for (to, amount) in split.remix.iter().chain(split.royalties.iter()) {
                Self::pay(buyer, to, *amount)?;
            }
            Self::pay(buyer, seller, split.seller)?;

            <pallet_nfts::Pallet<T> as Transfer<T::AccountId>>::transfer(
                &collection,
                &item,
                buyer,
            )?;
            Listings::<T>::remove(collection, item);

            Self::deposit_event(Event::Sold {
                collection,
                item,
                from: seller.clone(),
                to: buyer.clone(),
                price,
                source: asset.derived_from,
                remix: split.remix,
                royalties: split.royalties,
                seller_received: split.seller,
            });
            Ok(())
        }

        /// Whether `to` can be paid `amount`: the check `pay` makes.
        fn can_receive(to: &T::AccountId, amount: Balance) -> bool {
            <<T as Config>::Currency as Inspect<T::AccountId>>::can_deposit(
                to,
                amount,
                Provenance::Extant,
            ) == DepositConsequence::Success
        }

        /// Move `amount` from the buyer, who must keep their account alive. A
        /// payment to the buyer themselves, or of nothing, moves nothing.
        fn pay(buyer: &T::AccountId, to: &T::AccountId, amount: Balance) -> DispatchResult {
            if amount == 0 || to == buyer {
                return Ok(());
            }
            ensure!(
                Self::can_receive(to, amount),
                Error::<T>::PaymentCannotBeReceived
            );
            <<T as Config>::Currency as Mutate<T::AccountId>>::transfer(
                buyer,
                to,
                amount,
                Preservation::Preserve,
            )?;
            Ok(())
        }

        /// An asset's royalty terms, if its creator set them.
        pub fn terms(collection: CollectionId, item: ItemId) -> Option<Terms<T>> {
            RoyaltyTerms::<T>::get(collection, item)
        }

        /// An asset's listing, if it is offered for sale.
        pub fn listing(collection: CollectionId, item: ItemId) -> Option<Listing<T::AccountId>> {
            Listings::<T>::get(collection, item)
        }

        /// What a sale of this asset at `price` would pay, and whether it could
        /// settle. `None` if it is not a DRC-369 asset anyone holds. The asset
        /// need not be listed: a seller asks before choosing a price, a buyer
        /// before paying one. Nothing is moved.
        ///
        /// The parts are [`split`]'s, read through the function `buy` uses.
        /// `refusal` is found in one of two ways:
        ///
        /// - **With a buyer**, the sale itself is run — every payment and the
        ///   handover — and then undone, so the answer is the one the
        ///   transaction would get: a buyer who cannot pay, a part that cannot
        ///   be received, an asset held in place by nesting, or the holder
        ///   buying from themselves.
        /// - **Without one**, only what would stop every buyer is reported: a
        ///   part that cannot be received, and an asset held in place.
        ///
        /// A price of nothing is refused either way, as `list` refuses it.
        /// Whether the asset is listed, at what price, and what content a
        /// buyer expects are not part of the question and are not checked.
        pub fn sale_preview(
            collection: CollectionId,
            item: ItemId,
            price: Balance,
            buyer: Option<T::AccountId>,
        ) -> Option<SalePreview<T::AccountId>> {
            let asset = pallet_drc369::Pallet::<T>::asset(collection, item)?;
            let seller = Self::holder(collection, item)?;
            let parts = Self::parts(collection, item, &asset, price);
            let refusal = Self::would_settle(
                buyer.as_ref(),
                &seller,
                collection,
                item,
                &asset,
                price,
                &parts,
            )
            .err();
            Some(SalePreview {
                source: asset.derived_from,
                remix: parts.remix,
                royalties: parts.royalties,
                seller,
                seller_receives: parts.seller,
                refusal,
            })
        }

        /// The first reason a sale at `price` would be refused, if any. See
        /// [`Pallet::sale_preview`].
        fn would_settle(
            buyer: Option<&T::AccountId>,
            seller: &T::AccountId,
            collection: CollectionId,
            item: ItemId,
            asset: &pallet_drc369::Asset,
            price: Balance,
            parts: &Split<T::AccountId>,
        ) -> DispatchResult {
            ensure!(price > 0, Error::<T>::ZeroPrice);
            let Some(buyer) = buyer else {
                // An account a part of this sale has already reached exists by
                // the time the next part arrives, so only its first is checked.
                let mut reached: Vec<&T::AccountId> = Vec::new();
                let seller_part = (seller.clone(), parts.seller);
                for (to, amount) in parts
                    .remix
                    .iter()
                    .chain(parts.royalties.iter())
                    .chain(core::iter::once(&seller_part))
                {
                    if *amount == 0 || reached.contains(&to) {
                        continue;
                    }
                    ensure!(
                        Self::can_receive(to, *amount),
                        Error::<T>::PaymentCannotBeReceived
                    );
                    reached.push(to);
                }
                // What `pallet-nfts` asks before the transfer a sale ends with.
                ensure!(
                    !<T as pallet_nfts::Config>::Locker::is_locked(collection, item),
                    pallet_nfts::Error::<T>::ItemLocked
                );
                return Ok(());
            };

            ensure!(buyer != seller, Error::<T>::OwnListing);
            // The real settlement, undone whatever it returns.
            with_transaction(|| {
                TransactionOutcome::Rollback(Self::settle(
                    buyer, seller, collection, item, asset, price,
                ))
            })
        }
    }
}

/// The runtime API a client asks about a sale through, besides raw storage.
pub mod runtime_api {
    use super::*;

    sp_api::decl_runtime_apis! {
        /// DRC-369 sales, asked of chain state. Nothing here moves anything.
        pub trait Drc369RoyaltiesApi<AccountId> where AccountId: codec::Codec {
            /// What a sale of `(collection, item)` at `price` would pay, and
            /// whether it could settle; `None` if it is not a DRC-369 asset.
            /// With `buyer`, the sale is tried for that account and undone.
            fn sale_preview(
                collection: CollectionId,
                item: ItemId,
                price: Balance,
                buyer: Option<AccountId>,
            ) -> Option<SalePreview<AccountId>>;
        }
    }
}
