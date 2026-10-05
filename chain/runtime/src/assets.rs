//! Assets: `pallet-nfts` as the ownership ledger and `pallet-drc369` over it
//! (ADR-025, ADR-047, M4.1), with a source recorded for every value (ADR-052).
//! Nesting (M4.2, M4.5) adds two bounds and makes `pallet-drc369` the ledger's
//! `Locker`.
//!
//! # Where each value comes from
//!
//! - **Identifiers** `u32`/`u32`: ADR-047, decision 13 row 5.
//! - **`StringLimit`, `KeyLimit`, `ValueLimit`** 256/64/256: ADR-047, decision 13
//!   row 6.
//! - **`ApprovalsLimit`, `ItemAttributesApprovalsLimit`, `MaxTips`,
//!   `MaxAttributesPerCall`, `MaxDeadlineDuration` and `Features`**: Asset Hub
//!   Westend's `pallet_nfts::Config` at the pinned tag, `polkadot-stable2606-1`
//!   (`cumulus/parachains/runtimes/assets/asset-hub-westend/src/lib.rs`), read on
//!   22 September 2026. ADR-047 names these bounds as wire format and sets none
//!   of them; the ecosystem's own asset chain is the one configuration every
//!   wallet has already met. Its deadline is twelve thirty-day months of
//!   six-second blocks, and this chain's blocks are six seconds too.
//! - **`MaxNestingDepth`, `MaxChildren`** 8/64: ADR-047, decision 13 row 7.
//! - **Deposits: PLACEHOLDERS, not decided values.** See [`deposits`]. Nesting
//!   adds none: a nest is one small entry on an asset whose own deposit already
//!   bounds how many can exist, as ADR-061 reasoned for a listing.
//! - **Weights**: `pallet-nfts`'s own reference weights, and for `pallet-drc369`
//!   placeholders built from them. Neither is benchmarked on this chain's
//!   hardware; both are debt owed to M7.2.
//!
//! # One way in
//!
//! Every DRC-369 asset carries a content reference (ADR-047), and only
//! `pallet-drc369`'s `mint` writes one. So the base call filter
//! ([`AssetCallFilter`]) lets through, of `pallet-nfts`'s calls, only moving an
//! asset and approving someone else to move it, and of `pallet-utility`'s, only
//! the atomic batch (M4.6). Creating a collection, minting,
//! burning, changing a collection's team, metadata, attributes, prices and swaps
//! are all refused, because each would make or change an item behind DRC-369's
//! back: a bare item with no fingerprint, an asset whose record outlives it, or a
//! collection whose creator no longer controls it. `CreateOrigin` refuses
//! everyone as well, so a collection cannot be created through `pallet-nfts`'s
//! call even if the filter were loosened. A root call through `pallet-sudo`
//! bypasses the filter, as it bypasses everything, on development and test
//! networks only (ADR-037).
//!
//! # A nested asset stays where it is
//!
//! The filter still lets `Nfts::transfer` through, and a nested asset must not
//! leave its parent by it. That is not the filter's job and is not done there:
//! `pallet-nfts` asks its `Locker` inside every transfer and burn, and the
//! `Locker` is `pallet-drc369`, which answers that an asset is locked while it
//! is nested or holds a nested asset (ADR-025). So the refusal is `pallet-nfts`'s
//! own `ItemLocked`, and it holds for a bare transfer, one inside `batch_all`,
//! one by an approved account, and the transfer a sale ends with.

use super::*;

use frame_support::traits::{AsEnsureOriginWithArg, Contains, NeverEnsureOrigin};
use pallet_nfts::PalletFeatures;

pub mod deposits {
    //! **PLACEHOLDERS (U-14).** What `pallet-nfts` reserves from whoever stores an
    //! asset, a collection or a name.
    //!
    //! ADR-030 says each of these "needs its own sizing", and nothing has sized
    //! them: `docs/architecture/SPONSORSHIP.md` says who pays, not how much, and
    //! it is a proposal. They are listed as undecided in
    //! `docs/economics/OPEN_QUESTIONS.md` as U-14, and AGENTS.md §5 forbids
    //! presenting such a value as decided. The development chain still needs a
    //! number, so each is **derived, not picked**, by the arithmetic ADR-030
    //! applied to the existential deposit: one account entry of about 160 bytes
    //! costs one existential deposit (ADR-036, 100 CGT). So every storage entry
    //! an asset creates costs one existential deposit, and every byte of a name
    //! costs a hundred-and-sixtieth of one.
    //!
    //! Replacing them is a change to this module and to U-14, nothing else.
    use super::super::{Balance, EXISTENTIAL_DEPOSIT};
    use polkadot_sdk::sp_arithmetic::{
        helpers_128bit::multiply_by_rational_with_rounding, per_things::Rounding,
    };

