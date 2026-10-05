//! ARQ Wallets: one keyless payout account per published game (ADR-070).
//!
//! # What it is
//!
//! A game published on ARQADE is a DRC-369 asset, its **Cartridge**. Each
//! Cartridge may have one ARQ Wallet: an account derived from this pallet's
//! [`Config::PalletId`] and the Cartridge's id, which **no key can sign for**.
//! CGT leaves it only through this pallet's calls:
//!
//! - **The governor** is whoever holds the Cartridge, read from `pallet-nfts` at
//!   every call. Transferring the game transfers control of its wallet. The
//!   governor creates the wallet, sets its policy, registers or revokes its
//!   payout authority, pauses it, and withdraws from it.
//! - **The payout authority** is one account, held by the game's server, that
//!   can do one thing: pay a player within the policy.
//! - **A player** receives payouts, and claims what accrued when a payout could
//!   not reach them at once.
//!
//! # The rules, all checked here
//!
//! - **The policy** bounds every payout: the largest single payout, a budget per
//!   epoch, a cap per recipient per epoch, and the game rule versions a payout
//!   may name. Every value in it is the developer's.
//! - **Tightening takes effect at once; loosening waits.** A policy that loosens
//!   anything, and every withdrawal, is scheduled for `loosen_delay` blocks
//!   later, in public, and the governor can cancel it until then.
//! - **Each outcome is paid once.** A payout names an outcome id and the block
//!   it was issued at. It is refused if it was issued more than
//!   [`Config::OutcomeWindow`] blocks ago, and refused if the same outcome was
//!   already paid within the window, so a retried or replayed request cannot pay
//!   twice whatever the game's own records say.
//! - **What is owed is never withdrawn.** A payout that cannot reach its player
//!   — most often because it is below the existential deposit and the player
//!   has no account yet — is recorded as owed to them instead, and the wallet's
//!   owed total is kept out of every later payout and every withdrawal.
//! - **The wallet never dies.** Every transfer out keeps the account alive, so
//!   the existential deposit paid at creation stays.
//! - **A round's prize is held before anyone competes for it** (ADR-070
//!   decision 8, "proof of prize"). The authority opens a round with a prize no
//!   larger than the epoch budget; the prize is held at once and shows on chain.
//!   Held CGT is kept out of every payout and withdrawal, exactly as owed CGT is.
//!   Settling pays the winners from the hold (accruing what cannot reach them)
//!   and releases the rest; cancelling releases it all. A round settles at or
//!   after it closes and within [`Config::OutcomeWindow`] of closing; after that
//!   it can only be cancelled, by anyone, so a hold can never be stranded.
//!   Prizes are not counted against epoch budgets or recipient caps: the cap on a
//!   round is its own declared, held prize.
//!
//! # What it does not do yet
//!
//! The protocol's bounds — [`Config::MinLoosenDelay`],
//! [`Config::MaxAccrualExpiry`], [`Config::OutcomeWindow`],
//! [`Config::MaxPayoutsPerEpoch`] — are **U-16** and undecided; the runtime
//! carries marked placeholders.
//!
//! # Arithmetic
//!
//! Only additions and subtractions of amounts, each checked: an overflow is
//! refused, never wrapped or saturated into a wrong answer. No amount is ever
//! multiplied by another (AGENTS.md §5). The test
//! `the_largest_amounts_cannot_overflow` pins it.
//!
//! # Weights
//!
//! **Placeholders, not benchmarks** — debt owed to M7.2. See [`weights`].

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use polkadot_sdk::*;

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

pub use pallet_drc369::{CollectionId, ItemId};

/// An amount of CGT, in Sparks (AGENTS.md §5): the runtime's balance type.
pub type Balance = u128;

/// The game's own name for one outcome it pays. Unique per wallet within the
/// outcome window.
pub type OutcomeId = [u8; 32];

/// Which policy fields a change loosens, in the order the policy lists them.
/// Empty if the change only tightens or keeps them. See [`Policy`].
pub fn loosened<M: Get<u32>, L: Get<u32>>(
    current: &Policy<M, L>,
    next: &Policy<M, L>,
) -> Vec<&'static str> {
    let mut out = Vec::new();
    if next.max_payout > current.max_payout {
        out.push("max_payout");
    }
    // A shorter epoch refreshes its budgets sooner, so more can be paid per block.
    if next.epoch_blocks < current.epoch_blocks {
        out.push("epoch_blocks");
    }
    if next.epoch_budget > current.epoch_budget {
        out.push("epoch_budget");
    }
    if next.per_recipient_per_epoch > current.per_recipient_per_epoch {
        out.push("per_recipient_per_epoch");
    }
    if next
        .rule_versions
        .iter()
        .any(|v| !current.rule_versions.contains(v))
    {
        out.push("rule_versions");
    }
    if next.accrual_expiry > current.accrual_expiry {
        out.push("accrual_expiry");
    }
    if next.loosen_delay < current.loosen_delay {
        out.push("loosen_delay");
    }
    out
}

use frame_support::{pallet_prelude::*, CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound};

