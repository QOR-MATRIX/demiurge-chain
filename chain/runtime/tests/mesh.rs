//! The Mesh's hosting escrow in the real runtime (ADR-095, ADR-096): what only this runtime decides. The pallet's own
//! rules are tested in `pallets/mesh/src/tests.rs`.
//!
//! - It is pallet twelve, from spec version 9, and its calls are reachable.
//! - The period, window and missed-period limit are ADR-096's, and the takedown list is the root origin's.
//! - An escrow is funded in CGT, held where no key signs, and returned in full.

use demiurge_runtime::{
    assets::AssetCallFilter,
    denomination::{CGT, EXISTENTIAL_DEPOSIT},
    mesh::{ANSWER_WINDOW, MAX_MISSED, PERIOD},
    AccountId, Balance, Balances, Mesh, Runtime, RuntimeCall, RuntimeOrigin, System, VERSION,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::{fungible::Inspect, Contains, Get},
};
use polkadot_sdk::*;
use sp_runtime::{traits::Dispatchable, BuildStorage, DispatchError};

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

#[test]
fn the_mesh_is_pallet_twelve_at_spec_version_nine_and_reachable() {
    // Named for the version that brought the Mesh (`docs/GATES.toml` requires this name). Pinned to the
    // current version, so a later one is looked at here: at 10 and 11 the Mesh is still pallet twelve,
    // the progress records (ADR-113) and rentals (ADR-119) being new calls in existing pallets only.
    assert_eq!(VERSION.spec_version, 11);
    let call = RuntimeCall::Mesh(pallet_mesh::Call::bind_node { node: [0; 32] });
    assert_eq!(codec::Encode::encode(&call)[0], 12);
    assert!(AssetCallFilter::contains(&call));
}

#[test]
fn the_period_window_and_limit_are_adr_096s() {
    assert_eq!(PERIOD, 600, "an hour in six-second blocks");
    assert_eq!(ANSWER_WINDOW, 100);
    assert_eq!(MAX_MISSED, 3);
    assert_eq!(
        <<Runtime as pallet_mesh::Config>::Period as Get<u32>>::get(),
        PERIOD
    );
    assert_eq!(
        <<Runtime as pallet_mesh::Config>::AnswerWindow as Get<u32>>::get(),
        ANSWER_WINDOW
    );
    assert_eq!(
        <<Runtime as pallet_mesh::Config>::MaxMissed as Get<u32>>::get(),
        MAX_MISSED
    );
    // The window closes inside its period, while the seed block's hash is still kept.
    const { assert!(ANSWER_WINDOW + 1 < PERIOD) };
    let kept: u32 = <Runtime as frame_system::Config>::BlockHashCount::get();
    assert!(ANSWER_WINDOW < kept, "{ANSWER_WINDOW} < {kept}");
}

#[test]
fn an_escrow_is_funded_in_cgt_held_where_no_key_signs_and_returned_in_full() {
    let uploader = account(1);
    chain_with(vec![(uploader.clone(), 1_000 * CGT)]).execute_with(|| {
        let deposit = 200 * CGT;
        assert!(deposit >= EXISTENTIAL_DEPOSIT);
        let call = RuntimeCall::Mesh(pallet_mesh::Call::open_escrow {
            root: [7; 32],
            size: 1 << 20,
            copies: 3,
            storage_price: CGT,
            serving_price_per_mib: CGT / 100,
            serving_cap_per_period: 1 << 30,
            deposit,
        });
        assert_ok!(call.dispatch(RuntimeOrigin::signed(uploader.clone())));
        let held = pallet_mesh::Pallet::<Runtime>::account_of(0);
        assert_eq!(Balances::balance(&held), deposit);
        assert_eq!(Balances::balance(&uploader), 1_000 * CGT - deposit);
        assert_ok!(Mesh::close_escrow(
            RuntimeOrigin::signed(uploader.clone()),
            0
        ));
        assert_eq!(
            Balances::balance(&uploader),
            1_000 * CGT,
            "nothing kept, nothing created"
        );
        assert_eq!(Balances::balance(&held), 0);
    });
}

#[test]
fn only_the_root_origin_keeps_the_takedown_list() {
    chain_with(vec![(account(1), 1_000 * CGT)]).execute_with(|| {
        assert_noop!(
            Mesh::set_takedown(RuntimeOrigin::signed(account(1)), [9; 32], true),
            DispatchError::BadOrigin
        );
        assert_ok!(Mesh::set_takedown(RuntimeOrigin::root(), [9; 32], true));
        assert!(pallet_mesh::Takedowns::<Runtime>::contains_key([9; 32]));
    });
}
