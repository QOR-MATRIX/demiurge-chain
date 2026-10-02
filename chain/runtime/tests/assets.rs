//! Assets in the real runtime (M4.1, M4.2): `pallet-nfts`, `pallet-drc369` and
//! `pallet-drc369-royalties` as they are configured, and the call filter that
//! makes `pallet-drc369` the only way an asset comes into existence and
//! `pallet-drc369-royalties` the only way one is sold.
//!
//! The pallet's own behaviour — authorisation, the content reference, revision,
//! permanence, events, the singles collection — is tested in
//! `pallets/drc369/src/tests.rs` against a small test runtime. What is tested
//! here is what only this runtime decides: the filter, the configured bounds and
//! their sources, the placeholder deposits and the arithmetic behind them, and
//! the runtime API.

use codec::Encode;
use demiurge_runtime::{
    assets::{deposits, AssetCallFilter, NftsDepositPerByte},
    denomination::{CGT, EXISTENTIAL_DEPOSIT},
    AccountId, Balance, Balances, Block, Drc369, Drc369Royalties, Nfts, Runtime, RuntimeCall,
    RuntimeOrigin, System, VERSION,
};
use pallet_drc369::runtime_api::runtime_decl_for_drc_369_api::Drc369ApiV1;
use pallet_drc369::{CommitId, ContentRef, HashAlgo};
use pallet_drc369_royalties::runtime_api::runtime_decl_for_drc_369_royalties_api::Drc369RoyaltiesApiV1;
use polkadot_sdk::*;

use frame_support::{
    assert_ok,
    traits::{Contains, Get},
    BoundedVec,
};
use sp_core::H256;
use sp_runtime::{traits::Dispatchable, BuildStorage, MultiAddress};

fn account(seed: u8) -> AccountId {
    sp_runtime::AccountId32::new([seed; 32])
}

fn chain_with(endowments: Vec<(AccountId, Balance)>) -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .expect("the system genesis builds");
    pallet_balances::GenesisConfig::<Runtime> {
        balances: endowments,
        ..Default::default()
    }
    .assimilate_storage(&mut storage)
    .expect("the balances genesis builds");
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn content(seed: u8) -> ContentRef {
    ContentRef {
        algo: HashAlgo::Blake3_256,
        root: H256::repeat_byte(seed),
        size: 512,
    }
}

fn refusal(call: RuntimeCall, who: &AccountId) -> sp_runtime::DispatchError {
    call.dispatch(RuntimeOrigin::signed(who.clone()))
        .expect_err("the call is refused")
        .error
}

fn name(bytes: &[u8]) -> BoundedVec<u8, <Runtime as pallet_nfts::Config>::StringLimit> {
    bytes.to_vec().try_into().expect("within the string limit")
}

fn mint(who: &AccountId, name_bytes: &[u8]) {
    assert_ok!(Drc369::mint(
        RuntimeOrigin::signed(who.clone()),
        content(1),
        Some(CommitId::Sha1([7; 20])),
        name(name_bytes),
        true,
        None,
    ));
}

fn filtered(call: RuntimeCall, who: &AccountId) -> bool {
    match call.dispatch(RuntimeOrigin::signed(who.clone())) {
        Err(e) => e.error == frame_system::Error::<Runtime>::CallFiltered.into(),
        Ok(_) => false,
    }
}

// ---------------------------------------------------------------------------
// One way in
// ---------------------------------------------------------------------------