/// What an ARQ Wallet may pay. Every value is the developer's own.
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
#[scale_info(skip_type_params(MaxVersions, MaxLen))]
#[codec(mel_bound())]
pub struct Policy<MaxVersions: Get<u32>, MaxLen: Get<u32>> {
    /// The largest single payout, in Sparks.
    pub max_payout: Balance,
    /// How long an epoch lasts, in blocks.
    pub epoch_blocks: u32,
    /// The most the wallet pays in one epoch, all recipients together.
    pub epoch_budget: Balance,
    /// The most one recipient receives in one epoch.
    pub per_recipient_per_epoch: Balance,
    /// The game rule versions a payout may name.
    pub rule_versions: BoundedVec<BoundedVec<u8, MaxLen>, MaxVersions>,
    /// How long an amount owed to a player waits to be claimed, in blocks.
    pub accrual_expiry: u32,
    /// How long a loosening or a withdrawal waits before it may apply, in blocks.
    pub loosen_delay: u32,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        traits::{
            fungible::{Inspect, Mutate},
            tokens::{DepositConsequence, Fortitude, Preservation, Provenance},
        },
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{traits::AccountIdConversion, SaturatedConversion};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: polkadot_sdk::frame_system::Config + pallet_drc369::Config {
        /// CGT. A wallet holds it and pays it, and nothing else.
        type Currency: Mutate<Self::AccountId, Balance = Balance>;

        /// What every wallet's account is derived from.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The shortest `loosen_delay` a policy may set. **U-16.**
        #[pallet::constant]
        type MinLoosenDelay: Get<u32>;

        /// The longest `accrual_expiry` a policy may set. **U-16.**
        #[pallet::constant]
        type MaxAccrualExpiry: Get<u32>;

        /// How many blocks after it was issued a payout may still be made, and
        /// so how long its outcome id is remembered. **U-16.**
        #[pallet::constant]
        type OutcomeWindow: Get<u32>;

        /// The most payouts one wallet makes in one epoch, whatever their size,
        /// which bounds the outcome records one epoch can write. **U-16.**
        #[pallet::constant]
        type MaxPayoutsPerEpoch: Get<u32>;

        /// How many rule versions a policy may list. An engineering bound.
        #[pallet::constant]
        type MaxRuleVersions: Get<u32>;

        /// The longest rule version, in bytes. An engineering bound.
        #[pallet::constant]
        type MaxRuleVersionLen: Get<u32>;

        /// How many rounds one wallet may hold prizes for at once. An engineering
        /// bound on storage.
        #[pallet::constant]
        type MaxOpenRounds: Get<u32>;

        /// How many winners one round may pay. An engineering bound on a
        /// settlement's weight.
        #[pallet::constant]
        type MaxWinners: Get<u32>;

        /// Placeholders until M7.2 benchmarks them. See [`crate::weights`].
        type WeightInfo: WeightInfo;
    }

    /// This runtime's policy type.
    pub type PolicyOf<T> = Policy<<T as Config>::MaxRuleVersions, <T as Config>::MaxRuleVersionLen>;

    /// A rule version, as a payout names it.
    pub type RuleVersionOf<T> = BoundedVec<u8, <T as Config>::MaxRuleVersionLen>;

    /// One wallet's state.
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
    pub struct Wallet<T: Config> {
        /// The policy in force.
        pub policy: PolicyOf<T>,
        /// The account allowed to pay out, if one is registered.
        pub authority: Option<T::AccountId>,
        /// Whether payouts are paused.
        pub paused: bool,
        /// What players are owed and have not claimed. Never paid out to anyone
        /// else and never withdrawn.
        pub owed: Balance,
        /// The epoch `epoch_paid` and `epoch_payouts` count.
        pub epoch: u32,
        /// What was paid or owed in that epoch.
        pub epoch_paid: Balance,
        /// How many payouts were made in that epoch.
        pub epoch_payouts: u32,
    }

    /// A change waiting for its delay: a looser policy or a withdrawal.
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
    pub enum Pending<T: Config> {
        /// A policy that loosens something.
        Policy { policy: PolicyOf<T>, due: u32 },
        /// A withdrawal to `to`.
        Withdrawal {
            to: T::AccountId,
            amount: Balance,
            due: u32,
        },
    }

    /// Every ARQ Wallet, by its Cartridge.
    #[pallet::storage]
    pub type Wallets<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CollectionId,
        Blake2_128Concat,
        ItemId,
        Wallet<T>,
        OptionQuery,
    >;

    /// A wallet's pending looser policy, if any.
    #[pallet::storage]
    pub type PendingPolicy<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CollectionId,
        Blake2_128Concat,
        ItemId,
        Pending<T>,
        OptionQuery,
    >;

    /// A wallet's pending withdrawal, if any.
    #[pallet::storage]
    pub type PendingWithdrawal<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CollectionId,
        Blake2_128Concat,
        ItemId,
        Pending<T>,
        OptionQuery,
    >;

    /// What each recipient was paid or owed in the epoch named, per wallet.
    #[pallet::storage]
    pub type RecipientPaid<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (CollectionId, ItemId),
        Blake2_128Concat,
        T::AccountId,
        (u32, Balance),
        OptionQuery,
    >;

