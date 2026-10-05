//! The Demiurge runtime.
//!
//! A purpose-built Substrate L1 on the Polkadot SDK (ADR-013). It replaced the
//! custom Rust devnet, which was retired and deleted at M3.5; nothing here ever
//! depended on it.
//!
//! # What this first increment contains, and what it deliberately does not
//!
//! Standard components: system, timestamp, Aura block production (ADR-019),
//! GRANDPA finality (ADR-018), balances, session and a governance-chosen
//! validator set (ADR-020), `pallet-nfts` as the asset ledger (ADR-025), and
//! `pallet-sudo` behind a feature for development and test networks (ADR-037).
//!
//! Two custom pallets in the creative layer: `pallet-drc369`, DRC-369's
//! semantics over `pallet-nfts` (M4.1, ADR-047), and `pallet-drc369-royalties`,
//! royalty terms and the sale settled in CGT they bind to (M4.2, ADR-061).
//! Their configuration, every value's source and the call filter that makes
//! `pallet-drc369` the only way an asset is created are in [`assets`].
//!
//! **There is no transaction payment yet, on purpose.** Fee classes and the
//! burn share per class are OPEN-4 in `docs/economics/OPEN_QUESTIONS.md`, and
//! AGENTS.md §5 forbids inventing a value that file lists as undecided. A
//! `WeightToFee` is such a value. The custom chain charged no fee either, so
//! this is not a regression, and a development network is usable without one.
//!
//! **There is no issuance, no genesis allocation and no reward.** The base
//! supply's split is OPEN-2 and the issuance rate is OPEN-1. Endowments exist
//! only in development chain specifications.
//!
//! # The parachain condition (ADR-018)
//!
//! The runtime is written so a later move to a parachain does not require
//! rewriting it: privileged origins are `EnsureOrigin` types, and no runtime
//! logic depends on GRANDPA state, on one block per slot, or on a fixed slot
//! time.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` expands to a large enum; this is the SDK's own setting.
#![recursion_limit = "256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;

use alloc::vec::Vec;
use polkadot_sdk::*;

use frame_support::{
    construct_runtime, derive_impl, parameter_types,
    traits::{ConstBool, ConstU32, ConstU64, ConstU8, VariantCountOf},
    weights::{
        constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
        Weight,
    },
};
use frame_system::limits::{BlockLength, BlockWeights};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, IdentifyAccount, Verify},
    MultiAddress, MultiSignature, Perbill,
};
use sp_version::RuntimeVersion;

pub mod assets;
pub mod denomination;

pub use denomination::{DECIMALS, EXISTENTIAL_DEPOSIT, SS58_PREFIX, TOKEN_NAME, TOKEN_SYMBOL};

/// Signatures are `MultiSignature`, so Sr25519 account keys are supported
/// alongside the Ed25519 that GRANDPA uses for its own authority keys
/// (ADR-023: account keys are Sr25519).
pub type Signature = MultiSignature;

/// An account identifier, derived from the public key that signed.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// How a transaction names its sender, and how a call names an account it acts
/// on: `MultiAddress`, as the rest of the ecosystem does (ADR-041).
///
/// This is the SDK's own `SolochainDefaultConfig` value and every relay chain's.
/// In practice only the `Id` variant is used, since `pallet-indices` is not
/// mounted, so the cost is one byte per transaction. The reason to pay it is
/// that every wallet, tool and integrator has met this shape and none has met
/// the alternative, and a client that assumes it and is wrong gets a refused
/// transaction whose error does not say why.
pub type Address = MultiAddress<AccountId, ()>;

/// Balances are `u128` counts of Sparks, the atomic unit (ADR-035).
pub type Balance = u128;

/// Index of a transaction in an account's sequence.
pub type Nonce = u32;

/// Block number.
pub type BlockNumber = u32;

/// A hash of some data.
pub type Hash = sp_core::H256;

/// The header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// The block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// The extrinsic type as expected by the runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// The transaction extensions applied to every signed transaction.
///
/// `CheckGenesis` and `CheckMortality` put the genesis hash into the signed
/// payload, which is what meets D-009: a transaction signed for one network is
/// not valid on another. The custom chain had no chain identifier in what it
/// signed.
pub type TxExtension = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
);