/// Every `pallet-nfts` call that would make, destroy or change an asset behind
/// `pallet-drc369` is refused by the filter, even for the creator who owns and
/// administers the collection; moving an asset is not.
#[test]
fn only_drc369_can_create_or_change_an_asset() {
    let alice = account(1);
    let bob = account(2);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        // Alice's first mint makes her the owner, admin and issuer of
        // collection 0.
        mint(&alice, b"demo");
        let me = MultiAddress::Id(alice.clone());

        let refused: Vec<(&str, RuntimeCall)> = vec![
            (
                "create",
                RuntimeCall::Nfts(pallet_nfts::Call::create {
                    admin: me.clone(),
                    config: pallet_nfts::CollectionConfig {
                        settings: pallet_nfts::CollectionSettings::all_enabled(),
                        max_supply: None,
                        mint_settings: pallet_nfts::MintSettings::default(),
                    },
                }),
            ),
            (
                "mint",
                RuntimeCall::Nfts(pallet_nfts::Call::mint {
                    collection: 0,
                    item: 99,
                    mint_to: me.clone(),
                    witness_data: None,
                }),
            ),
            (
                "burn",
                RuntimeCall::Nfts(pallet_nfts::Call::burn {
                    collection: 0,
                    item: 0,
                }),
            ),
            (
                "set_team",
                RuntimeCall::Nfts(pallet_nfts::Call::set_team {
                    collection: 0,
                    issuer: Some(MultiAddress::Id(bob.clone())),
                    admin: Some(MultiAddress::Id(bob.clone())),
                    freezer: Some(MultiAddress::Id(bob.clone())),
                }),
            ),
            (
                "set_metadata",
                RuntimeCall::Nfts(pallet_nfts::Call::set_metadata {
                    collection: 0,
                    item: 0,
                    data: name(b"renamed"),
                }),
            ),
            (
                "lock_item_transfer",
                RuntimeCall::Nfts(pallet_nfts::Call::lock_item_transfer {
                    collection: 0,
                    item: 0,
                }),
            ),
            (
                "set_price",
                RuntimeCall::Nfts(pallet_nfts::Call::set_price {
                    collection: 0,
                    item: 0,
                    price: Some(CGT),
                    whitelisted_buyer: None,
                }),
            ),
            // `pallet-nfts`'s own sale pays no royalty (inventory F-D5), so the
            // only sale this chain settles is `pallet-drc369-royalties`'s.
            (
                "buy_item",
                RuntimeCall::Nfts(pallet_nfts::Call::buy_item {
                    collection: 0,
                    item: 0,
                    bid_price: CGT,
                }),
            ),
        ];
        for (label, call) in refused {
            assert!(filtered(call, &alice), "Nfts::{label} was not filtered");
        }
        assert_eq!(Nfts::owner(0, 99), None);
        assert_eq!(Nfts::owner(0, 0), Some(alice.clone()));

        // Moving an asset is allowed, and the ledger follows.
        let transfer = RuntimeCall::Nfts(pallet_nfts::Call::transfer {
            collection: 0,
            item: 0,
            dest: MultiAddress::Id(bob.clone()),
        });
        assert!(AssetCallFilter::contains(&transfer));
        assert_ok!(transfer.dispatch(RuntimeOrigin::signed(alice.clone())));
        assert_eq!(Nfts::owner(0, 0), Some(bob.clone()));

        // And DRC-369's own calls pass the filter, royalties and sale included.
        assert!(AssetCallFilter::contains(&RuntimeCall::Drc369(
            pallet_drc369::Call::make_permanent {
                collection: 0,
                item: 0
            }
        )));
        for call in [
            pallet_drc369_royalties::Call::set_terms {
                collection: 0,
                item: 0,
                recipients: Default::default(),
                remix: Default::default(),
            },
            pallet_drc369_royalties::Call::list {
                collection: 0,
                item: 0,
                price: CGT,
            },
            pallet_drc369_royalties::Call::unlist {
                collection: 0,
                item: 0,
            },
            pallet_drc369_royalties::Call::buy {
                collection: 0,
                item: 0,
                max_price: CGT,
            },
        ] {
            assert!(AssetCallFilter::contains(&RuntimeCall::Drc369Royalties(
                call
            )));
        }
    });
}

// ---------------------------------------------------------------------------
// A trade of several assets (M4.6)
// ---------------------------------------------------------------------------

fn transfer_of(item: u32, to: &AccountId) -> RuntimeCall {
    RuntimeCall::Nfts(pallet_nfts::Call::transfer {
        collection: 0,
        item,
        dest: MultiAddress::Id(to.clone()),
    })
}

/// Several assets move in one transaction, which is what a trade is.
#[test]
fn several_assets_move_in_one_transaction() {
    let alice = account(1);
    let bob = account(2);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        mint(&alice, b"one");
        mint(&alice, b"two");

        let trade = RuntimeCall::Utility(pallet_utility::Call::batch_all {
            calls: vec![transfer_of(0, &bob), transfer_of(1, &bob)],
        });
        assert!(AssetCallFilter::contains(&trade));
        assert_ok!(trade.dispatch(RuntimeOrigin::signed(alice.clone())));

        assert_eq!(Nfts::owner(0, 0), Some(bob.clone()));
        assert_eq!(Nfts::owner(0, 1), Some(bob.clone()));
        assert!(<Runtime as Drc369ApiV1<Block, AccountId>>::assets_of(alice).is_empty());
        assert_eq!(
            <Runtime as Drc369ApiV1<Block, AccountId>>::assets_of(bob).len(),
            2
        );
    });
}