    /// Outcomes already paid, until the block after which no payout naming them
    /// could be made again.
    #[pallet::storage]
    pub type Outcomes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (CollectionId, ItemId),
        Blake2_128Concat,
        OutcomeId,
        u32,
        OptionQuery,
    >;

    /// What each player is owed by each wallet, and the block after which it
    /// expires.
    #[pallet::storage]
    pub type Accruals<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (CollectionId, ItemId),
        Blake2_128Concat,
        T::AccountId,
        (Balance, u32),
        OptionQuery,
    >;

    /// A round whose prize is held.
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
    pub struct Round<T: Config> {
        /// The CGT held for its winners.
        pub prize: Balance,
        /// The game rule version it is played under.
        pub rule_version: RuleVersionOf<T>,
        /// The block from which it may be settled.
        pub closes_at: u32,
    }

    /// The game's own name for a round, unique among a wallet's open rounds.
    pub type RoundId = [u8; 32];

    /// Each wallet's open rounds.
    #[pallet::storage]
    pub type Rounds<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (CollectionId, ItemId),
        Blake2_128Concat,
        RoundId,
        Round<T>,
        OptionQuery,
    >;

    /// What each wallet holds for open rounds, in total. Kept apart from the
    /// wallet's own record so that adding rounds changed no stored encoding.
    #[pallet::storage]
    pub type Held<T: Config> =
        StorageMap<_, Blake2_128Concat, (CollectionId, ItemId), Balance, ValueQuery>;

    /// How many rounds each wallet has open.
    #[pallet::storage]
    pub type OpenRounds<T: Config> =
        StorageMap<_, Blake2_128Concat, (CollectionId, ItemId), u32, ValueQuery>;