    /// PLACEHOLDER (U-14). One storage entry, priced as ADR-030 priced one
    /// account entry.
    pub const PER_ENTRY: Balance = EXISTENTIAL_DEPOSIT;

    /// The size of the account entry ADR-030 priced, in bytes.
    pub const BYTES_IN_THE_ENTRY_ADR_030_PRICED: u128 = 160;

    /// PLACEHOLDER (U-14). A collection is five entries: `pallet-nfts`'s
    /// `Collection`, `CollectionRoleOf`, `CollectionConfigOf` and
    /// `CollectionAccount`, and `pallet-drc369`'s `Singles`.
    pub const COLLECTION: Balance = 5 * PER_ENTRY;

    /// PLACEHOLDER (U-14). An asset is four entries: `pallet-nfts`'s `Item`,
    /// `Account` and `ItemConfigOf`, and `pallet-drc369`'s `Assets`.
    pub const ITEM: Balance = 4 * PER_ENTRY;

    /// PLACEHOLDER (U-14). A name is one entry, `ItemMetadataOf`, plus its bytes.
    pub const METADATA_BASE: Balance = PER_ENTRY;

    /// PLACEHOLDER (U-14). An attribute is one entry, plus its bytes. Nothing
    /// sets one yet: the call filter refuses `pallet-nfts`'s attribute calls.
    pub const ATTRIBUTE_BASE: Balance = PER_ENTRY;

    /// PLACEHOLDER (U-14). One byte: an entry's price over the entry's size, by
    /// the SDK's own ratio helper (ADR-035), never a hand-written division of a
    /// product. `10^20 / 160` is exact, so no rounding happens.
    pub fn per_byte() -> Balance {
        multiply_by_rational_with_rounding(
            PER_ENTRY,
            1,
            BYTES_IN_THE_ENTRY_ADR_030_PRICED,
            Rounding::Down,
        )
        .expect("a ratio below one of a u128 cannot overflow")
    }
}

/// Blocks in a day at this chain's target block time.
const DAYS: BlockNumber = (24 * 60 * 60 * 1000 / MILLISECS_PER_BLOCK) as BlockNumber;

parameter_types! {
    pub const NftsCollectionDeposit: Balance = deposits::COLLECTION;
    pub const NftsItemDeposit: Balance = deposits::ITEM;
    pub const NftsMetadataDepositBase: Balance = deposits::METADATA_BASE;
    pub const NftsAttributeDepositBase: Balance = deposits::ATTRIBUTE_BASE;
    pub NftsDepositPerByte: Balance = deposits::per_byte();
    /// Asset Hub's value at the pinned tag: twelve thirty-day months.
    pub const NftsMaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
    /// Asset Hub's value at the pinned tag. The call filter, not this, is what
    /// keeps trading and swaps closed.
    pub NftsFeatures: PalletFeatures = PalletFeatures::all_enabled();
}

impl pallet_nfts::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = pallet_drc369::CollectionId;
    type ItemId = pallet_drc369::ItemId;
    type Currency = Balances;
    /// Nobody creates a collection through `pallet-nfts`'s own call: a
    /// creator's singles collection comes from `pallet-drc369`'s first mint.
    type CreateOrigin = AsEnsureOriginWithArg<NeverEnsureOrigin<AccountId>>;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    /// A nested asset, and an asset holding one, cannot be transferred or
    /// burned (ADR-025). See the module documentation.
    type Locker = Drc369;
    type CollectionDeposit = NftsCollectionDeposit;
    type ItemDeposit = NftsItemDeposit;
    type MetadataDepositBase = NftsMetadataDepositBase;
    type AttributeDepositBase = NftsAttributeDepositBase;
    type DepositPerByte = NftsDepositPerByte;
    type StringLimit = ConstU32<256>;
    type KeyLimit = ConstU32<64>;
    type ValueLimit = ConstU32<256>;
    type ApprovalsLimit = ConstU32<20>;
    type ItemAttributesApprovalsLimit = ConstU32<30>;
    type MaxTips = ConstU32<10>;
    type MaxDeadlineDuration = NftsMaxDeadlineDuration;
    type MaxAttributesPerCall = ConstU32<10>;
    type Features = NftsFeatures;
    type OffchainSignature = Signature;
    type OffchainPublic = <Signature as Verify>::Signer;
    /// The SDK's reference weights, not benchmarked here (M7.2).
    type WeightInfo = pallet_nfts::weights::SubstrateWeight<Runtime>;
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = ();
    /// A standalone chain counts its own blocks (ADR-018).
    type BlockNumberProvider = System;
}