/// A trade that cannot finish moves nothing. This is the reason the pallet is
/// mounted at all: `batch` carries on past a failure, so a trade built on it
/// would hand over some assets and keep the rest, with nothing to undo it.
#[test]
fn a_trade_that_cannot_finish_moves_nothing() {
    let alice = account(1);
    let bob = account(2);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        mint(&alice, b"one");
        mint(&alice, b"two");

        // The second asset of this trade does not exist.
        let trade = RuntimeCall::Utility(pallet_utility::Call::batch_all {
            calls: vec![transfer_of(0, &bob), transfer_of(9, &bob)],
        });
        assert!(trade
            .dispatch(RuntimeOrigin::signed(alice.clone()))
            .is_err());

        assert_eq!(Nfts::owner(0, 0), Some(alice.clone()));
        assert_eq!(Nfts::owner(0, 1), Some(alice.clone()));
        assert_eq!(
            <Runtime as Drc369ApiV1<Block, AccountId>>::assets_of(bob).len(),
            0
        );
    });
}

/// A batch is not a second way in. Every call the filter refuses on its own is
/// refused inside a batch, because a signed origin's inner calls are dispatched
/// through this runtime's own filter.
#[test]
fn a_batch_cannot_smuggle_a_call_the_filter_refuses() {
    let alice = account(1);
    let bob = account(2);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        mint(&alice, b"one");

        let smuggled = RuntimeCall::Utility(pallet_utility::Call::batch_all {
            calls: vec![
                // Allowed on its own.
                transfer_of(0, &bob),
                // Never allowed: it would make an item with no DRC-369 record.
                RuntimeCall::Nfts(pallet_nfts::Call::mint {
                    collection: 0,
                    item: 99,
                    mint_to: MultiAddress::Id(alice.clone()),
                    witness_data: None,
                }),
            ],
        });
        // The batch itself passes the filter; its contents are what is refused.
        assert!(AssetCallFilter::contains(&smuggled));
        let refusal = smuggled
            .dispatch(RuntimeOrigin::signed(alice.clone()))
            .expect_err("a batch carrying a filtered call fails");
        assert_eq!(
            refusal.error,
            frame_system::Error::<Runtime>::CallFiltered.into()
        );

        // Nothing was made, and the allowed call before it was rolled back.
        assert_eq!(Nfts::owner(0, 99), None);
        assert_eq!(Nfts::owner(0, 0), Some(alice));
    });
}

/// Only the atomic batch is reachable. `batch` and `force_batch` carry on past
/// a failure; `as_derivative` acts from an account derived from the signer;
/// the rest are root-only. Each is refused by the filter.
#[test]
fn only_the_atomic_batch_of_pallet_utility_is_reachable() {
    let alice = account(1);
    let bob = account(2);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        mint(&alice, b"one");

        let refused: Vec<(&str, RuntimeCall)> = vec![
            (
                "batch",
                RuntimeCall::Utility(pallet_utility::Call::batch {
                    calls: vec![transfer_of(0, &bob)],
                }),
            ),
            (
                "force_batch",
                RuntimeCall::Utility(pallet_utility::Call::force_batch {
                    calls: vec![transfer_of(0, &bob)],
                }),
            ),
            (
                "as_derivative",
                RuntimeCall::Utility(pallet_utility::Call::as_derivative {
                    index: 0,
                    call: Box::new(transfer_of(0, &bob)),
                }),
            ),
            (
                "with_weight",
                RuntimeCall::Utility(pallet_utility::Call::with_weight {
                    call: Box::new(transfer_of(0, &bob)),
                    weight: Default::default(),
                }),
            ),
        ];

        for (label, call) in refused {
            assert!(!AssetCallFilter::contains(&call), "Utility::{label} passed");
            assert!(filtered(call, &alice), "Utility::{label} was not filtered");
        }
        assert_eq!(Nfts::owner(0, 0), Some(alice));
    });
}

// ---------------------------------------------------------------------------
// The configured values, and where they came from
// ---------------------------------------------------------------------------