    /// Every event carries what an indexer needs (ADR-028): which wallet, who,
    /// how much, and for payouts the outcome and the rule version.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A wallet was created for a Cartridge and funded.
        Created {
            collection: CollectionId,
            item: ItemId,
            account: T::AccountId,
            by: T::AccountId,
            funded: Balance,
        },
        /// A policy took effect.
        PolicySet {
            collection: CollectionId,
            item: ItemId,
            policy: PolicyOf<T>,
        },
        /// A looser policy was scheduled; `loosens` names what it loosens.
        PolicyScheduled {
            collection: CollectionId,
            item: ItemId,
            policy: PolicyOf<T>,
            due: u32,
            loosens: Vec<Vec<u8>>,
        },
        /// A payout authority was registered, replaced or revoked (`None`).
        AuthoritySet {
            collection: CollectionId,
            item: ItemId,
            authority: Option<T::AccountId>,
        },
        /// Payouts were paused or resumed.
        PausedSet {
            collection: CollectionId,
            item: ItemId,
            paused: bool,
        },
        /// A player was paid.
        Paid {
            collection: CollectionId,
            item: ItemId,
            outcome: OutcomeId,
            rule_version: Vec<u8>,
            to: T::AccountId,
            amount: Balance,
        },
        /// A payout could not reach its player, and is owed to them instead.
        Accrued {
            collection: CollectionId,
            item: ItemId,
            outcome: OutcomeId,
            rule_version: Vec<u8>,
            to: T::AccountId,
            amount: Balance,
            expires: u32,
        },
        /// A player claimed what they were owed.
        Claimed {
            collection: CollectionId,
            item: ItemId,
            to: T::AccountId,
            amount: Balance,
        },
        /// An unclaimed amount expired and returned to the wallet.
        AccrualExpired {
            collection: CollectionId,
            item: ItemId,
            of: T::AccountId,
            amount: Balance,
        },
        /// A withdrawal was scheduled.
        WithdrawalScheduled {
            collection: CollectionId,
            item: ItemId,
            to: T::AccountId,
            amount: Balance,
            due: u32,
        },
        /// A withdrawal was made.
        Withdrawn {
            collection: CollectionId,
            item: ItemId,
            to: T::AccountId,
            amount: Balance,
        },
        /// The governor cancelled what was pending.
        PendingCancelled {
            collection: CollectionId,
            item: ItemId,
        },
        /// A round opened, its prize held.
        RoundOpened {
            collection: CollectionId,
            item: ItemId,
            round: RoundId,
            prize: Balance,
            rule_version: Vec<u8>,
            closes_at: u32,
        },
        /// A round was settled: `paid` reached their winners, `accrued` is owed
        /// to winners who could not receive it yet, `released` returned to the
        /// wallet. Together they are the prize.
        RoundSettled {
            collection: CollectionId,
            item: ItemId,
            round: RoundId,
            paid: Vec<(T::AccountId, Balance)>,
            accrued: Vec<(T::AccountId, Balance)>,
            released: Balance,
        },
        /// A round was cancelled and its whole prize released.
        RoundCancelled {
            collection: CollectionId,
            item: ItemId,
            round: RoundId,
            prize: Balance,
            by: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No DRC-369 asset has that collection and item.
        UnknownCartridge,
        /// Only whoever holds the Cartridge may do that.
        NotGovernor,
        /// The Cartridge already has a wallet.
        WalletExists,
        /// The Cartridge has no wallet.
        NoWallet,
        /// The first deposit must be at least the existential deposit, which
        /// keeps the wallet's account alive.
        BelowExistentialDeposit,
        /// A cap or budget is zero, so nothing could ever be paid.
        ZeroLimit,
        /// The largest payout is above the cap per recipient, or the cap per
        /// recipient above the epoch budget.
        LimitsOutOfOrder,
        /// An epoch must last at least one block.
        ZeroEpoch,
        /// The accrual expiry is zero, or above the protocol's maximum.
        AccrualExpiryOutOfBounds,
        /// The loosening delay is below the protocol's minimum.
        DelayTooShort,
        /// A policy must list at least one rule version, each once.
        BadRuleVersions,
        /// Only the wallet's payout authority may pay out.
        NotAuthority,
        /// Payouts are paused.
        Paused,
        /// A payout of nothing.
        ZeroAmount,
        /// Above the policy's largest single payout.
        AboveMaxPayout,
        /// The payout names a rule version the policy does not list.
        RuleVersionNotAllowed,
        /// The payout was issued in the future, or longer ago than the outcome
        /// window.
        OutsideOutcomeWindow,
        /// This outcome was already paid.
        OutcomePaid,
        /// The epoch's budget would be exceeded.
        EpochBudgetExceeded,
        /// The recipient's cap for this epoch would be exceeded.
        RecipientCapExceeded,
        /// The epoch's number of payouts is reached.
        TooManyPayouts,
        /// The wallet does not hold enough beyond what it owes and the deposit
        /// that keeps it alive.
        InsufficientFunds,
        /// An amount would overflow. Refused, never wrapped.
        Overflow,
        /// Nothing is owed to that account.
        NothingOwed,
        /// What is owed has expired.
        AccrualExpired,
        /// What is owed has not expired yet.
        AccrualNotExpired,
        /// The amount owed still cannot reach the account: it is below the
        /// existential deposit and the account does not exist.
        CannotReceiveYet,
        /// Nothing is pending.
        NothingPending,
        /// The change is still waiting for its delay.
        NotDue,
        /// A withdrawal is already pending.
        WithdrawalPending,
        /// The outcome's record is still needed.
        OutcomeStillRemembered,
        /// A round with that id is already open.
        RoundExists,
        /// No open round has that id.
        NoRound,
        /// The wallet has as many rounds open as it may.
        TooManyRounds,
        /// A round's prize may not exceed the policy's epoch budget.
        PrizeAboveBudget,
        /// A round must close after the block it opens in.
        ClosesInPast,
        /// The round has not closed yet.
        RoundNotClosed,
        /// The round closed longer ago than the outcome window; it can only be
        /// cancelled.
        RoundExpired,
        /// The winners' amounts add up to more than the prize.
        WinnersExceedPrize,
        /// The same account is named twice among the winners.
        DuplicateWinner,
        /// Only the authority or the governor may cancel an open round before it
        /// expires.
        NotAllowedToCancel,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create the Cartridge's wallet with `policy`, and fund it with
        /// `funding` from the caller, who must hold the Cartridge. `funding`
        /// must be at least the existential deposit, which stays.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create())]
        pub fn create(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            policy: PolicyOf<T>,
            funding: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_governor(&who, collection, item)?;
            ensure!(
                !Wallets::<T>::contains_key(collection, item),
                Error::<T>::WalletExists
            );
            Self::validate(&policy)?;
            ensure!(
                funding >= <<T as Config>::Currency as Inspect<T::AccountId>>::minimum_balance(),
                Error::<T>::BelowExistentialDeposit
            );
            let account = Self::account_of(collection, item);
            <<T as Config>::Currency as Mutate<T::AccountId>>::transfer(
                &who,
                &account,
                funding,
                Preservation::Preserve,
            )?;
            Wallets::<T>::insert(
                collection,
                item,
                Wallet::<T> {
                    policy: policy.clone(),
                    authority: None,
                    paused: false,
                    owed: 0,
                    epoch: Self::epoch_of(&policy),
                    epoch_paid: 0,
                    epoch_payouts: 0,
                },
            );
            Self::deposit_event(Event::Created {
                collection,
                item,
                account,
                by: who,
                funded: funding,
            });
            Self::deposit_event(Event::PolicySet {
                collection,
                item,
                policy,
            });
            Ok(())
        }

        /// Set a new policy. If it only tightens, it takes effect at once;
        /// if it loosens anything, it is scheduled for the current policy's
        /// `loosen_delay` later and applied by [`Pallet::apply_policy`].
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::set_policy())]
        pub fn set_policy(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            policy: PolicyOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_governor(&who, collection, item)?;
            Self::validate(&policy)?;
            let mut wallet = Wallets::<T>::get(collection, item).ok_or(Error::<T>::NoWallet)?;
            let loosens = loosened(&wallet.policy, &policy);
            if loosens.is_empty() {
                wallet.policy = policy.clone();
                Wallets::<T>::insert(collection, item, wallet);
                PendingPolicy::<T>::remove(collection, item);
                Self::deposit_event(Event::PolicySet {
                    collection,
                    item,
                    policy,
                });
            } else {
                let due = Self::now().saturating_add(wallet.policy.loosen_delay);
                PendingPolicy::<T>::insert(
                    collection,
                    item,
                    Pending::<T>::Policy {
                        policy: policy.clone(),
                        due,
                    },
                );
                Self::deposit_event(Event::PolicyScheduled {
                    collection,
                    item,
                    policy,
                    due,
                    loosens: loosens.into_iter().map(|f| f.as_bytes().to_vec()).collect(),
                });
            }
            Ok(())
        }

        /// Apply a scheduled policy once its delay has passed. Anyone may.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::apply_policy())]
        pub fn apply_policy(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            let Some(Pending::Policy { policy, due }) = PendingPolicy::<T>::get(collection, item)
            else {
                return Err(Error::<T>::NothingPending.into());
            };
            ensure!(Self::now() >= due, Error::<T>::NotDue);
            Wallets::<T>::try_mutate(collection, item, |w| -> DispatchResult {
                let wallet = w.as_mut().ok_or(Error::<T>::NoWallet)?;
                wallet.policy = policy.clone();
                Ok(())
            })?;
            PendingPolicy::<T>::remove(collection, item);
            Self::deposit_event(Event::PolicySet {
                collection,
                item,
                policy,
            });
            Ok(())
        }

        /// Cancel a pending policy and a pending withdrawal. The governor only.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_pending())]
        pub fn cancel_pending(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_governor(&who, collection, item)?;
            ensure!(
                PendingPolicy::<T>::contains_key(collection, item)
                    || PendingWithdrawal::<T>::contains_key(collection, item),
                Error::<T>::NothingPending
            );
            PendingPolicy::<T>::remove(collection, item);
            PendingWithdrawal::<T>::remove(collection, item);
            Self::deposit_event(Event::PendingCancelled { collection, item });
            Ok(())
        }

        /// Register, replace or revoke (`None`) the payout authority. At once.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::set_authority())]
        pub fn set_authority(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            authority: Option<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_governor(&who, collection, item)?;
            Wallets::<T>::try_mutate(collection, item, |w| -> DispatchResult {
                w.as_mut().ok_or(Error::<T>::NoWallet)?.authority = authority.clone();
                Ok(())
            })?;
            Self::deposit_event(Event::AuthoritySet {
                collection,
                item,
                authority,
            });
            Ok(())
        }

        /// Pause or resume payouts. At once.
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::set_paused())]
        pub fn set_paused(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            paused: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_governor(&who, collection, item)?;
            Wallets::<T>::try_mutate(collection, item, |w| -> DispatchResult {
                w.as_mut().ok_or(Error::<T>::NoWallet)?.paused = paused;
                Ok(())
            })?;
            Self::deposit_event(Event::PausedSet {
                collection,
                item,
                paused,
            });
            Ok(())
        }

        /// Pay `amount` to `to` for `outcome`, issued at block `issued_at` under
        /// `rule_version`. The payout authority only, within the policy. If the
        /// amount cannot reach `to` now, it is owed to them instead.
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::payout())]
        // Every argument is a field an indexer and an auditor read from the call
        // itself; bundling them in a struct would only hide them in metadata.
        #[allow(clippy::too_many_arguments)]
        pub fn payout(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            outcome: OutcomeId,
            issued_at: u32,
            rule_version: RuleVersionOf<T>,
            to: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut wallet = Wallets::<T>::get(collection, item).ok_or(Error::<T>::NoWallet)?;
            ensure!(
                wallet.authority.as_ref() == Some(&who),
                Error::<T>::NotAuthority
            );
            ensure!(!wallet.paused, Error::<T>::Paused);
            ensure!(amount > 0, Error::<T>::ZeroAmount);
            ensure!(
                amount <= wallet.policy.max_payout,
                Error::<T>::AboveMaxPayout
            );
            ensure!(
                wallet.policy.rule_versions.contains(&rule_version),
                Error::<T>::RuleVersionNotAllowed
            );

            let now = Self::now();
            let window = T::OutcomeWindow::get();
            ensure!(
                issued_at <= now && now - issued_at <= window,
                Error::<T>::OutsideOutcomeWindow
            );
            let key = (collection, item);
            ensure!(
                !Outcomes::<T>::contains_key(key, outcome),
                Error::<T>::OutcomePaid
            );

            let epoch = Self::epoch_of(&wallet.policy);
            if epoch != wallet.epoch {
                wallet.epoch = epoch;
                wallet.epoch_paid = 0;
                wallet.epoch_payouts = 0;
            }
            ensure!(
                wallet.epoch_payouts < T::MaxPayoutsPerEpoch::get(),
                Error::<T>::TooManyPayouts
            );
            let epoch_paid = wallet
                .epoch_paid
                .checked_add(amount)
                .ok_or(Error::<T>::Overflow)?;
            ensure!(
                epoch_paid <= wallet.policy.epoch_budget,
                Error::<T>::EpochBudgetExceeded
            );
            let recipient_paid = match RecipientPaid::<T>::get(key, &to) {
                Some((e, paid)) if e == epoch => paid,
                _ => 0,
            }
            .checked_add(amount)
            .ok_or(Error::<T>::Overflow)?;
            ensure!(
                recipient_paid <= wallet.policy.per_recipient_per_epoch,
                Error::<T>::RecipientCapExceeded
            );
            ensure!(
                Self::available(collection, item, wallet.owed) >= amount,
                Error::<T>::InsufficientFunds
            );

            wallet.epoch_paid = epoch_paid;
            wallet.epoch_payouts = wallet.epoch_payouts.saturating_add(1);
            RecipientPaid::<T>::insert(key, &to, (epoch, recipient_paid));
            Outcomes::<T>::insert(key, outcome, issued_at.saturating_add(window));

            let rule_version = rule_version.into_inner();
            if Self::can_receive(&to, amount) {
                let account = Self::account_of(collection, item);
                <<T as Config>::Currency as Mutate<T::AccountId>>::transfer(
                    &account,
                    &to,
                    amount,
                    Preservation::Preserve,
                )?;
                Wallets::<T>::insert(collection, item, wallet);
                Self::deposit_event(Event::Paid {
                    collection,
                    item,
                    outcome,
                    rule_version,
                    to,
                    amount,
                });
            } else {
                let expires = now.saturating_add(wallet.policy.accrual_expiry);
                let owed_to = match Accruals::<T>::get(key, &to) {
                    Some((owed, _)) => owed,
                    None => 0,
                }
                .checked_add(amount)
                .ok_or(Error::<T>::Overflow)?;
                wallet.owed = wallet
                    .owed
                    .checked_add(amount)
                    .ok_or(Error::<T>::Overflow)?;
                Accruals::<T>::insert(key, &to, (owed_to, expires));
                Wallets::<T>::insert(collection, item, wallet);
                Self::deposit_event(Event::Accrued {
                    collection,
                    item,
                    outcome,
                    rule_version,
                    to,
                    amount,
                    expires,
                });
            }
            Ok(())
        }

        /// Claim everything the wallet owes the caller, if it can reach them.
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::claim())]
        pub fn claim(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let key = (collection, item);
            let (amount, expires) = Accruals::<T>::get(key, &who).ok_or(Error::<T>::NothingOwed)?;
            ensure!(Self::now() <= expires, Error::<T>::AccrualExpired);
            ensure!(
                Self::can_receive(&who, amount),
                Error::<T>::CannotReceiveYet
            );
            let account = Self::account_of(collection, item);
            <<T as Config>::Currency as Mutate<T::AccountId>>::transfer(
                &account,
                &who,
                amount,
                Preservation::Preserve,
            )?;
            Accruals::<T>::remove(key, &who);
            Wallets::<T>::try_mutate(collection, item, |w| -> DispatchResult {
                let wallet = w.as_mut().ok_or(Error::<T>::NoWallet)?;
                wallet.owed = wallet.owed.saturating_sub(amount);
                Ok(())
            })?;
            Self::deposit_event(Event::Claimed {
                collection,
                item,
                to: who,
                amount,
            });
            Ok(())
        }

        /// Return an expired, unclaimed amount to the wallet. Anyone may.
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::expire_accrual())]
        pub fn expire_accrual(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            of: T::AccountId,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            let key = (collection, item);
            let (amount, expires) = Accruals::<T>::get(key, &of).ok_or(Error::<T>::NothingOwed)?;
            ensure!(Self::now() > expires, Error::<T>::AccrualNotExpired);
            Accruals::<T>::remove(key, &of);
            Wallets::<T>::try_mutate(collection, item, |w| -> DispatchResult {
                let wallet = w.as_mut().ok_or(Error::<T>::NoWallet)?;
                wallet.owed = wallet.owed.saturating_sub(amount);
                Ok(())
            })?;
            Self::deposit_event(Event::AccrualExpired {
                collection,
                item,
                of,
                amount,
            });
            Ok(())
        }

        /// Schedule a withdrawal of `amount` to `to`, for the policy's
        /// `loosen_delay` later. The governor only, and never more than the
        /// wallet holds beyond what it owes and its deposit.
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::schedule_withdrawal())]
        pub fn schedule_withdrawal(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            to: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_governor(&who, collection, item)?;
            let wallet = Wallets::<T>::get(collection, item).ok_or(Error::<T>::NoWallet)?;
            ensure!(amount > 0, Error::<T>::ZeroAmount);
            ensure!(
                !PendingWithdrawal::<T>::contains_key(collection, item),
                Error::<T>::WithdrawalPending
            );
            ensure!(
                Self::available(collection, item, wallet.owed) >= amount,
                Error::<T>::InsufficientFunds
            );
            let due = Self::now().saturating_add(wallet.policy.loosen_delay);
            PendingWithdrawal::<T>::insert(
                collection,
                item,
                Pending::<T>::Withdrawal {
                    to: to.clone(),
                    amount,
                    due,
                },
            );
            Self::deposit_event(Event::WithdrawalScheduled {
                collection,
                item,
                to,
                amount,
                due,
            });
            Ok(())
        }

        /// Make a scheduled withdrawal once its delay has passed. Anyone may.
        /// Refused if, by then, the wallet owes too much for it.
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::execute_withdrawal())]
        pub fn execute_withdrawal(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            let Some(Pending::Withdrawal { to, amount, due }) =
                PendingWithdrawal::<T>::get(collection, item)
            else {
                return Err(Error::<T>::NothingPending.into());
            };
            ensure!(Self::now() >= due, Error::<T>::NotDue);
            let wallet = Wallets::<T>::get(collection, item).ok_or(Error::<T>::NoWallet)?;
            ensure!(
                Self::available(collection, item, wallet.owed) >= amount,
                Error::<T>::InsufficientFunds
            );
            let account = Self::account_of(collection, item);
            <<T as Config>::Currency as Mutate<T::AccountId>>::transfer(
                &account,
                &to,
                amount,
                Preservation::Preserve,
            )?;
            PendingWithdrawal::<T>::remove(collection, item);
            Self::deposit_event(Event::Withdrawn {
                collection,
                item,
                to,
                amount,
            });
            Ok(())
        }

        /// Forget an outcome no payout could name again. Anyone may.
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::prune_outcome())]
        pub fn prune_outcome(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            outcome: OutcomeId,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            let key = (collection, item);
            let until = Outcomes::<T>::get(key, outcome).ok_or(Error::<T>::NothingPending)?;
            ensure!(Self::now() > until, Error::<T>::OutcomeStillRemembered);
            Outcomes::<T>::remove(key, outcome);
            Ok(())
        }

        /// Open a round and hold its prize. The payout authority only, within
        /// the policy's epoch budget and the wallet's available funds.
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::open_round())]
        pub fn open_round(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            round: RoundId,
            prize: Balance,
            rule_version: RuleVersionOf<T>,
            closes_at: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let wallet = Wallets::<T>::get(collection, item).ok_or(Error::<T>::NoWallet)?;
            ensure!(
                wallet.authority.as_ref() == Some(&who),
                Error::<T>::NotAuthority
            );
            ensure!(!wallet.paused, Error::<T>::Paused);
            ensure!(prize > 0, Error::<T>::ZeroAmount);
            ensure!(
                prize <= wallet.policy.epoch_budget,
                Error::<T>::PrizeAboveBudget
            );
            ensure!(
                wallet.policy.rule_versions.contains(&rule_version),
                Error::<T>::RuleVersionNotAllowed
            );
            ensure!(closes_at > Self::now(), Error::<T>::ClosesInPast);
            let key = (collection, item);
            ensure!(
                !Rounds::<T>::contains_key(key, round),
                Error::<T>::RoundExists
            );
            let open = OpenRounds::<T>::get(key);
            ensure!(open < T::MaxOpenRounds::get(), Error::<T>::TooManyRounds);
            ensure!(
                Self::available(collection, item, wallet.owed) >= prize,
                Error::<T>::InsufficientFunds
            );
            let held = Held::<T>::get(key)
                .checked_add(prize)
                .ok_or(Error::<T>::Overflow)?;

            Held::<T>::insert(key, held);
            OpenRounds::<T>::insert(key, open.saturating_add(1));
            Rounds::<T>::insert(
                key,
                round,
                Round::<T> {
                    prize,
                    rule_version: rule_version.clone(),
                    closes_at,
                },
            );
            Self::deposit_event(Event::RoundOpened {
                collection,
                item,
                round,
                prize,
                rule_version: rule_version.into_inner(),
                closes_at,
            });
            Ok(())
        }

        /// Settle a closed round: pay each winner from the held prize, accrue
        /// what cannot reach them yet, and release the rest. The payout
        /// authority only, from the round's close until the outcome window
        /// after it.
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::settle_round())]
        pub fn settle_round(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            round: RoundId,
            winners: BoundedVec<(T::AccountId, Balance), T::MaxWinners>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut wallet = Wallets::<T>::get(collection, item).ok_or(Error::<T>::NoWallet)?;
            ensure!(
                wallet.authority.as_ref() == Some(&who),
                Error::<T>::NotAuthority
            );
            ensure!(!wallet.paused, Error::<T>::Paused);
            let key = (collection, item);
            let r = Rounds::<T>::get(key, round).ok_or(Error::<T>::NoRound)?;
            let now = Self::now();
            ensure!(now >= r.closes_at, Error::<T>::RoundNotClosed);
            ensure!(
                now - r.closes_at <= T::OutcomeWindow::get(),
                Error::<T>::RoundExpired
            );

            let mut total: Balance = 0;
            for (index, (account, amount)) in winners.iter().enumerate() {
                ensure!(*amount > 0, Error::<T>::ZeroAmount);
                ensure!(
                    !winners[..index].iter().any(|(other, _)| other == account),
                    Error::<T>::DuplicateWinner
                );
                total = total.checked_add(*amount).ok_or(Error::<T>::Overflow)?;
            }
            ensure!(total <= r.prize, Error::<T>::WinnersExceedPrize);

            // The prize leaves the hold first, so what the winners are paid is
            // checked against the wallet's balance, not double-counted.
            Held::<T>::mutate(key, |held| *held = held.saturating_sub(r.prize));
            OpenRounds::<T>::mutate(key, |open| *open = open.saturating_sub(1));
            Rounds::<T>::remove(key, round);

            let account = Self::account_of(collection, item);
            let expires = now.saturating_add(wallet.policy.accrual_expiry);
            let mut paid = Vec::new();
            let mut accrued = Vec::new();
            for (to, amount) in winners.into_inner() {
                if Self::can_receive(&to, amount) {
                    <<T as Config>::Currency as Mutate<T::AccountId>>::transfer(
                        &account,
                        &to,
                        amount,
                        Preservation::Preserve,
                    )?;
                    paid.push((to, amount));
                } else {
                    let owed_to = Accruals::<T>::get(key, &to)
                        .map(|(owed, _)| owed)
                        .unwrap_or(0)
                        .checked_add(amount)
                        .ok_or(Error::<T>::Overflow)?;
                    wallet.owed = wallet
                        .owed
                        .checked_add(amount)
                        .ok_or(Error::<T>::Overflow)?;
                    Accruals::<T>::insert(key, &to, (owed_to, expires));
                    accrued.push((to, amount));
                }
            }
            Wallets::<T>::insert(collection, item, wallet);
            Self::deposit_event(Event::RoundSettled {
                collection,
                item,
                round,
                paid,
                accrued,
                released: r.prize - total,
            });
            Ok(())
        }

        /// Cancel an open round and release its whole prize. The authority or
        /// the governor at any time; anyone once the round can no longer be
        /// settled, so a hold is never stranded.
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_round())]
        pub fn cancel_round(
            origin: OriginFor<T>,
            collection: CollectionId,
            item: ItemId,
            round: RoundId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let wallet = Wallets::<T>::get(collection, item).ok_or(Error::<T>::NoWallet)?;
            let key = (collection, item);
            let r = Rounds::<T>::get(key, round).ok_or(Error::<T>::NoRound)?;
            let expired = Self::now().saturating_sub(r.closes_at) > T::OutcomeWindow::get()
                && Self::now() > r.closes_at;
            let entitled = wallet.authority.as_ref() == Some(&who)
                || pallet_nfts::Pallet::<T>::owner(collection, item).as_ref() == Some(&who);
            ensure!(entitled || expired, Error::<T>::NotAllowedToCancel);

            Held::<T>::mutate(key, |held| *held = held.saturating_sub(r.prize));
            OpenRounds::<T>::mutate(key, |open| *open = open.saturating_sub(1));
            Rounds::<T>::remove(key, round);
            Self::deposit_event(Event::RoundCancelled {
                collection,
                item,
                round,
                prize: r.prize,
                by: who,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// The wallet account of a Cartridge, whether or not it was created.
        pub fn account_of(collection: CollectionId, item: ItemId) -> T::AccountId {
            T::PalletId::get().into_sub_account_truncating((collection, item))
        }

        /// What the wallet can pay out or withdraw: everything above its
        /// deposit, less what it owes and what it holds for open rounds.
        pub fn available(collection: CollectionId, item: ItemId, owed: Balance) -> Balance {
            <<T as Config>::Currency as Inspect<T::AccountId>>::reducible_balance(
                &Self::account_of(collection, item),
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .saturating_sub(owed)
            .saturating_sub(Held::<T>::get((collection, item)))
        }

        fn now() -> u32 {
            frame_system::Pallet::<T>::block_number().saturated_into::<u32>()
        }

        fn epoch_of(policy: &PolicyOf<T>) -> u32 {
            Self::now() / policy.epoch_blocks.max(1)
        }

        fn ensure_governor(
            who: &T::AccountId,
            collection: CollectionId,
            item: ItemId,
        ) -> DispatchResult {
            ensure!(
                pallet_drc369::Pallet::<T>::asset(collection, item).is_some(),
                Error::<T>::UnknownCartridge
            );
            ensure!(
                pallet_nfts::Pallet::<T>::owner(collection, item).as_ref() == Some(who),
                Error::<T>::NotGovernor
            );
            Ok(())
        }

        fn can_receive(to: &T::AccountId, amount: Balance) -> bool {
            <<T as Config>::Currency as Inspect<T::AccountId>>::can_deposit(
                to,
                amount,
                Provenance::Extant,
            ) == DepositConsequence::Success
        }

        /// The checks every policy passes, the SDK's `validatePolicy` in Rust.
        pub fn validate(p: &PolicyOf<T>) -> DispatchResult {
            ensure!(
                p.max_payout > 0 && p.epoch_budget > 0 && p.per_recipient_per_epoch > 0,
                Error::<T>::ZeroLimit
            );
            ensure!(
                p.max_payout <= p.per_recipient_per_epoch
                    && p.per_recipient_per_epoch <= p.epoch_budget,
                Error::<T>::LimitsOutOfOrder
            );
            ensure!(p.epoch_blocks > 0, Error::<T>::ZeroEpoch);
            ensure!(
                p.accrual_expiry > 0 && p.accrual_expiry <= T::MaxAccrualExpiry::get(),
                Error::<T>::AccrualExpiryOutOfBounds
            );
            ensure!(
                p.loosen_delay >= T::MinLoosenDelay::get(),
                Error::<T>::DelayTooShort
            );
            ensure!(!p.rule_versions.is_empty(), Error::<T>::BadRuleVersions);
            for (i, v) in p.rule_versions.iter().enumerate() {
                ensure!(
                    !v.is_empty() && !p.rule_versions[..i].contains(v),
                    Error::<T>::BadRuleVersions
                );
            }
            Ok(())
        }
    }
}
