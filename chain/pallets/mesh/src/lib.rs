//! The Mesh's hosting escrow and its proofs (ADR-095, ADR-096; `docs/architecture/MESH_PROOFS.md`).
//!
//! # What it is
//!
//! Whoever puts a file on the Mesh opens an **escrow** for its BLAKE3 root: the size, how many copies they want held, a
//! price for each answered storage challenge, a price per MiB served, a cap on bytes paid for each period, and a deposit.
//! The deposit sits in an account derived from this pallet's [`Config::PalletId`] and the escrow's id, which **no key
//! can sign for**. CGT leaves it only to pay a seeder for proven work, or back to the uploader when they close it.
//! Nothing here creates CGT (ADR-095 decision 4).
//!
//! - **Seeders** bind their launcher's Mesh node to their account ([`Pallet::bind_node`]), take an assignment on an
//!   escrow while it has copies to fill, and are paid:
//! - **for storage**, once each period, by answering a challenge: the chain names one 16 KiB group of the file, chosen
//!   from a block hash after the period opens, and the seeder sends the group and its BLAKE3 proof, checked here against
//!   the root ([`mesh_proof::verify`]);
//! - **for serving**, by receipts the receiving account signs: the escrow, the period, the seeder and the bytes received
//!   from that seeder in the period so far. Receipts are cumulative, so the chain pays only what is new and a receipt
//!   can never be paid twice.
//!
//! # The rules, all checked here
//!
//! - A seeder is neither the escrow's uploader nor, for a receipt, its receiver; the receiver is not the uploader.
//! - One receiver is paid for at most the file's size per period, across seeders; an escrow pays for at most its cap
//!   per period, across receivers. That a receiver has a QOR ID is not visible to the chain (ADR-096 decision 2).
//! - A challenge is answered within [`Config::AnswerWindow`] blocks of its seed block, once per period, and only for an
//!   assignment taken in an earlier period. An assignment unanswered for more than [`Config::MaxMissed`] periods may be
//!   ended by anyone.
//! - A root on the takedown list, kept by [`Config::TakedownOrigin`], cannot be escrowed, and its escrows pay nothing
//!   more; they can still be closed.
//!
//! # Arithmetic
//!
//! Additions and subtractions of amounts are checked. The one ratio, a price per MiB times bytes, goes through
//! `multiply_by_rational_with_rounding` (ADR-035), rounding down, and is refused if the result would not fit. The test
//! `the_largest_amounts_cannot_overflow` pins it.
//!
//! # Weights
//!
//! **Placeholders, not benchmarks** — debt owed to M7.2. See [`weights`].

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use polkadot_sdk::*;

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

/// An amount of CGT, in Sparks (AGENTS.md §5): the runtime's balance type.
pub type Balance = u128;

/// An escrow's number, given in order.
pub type EscrowId = u64;

/// A BLAKE3-256 root (ADR-047).
pub type Root = [u8; 32];

/// A launcher's Mesh node: its iroh public key.
pub type NodeKey = [u8; 32];

/// One MiB, the unit a serving price is per.
pub const MIB: u128 = 1024 * 1024;

/// What every receipt's signed bytes begin with, so a signature made for anything else is never a receipt.
pub const RECEIPT_CONTEXT: &[u8] = b"demiurge/mesh-receipt/v1";

use frame_support::{pallet_prelude::*, CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound};

