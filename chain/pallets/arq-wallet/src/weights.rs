//! Weights for `pallet-arq-wallet`. **Placeholders, not benchmarks: debt owed to
//! M7.2.**
//!
//! The database reads and writes each call makes, priced by the runtime's
//! `DbWeight`, plus one balance transfer where a call moves CGT. An estimate of
//! the right shape, not a measurement. The chain charges no fee (OPEN-4), so a
//! weight bounds how much fits in a block and nothing else.

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};
use polkadot_sdk::*;

/// What each call costs.
pub trait WeightInfo {
    fn create() -> Weight;
    fn set_policy() -> Weight;
    fn apply_policy() -> Weight;
    fn cancel_pending() -> Weight;
    fn set_authority() -> Weight;
    fn set_paused() -> Weight;
    fn payout() -> Weight;
    fn claim() -> Weight;
    fn expire_accrual() -> Weight;
    fn schedule_withdrawal() -> Weight;
    fn execute_withdrawal() -> Weight;
    fn prune_outcome() -> Weight;
}

/// The placeholder the runtime uses until M7.2.
pub struct PlaceholderWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for PlaceholderWeight<T> {
    /// The asset record, its holder, the wallet; both accounts of a transfer; writes the wallet.
    fn create() -> Weight {
        T::DbWeight::get().reads_writes(5, 3)
    }
    /// The asset record, its holder, the wallet; writes the wallet or the pending change.
    fn set_policy() -> Weight {
        T::DbWeight::get().reads_writes(3, 2)
    }
    fn apply_policy() -> Weight {
        T::DbWeight::get().reads_writes(2, 2)
    }
    fn cancel_pending() -> Weight {
        T::DbWeight::get().reads_writes(4, 2)
    }
    fn set_authority() -> Weight {
        T::DbWeight::get().reads_writes(3, 1)
    }
    fn set_paused() -> Weight {
        T::DbWeight::get().reads_writes(3, 1)
    }
    /// The wallet, the outcome, the recipient's epoch, the accrual, both
    /// accounts; writes the wallet, the outcome, the recipient's epoch, and
    /// both accounts or the accrual.
    fn payout() -> Weight {
        T::DbWeight::get().reads_writes(7, 5)
    }
    fn claim() -> Weight {
        T::DbWeight::get().reads_writes(4, 4)
    }
    fn expire_accrual() -> Weight {
        T::DbWeight::get().reads_writes(2, 2)
    }
    fn schedule_withdrawal() -> Weight {
        T::DbWeight::get().reads_writes(5, 1)
    }
    fn execute_withdrawal() -> Weight {
        T::DbWeight::get().reads_writes(5, 3)
    }
    fn prune_outcome() -> Weight {
        T::DbWeight::get().reads_writes(1, 1)
    }
}

/// For tests.
impl WeightInfo for () {
    fn create() -> Weight {
        Weight::zero()
    }
    fn set_policy() -> Weight {
        Weight::zero()
    }
    fn apply_policy() -> Weight {
        Weight::zero()
    }
    fn cancel_pending() -> Weight {
        Weight::zero()
    }
    fn set_authority() -> Weight {
        Weight::zero()
    }
    fn set_paused() -> Weight {
        Weight::zero()
    }
    fn payout() -> Weight {
        Weight::zero()
    }
    fn claim() -> Weight {
        Weight::zero()
    }
    fn expire_accrual() -> Weight {
        Weight::zero()
    }
    fn schedule_withdrawal() -> Weight {
        Weight::zero()
    }
    fn execute_withdrawal() -> Weight {
        Weight::zero()
    }
    fn prune_outcome() -> Weight {
        Weight::zero()
    }
}
