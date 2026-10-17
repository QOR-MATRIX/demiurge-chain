//! Weights for `pallet-mesh`. **Placeholders, not benchmarks: debt owed to M7.2.**
//!
//! The database reads and writes each call makes, priced by the runtime's `DbWeight`, plus one balance transfer where a
//! call moves CGT, and for a challenge the hashing of one 16 KiB group and its proof. An estimate of the right shape, not
//! a measurement. The chain charges no fee (OPEN-4), so a weight bounds how much fits in a block and nothing else.

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};
use polkadot_sdk::*;

/// What each call costs.
pub trait WeightInfo {
    fn bind_node() -> Weight;
    fn open_escrow() -> Weight;
    fn top_up() -> Weight;
    fn close_escrow() -> Weight;
    fn take_assignment() -> Weight;
    fn leave_assignment() -> Weight;
    fn end_lapsed() -> Weight;
    fn answer_challenge(depth: u32) -> Weight;
    fn submit_receipts(receipts: u32) -> Weight;
    fn set_takedown() -> Weight;
    fn forget_served() -> Weight;
}

/// The placeholder the runtime uses until M7.2.
pub struct PlaceholderWeight<T>(PhantomData<T>);

/// Hashing one 16 KiB group (sixteen BLAKE3 chunks) and merging one level of its proof: a generous estimate.
const GROUP_HASH: u64 = 40_000_000;
const MERGE: u64 = 1_000_000;
/// Checking one Sr25519 signature: a generous estimate.
const SIGNATURE: u64 = 60_000_000;

impl<T: crate::Config> WeightInfo for PlaceholderWeight<T> {
    fn bind_node() -> Weight {
        T::DbWeight::get().writes(1)
    }
    /// The takedown list, the next id, both accounts; writes the id, the escrow and both accounts.
    fn open_escrow() -> Weight {
        T::DbWeight::get().reads_writes(4, 4)
    }
    fn top_up() -> Weight {
        T::DbWeight::get().reads_writes(3, 2)
    }
    /// The escrow, both accounts, up to the most copies' assignments; writes them all.
    fn close_escrow() -> Weight {
        T::DbWeight::get().reads_writes(3, 3 + 16)
    }
    fn take_assignment() -> Weight {
        T::DbWeight::get().reads_writes(4, 2)
    }
    fn leave_assignment() -> Weight {
        T::DbWeight::get().reads_writes(2, 2)
    }
    fn end_lapsed() -> Weight {
        T::DbWeight::get().reads_writes(2, 2)
    }
    /// The escrow, the takedown list, the assignment, the seed's hash, both accounts; writes the assignment and both
    /// accounts; and the hashing.
    fn answer_challenge(depth: u32) -> Weight {
        T::DbWeight::get()
            .reads_writes(6, 3)
            .saturating_add(Weight::from_parts(GROUP_HASH + MERGE * u64::from(depth), 0))
    }
    /// For each receipt: the genesis hash, the escrow, the takedown list, the assignment, three served records, both
    /// accounts; writes three records and both accounts; and a signature.
    fn submit_receipts(receipts: u32) -> Weight {
        T::DbWeight::get()
            .reads_writes(9, 5)
            .saturating_add(Weight::from_parts(SIGNATURE, 0))
            .saturating_mul(u64::from(receipts.max(1)))
    }
    fn set_takedown() -> Weight {
        T::DbWeight::get().writes(1)
    }
    /// Up to the most receipts' records in each of two maps, and one more.
    fn forget_served() -> Weight {
        T::DbWeight::get().writes(2 * 64 + 1)
    }
}

impl WeightInfo for () {
    fn bind_node() -> Weight {
        Weight::zero()
    }
    fn open_escrow() -> Weight {
        Weight::zero()
    }
    fn top_up() -> Weight {
        Weight::zero()
    }
    fn close_escrow() -> Weight {
        Weight::zero()
    }
    fn take_assignment() -> Weight {
        Weight::zero()
    }
    fn leave_assignment() -> Weight {
        Weight::zero()
    }
    fn end_lapsed() -> Weight {
        Weight::zero()
    }
    fn answer_challenge(_: u32) -> Weight {
        Weight::zero()
    }
    fn submit_receipts(_: u32) -> Weight {
        Weight::zero()
    }
    fn set_takedown() -> Weight {
        Weight::zero()
    }
    fn forget_served() -> Weight {
        Weight::zero()
    }
}