/// The bounds are the ones `assets.rs` gives a source for: ADR-047 for the
/// strings, Asset Hub Westend at `polkadot-stable2606-1` for the rest. Changing
/// one without changing its recorded source fails here.
#[test]
fn the_asset_bounds_are_the_ones_recorded_with_a_source() {
    type N = <Runtime as pallet_nfts::Config>::StringLimit;
    assert_eq!(<N as Get<u32>>::get(), 256);
    assert_eq!(
        <<Runtime as pallet_nfts::Config>::KeyLimit as Get<u32>>::get(),
        64
    );
    assert_eq!(
        <<Runtime as pallet_nfts::Config>::ValueLimit as Get<u32>>::get(),
        256
    );
    assert_eq!(
        <<Runtime as pallet_nfts::Config>::ApprovalsLimit as Get<u32>>::get(),
        20
    );
    assert_eq!(
        <<Runtime as pallet_nfts::Config>::ItemAttributesApprovalsLimit as Get<u32>>::get(),
        30
    );
    assert_eq!(
        <<Runtime as pallet_nfts::Config>::MaxTips as Get<u32>>::get(),
        10
    );
    assert_eq!(
        <<Runtime as pallet_nfts::Config>::MaxAttributesPerCall as Get<u32>>::get(),
        10
    );
    // Twelve thirty-day months of six-second blocks.
    assert_eq!(
        <<Runtime as pallet_nfts::Config>::MaxDeadlineDuration as Get<u32>>::get(),
        12 * 30 * 14_400
    );
    // ADR-047 decision 13 row 7; the recipient bound confirmed by ADR-057.
    assert_eq!(
        <<Runtime as pallet_drc369::Config>::MaxRemixDepth as Get<u8>>::get(),
        16
    );
    assert_eq!(
        <<Runtime as pallet_drc369_royalties::Config>::MaxRoyaltyRecipients as Get<u32>>::get(),
        8
    );
    // ADR-047 decision 13 row 7 again: how deep an asset nests, which is also
    // the most reads the cycle check makes, and how many assets one may hold.
    assert_eq!(
        <<Runtime as pallet_drc369::Config>::MaxNestingDepth as Get<u8>>::get(),
        8
    );
    assert_eq!(
        <<Runtime as pallet_drc369::Config>::MaxChildren as Get<u32>>::get(),
        64
    );
}

/// The deposits are placeholders (U-14), derived by ADR-030's arithmetic from
/// the existential deposit: an entry costs one existential deposit, a byte a
/// hundred-and-sixtieth of one.
#[test]
fn the_placeholder_deposits_are_derived_from_the_existential_deposit() {
    assert_eq!(deposits::PER_ENTRY, EXISTENTIAL_DEPOSIT);
    assert_eq!(deposits::COLLECTION, 5 * EXISTENTIAL_DEPOSIT);
    assert_eq!(deposits::ITEM, 4 * EXISTENTIAL_DEPOSIT);
    assert_eq!(deposits::METADATA_BASE, EXISTENTIAL_DEPOSIT);
    assert_eq!(deposits::ATTRIBUTE_BASE, EXISTENTIAL_DEPOSIT);
    // Exact: 10^20 / 160 leaves no remainder, so no rounding happened.
    assert_eq!(deposits::per_byte() * 160, EXISTENTIAL_DEPOSIT);
    assert_eq!(deposits::per_byte(), 625_000_000_000_000_000);

    assert_eq!(
        <<Runtime as pallet_nfts::Config>::CollectionDeposit as Get<Balance>>::get(),
        deposits::COLLECTION
    );
    assert_eq!(
        <<Runtime as pallet_nfts::Config>::ItemDeposit as Get<Balance>>::get(),
        deposits::ITEM
    );
    assert_eq!(NftsDepositPerByte::get(), deposits::per_byte());
}

/// ADR-035: every pallet that moves CGT pins the largest intermediate its money
/// paths form. A mint's is `pallet-nfts`'s metadata deposit, `DepositPerByte`
/// times the longest name, which it computes with a saturating multiply: a
/// result that had saturated would read as `u128::MAX` rather than fail. This
/// mints with the longest name and checks the held amount exactly.
#[test]
fn the_largest_deposit_a_mint_holds_is_exact() {
    let alice = account(1);
    chain_with(vec![(alice.clone(), 10_000 * CGT)]).execute_with(|| {
        let longest = [b'x'; 256];

        // The intermediate, checked rather than saturated: 256 bytes at
        // 0.625 CGT is 160 CGT, some eighteen orders of magnitude below the
        // ceiling.
        let largest_intermediate = deposits::per_byte().checked_mul(256).expect("no overflow");
        assert_eq!(largest_intermediate, 160 * CGT);

        mint(&alice, &longest);
        let expected =
            deposits::COLLECTION + deposits::ITEM + deposits::METADATA_BASE + largest_intermediate;
        assert_eq!(expected, 1_160 * CGT);
        assert_eq!(Balances::reserved_balance(&alice), expected);
    });
}