/// Executive: dispatches incoming extrinsics to the right pallet.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

/// Opaque types, for the node, which does not know the runtime's concrete types.
pub mod opaque {
    use super::*;
    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
        pub grandpa: Grandpa,
    }
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: alloc::borrow::Cow::Borrowed("demiurge"),
    impl_name: alloc::borrow::Cow::Borrowed("demiurge"),
    authoring_version: 1,
    // 2: `pallet-nfts` and `pallet-drc369` (M4.1). 3: `pallet-utility`, for a
    // trade of several assets that either happens entirely or not at all
    // (M4.6). Existing calls kept their encoding in both. 4: remix provenance
    // and `pallet-drc369-royalties` (M4.2, ADR-061); `Drc369::mint` gained its
    // `derived_from` argument, so its encoding changed and `transaction_version`
    // moved to 2. 5: nesting (M4.2, M4.5, R-2): `Drc369::nest` and `unnest`
    // are new calls and `pallet-drc369` became `pallet-nfts`'s `Locker`.
    // Existing calls kept their encoding, so `transaction_version` stays.
    // 6: `Drc369Royalties::buy_exact`, a new call, and the runtime API
    // `Drc369RoyaltiesApi`. `buy` kept its index and its encoding, which
    // `tests/assets.rs` pins, so `transaction_version` stays again.
    // 7: `pallet-arq-wallet`, ARQ Wallets (ADR-070), a new pallet at index 11.
    // No existing call changed, so `transaction_version` stays.
    // 8: `ArqWallet::open_round`, `settle_round` and `cancel_round`, prizes held
    // before a paid round (ADR-070 decision 8). New calls and storage only; no
    // stored value's encoding changed, so no migration and the same
    // `transaction_version`.
    spec_version: 8,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 2,
    system_version: 1,
};

/// Target block time. Behind a configuration type, so no runtime logic depends
/// on a fixed slot time (ADR-018's parachain condition).
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

/// A block may use at most two seconds of compute and 5 MB of proof.
const MAXIMUM_BLOCK_WEIGHT: Weight =
    Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), u64::MAX);

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub const SS58Prefix: u16 = SS58_PREFIX;
    /// 5 MB for normal extrinsics, so a block's proof stays small enough for a
    /// later parachain move (ADR-018's condition).
    pub RuntimeBlockLength: BlockLength = BlockLength::builder()
        .max_length(5 * 1024 * 1024)
        .modify_max_length_for_class(
            frame_support::dispatch::DispatchClass::Normal,
            |max| *max = NORMAL_DISPATCH_RATIO * *max,
        )
        .build();
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::with_sensible_defaults(
        MAXIMUM_BLOCK_WEIGHT,
        NORMAL_DISPATCH_RATIO,
    );
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
    /// Everything, except the `pallet-nfts` calls that would make or change an
    /// asset behind `pallet-drc369` (see [`assets`]).
    type BaseCallFilter = assets::AssetCallFilter;
    type Block = Block;
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = RuntimeBlockLength;
    type AccountId = AccountId;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type Nonce = Nonce;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountData = pallet_balances::AccountData<Balance>;
    type DbWeight = RocksDbWeight;
    type Version = Version;
    /// Prefix 42 on development and test networks (ADR-024). The mainnet prefix
    /// is a Public Release criterion and is not decided here.
    type SS58Prefix = SS58Prefix;
}

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<64>;
    type AllowMultipleBlocksPerSlot = ConstBool<false>;
    type SlotDuration = ConstU64<SLOT_DURATION>;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<64>;
    type MaxNominators = ConstU32<0>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

parameter_types! {
    /// 100 CGT, in Sparks (ADR-036, at the eighteen decimals of ADR-035).
    ///
    /// Derived from a target, never picked: sponsoring a million accounts costs
    /// a negligible share of the sponsorship budget, while dust spam stays
    /// uneconomic (ADR-030). It is revisitable before mainnet only, and only if
    /// OPEN-2's sponsorship budget changes that arithmetic.
    pub const ExistentialDepositConst: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type WeightInfo = ();
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDepositConst;
    type AccountStore = System;
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
    type DoneSlashHandler = ();
}

