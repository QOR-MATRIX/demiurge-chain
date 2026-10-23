//! Weights for `pallet-drc369`. **Placeholders, not benchmarks: debt owed to M7.2.**
//!
//! No call here has been benchmarked. Each weight is assembled from two things
//! that do exist: `pallet-nfts`'s own reference weights (its `SubstrateWeight`,
//! benchmarked by the SDK on the SDK's reference hardware, not on ours) for the
//! work this pallet asks `pallet-nfts` to do, and the database reads and writes
//! this pallet makes itself, priced by the runtime's `DbWeight`. That is an
//! estimate of the right shape, not a measurement.
//!
//! Roadmap item M7.2 requires a benchmarked weight for every call before a public
//! network, and these are ten of them. Until then the chain charges no fee at
//! all (OPEN-4), so a weight here bounds how much fits in a block and nothing
//! else.

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};
use pallet_nfts::WeightInfo as NftsWeightInfo;
use polkadot_sdk::*;

/// What each call costs.
pub trait WeightInfo {
    fn mint() -> Weight;
    fn revise() -> Weight;
    fn make_permanent() -> Weight;
    /// `depth` is the most `ParentOf` reads the cycle check can make: the
    /// pallet passes `MaxNestingDepth`, so the worst case is what is charged.
    fn nest(depth: u32) -> Weight;
    fn unnest() -> Weight;
    fn set_home_game() -> Weight;
    fn authorise_writer() -> Weight;
    fn revoke_writer() -> Weight;
    fn record_progress() -> Weight;
    fn correct_progress() -> Weight;
}

/// The placeholder the runtime uses until M7.2.
pub struct PlaceholderWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for PlaceholderWeight<T> {
    /// The worst case, a creator's first mint of a remix: `pallet-nfts` creates
    /// the singles collection, mints the item and sets its metadata. This pallet
    /// reads the source's `Assets` record for its remix depth and writes its
    /// `RemixCount`, reads and writes the creator's `Singles` entry and writes
    /// the new `Assets` record.
    fn mint() -> Weight {
        pallet_nfts::weights::SubstrateWeight::<T>::create()
            .saturating_add(pallet_nfts::weights::SubstrateWeight::<T>::mint())
            .saturating_add(pallet_nfts::weights::SubstrateWeight::<T>::set_metadata())
            .saturating_add(T::DbWeight::get().reads_writes(3, 3))
    }

    /// Reads the `Assets` record and `pallet-nfts`'s `Item` for the owner,
    /// writes the record. `pallet-nfts`'s `set_metadata`, which reads the item
    /// and writes one small record, stands in for the computation.
    fn revise() -> Weight {
        pallet_nfts::weights::SubstrateWeight::<T>::set_metadata()
            .saturating_add(T::DbWeight::get().reads_writes(2, 1))
    }

    /// The same accesses as `revise`.
    fn make_permanent() -> Weight {
        pallet_nfts::weights::SubstrateWeight::<T>::set_metadata()
            .saturating_add(T::DbWeight::get().reads_writes(2, 1))
    }

    /// Reads both `Assets` records and both `pallet-nfts` `Item`s for their
    /// owners, the child's `ParentOf` and `ChildCount` and the parent's
    /// `ChildCount`, then walks `ParentOf` towards the root, one read a step and
    /// at most `depth` of them. Writes the child's `ParentOf` and the parent's
    /// `ChildCount`. `set_metadata` stands in for the computation, as above.
    fn nest(depth: u32) -> Weight {
        pallet_nfts::weights::SubstrateWeight::<T>::set_metadata()
            .saturating_add(T::DbWeight::get().reads_writes(7, 2))
            .saturating_add(T::DbWeight::get().reads(depth.into()))
    }

    /// Reads the child's `ParentOf` and its `Item` for the owner, and the
    /// parent's `ChildCount`; writes `ParentOf` and `ChildCount`.
    fn unnest() -> Weight {
        pallet_nfts::weights::SubstrateWeight::<T>::set_metadata()
            .saturating_add(T::DbWeight::get().reads_writes(3, 2))
    }

    /// Reads `pallet-nfts`'s `Collection` for the owner, the game's `Assets`
    /// record and the collection's `HomeGame`; writes `HomeGame`.
    fn set_home_game() -> Weight {
        pallet_nfts::weights::SubstrateWeight::<T>::set_metadata()
            .saturating_add(T::DbWeight::get().reads_writes(3, 1))
    }

    /// Reads the Cartridge's `Assets` record and `Item` for its holder, the
    /// writer's `Writers` entry and the game's `WriterCount`; writes both.
    fn authorise_writer() -> Weight {
        pallet_nfts::weights::SubstrateWeight::<T>::set_metadata()
            .saturating_add(T::DbWeight::get().reads_writes(4, 2))
    }

    /// The same accesses as `authorise_writer`.
    fn revoke_writer() -> Weight {
        pallet_nfts::weights::SubstrateWeight::<T>::set_metadata()
            .saturating_add(T::DbWeight::get().reads_writes(4, 2))
    }

    /// Reads the asset's `Assets` record, the collection's `HomeGame`, the
    /// writer's `Writers` entry, the listing the runtime checks and the record;
    /// writes the record. Placing at most `MaxStats` stats is computation that
    /// `set_metadata` stands in for, as above.
    fn record_progress() -> Weight {
        pallet_nfts::weights::SubstrateWeight::<T>::set_metadata()
            .saturating_add(T::DbWeight::get().reads_writes(5, 1))
    }

    /// Reads the Cartridge's `Assets` record and `Item` for its holder, the
    /// record and the listing the runtime checks; writes the record.
    fn correct_progress() -> Weight {
        pallet_nfts::weights::SubstrateWeight::<T>::set_metadata()
            .saturating_add(T::DbWeight::get().reads_writes(4, 1))
    }
}

/// For tests.
impl WeightInfo for () {
    fn mint() -> Weight {
        Weight::zero()
    }
    fn revise() -> Weight {
        Weight::zero()
    }
    fn make_permanent() -> Weight {
        Weight::zero()
    }
    fn nest(_depth: u32) -> Weight {
        Weight::zero()
    }
    fn unnest() -> Weight {
        Weight::zero()
    }
    fn set_home_game() -> Weight {
        Weight::zero()
    }
    fn authorise_writer() -> Weight {
        Weight::zero()
    }
    fn revoke_writer() -> Weight {
        Weight::zero()
    }
    fn record_progress() -> Weight {
        Weight::zero()
    }
    fn correct_progress() -> Weight {
        Weight::zero()
    }
}
