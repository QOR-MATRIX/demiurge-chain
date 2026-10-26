//! Weights for `pallet-drc369-royalties`. **Placeholders, not benchmarks: debt
//! owed to M7.2.**
//!
//! Built the way `pallet-drc369`'s are: from `pallet-nfts`'s own reference
//! weights for the work this pallet asks it to do, plus the database reads and
//! writes this pallet makes, priced by the runtime's `DbWeight`. An estimate of
//! the right shape, not a measurement. The chain charges no fee (OPEN-4), so a
//! weight bounds how much fits in a block and nothing else.

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};
use pallet_nfts::WeightInfo as NftsWeightInfo;
use polkadot_sdk::*;

/// What each call costs.
pub trait WeightInfo {
    fn set_terms() -> Weight;
    fn list() -> Weight;
    fn unlist() -> Weight;
    fn buy() -> Weight;
    fn buy_exact() -> Weight;
    fn offer_rental() -> Weight;
    fn withdraw_rental_offer() -> Weight;
    fn rent() -> Weight;
    fn end_rental() -> Weight;
}

/// The placeholder the runtime uses until M7.2.
pub struct PlaceholderWeight<T>(PhantomData<T>);

impl<T: crate::Config> WeightInfo for PlaceholderWeight<T> {
    /// Reads the asset's record, its holder, its collection's owner, its remix
    /// count and its current terms; writes the terms.
    fn set_terms() -> Weight {
        T::DbWeight::get().reads_writes(5, 1)
    }

    /// Reads the asset's record and its holder; writes the listing.
    fn list() -> Weight {
        T::DbWeight::get().reads_writes(2, 1)
    }

    /// Reads the listing and the holder; removes the listing.
    fn unlist() -> Weight {
        T::DbWeight::get().reads_writes(2, 1)
    }

    /// The worst case: a remix whose source and itself both name the most
    /// recipients. Reads the listing, the record, the holder and both sets of
    /// terms; pays every recipient of both and the seller, each payment reading
    /// and writing the payee's account and the buyer's; hands the asset over
    /// through `pallet-nfts`; removes the listing.
    fn buy() -> Weight {
        let payees = u64::from(T::MaxRoyaltyRecipients::get())
            .saturating_mul(2)
            .saturating_add(1);
        pallet_nfts::weights::SubstrateWeight::<T>::transfer()
            .saturating_add(T::DbWeight::get().reads_writes(5, 1))
            .saturating_add(T::DbWeight::get().reads_writes(2, 2).saturating_mul(payees))
    }

    /// `buy`, and one comparison against the asset's record, which `buy`
    /// already reads. A placeholder as `buy`'s is, owed to M7.2 with it.
    fn buy_exact() -> Weight {
        Self::buy()
    }

    /// Reads the asset's record and its holder; writes the offer (ADR-119).
    fn offer_rental() -> Weight {
        T::DbWeight::get().reads_writes(2, 1)
    }

    /// Reads the offer and the holder; removes the offer.
    fn withdraw_rental_offer() -> Weight {
        T::DbWeight::get().reads_writes(2, 1)
    }

    /// The worst case, as `buy`'s: a remix whose source and itself both name the
    /// most recipients. Reads the offer, the record, the holder, the listing, the
    /// rental and both sets of terms; pays every recipient of both and the
    /// holder, each payment reading and writing the payee's account and the
    /// renter's; clears an ended rental and its index entry, and writes the new
    /// rental and its index entry. Nothing is handed over.
    fn rent() -> Weight {
        let payees = u64::from(T::MaxRoyaltyRecipients::get())
            .saturating_mul(2)
            .saturating_add(1);
        T::DbWeight::get()
            .reads_writes(7, 4)
            .saturating_add(T::DbWeight::get().reads_writes(2, 2).saturating_mul(payees))
    }

    /// Reads the rental; removes it and its index entry.
    fn end_rental() -> Weight {
        T::DbWeight::get().reads_writes(1, 2)
    }
}

/// For tests.
impl WeightInfo for () {
    fn set_terms() -> Weight {
        Weight::zero()
    }
    fn list() -> Weight {
        Weight::zero()
    }
    fn unlist() -> Weight {
        Weight::zero()
    }
    fn buy() -> Weight {
        Weight::zero()
    }
    fn buy_exact() -> Weight {
        Weight::zero()
    }
    fn offer_rental() -> Weight {
        Weight::zero()
    }
    fn withdraw_rental_offer() -> Weight {
        Weight::zero()
    }
    fn rent() -> Weight {
        Weight::zero()
    }
    fn end_rental() -> Weight {
        Weight::zero()
    }
}