/// A creator's first mint holds the collection, the item and the name; a
/// second holds only the item and its name.
#[test]
fn a_first_mint_holds_the_collection_and_later_mints_do_not() {
    let alice = account(1);
    chain_with(vec![(alice.clone(), 10_000 * CGT)]).execute_with(|| {
        let name_cost = 4 * deposits::per_byte();
        mint(&alice, b"demo");
        let first = deposits::COLLECTION + deposits::ITEM + deposits::METADATA_BASE + name_cost;
        assert_eq!(Balances::reserved_balance(&alice), first);
        // 1,002.5 CGT.
        assert_eq!(first, 1_002 * CGT + CGT / 2);

        mint(&alice, b"demo");
        assert_eq!(
            Balances::reserved_balance(&alice),
            first + deposits::ITEM + deposits::METADATA_BASE + name_cost
        );
    });
}

// ---------------------------------------------------------------------------
// Reading assets back
// ---------------------------------------------------------------------------

/// The runtime API answers from chain state: what an account holds, and one
/// asset's record.
#[test]
fn the_runtime_api_lists_what_an_account_holds() {
    let alice = account(1);
    let bob = account(2);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        assert!(<Runtime as Drc369ApiV1<Block, AccountId>>::assets_of(alice.clone()).is_empty());

        mint(&alice, b"demo");
        let held = <Runtime as Drc369ApiV1<Block, AccountId>>::assets_of(alice.clone());
        assert_eq!(held.len(), 1);
        assert_eq!((held[0].collection, held[0].item), (0, 0));
        assert_eq!(held[0].name, b"demo".to_vec());
        assert_eq!(held[0].asset.origin, content(1));
        assert_eq!(held[0].asset.commit, Some(CommitId::Sha1([7; 20])));
        assert!(held[0].asset.revisable);

        assert_eq!(
            <Runtime as Drc369ApiV1<Block, AccountId>>::asset(0, 0),
            Some(held[0].asset)
        );
        assert_eq!(
            <Runtime as Drc369ApiV1<Block, AccountId>>::asset(0, 1),
            None
        );
        assert!(<Runtime as Drc369ApiV1<Block, AccountId>>::assets_of(bob).is_empty());
    });
}

// ---------------------------------------------------------------------------
// A sale settled in CGT (M4.2, ADR-061)
// ---------------------------------------------------------------------------

/// A sale in this runtime pays its royalties and its seller in CGT and hands the
/// asset over, at this runtime's existential deposit and denomination. The
/// arithmetic itself is tested in `pallets/drc369-royalties/src/tests.rs`.
#[test]
fn a_sale_pays_royalties_in_cgt_and_hands_the_asset_over() {
    let alice = account(1);
    let bob = account(2);
    let dave = account(4);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
        (dave.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        mint(&alice, b"song");
        let recipients: BoundedVec<_, _> =
            vec![(dave.clone(), sp_runtime::Permill::from_percent(10))]
                .try_into()
                .unwrap();
        assert_ok!(Drc369Royalties::set_terms(
            RuntimeOrigin::signed(alice.clone()),
            0,
            0,
            recipients,
            sp_runtime::Permill::zero(),
        ));
        assert_ok!(Drc369Royalties::list(
            RuntimeOrigin::signed(alice.clone()),
            0,
            0,
            1_000 * CGT
        ));

        let alice_before = Balances::free_balance(&alice);
        let dave_before = Balances::free_balance(&dave);
        let bob_before = Balances::free_balance(&bob);
        assert_ok!(Drc369Royalties::buy(
            RuntimeOrigin::signed(bob.clone()),
            0,
            0,
            1_000 * CGT
        ));

        assert_eq!(Balances::free_balance(&dave) - dave_before, 100 * CGT);
        assert_eq!(Balances::free_balance(&alice) - alice_before, 900 * CGT);
        assert_eq!(bob_before - Balances::free_balance(&bob), 1_000 * CGT);
        assert_eq!(Nfts::owner(0, 0), Some(bob.clone()));
    });
}

// ---------------------------------------------------------------------------
// Nesting (M4.2, M4.5, requirement R-2)
// ---------------------------------------------------------------------------

fn nest_of(item: u32, parent: u32) -> RuntimeCall {
    RuntimeCall::Drc369(pallet_drc369::Call::nest {
        collection: 0,
        item,
        parent: (0, parent),
    })
}

fn unnest_of(item: u32) -> RuntimeCall {
    RuntimeCall::Drc369(pallet_drc369::Call::unnest {
        collection: 0,
        item,
    })
}

