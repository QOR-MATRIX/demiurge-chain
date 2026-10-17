//! A test runtime: `frame_system`, `pallet-balances` and this pallet.
//!
//! Every amount and bound here is a **test value**, not the runtime's: the existential deposit is 10, a period is 20
//! blocks and its answer window 5, so a test reaches them in a few blocks. Prices are whatever a test's uploader sets.

use frame_support::{
    derive_impl, parameter_types,
    traits::{ConstU128, ConstU32, ConstU8, Hooks},
    PalletId,
};
use polkadot_sdk::*;
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
    traits::{IdentifyAccount, IdentityLookup, Verify},
    BuildStorage, MultiSignature,
};

use crate as pallet_mesh;
use crate::Balance;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Mesh: pallet_mesh,
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

pub const EXISTENTIAL_DEPOSIT: Balance = 10;

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type Balance = Balance;
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
}

pub const PERIOD: u32 = 20;
pub const WINDOW: u32 = 5;
pub const MAX_MISSED: u32 = 3;
pub const MAX_COPIES: u8 = 3;

parameter_types! {
    pub const MeshPalletId: PalletId = PalletId(*b"dmg/mesh");
}

impl pallet_mesh::Config for Test {
    type Currency = Balances;
    type PalletId = MeshPalletId;
    type Period = ConstU32<PERIOD>;
    type AnswerWindow = ConstU32<WINDOW>;
    type MaxMissed = ConstU32<MAX_MISSED>;
    type MaxCopies = ConstU8<MAX_COPIES>;
    type MaxReceipts = ConstU32<8>;
    type TakedownOrigin = frame_system::EnsureRoot<AccountId>;
    type ReceiptSignature = Signature;
    type ReceiptSigner = AccountPublic;
    type WeightInfo = ();
}

/// A test person's key, made from a seed.
pub fn pair(id: u8) -> sr25519::Pair {
    sr25519::Pair::from_seed(&[id; 32])
}

pub fn account(id: u8) -> AccountId {
    AccountPublic::from(pair(id).public()).into_account()
}

pub const UPLOADER: u8 = 1;
pub const SEEDER: u8 = 2;
pub const OTHER_SEEDER: u8 = 3;
pub const RECEIVER: u8 = 4;
pub const OTHER_RECEIVER: u8 = 5;
pub const STRANGER: u8 = 6;
pub const START: Balance = 1_000_000_000;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: (1..=6).map(|id| (account(id), START)).collect(),
        ..Default::default()
    }
    .assimilate_storage(&mut storage)
    .unwrap();
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        // Block 1, with block 0's hash known as the genesis hash a receipt names.
        System::initialize(&1, &H256::repeat_byte(0xaa), &Default::default());
        <Mesh as Hooks<u64>>::integrity_test();
    });
    ext
}

/// Advance to block `n`, each block's parent hash distinct, as `frame_system` records it.
pub fn run_to(n: u64) {
    while System::block_number() < n {
        let next = System::block_number() + 1;
        System::initialize(
            &next,
            &H256::repeat_byte((next % 251) as u8),
            &Default::default(),
        );
    }
}

/// The first block of `period`'s answer window.
pub fn window_of(period: u32) -> u64 {
    u64::from(period * PERIOD + 2)
}