/// What a receiver signs: everything received from one seeder for one escrow in one period, so far.
#[derive(
    Clone, PartialEq, Eq, Debug, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo,
)]
pub struct Receipt<AccountId> {
    pub escrow: EscrowId,
    pub period: u32,
    pub seeder: AccountId,
    pub receiver: AccountId,
    pub bytes: u64,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        traits::{
            fungible::{Inspect, Mutate},
            tokens::{Fortitude, Preservation},
        },
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        traits::{AccountIdConversion, IdentifyAccount, Verify},
        Rounding, SaturatedConversion,
    };

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: polkadot_sdk::frame_system::Config {
        /// CGT. An escrow holds it and pays it, and nothing else.
        type Currency: Mutate<Self::AccountId, Balance = Balance>;

        /// What every escrow's account is derived from.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Blocks in a period: how often each assignment is challenged (ADR-096: 600). A chain's genesis may set
        /// another, which only a development chain does, so a test reaches a challenge in minutes ([`GenesisConfig`]).
        #[pallet::constant]
        type Period: Get<u32>;

        /// Blocks after a period's seed block in which its challenge may be answered (ADR-096: 100). Must be less than
        /// the period, and within the block hashes the chain keeps. Genesis may set another, as for the period.
        #[pallet::constant]
        type AnswerWindow: Get<u32>;

        /// Periods unanswered after which anyone may end an assignment (ADR-096: 3).
        #[pallet::constant]
        type MaxMissed: Get<u32>;

        /// The most copies an escrow may ask for. An engineering bound on its assignments.
        #[pallet::constant]
        type MaxCopies: Get<u8>;

        /// The most receipts one call submits. An engineering bound on a call's weight.
        #[pallet::constant]
        type MaxReceipts: Get<u32>;

        /// Who keeps the takedown list (ADR-096 decision 4: the root origin).
        type TakedownOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// A receiver's signature on a receipt, and the key it is checked against.
        type ReceiptSignature: Verify<Signer = Self::ReceiptSigner> + Parameter;
        type ReceiptSigner: IdentifyAccount<AccountId = Self::AccountId>;

        /// Placeholders until M7.2 benchmarks them. See [`crate::weights`].
        type WeightInfo: WeightInfo;
    }

    /// Receipts and their signatures, as one call submits them.
    pub type ReceiptsOf<T> = BoundedVec<
        (
            Receipt<<T as frame_system::Config>::AccountId>,
            <T as Config>::ReceiptSignature,
        ),
        <T as Config>::MaxReceipts,
    >;

    /// One escrow.
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
    pub struct Escrow<T: Config> {
        pub uploader: T::AccountId,
        pub root: Root,
        /// The file's size in bytes, which places a challenged group in its tree (`mesh_proof::verify`).
        pub size: u64,
        /// How many seeders it pays to hold the file.
        pub copies: u8,
        /// Paid to a seeder for each answered challenge. The uploader's own price.
        pub storage_price: Balance,
        /// Paid per MiB served, as receipts prove it. The uploader's own price.
        pub serving_price_per_mib: Balance,
        /// The most bytes served the escrow pays for in one period, across every receiver.
        pub serving_cap_per_period: u64,
        /// How many assignments it has now.
        pub assigned: u8,
    }

    /// A seeder's hold on an escrow.
    #[derive(
        Clone, PartialEq, Eq, Debug, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo,
    )]
    pub struct Assignment {
        /// The period it was taken in.
        pub taken: u32,
        /// The last period it answered, if any.
        pub answered: Option<u32>,
    }

    #[pallet::type_value]
    pub fn DefaultPeriod<T: Config>() -> u32 {
        T::Period::get()
    }

    #[pallet::type_value]
    pub fn DefaultAnswerWindow<T: Config>() -> u32 {
        T::AnswerWindow::get()
    }

    /// Blocks in a period on this chain: [`Config::Period`] unless genesis set another. Nothing changes it after.
    #[pallet::storage]
    pub type PeriodLength<T> = StorageValue<_, u32, ValueQuery, DefaultPeriod<T>>;

    /// The answer window on this chain: [`Config::AnswerWindow`] unless genesis set another. Nothing changes it after.
    #[pallet::storage]
    pub type AnswerWindowLength<T> = StorageValue<_, u32, ValueQuery, DefaultAnswerWindow<T>>;

    /// A development chain's shorter period and window. Every other chain leaves both `None`, and a chain that gains
    /// this pallet by a runtime upgrade has no genesis for it, so both keep ADR-096's values.
    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub period: Option<u32>,
        pub answer_window: Option<u32>,
        #[serde(skip)]
        pub _config: core::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let period = self.period.unwrap_or_else(T::Period::get);
            let window = self.answer_window.unwrap_or_else(T::AnswerWindow::get);
            assert!(
                timing_holds(period, window),
                "a Mesh period of {period} blocks cannot hold an answer window of {window}"
            );
            if let Some(period) = self.period {
                PeriodLength::<T>::put(period);
            }
            if let Some(window) = self.answer_window {
                AnswerWindowLength::<T>::put(window);
            }
        }
    }

    #[pallet::storage]
    pub type NextEscrowId<T> = StorageValue<_, EscrowId, ValueQuery>;

    #[pallet::storage]
    pub type Escrows<T: Config> = StorageMap<_, Blake2_128Concat, EscrowId, Escrow<T>>;

    #[pallet::storage]
    pub type Assignments<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, EscrowId, Blake2_128Concat, T::AccountId, Assignment>;

    /// Each account's Mesh node, so a launcher can find who holds a root (ADR-095 decision 1).
    #[pallet::storage]
    pub type Nodes<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, NodeKey>;

    /// Roots seeders refuse, and escrows cannot hold.
    #[pallet::storage]
    pub type Takedowns<T> = StorageMap<_, Blake2_128Concat, Root, ()>;

    /// Bytes paid for, per escrow, period, receiver and seeder: what a receipt has already been paid up to.
    #[pallet::storage]
    pub type Served<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, EscrowId>,
            NMapKey<Twox64Concat, u32>,
            NMapKey<Blake2_128Concat, T::AccountId>,
            NMapKey<Blake2_128Concat, T::AccountId>,
        ),
        u64,
        ValueQuery,
    >;

    /// Bytes paid for, per escrow, period and receiver, across seeders.
    #[pallet::storage]
    pub type ReceiverServed<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, EscrowId>,
            NMapKey<Twox64Concat, u32>,
            NMapKey<Blake2_128Concat, T::AccountId>,
        ),
        u64,
        ValueQuery,
    >;

    /// Bytes paid for, per escrow and period, across receivers.
    #[pallet::storage]
    pub type PeriodServed<T> =
        StorageDoubleMap<_, Blake2_128Concat, EscrowId, Twox64Concat, u32, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NodeBound {
            account: T::AccountId,
            node: NodeKey,
        },
        EscrowOpened {
            escrow: EscrowId,
            uploader: T::AccountId,
            root: Root,
            size: u64,
            deposit: Balance,
        },
        EscrowToppedUp {
            escrow: EscrowId,
            amount: Balance,
        },
        EscrowClosed {
            escrow: EscrowId,
            returned: Balance,
        },
        AssignmentTaken {
            escrow: EscrowId,
            seeder: T::AccountId,
        },
        AssignmentLeft {
            escrow: EscrowId,
            seeder: T::AccountId,
        },
        AssignmentLapsed {
            escrow: EscrowId,
            seeder: T::AccountId,
        },
        ChallengeAnswered {
            escrow: EscrowId,
            seeder: T::AccountId,
            period: u32,
            paid: Balance,
        },
        ServingPaid {
            escrow: EscrowId,
            seeder: T::AccountId,
            receiver: T::AccountId,
            period: u32,
            bytes: u64,
            paid: Balance,
        },
        TakedownSet {
            root: Root,
            listed: bool,
        },
        ServedForgotten {
            escrow: EscrowId,
            period: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No escrow has that number.
        NoSuchEscrow,
        /// Only the escrow's uploader may do that.
        NotUploader,
        /// That root is on the takedown list.
        TakenDown,
        /// An escrow holds a file of at least one byte, for one copy or more, up to the most copies allowed.
        BadEscrow,
        /// A deposit must leave the escrow at least the existential deposit.
        BelowExistentialDeposit,
        /// A seeder binds its node before taking an assignment.
        NoNode,
        /// The uploader does not seed their own escrow.
        UploaderCannotSeed,
        /// Every copy the escrow asks for is held already.
        NoCopiesLeft,
        /// That seeder already holds it.
        AlreadyAssigned,
        /// That seeder holds no assignment on the escrow.
        NotAssigned,
        /// This period's challenge is not open now.
        OutsideWindow,
        /// An assignment taken this period is first challenged in the next.
        TooSoon,
        /// This period's challenge was answered already.
        AlreadyAnswered,
        /// The group and its proof do not show the root.
        BadProof,
        /// The assignment has not missed enough periods to end.
        NotLapsed,
        /// The escrow does not hold enough to pay.
        EscrowExhausted,
        /// A receipt's signature does not check, or it names another seeder.
        BadReceipt,
        /// A receipt for a period that is neither this one nor the last.
        StaleReceipt,
        /// The receiver is the seeder or the uploader.
        SelfDealing,
        /// A receipt claims no more than was already paid.
        NothingNew,
        /// A period's records are forgotten only once no receipt can name it.
        PeriodOpen,
        /// An amount would not fit.
        Overflow,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn integrity_test() {
            assert!(
                timing_holds(T::Period::get(), T::AnswerWindow::get()),
                "the answer window must be at least a block, close inside its period, and close while the seed                  block's hash is kept"
            );
            assert!(
                T::MaxCopies::get() > 0,
                "an escrow must be able to ask for one copy"
            );
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Tie the caller's account to its launcher's Mesh node, or move it to another node.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::bind_node())]
        pub fn bind_node(origin: OriginFor<T>, node: NodeKey) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Nodes::<T>::insert(&who, node);
            Self::deposit_event(Event::NodeBound { account: who, node });
            Ok(())
        }

        /// Open an escrow for `root`, at the caller's own prices, funded with `deposit`.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::open_escrow())]
        #[allow(clippy::too_many_arguments)]
        pub fn open_escrow(
            origin: OriginFor<T>,
            root: Root,
            size: u64,
            copies: u8,
            storage_price: Balance,
            serving_price_per_mib: Balance,
            serving_cap_per_period: u64,
            deposit: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!Takedowns::<T>::contains_key(root), Error::<T>::TakenDown);
            ensure!(
                size > 0 && copies > 0 && copies <= T::MaxCopies::get(),
                Error::<T>::BadEscrow
            );
            ensure!(
                deposit >= <T::Currency as Inspect<T::AccountId>>::minimum_balance(),
                Error::<T>::BelowExistentialDeposit
            );
            let escrow = NextEscrowId::<T>::get();
            let next = escrow.checked_add(1).ok_or(Error::<T>::Overflow)?;
            <T::Currency as Mutate<T::AccountId>>::transfer(
                &who,
                &Self::account_of(escrow),
                deposit,
                Preservation::Preserve,
            )?;
            NextEscrowId::<T>::put(next);
            Escrows::<T>::insert(
                escrow,
                Escrow::<T> {
                    uploader: who.clone(),
                    root,
                    size,
                    copies,
                    storage_price,
                    serving_price_per_mib,
                    serving_cap_per_period,
                    assigned: 0,
                },
            );
            Self::deposit_event(Event::EscrowOpened {
                escrow,
                uploader: who,
                root,
                size,
                deposit,
            });
            Ok(())
        }

        /// Add to an escrow. Anyone may.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::top_up())]
        pub fn top_up(origin: OriginFor<T>, escrow: EscrowId, amount: Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Escrows::<T>::contains_key(escrow), Error::<T>::NoSuchEscrow);
            <T::Currency as Mutate<T::AccountId>>::transfer(
                &who,
                &Self::account_of(escrow),
                amount,
                Preservation::Preserve,
            )?;
            Self::deposit_event(Event::EscrowToppedUp { escrow, amount });
            Ok(())
        }

        /// Close an escrow: everything it holds goes back to the uploader, and its assignments end.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::close_escrow())]
        pub fn close_escrow(origin: OriginFor<T>, escrow: EscrowId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let e = Escrows::<T>::get(escrow).ok_or(Error::<T>::NoSuchEscrow)?;
            ensure!(e.uploader == who, Error::<T>::NotUploader);
            let account = Self::account_of(escrow);
            let returned = <T::Currency as Inspect<T::AccountId>>::balance(&account);
            if returned > 0 {
                <T::Currency as Mutate<T::AccountId>>::transfer(
                    &account,
                    &who,
                    returned,
                    Preservation::Expendable,
                )?;
            }
            let _ = Assignments::<T>::clear_prefix(escrow, u32::from(T::MaxCopies::get()), None);
            Escrows::<T>::remove(escrow);
            Self::deposit_event(Event::EscrowClosed { escrow, returned });
            Ok(())
        }

        /// Take one of the escrow's copies to hold.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::take_assignment())]
        pub fn take_assignment(origin: OriginFor<T>, escrow: EscrowId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut e = Escrows::<T>::get(escrow).ok_or(Error::<T>::NoSuchEscrow)?;
            ensure!(!Takedowns::<T>::contains_key(e.root), Error::<T>::TakenDown);
            ensure!(Nodes::<T>::contains_key(&who), Error::<T>::NoNode);
            ensure!(e.uploader != who, Error::<T>::UploaderCannotSeed);
            ensure!(
                !Assignments::<T>::contains_key(escrow, &who),
                Error::<T>::AlreadyAssigned
            );
            ensure!(e.assigned < e.copies, Error::<T>::NoCopiesLeft);
            e.assigned += 1;
            Assignments::<T>::insert(
                escrow,
                &who,
                Assignment {
                    taken: Self::period_now(),
                    answered: None,
                },
            );
            Escrows::<T>::insert(escrow, e);
            Self::deposit_event(Event::AssignmentTaken {
                escrow,
                seeder: who,
            });
            Ok(())
        }

        /// Give up an assignment.
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::leave_assignment())]
        pub fn leave_assignment(origin: OriginFor<T>, escrow: EscrowId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::end(escrow, &who)?;
            Self::deposit_event(Event::AssignmentLeft {
                escrow,
                seeder: who,
            });
            Ok(())
        }

        /// End an assignment that has gone unanswered for more than [`Config::MaxMissed`] periods. Anyone may.
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::end_lapsed())]
        pub fn end_lapsed(
            origin: OriginFor<T>,
            escrow: EscrowId,
            seeder: T::AccountId,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            let a = Assignments::<T>::get(escrow, &seeder).ok_or(Error::<T>::NotAssigned)?;
            let last = a.answered.unwrap_or(a.taken);
            ensure!(
                Self::period_now().saturating_sub(last) > T::MaxMissed::get(),
                Error::<T>::NotLapsed
            );
            Self::end(escrow, &seeder)?;
            Self::deposit_event(Event::AssignmentLapsed { escrow, seeder });
            Ok(())
        }

        /// Answer this period's challenge: the group the chain names, and its proof. Pays the storage price.
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::answer_challenge(proof.len() as u32))]
        pub fn answer_challenge(
            origin: OriginFor<T>,
            escrow: EscrowId,
            group: BoundedVec<u8, ConstU32<{ mesh_proof::GROUP as u32 }>>,
            proof: BoundedVec<[u8; 32], ConstU32<{ mesh_proof::MAX_DEPTH as u32 }>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let e = Escrows::<T>::get(escrow).ok_or(Error::<T>::NoSuchEscrow)?;
            ensure!(!Takedowns::<T>::contains_key(e.root), Error::<T>::TakenDown);
            let mut a = Assignments::<T>::get(escrow, &who).ok_or(Error::<T>::NotAssigned)?;
            let (period, index) =
                Self::challenge(escrow, &who, &e).ok_or(Error::<T>::OutsideWindow)?;
            ensure!(a.taken < period, Error::<T>::TooSoon);
            ensure!(a.answered != Some(period), Error::<T>::AlreadyAnswered);
            ensure!(
                mesh_proof::verify(&e.root, e.size, index, &group, &proof),
                Error::<T>::BadProof
            );
            Self::pay(escrow, &who, e.storage_price)?;
            a.answered = Some(period);
            Assignments::<T>::insert(escrow, &who, a);
            Self::deposit_event(Event::ChallengeAnswered {
                escrow,
                seeder: who,
                period,
                paid: e.storage_price,
            });
            Ok(())
        }

        /// Be paid for serving, by receipts the receivers signed. Each pays what is new since that receipt's last
        /// payment, within the receiver's and the escrow's caps for the period.
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_receipts(receipts.len() as u32))]
        pub fn submit_receipts(origin: OriginFor<T>, receipts: ReceiptsOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            for (receipt, signature) in receipts {
                Self::settle(&who, receipt, signature)?;
            }
            Ok(())
        }

        /// Put a root on the takedown list, or take it off.
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::set_takedown())]
        pub fn set_takedown(origin: OriginFor<T>, root: Root, listed: bool) -> DispatchResult {
            T::TakedownOrigin::ensure_origin(origin)?;
            if listed {
                Takedowns::<T>::insert(root, ());
            } else {
                Takedowns::<T>::remove(root);
            }
            Self::deposit_event(Event::TakedownSet { root, listed });
            Ok(())
        }

        /// Forget the records of a period no receipt can name any more. Anyone may.
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::forget_served())]
        pub fn forget_served(
            origin: OriginFor<T>,
            escrow: EscrowId,
            period: u32,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            ensure!(
                period.saturating_add(1) < Self::period_now(),
                Error::<T>::PeriodOpen
            );
            let limit = T::MaxReceipts::get();
            let _ = Served::<T>::clear_prefix((escrow, period), limit, None);
            let _ = ReceiverServed::<T>::clear_prefix((escrow, period), limit, None);
            PeriodServed::<T>::remove(escrow, period);
            Self::deposit_event(Event::ServedForgotten { escrow, period });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// The account an escrow's CGT sits in. No key signs for it.
        pub fn account_of(escrow: EscrowId) -> T::AccountId {
            T::PalletId::get().into_sub_account_truncating(escrow)
        }

        /// The period the current block is in.
        pub fn period_now() -> u32 {
            let now: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
            now / PeriodLength::<T>::get()
        }

        /// The block whose hash seeds `period`'s challenges: the one after the period's first.
        pub fn seed_block(period: u32) -> u32 {
            period
                .saturating_mul(PeriodLength::<T>::get())
                .saturating_add(1)
        }

        /// This period's challenge for `seeder` on `escrow`, if its window is open now: the period and the group.
        pub fn challenge(
            escrow: EscrowId,
            seeder: &T::AccountId,
            e: &Escrow<T>,
        ) -> Option<(u32, u64)> {
            let now: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
            let period = now / PeriodLength::<T>::get();
            let seed = Self::seed_block(period);
            if now <= seed || now > seed.saturating_add(AnswerWindowLength::<T>::get()) {
                return None;
            }
            let hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::from(seed));
            let draw = sp_io::hashing::blake2_256(&(hash, escrow, seeder).encode());
            let mut first = [0u8; 8];
            first.copy_from_slice(&draw[..8]);
            Some((
                period,
                u64::from_le_bytes(first) % mesh_proof::groups(e.size),
            ))
        }

        /// The bytes a receiver signs for `receipt` on this chain.
        pub fn receipt_message(receipt: &Receipt<T::AccountId>) -> alloc::vec::Vec<u8> {
            let genesis = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::from(0u32));
            (RECEIPT_CONTEXT, genesis, receipt).encode()
        }

        /// What `bytes` cost at `price_per_mib`, rounded down; `None` if it would not fit.
        pub fn serving_cost(price_per_mib: Balance, bytes: u64) -> Option<Balance> {
            sp_runtime::helpers_128bit::multiply_by_rational_with_rounding(
                price_per_mib,
                u128::from(bytes),
                MIB,
                Rounding::Down,
            )
        }

        fn end(escrow: EscrowId, seeder: &T::AccountId) -> DispatchResult {
            ensure!(
                Assignments::<T>::contains_key(escrow, seeder),
                Error::<T>::NotAssigned
            );
            Assignments::<T>::remove(escrow, seeder);
            Escrows::<T>::mutate(escrow, |e| {
                if let Some(e) = e {
                    e.assigned = e.assigned.saturating_sub(1);
                }
            });
            Ok(())
        }

        /// Pay `amount` from the escrow, keeping its existential deposit, or refuse and move nothing.
        fn pay(escrow: EscrowId, to: &T::AccountId, amount: Balance) -> DispatchResult {
            if amount == 0 {
                return Ok(());
            }
            let account = Self::account_of(escrow);
            let available = <T::Currency as Inspect<T::AccountId>>::reducible_balance(
                &account,
                Preservation::Preserve,
                Fortitude::Polite,
            );
            ensure!(available >= amount, Error::<T>::EscrowExhausted);
            <T::Currency as Mutate<T::AccountId>>::transfer(
                &account,
                to,
                amount,
                Preservation::Preserve,
            )?;
            Ok(())
        }

        fn settle(
            seeder: &T::AccountId,
            receipt: Receipt<T::AccountId>,
            signature: T::ReceiptSignature,
        ) -> DispatchResult {
            ensure!(receipt.seeder == *seeder, Error::<T>::BadReceipt);
            ensure!(
                signature.verify(&Self::receipt_message(&receipt)[..], &receipt.receiver),
                Error::<T>::BadReceipt
            );
            let now = Self::period_now();
            ensure!(
                receipt.period == now || receipt.period.saturating_add(1) == now,
                Error::<T>::StaleReceipt
            );
            let e = Escrows::<T>::get(receipt.escrow).ok_or(Error::<T>::NoSuchEscrow)?;
            ensure!(!Takedowns::<T>::contains_key(e.root), Error::<T>::TakenDown);
            ensure!(
                Assignments::<T>::contains_key(receipt.escrow, seeder),
                Error::<T>::NotAssigned
            );
            ensure!(
                receipt.receiver != *seeder && receipt.receiver != e.uploader,
                Error::<T>::SelfDealing
            );
            let key = (
                receipt.escrow,
                receipt.period,
                receipt.receiver.clone(),
                seeder.clone(),
            );
            let paid = Served::<T>::get(&key);
            ensure!(receipt.bytes > paid, Error::<T>::NothingNew);
            // What is new from this seeder, within what this receiver and this escrow may still be paid for.
            let receiver_key = (receipt.escrow, receipt.period, receipt.receiver.clone());
            let receiver_paid = ReceiverServed::<T>::get(&receiver_key);
            let period_paid = PeriodServed::<T>::get(receipt.escrow, receipt.period);
            let new = (receipt.bytes - paid)
                .min(e.size.saturating_sub(receiver_paid))
                .min(e.serving_cap_per_period.saturating_sub(period_paid));
            ensure!(new > 0, Error::<T>::NothingNew);
            let cost =
                Self::serving_cost(e.serving_price_per_mib, new).ok_or(Error::<T>::Overflow)?;
            Self::pay(receipt.escrow, seeder, cost)?;
            Served::<T>::insert(&key, paid.checked_add(new).ok_or(Error::<T>::Overflow)?);
            ReceiverServed::<T>::insert(
                &receiver_key,
                receiver_paid.checked_add(new).ok_or(Error::<T>::Overflow)?,
            );
            PeriodServed::<T>::insert(
                receipt.escrow,
                receipt.period,
                period_paid.checked_add(new).ok_or(Error::<T>::Overflow)?,
            );
            Self::deposit_event(Event::ServingPaid {
                escrow: receipt.escrow,
                seeder: seeder.clone(),
                receiver: receipt.receiver,
                period: receipt.period,
                bytes: new,
                paid: cost,
            });
            Ok(())
        }
    }
}

/// Whether a period of `period` blocks holds an answer window of `window`: at least one block, closing inside its period,
/// and closing while the chain still keeps the seed block's hash (it keeps 256).
pub fn timing_holds(period: u32, window: u32) -> bool {
    window > 0 && window.saturating_add(1) < period && window < 256
}

pub mod runtime_api {
    use super::*;

    sp_api::decl_runtime_apis! {
        /// The Mesh, asked of chain state, for a seeder's launcher. Nothing here moves anything.
        pub trait MeshApi<AccountId> where AccountId: codec::Codec {
            /// The period and the group `seeder` must answer for `escrow` now, if the window is open and it holds an
            /// assignment there.
            fn challenge(escrow: EscrowId, seeder: AccountId) -> Option<(u32, u64)>;
            /// The bytes a receiver signs for `receipt` on this chain.
            fn receipt_message(receipt: Receipt<AccountId>) -> alloc::vec::Vec<u8>;
            /// The period the chain is in.
            fn period() -> u32;
        }
    }
}