/// In this runtime `pallet-drc369` is `pallet-nfts`'s `Locker` (ADR-025), so the
/// transfer the call filter lets through cannot take a nested asset out of its
/// parent, or move the parent from under it, for the owner or for an account the
/// owner approved.
#[test]
fn a_nested_asset_cannot_be_transferred_in_this_runtime() {
    let alice = account(1);
    let bob = account(2);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        mint(&alice, b"chest");
        mint(&alice, b"sword");
        assert_ok!(Nfts::approve_transfer(
            RuntimeOrigin::signed(alice.clone()),
            0,
            1,
            MultiAddress::Id(bob.clone()),
            None
        ));

        // Nesting and un-nesting are DRC-369's own calls, and pass the filter.
        assert!(AssetCallFilter::contains(&nest_of(1, 0)));
        assert!(AssetCallFilter::contains(&unnest_of(1)));
        assert_ok!(nest_of(1, 0).dispatch(RuntimeOrigin::signed(alice.clone())));

        let locked: sp_runtime::DispatchError = pallet_nfts::Error::<Runtime>::ItemLocked.into();
        assert_eq!(refusal(transfer_of(1, &bob), &alice), locked);
        assert_eq!(refusal(transfer_of(1, &bob), &bob), locked);
        assert_eq!(refusal(transfer_of(0, &bob), &alice), locked);
        assert_eq!(Nfts::owner(0, 0), Some(alice.clone()));
        assert_eq!(Nfts::owner(0, 1), Some(alice.clone()));

        // A cycle is refused here as it is in the pallet's own tests (R-2).
        assert_eq!(
            refusal(nest_of(0, 1), &alice),
            pallet_drc369::Error::<Runtime>::NestingCycle.into()
        );

        assert_ok!(unnest_of(1).dispatch(RuntimeOrigin::signed(alice.clone())));
        assert_ok!(transfer_of(1, &bob).dispatch(RuntimeOrigin::signed(alice.clone())));
        assert_eq!(Nfts::owner(0, 1), Some(bob));
    });
}

/// A batch is not a way round nesting. A nested asset inside `batch_all` is
/// refused by the ledger itself and takes the whole batch with it; the calls
/// that could change an asset behind DRC-369 stay filtered inside one; and the
/// one honest route, taking the asset out first, is the owner's alone.
#[test]
fn a_batch_cannot_move_a_nested_asset() {
    let alice = account(1);
    let bob = account(2);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        mint(&alice, b"chest");
        mint(&alice, b"sword");
        mint(&alice, b"loose");
        assert_ok!(nest_of(1, 0).dispatch(RuntimeOrigin::signed(alice.clone())));

        let locked: sp_runtime::DispatchError = pallet_nfts::Error::<Runtime>::ItemLocked.into();
        let batch = |calls: Vec<RuntimeCall>| {
            RuntimeCall::Utility(pallet_utility::Call::batch_all { calls })
        };

        // A free asset first, then the nested one: neither moves.
        assert_eq!(
            refusal(
                batch(vec![transfer_of(2, &bob), transfer_of(1, &bob)]),
                &alice
            ),
            locked
        );
        // Nor does the parent. And a batch inside a batch is not a deeper way
        // in: `pallet-utility` refuses to nest `batch_all` at all.
        assert_eq!(refusal(batch(vec![transfer_of(0, &bob)]), &alice), locked);
        assert_eq!(
            refusal(batch(vec![batch(vec![transfer_of(1, &bob)])]), &alice),
            frame_system::Error::<Runtime>::CallFiltered.into()
        );
        // Burning it out of the tree is filtered, as it always was.
        assert_eq!(
            refusal(
                batch(vec![RuntimeCall::Nfts(pallet_nfts::Call::burn {
                    collection: 0,
                    item: 1,
                })]),
                &alice
            ),
            frame_system::Error::<Runtime>::CallFiltered.into()
        );
        // Bob cannot take it out for himself in a batch of his own.
        assert_eq!(
            refusal(batch(vec![unnest_of(1), transfer_of(1, &bob)]), &bob),
            pallet_drc369::Error::<Runtime>::NotOwner.into()
        );
        for item in 0..3 {
            assert_eq!(Nfts::owner(0, item), Some(alice.clone()));
        }
        assert_eq!(Drc369::parent_of(0, 1), Some((0, 0)));

        // The owner taking it out and then moving it is two acts in one
        // signature, and is allowed: the asset is no longer nested when it moves.
        assert_ok!(batch(vec![unnest_of(1), transfer_of(1, &bob)])
            .dispatch(RuntimeOrigin::signed(alice.clone())));
        assert_eq!(Nfts::owner(0, 1), Some(bob));
        assert_eq!(Drc369::parent_of(0, 1), None);
    });
}

