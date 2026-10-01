//! A test runtime for the pallet: `frame_system`, `pallet-balances`, `pallet-nfts`
//! and this pallet.
//!
//! Accounts are `AccountId32` and signatures `MultiSignature`, as in the real
//! runtime, because `pallet-nfts` ties its account type to its off-chain
//! signature type. The deposit values are **test values**, chosen so the tests can
//! tell the deposits apart; they are not the runtime's.

use frame_support::{
    derive_impl, parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, ConstU64, NeverEnsureOrigin},
};
use pallet_nfts::PalletFeatures;
use polkadot_sdk::*;
use sp_runtime::{
    traits::{IdentifyAccount, IdentityLookup, Verify},
    AccountId32, BuildStorage, MultiSignature,
};

use crate as pallet_drc369;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Nfts: pallet_nfts,
        Drc369: pallet_drc369,
    }
);

pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub type Balance = u128;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type AccountData = pallet_balances::AccountData<Balance>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type Balance = Balance;
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
}

/// Test deposits, each a different power of ten so a reserved total says which
/// deposits made it up.
pub const COLLECTION_DEPOSIT: Balance = 1_000;
pub const ITEM_DEPOSIT: Balance = 100;
pub const METADATA_DEPOSIT_BASE: Balance = 10;
pub const DEPOSIT_PER_BYTE: Balance = 1;

parameter_types! {
    pub storage Features: PalletFeatures = PalletFeatures::all_enabled();
}

impl pallet_nfts::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = u32;
    type ItemId = u32;
    type Currency = Balances;
    // As in the runtime: nobody creates a collection through `pallet-nfts`'s
    // own call. The singles collection comes from this pallet.
    type CreateOrigin = AsEnsureOriginWithArg<NeverEnsureOrigin<AccountId>>;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    // As in the runtime: a nested asset, and the asset holding it, are locked.
    type Locker = Drc369;
    type CollectionDeposit = ConstU128<COLLECTION_DEPOSIT>;
    type ItemDeposit = ConstU128<ITEM_DEPOSIT>;
    type MetadataDepositBase = ConstU128<METADATA_DEPOSIT_BASE>;
    type AttributeDepositBase = ConstU128<METADATA_DEPOSIT_BASE>;
    type DepositPerByte = ConstU128<DEPOSIT_PER_BYTE>;
    type StringLimit = ConstU32<256>;
    type KeyLimit = ConstU32<64>;
    type ValueLimit = ConstU32<256>;
    type ApprovalsLimit = ConstU32<20>;
    type ItemAttributesApprovalsLimit = ConstU32<30>;
    type MaxTips = ConstU32<10>;
    type MaxDeadlineDuration = ConstU64<10_000>;
    type MaxAttributesPerCall = ConstU32<10>;
    type Features = Features;
    type OffchainSignature = Signature;
    type OffchainPublic = AccountPublic;
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = ();
    type BlockNumberProvider = frame_system::Pallet<Test>;
}

/// Smaller than the runtime's 16, so a test can reach it in a few mints.
pub const MAX_REMIX_DEPTH: u8 = 3;

/// Smaller than the runtime's 8, so a test can reach it in a few nests.
pub const MAX_NESTING_DEPTH: u8 = 3;

/// Smaller than the runtime's 64, for the same reason.
pub const MAX_CHILDREN: u32 = 2;

impl pallet_drc369::Config for Test {
    type WeightInfo = ();
    type MaxRemixDepth = frame_support::traits::ConstU8<MAX_REMIX_DEPTH>;
    type MaxNestingDepth = frame_support::traits::ConstU8<MAX_NESTING_DEPTH>;
    type MaxChildren = ConstU32<MAX_CHILDREN>;
}

pub fn account(id: u8) -> AccountId {
    AccountId32::new([id; 32])
}

pub const ALICE: u8 = 1;
pub const BOB: u8 = 2;
/// Holds nothing at all.
pub const NOBODY: u8 = 9;

/// Alice and Bob each hold plenty; `NOBODY` holds nothing.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(account(ALICE), 1_000_000), (account(BOB), 1_000_000)],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext: sp_io::TestExternalities = t.into();
    // Events are not deposited at block zero.
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn events() -> Vec<pallet_drc369::Event<Test>> {
    System::events()
        .into_iter()
        .filter_map(|r| {
            if let RuntimeEvent::Drc369(e) = r.event {
                Some(e)
            } else {
                None
            }
        })
        .collect()
}

/// How many collections `pallet-nfts` reports having created.
pub fn nfts_collections_created() -> usize {
    System::events()
        .into_iter()
        .filter(|r| {
            matches!(
                r.event,
                RuntimeEvent::Nfts(pallet_nfts::Event::Created { .. })
            )
        })
        .count()
}
