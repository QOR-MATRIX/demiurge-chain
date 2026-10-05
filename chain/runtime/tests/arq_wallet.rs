//! ARQ Wallets in the real runtime (ADR-070): what only this runtime decides.
//! The pallet's own rules are tested in `pallets/arq-wallet/src/tests.rs`.
//!
//! - The calls are reachable: the base call filter lets them through.
//! - A wallet is created for a Cartridge, funded in CGT and pays a player.
//! - The protocol's bounds are the marked U-16 placeholders, and nothing else.

use demiurge_runtime::{
    assets::{arq_wallet_placeholders as placeholder, AssetCallFilter},
    denomination::{CGT, EXISTENTIAL_DEPOSIT},
    AccountId, ArqWallet, Balance, Balances, Drc369, Runtime, RuntimeCall, RuntimeOrigin, System,
    VERSION,
};
use frame_support::{
    assert_ok,
    traits::{fungible::Inspect, Contains, Get},
};
use pallet_drc369::{CommitId, ContentRef, HashAlgo};
use polkadot_sdk::*;
use sp_core::H256;
use sp_runtime::{traits::Dispatchable, BuildStorage};

fn account(seed: u8) -> AccountId {
    sp_runtime::AccountId32::new([seed; 32])
}

fn chain_with(endowments: Vec<(AccountId, Balance)>) -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();
    pallet_balances::GenesisConfig::<Runtime> {
        balances: endowments,
        ..Default::default()
    }
    .assimilate_storage(&mut storage)
    .unwrap();
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn policy() -> pallet_arq_wallet::PolicyOf<Runtime> {
    pallet_arq_wallet::Policy {
        max_payout: 10 * CGT,
        epoch_blocks: 14_400,
        epoch_budget: 1_000 * CGT,
        per_recipient_per_epoch: 50 * CGT,
        rule_versions: vec![b"flux@1".to_vec().try_into().unwrap()]
            .try_into()
            .unwrap(),
        accrual_expiry: 14_400,
        loosen_delay: placeholder::MIN_LOOSEN_DELAY,
    }
}

#[test]
fn the_arq_wallet_is_pallet_eleven_at_spec_version_seven() {
    assert_eq!(VERSION.spec_version, 7);
    assert_eq!(VERSION.transaction_version, 2);
    let call = RuntimeCall::ArqWallet(pallet_arq_wallet::Call::set_paused {
        collection: 0,
        item: 0,
        paused: true,
    });
    assert_eq!(codec::Encode::encode(&call)[0], 11);
    assert!(AssetCallFilter::contains(&call));
}

#[test]
fn the_protocol_bounds_are_the_marked_placeholders() {
    // U-16 is open; these are placeholders, pinned so that changing one is a
    // visible decision rather than an edit.
    assert_eq!(
        <<Runtime as pallet_arq_wallet::Config>::MinLoosenDelay as Get<u32>>::get(),
        placeholder::MIN_LOOSEN_DELAY
    );
    assert_eq!(
        <<Runtime as pallet_arq_wallet::Config>::OutcomeWindow as Get<u32>>::get(),
        placeholder::OUTCOME_WINDOW
    );
    assert_eq!(
        <<Runtime as pallet_arq_wallet::Config>::MaxAccrualExpiry as Get<u32>>::get(),
        placeholder::MAX_ACCRUAL_EXPIRY
    );
    assert_eq!(
        <<Runtime as pallet_arq_wallet::Config>::MaxPayoutsPerEpoch as Get<u32>>::get(),
        placeholder::MAX_PAYOUTS_PER_EPOCH
    );
}

#[test]
fn a_wallet_is_created_for_a_cartridge_and_pays_a_player_in_cgt() {
    let (dev, server, player) = (account(1), account(2), account(3));
    chain_with(vec![
        (dev.clone(), 10_000 * CGT),
        (server.clone(), 1_000 * CGT),
        (player.clone(), 1_000 * CGT),
    ])
    .execute_with(|| {
        assert_ok!(Drc369::mint(
            RuntimeOrigin::signed(dev.clone()),
            ContentRef {
                algo: HashAlgo::Blake3_256,
                root: H256::repeat_byte(1),
                size: 512
            },
            Some(CommitId::Sha1([7; 20])),
            b"game".to_vec().try_into().unwrap(),
            true,
            None,
        ));
        let singles = pallet_drc369::Singles::<Runtime>::get(&dev).unwrap();
        let (c, i) = (singles.collection, singles.next_item - 1);

        // Every call goes through the runtime's dispatch, filter included.
        let create = RuntimeCall::ArqWallet(pallet_arq_wallet::Call::create {
            collection: c,
            item: i,
            policy: policy(),
            funding: 1_000 * CGT,
        });
        assert_ok!(create.dispatch(RuntimeOrigin::signed(dev.clone())));
        let authority = RuntimeCall::ArqWallet(pallet_arq_wallet::Call::set_authority {
            collection: c,
            item: i,
            authority: Some(server.clone()),
        });
        assert_ok!(authority.dispatch(RuntimeOrigin::signed(dev.clone())));
        let payout = RuntimeCall::ArqWallet(pallet_arq_wallet::Call::payout {
            collection: c,
            item: i,
            outcome: [9; 32],
            issued_at: 1,
            rule_version: b"flux@1".to_vec().try_into().unwrap(),
            to: player.clone(),
            amount: 10 * CGT,
        });
        assert_ok!(payout.dispatch(RuntimeOrigin::signed(server.clone())));

        assert_eq!(
            <Balances as Inspect<AccountId>>::balance(&player),
            1_010 * CGT
        );
        let wallet = ArqWallet::account_of(c, i);
        assert_eq!(
            <Balances as Inspect<AccountId>>::balance(&wallet),
            990 * CGT
        );
        // What it can still pay keeps the existential deposit back.
        assert_eq!(
            ArqWallet::available(c, i, 0),
            990 * CGT - EXISTENTIAL_DEPOSIT
        );
    });
}