/// A sale ends with a transfer, and the ledger refuses it for a nested asset, so
/// the sale is refused whole: no CGT moves and the listing is left as it was.
#[test]
fn a_nested_asset_cannot_be_sold() {
    let alice = account(1);
    let bob = account(2);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        mint(&alice, b"chest");
        mint(&alice, b"sword");
        for item in 0..2 {
            assert_ok!(Drc369Royalties::list(
                RuntimeOrigin::signed(alice.clone()),
                0,
                item,
                1_000 * CGT
            ));
        }
        assert_ok!(nest_of(1, 0).dispatch(RuntimeOrigin::signed(alice.clone())));

        let alice_before = Balances::free_balance(&alice);
        let bob_before = Balances::free_balance(&bob);
        for item in 0..2 {
            let buy = RuntimeCall::Drc369Royalties(pallet_drc369_royalties::Call::buy {
                collection: 0,
                item,
                max_price: 1_000 * CGT,
            });
            assert_eq!(
                refusal(buy, &bob),
                pallet_nfts::Error::<Runtime>::ItemLocked.into()
            );
            assert_eq!(Nfts::owner(0, item), Some(alice.clone()));
            assert!(Drc369Royalties::listing(0, item).is_some());
        }
        assert_eq!(Balances::free_balance(&alice), alice_before);
        assert_eq!(Balances::free_balance(&bob), bob_before);

        // Taken out, the same listing sells.
        assert_ok!(unnest_of(1).dispatch(RuntimeOrigin::signed(alice.clone())));
        assert_ok!(Drc369Royalties::buy(
            RuntimeOrigin::signed(bob.clone()),
            0,
            1,
            1_000 * CGT
        ));
        assert_eq!(Nfts::owner(0, 1), Some(bob));
    });
}

// ---------------------------------------------------------------------------
// The work a buyer agreed to, and asking before paying
// ---------------------------------------------------------------------------

/// `buy_exact` is a new call beside `buy`, not a change to it: `buy` keeps its
/// index and its bytes, so a client built against the earlier runtime still
/// sends a call this one accepts, and `transaction_version` has no reason to
/// move. Through the filter and the dispatcher, `buy_exact` refuses an asset
/// revised after the buyer looked, and nothing moves.
#[test]
fn buy_exact_is_a_new_call_and_buy_keeps_its_encoding() {
    let buy = RuntimeCall::Drc369Royalties(pallet_drc369_royalties::Call::buy {
        collection: 1,
        item: 2,
        max_price: 3,
    });
    // Pallet 10, call 3, then two `u32`s and a `u128`, little-endian.
    let mut bytes = vec![10u8, 3];
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(2u32.to_le_bytes());
    bytes.extend(3u128.to_le_bytes());
    assert_eq!(buy.encode(), bytes);
    assert_eq!(VERSION.transaction_version, 2);

    let exact = |item: u32, max_price: Balance, seed: u8| {
        RuntimeCall::Drc369Royalties(pallet_drc369_royalties::Call::buy_exact {
            collection: 0,
            item,
            max_price,
            content: content(seed),
        })
    };
    // Call 4: the same arguments, then the 41-byte content reference.
    let mut bytes = vec![10u8, 4];
    bytes.extend(0u32.to_le_bytes());
    bytes.extend(2u32.to_le_bytes());
    bytes.extend(3u128.to_le_bytes());
    bytes.push(0);
    bytes.extend([1u8; 32]);
    bytes.extend(512u64.to_le_bytes());
    assert_eq!(exact(2, 3, 1).encode(), bytes);
    assert!(AssetCallFilter::contains(&exact(2, 3, 1)));

    let alice = account(1);
    let bob = account(2);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        mint(&alice, b"song");
        assert_ok!(Drc369Royalties::list(
            RuntimeOrigin::signed(alice.clone()),
            0,
            0,
            1_000 * CGT
        ));
        assert_ok!(Drc369::revise(
            RuntimeOrigin::signed(alice.clone()),
            0,
            0,
            content(2),
            None
        ));

        let alice_before = Balances::free_balance(&alice);
        let bob_before = Balances::free_balance(&bob);
        assert_eq!(
            refusal(exact(0, 1_000 * CGT, 1), &bob),
            pallet_drc369_royalties::Error::<Runtime>::ContentChanged.into()
        );
        assert_eq!(Balances::free_balance(&alice), alice_before);
        assert_eq!(Balances::free_balance(&bob), bob_before);
        assert_eq!(Nfts::owner(0, 0), Some(alice.clone()));

        assert_ok!(exact(0, 1_000 * CGT, 2).dispatch(RuntimeOrigin::signed(bob.clone())));
        assert_eq!(Balances::free_balance(&alice) - alice_before, 1_000 * CGT);
        assert_eq!(Nfts::owner(0, 0), Some(bob));
    });
}

