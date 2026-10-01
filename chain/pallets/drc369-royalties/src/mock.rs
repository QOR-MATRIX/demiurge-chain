//! A test runtime for the pallet: `frame_system`, `pallet-balances`,
//! `pallet-nfts`, `pallet-drc369` and this pallet.
//!
//! Accounts are `AccountId32` and signatures `MultiSignature`, as in the real
//! runtime. Every amount here is a **test value**, not the runtime's: the
//! existential deposit is 10 so a test can reach it, and the deposits are small
//! powers of ten.

use frame_support::{
    derive_impl, parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, ConstU64, ConstU8, NeverEnsureOrigin},
};
use pallet_nfts::PalletFeatures;
use polkadot_sdk::*;
use sp_runtime::{
    traits::{IdentifyAccount, IdentityLookup, Verify},
    AccountId32, BuildStorage, MultiSignature,
};

use crate as pallet_drc369_royalties;
use crate::Balance;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Nfts: pallet_nfts,
        Drc369: pallet_drc369,
        Royalties: pallet_drc369_royalties,
    }
);

pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type AccountData = pallet_balances::AccountData<Balance>;
}

/// A test value, small enough that a small royalty can fall below it.
pub const EXISTENTIAL_DEPOSIT: Balance = 10;

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type Balance = Balance;
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
}

parameter_types! {
    pub storage Features: PalletFeatures = PalletFeatures::all_enabled();
}

impl pallet_nfts::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = u32;
    type ItemId = u32;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<NeverEnsureOrigin<AccountId>>;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type Locker = ();
    type CollectionDeposit = ConstU128<1_000>;
    type ItemDeposit = ConstU128<100>;
    type MetadataDepositBase = ConstU128<10>;
    type AttributeDepositBase = ConstU128<10>;
    type DepositPerByte = ConstU128<1>;
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

impl pallet_drc369::Config for Test {
    type WeightInfo = ();
    type MaxRemixDepth = ConstU8<16>;
    type MaxNestingDepth = ConstU8<8>;
    type MaxChildren = ConstU32<64>;
}

/// The runtime's value (ADR-057), so the bound is tested as it will run.
pub const MAX_RECIPIENTS: u32 = 8;

impl pallet_drc369_royalties::Config for Test {
    type Currency = Balances;
    type MaxRoyaltyRecipients = ConstU32<MAX_RECIPIENTS>;
    type WeightInfo = ();
}

pub fn account(id: u8) -> AccountId {
    AccountId32::new([id; 32])
}

pub const ALICE: u8 = 1;
pub const BOB: u8 = 2;
pub const CAROL: u8 = 3;
pub const DAVE: u8 = 4;
/// Holds nothing at all, so has no account.
pub const NOBODY: u8 = 9;

/// Every named account starts with this much.
pub const START: Balance = 1_000_000;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: [ALICE, BOB, CAROL, DAVE]
            .into_iter()
            .map(|who| (account(who), START))
            .collect(),
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext: sp_io::TestExternalities = t.into();
    // Events are not deposited at block zero.
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn events() -> Vec<pallet_drc369_royalties::Event<Test>> {
    System::events()
        .into_iter()
        .filter_map(|r| {
            if let RuntimeEvent::Royalties(e) = r.event {
                Some(e)
            } else {
                None
            }
        })
        .collect()
}
