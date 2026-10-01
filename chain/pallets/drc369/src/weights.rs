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
//! network, and these are five of them. Until then the chain charges no fee at
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
}