/// The runtime API answers what a sale would pay and whether it could settle,
/// at this runtime's existential deposit and with this runtime's `Locker`: a
/// part below the existential deposit for an account that does not exist, and
/// an asset held in place by nesting, are both reported, and asking moves
/// nothing.
#[test]
fn the_runtime_api_previews_what_a_sale_would_pay() {
    let alice = account(1);
    let bob = account(2);
    let dave = account(4);
    // Holds nothing, so has no account.
    let eve = account(5);
    chain_with(vec![
        (alice.clone(), 10_000 * CGT),
        (bob.clone(), 10_000 * CGT),
        (dave.clone(), 10_000 * CGT),
    ])
    .execute_with(|| {
        let preview = |item: u32, price: Balance, buyer: Option<&AccountId>| {
            <Runtime as Drc369RoyaltiesApiV1<Block, AccountId>>::sale_preview(
                0,
                item,
                price,
                buyer.cloned(),
            )
        };
        assert_eq!(preview(0, 1_000 * CGT, None), None);

        mint(&alice, b"song");
        let terms = |who: &AccountId| -> BoundedVec<_, _> {
            vec![(who.clone(), sp_runtime::Permill::from_percent(10))]
                .try_into()
                .unwrap()
        };
        assert_ok!(Drc369Royalties::set_terms(
            RuntimeOrigin::signed(alice.clone()),
            0,
            0,
            terms(&dave),
            sp_runtime::Permill::zero(),
        ));

        let balances = || [&alice, &bob, &dave].map(Balances::free_balance);
        let before = balances();
        for buyer in [None, Some(&bob)] {
            let answer = preview(0, 1_000 * CGT, buyer).expect("the asset exists");
            assert_eq!(answer.source, None);
            assert!(answer.remix.is_empty());
            assert_eq!(answer.royalties, vec![(dave.clone(), 100 * CGT)]);
            assert_eq!(answer.seller, alice);
            assert_eq!(answer.seller_receives, 900 * CGT);
            assert_eq!(answer.refusal, None);
        }
        assert_eq!(balances(), before);
        assert_eq!(Nfts::owner(0, 0), Some(alice.clone()));
        // The buyer named is the buyer tried: the holder cannot buy from
        // themselves, which only an answer for that account can say.
        assert_eq!(
            preview(0, 1_000 * CGT, Some(&alice)).unwrap().refusal,
            Some(pallet_drc369_royalties::Error::<Runtime>::OwnListing.into())
        );

        // Eve has no account, and 10% of 500 CGT is 50 CGT, half this
        // runtime's existential deposit: no buyer can make that sale settle.
        assert_ok!(Drc369Royalties::set_terms(
            RuntimeOrigin::signed(alice.clone()),
            0,
            0,
            terms(&eve),
            sp_runtime::Permill::zero(),
        ));
        assert_eq!(EXISTENTIAL_DEPOSIT, 100 * CGT);
        let unreceivable: sp_runtime::DispatchError =
            pallet_drc369_royalties::Error::<Runtime>::PaymentCannotBeReceived.into();
        for buyer in [None, Some(&bob)] {
            let answer = preview(0, 500 * CGT, buyer).expect("the asset exists");
            assert_eq!(answer.royalties, vec![(eve.clone(), 50 * CGT)]);
            assert_eq!(answer.refusal, Some(unreceivable));
            assert_eq!(preview(0, 1_000 * CGT, buyer).unwrap().refusal, None);
        }

        // A nested asset and the asset holding it: the ledger would refuse the
        // handover, and the preview says so before anyone pays.
        mint(&alice, b"chest");
        mint(&alice, b"sword");
        assert_eq!(preview(2, 1_000 * CGT, None).unwrap().refusal, None);
        assert_ok!(nest_of(2, 1).dispatch(RuntimeOrigin::signed(alice.clone())));
        let locked: sp_runtime::DispatchError = pallet_nfts::Error::<Runtime>::ItemLocked.into();
        // The two mints held deposits; the previews after them move nothing.
        let before = balances();
        for item in [1, 2] {
            for buyer in [None, Some(&bob)] {
                let answer = preview(item, 1_000 * CGT, buyer).expect("the asset exists");
                assert_eq!(answer.seller_receives, 1_000 * CGT);
                assert_eq!(answer.refusal, Some(locked));
            }
        }
        assert_eq!(balances(), before);
    });
}