impl pallet_drc369::Config for Runtime {
    /// Placeholders, not benchmarks: M7.2 debt.
    type WeightInfo = pallet_drc369::weights::PlaceholderWeight<Runtime>;
    /// ADR-047 decision 13 row 7: an engineering bound, part of the wire format.
    type MaxRemixDepth = ConstU8<16>;
    /// ADR-047 decision 13 row 7: an engineering bound, part of the wire format.
    /// It is also the most reads the cycle check makes (R-2).
    type MaxNestingDepth = ConstU8<8>;
    /// ADR-047 decision 13 row 7: an engineering bound, part of the wire format.
    type MaxChildren = ConstU32<64>;
}

impl pallet_drc369_royalties::Config for Runtime {
    /// A sale is paid in CGT, and only in CGT.
    type Currency = Balances;
    /// ADR-047 decision 13 row 7, confirmed by the owner in ADR-057.
    type MaxRoyaltyRecipients = ConstU32<8>;
    /// Placeholders, not benchmarks: M7.2 debt.
    type WeightInfo = pallet_drc369_royalties::weights::PlaceholderWeight<Runtime>;
}

frame_support::parameter_types! {
    /// What every ARQ Wallet's account is derived from (ADR-070). Changing it
    /// would move every wallet to an account nobody funded.
    pub const ArqWalletPalletId: frame_support::PalletId = frame_support::PalletId(*b"dmg/arqw");
}

/// **U-16 PLACEHOLDERS, not decided values.** The protocol's bounds on an ARQ
/// Wallet's policy are open (OPEN_QUESTIONS.md, U-16). These let a devnet run,
/// in six-second blocks: an hour's delay before a loosening or a withdrawal, an
/// hour's outcome window, thirty days before an unclaimed payout expires, and a
/// thousand payouts per wallet per epoch. None is presented as decided.
pub mod arq_wallet_placeholders {
    pub const MIN_LOOSEN_DELAY: u32 = 600;
    pub const OUTCOME_WINDOW: u32 = 600;
    pub const MAX_ACCRUAL_EXPIRY: u32 = 432_000;
    pub const MAX_PAYOUTS_PER_EPOCH: u32 = 1_000;
}

impl pallet_arq_wallet::Config for Runtime {
    /// A wallet holds CGT and pays it, and nothing else.
    type Currency = Balances;
    type PalletId = ArqWalletPalletId;
    type MinLoosenDelay = ConstU32<{ arq_wallet_placeholders::MIN_LOOSEN_DELAY }>;
    type MaxAccrualExpiry = ConstU32<{ arq_wallet_placeholders::MAX_ACCRUAL_EXPIRY }>;
    type OutcomeWindow = ConstU32<{ arq_wallet_placeholders::OUTCOME_WINDOW }>;
    type MaxPayoutsPerEpoch = ConstU32<{ arq_wallet_placeholders::MAX_PAYOUTS_PER_EPOCH }>;
    /// Engineering bounds: a game lists a few rule versions, each `name@number`
    /// of at most 58 bytes in the SDK's form.
    type MaxRuleVersions = ConstU32<8>;
    type MaxRuleVersionLen = ConstU32<64>;
    /// Placeholders, not benchmarks: M7.2 debt.
    type WeightInfo = pallet_arq_wallet::weights::PlaceholderWeight<Runtime>;
}

/// The runtime's base call filter: everything, except the `pallet-nfts` calls
/// that would make or change an asset behind `pallet-drc369`, and every
/// `pallet-utility` call but the atomic batch. See the module documentation.
pub struct AssetCallFilter;

impl Contains<RuntimeCall> for AssetCallFilter {
    fn contains(call: &RuntimeCall) -> bool {
        match call {
            RuntimeCall::Nfts(call) => matches!(
                call,
                pallet_nfts::Call::transfer { .. }
                    | pallet_nfts::Call::approve_transfer { .. }
                    | pallet_nfts::Call::cancel_approval { .. }
                    | pallet_nfts::Call::clear_all_transfer_approvals { .. }
            ),
            // `batch_all` is here for one reason: a trade of several assets
            // must either happen entirely or not at all. `batch` and
            // `force_batch` carry on past a failure, which would move some of
            // the assets and leave the rest; `as_derivative` acts from accounts
            // derived from the signer, which nothing needs and which muddies
            // who holds an asset's deposit. The rest are root-only anyway, and
            // refusing them here costs nothing.
            //
            // This does not widen what a batch may contain. A signed origin's
            // inner calls are dispatched through this same filter, so a
            // `batch_all` carrying `Nfts::mint` is refused exactly as a bare
            // `Nfts::mint` is; `tests/assets.rs` pins that.
            RuntimeCall::Utility(call) => matches!(call, pallet_utility::Call::batch_all { .. }),
            _ => true,
        }
    }
}