parameter_types! {
    /// Blocks per session, after which the validator set `pallet-session` uses
    /// is rotated.
    ///
    /// An operational parameter, not an economic one: nothing in
    /// `OPEN_QUESTIONS.md` governs it, and it does not touch supply, fees or
    /// rewards. 100 blocks is ten minutes at the six-second target, short enough
    /// that a set change can be observed on a devnet and long enough not to
    /// churn. Mainnet's value is a governance decision before launch.
    pub const SessionPeriod: BlockNumber = 100;
    pub const SessionOffset: BlockNumber = 0;
}

impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    /// A validator is identified by its account, so a parachain move can swap
    /// the session manager without changing identity (ADR-018).
    type ValidatorId = AccountId;
    type ValidatorIdOf = sp_runtime::traits::ConvertInto;
    type ShouldEndSession = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;
    /// The governance-chosen set (ADR-020).
    type SessionManager = ValidatorSet;
    type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type DisablingStrategy = ();
    type WeightInfo = ();
    type Currency = Balances;
    type KeyDeposit = ();
}

parameter_types! {
    /// The most validators the set may hold. Matches Aura's `MaxAuthorities`,
    /// because a set larger than Aura can hold would be silently truncated.
    pub const MaxValidators: u32 = 64;
    /// The fewest the set may hold.
    ///
    /// One, so that a single-validator development chain is legal. It is the
    /// floor that keeps the chain able to author at all: see the pallet's
    /// documentation for why nothing may take the set below it. A higher floor
    /// for mainnet is a decision before launch, not a default chosen here.
    pub const MinValidators: u32 = 1;
}

impl pallet_validator_set::Config for Runtime {
    /// Root for now. With `sudo` compiled in, that is the owner's key on a
    /// development network (ADR-037).
    ///
    /// **On the mainnet shape there is no path to this origin yet**, which is
    /// correct rather than an oversight: ADR-021 puts a collective in place
    /// first and full OpenGov later, and neither is built. Until one is, a
    /// mainnet runtime cannot change its validator set, and that is recorded
    /// rather than papered over with a weaker origin.
    type GovernanceOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxValidators = MaxValidators;
    type MinValidators = MinValidators;
}

/// Several calls in one transaction, so that a trade of several assets either
/// happens entirely or not at all (M4.6).
///
/// **Only `batch_all` is reachable**: the call filter refuses `pallet-utility`'s
/// other calls ([`assets::AssetCallFilter`]). `batch_all` is the atomic one — any
/// inner call failing reverts the whole transaction — and atomicity is the reason
/// the pallet is here. `batch` and `force_batch` carry on past a failure, which
/// for a trade means some assets moving and others not; `as_derivative` acts from
/// accounts derived from the signer, which nothing here needs and which would
/// complicate who holds an asset's deposit.
///
/// **It opens no second way to create an asset.** For a signed origin,
/// `batch_all` dispatches each inner call with the origin's own filter — this
/// runtime's `BaseCallFilter` — so every call refused outside a batch is refused
/// inside one. Only a root origin bypasses filters, and root exists on
/// development and test networks only (ADR-037). Both halves are pinned by tests
/// in `tests/assets.rs`.
impl pallet_utility::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    /// PLACEHOLDER, owed to M7.2 with every other unbenchmarked weight: the
    /// SDK's own reference weights for this pallet, measured on its reference
    /// hardware, not ours.
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

/// Development and test networks only (ADR-037). Absent from any mainnet
/// runtime, which is built without the `sudo` feature.
#[cfg(feature = "sudo")]
impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = ();
}

construct_runtime!(
    pub enum Runtime {
        System: frame_system = 0,
        Timestamp: pallet_timestamp = 1,
        Aura: pallet_aura = 2,
        Grandpa: pallet_grandpa = 3,
        Balances: pallet_balances = 4,
        // Session rotates the validator set; ValidatorSet decides what it is
        // (ADR-020). Aura and GRANDPA follow session's changes.
        Session: pallet_session = 5,
        ValidatorSet: pallet_validator_set = 6,
        // The asset ledger (ADR-025) and DRC-369 over it (ADR-047, M4.1).
        Nfts: pallet_nfts = 7,
        Drc369: pallet_drc369 = 8,
        // Several calls in one transaction, all or nothing (M4.6). Only
        // `batch_all` is reachable; see `assets::AssetCallFilter`.
        Utility: pallet_utility = 9,
        // Royalty terms, and the sale settled in CGT they bind to (M4.2,
        // ADR-061).
        Drc369Royalties: pallet_drc369_royalties = 10,
        // One keyless payout account per published game, governed by whoever
        // holds its Cartridge (ADR-070).
        ArqWallet: pallet_arq_wallet = 11,

        // Development and test networks only (ADR-037).
        #[cfg(feature = "sudo")]
        Sudo: pallet_sudo = 100,
    }
);

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion { VERSION }
        fn execute_block(block: <Block as sp_runtime::traits::Block>::LazyBlock) {
            Executive::execute_block(block);
        }
        fn initialize_block(header: &<Block as sp_runtime::traits::Block>::Header)
            -> sp_runtime::ExtrinsicInclusionMode
        { Executive::initialize_block(header) }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> sp_core::OpaqueMetadata {
            sp_core::OpaqueMetadata::new(Runtime::metadata().into())
        }
        fn metadata_at_version(version: u32) -> Option<sp_core::OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }
        fn metadata_versions() -> Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as sp_runtime::traits::Block>::Extrinsic)
            -> sp_runtime::ApplyExtrinsicResult
        { Executive::apply_extrinsic(extrinsic) }
        fn finalize_block() -> <Block as sp_runtime::traits::Block>::Header {
            Executive::finalize_block()
        }
        fn inherent_extrinsics(data: sp_inherents::InherentData)
            -> Vec<<Block as sp_runtime::traits::Block>::Extrinsic>
        { data.create_extrinsics() }
        fn check_inherents(
            block: <Block as sp_runtime::traits::Block>::LazyBlock,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: sp_runtime::transaction_validity::TransactionSource,
            tx: <Block as sp_runtime::traits::Block>::Extrinsic,
            block_hash: <Block as sp_runtime::traits::Block>::Hash,
        ) -> sp_runtime::transaction_validity::TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as sp_runtime::traits::Block>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(SLOT_DURATION)
        }
        fn authorities() -> Vec<AuraId> {
            pallet_aura::Authorities::<Runtime>::get().into_inner()
        }
    }

    impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
            Grandpa::grandpa_authorities()
        }
        fn current_set_id() -> sp_consensus_grandpa::SetId {
            Grandpa::current_set_id()
        }
        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as sp_runtime::traits::Block>::Hash,
                sp_runtime::traits::NumberFor<Block>,
            >,
            _key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> { None }
        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            _authority_id: GrandpaId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> { None }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(
            owner: Vec<u8>,
            seed: Option<Vec<u8>>,
        ) -> sp_session::OpaqueGeneratedSessionKeys {
            SessionKeys::generate(&owner, seed).into()
        }
        fn decode_session_keys(encoded: Vec<u8>)
            -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>>
        { SessionKeys::decode_into_raw_public_keys(&encoded) }
    }

    impl pallet_drc369::runtime_api::Drc369Api<Block, AccountId> for Runtime {
        fn assets_of(owner: AccountId) -> Vec<pallet_drc369::OwnedAsset> {
            Drc369::assets_of(&owner)
        }
        fn asset(
            collection: pallet_drc369::CollectionId,
            item: pallet_drc369::ItemId,
        ) -> Option<pallet_drc369::Asset> {
            Drc369::asset(collection, item)
        }
    }

    impl pallet_drc369_royalties::runtime_api::Drc369RoyaltiesApi<Block, AccountId> for Runtime {
        fn sale_preview(
            collection: pallet_drc369::CollectionId,
            item: pallet_drc369::ItemId,
            price: Balance,
            buyer: Option<AccountId>,
        ) -> Option<pallet_drc369_royalties::SalePreview<AccountId>> {
            Drc369Royalties::sale_preview(collection, item, price, buyer)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(account: AccountId) -> Nonce {
            System::account_nonce(account)
        }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
            frame_support::genesis_builder_helper::build_state::<RuntimeGenesisConfig>(config)
        }
        fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
            frame_support::genesis_builder_helper::get_preset::<RuntimeGenesisConfig>(id, |_| None)
        }
        fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
            Vec::new()
        }
    }
}
