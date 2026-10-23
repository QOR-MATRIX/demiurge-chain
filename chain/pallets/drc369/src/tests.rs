//! Tests for M4.1. Each of the five the roadmap names was shown to fail against
//! a planted fault before it was trusted; the commit that added them records how.
//!
//! The nesting tests are M4.2's nesting and M4.5's requirement R-2. Each of them
//! was shown to fail against a planted fault too. The progress tests at the end are
//! M4.2 and M4.5's state and XP (ADR-113).

use crate::{mock::*, *};
use codec::{DecodeAll, Encode, MaxEncodedLen};
use frame_support::{assert_noop, assert_ok, traits::Currency, BoundedVec};
use sp_runtime::{DispatchError, DispatchResult};

fn content(seed: u8) -> ContentRef {
    ContentRef {
        algo: HashAlgo::Blake3_256,
        root: H256::repeat_byte(seed),
        size: 1_000 + seed as u64,
    }
}

fn commit(seed: u8) -> Option<CommitId> {
    Some(CommitId::Sha1([seed; 20]))
}

fn name(text: &str) -> BoundedVec<u8, <Test as pallet_nfts::Config>::StringLimit> {
    text.as_bytes().to_vec().try_into().unwrap()
}

fn signed(who: u8) -> RuntimeOrigin {
    RuntimeOrigin::signed(account(who))
}

fn mint(who: u8, seed: u8) -> DispatchResult {
    Drc369::mint(
        signed(who),
        content(seed),
        commit(seed),
        name("a song"),
        true,
        None,
    )
}

fn remix(who: u8, seed: u8, source: (CollectionId, ItemId)) -> DispatchResult {
    Drc369::mint(
        signed(who),
        content(seed),
        commit(seed),
        name("a remix"),
        true,
        Some(source),
    )
}

// ---------------------------------------------------------------------------
// A mint must be authorised (requirement 7)
// ---------------------------------------------------------------------------

#[test]
fn an_unsigned_or_root_mint_is_refused() {
    new_test_ext().execute_with(|| {
        for origin in [RuntimeOrigin::none(), RuntimeOrigin::root()] {
            assert_noop!(
                Drc369::mint(origin, content(1), commit(1), name("x"), true, None),
                DispatchError::BadOrigin
            );
        }
        assert_eq!(Singles::<Test>::iter().count(), 0);
        assert_eq!(Assets::<Test>::iter().count(), 0);
        assert_eq!(nfts_collections_created(), 0);
    });
}

#[test]
fn a_mint_goes_to_the_signer_in_the_signers_own_collection() {
    new_test_ext().execute_with(|| {
        assert_ok!(mint(ALICE, 1));
        assert_ok!(mint(BOB, 2));

        let alice = Singles::<Test>::get(account(ALICE)).unwrap().collection;
        let bob = Singles::<Test>::get(account(BOB)).unwrap().collection;
        assert_ne!(alice, bob);

        assert_eq!(Nfts::owner(alice, 0), Some(account(ALICE)));
        assert_eq!(Nfts::collection_owner(alice), Some(account(ALICE)));
        assert_eq!(Nfts::owner(bob, 0), Some(account(BOB)));
        assert_eq!(Nfts::collection_owner(bob), Some(account(BOB)));

        // Nor can one touch the other's asset.
        assert_noop!(
            Drc369::revise(signed(BOB), alice, 0, content(3), commit(3)),
            Error::<Test>::NotOwner
        );
        assert_noop!(
            Drc369::make_permanent(signed(BOB), alice, 0),
            Error::<Test>::NotOwner
        );
    });
}

// ---------------------------------------------------------------------------
// The content reference (ADR-047, decisions 1 to 3)
// ---------------------------------------------------------------------------

#[test]
fn the_content_reference_is_41_bytes() {
    let reference = content(7);
    assert_eq!(reference.encode().len(), 41);
    assert_eq!(ContentRef::max_encoded_len(), 41);
    // Tag first, then the root, then the size, little-endian.
    let bytes = reference.encode();
    assert_eq!(bytes[0], 0);
    assert_eq!(&bytes[1..33], &[7u8; 32]);
    assert_eq!(&bytes[33..], &1_007u64.to_le_bytes());
}

#[test]
fn a_malformed_reference_is_refused() {
    let good = content(7).encode();
    assert_eq!(ContentRef::decode_all(&mut &good[..]), Ok(content(7)));

    // An algorithm tag that names nothing does not decode.
    for tag in [3u8, 0x7f, 0xff] {
        let mut bad = good.clone();
        bad[0] = tag;
        assert!(
            ContentRef::decode_all(&mut &bad[..]).is_err(),
            "tag {tag} decoded"
        );
    }

    // Nor does a reference of the wrong length, short or long.
    assert!(ContentRef::decode_all(&mut &good[..40]).is_err());
    let mut long = good.clone();
    long.push(0);
    assert!(ContentRef::decode_all(&mut &long[..]).is_err());

    // The same holds for the whole call, which is how the runtime meets it: a
    // mint carrying an unknown tag is refused before it is dispatched.
    let call = RuntimeCall::Drc369(Call::mint {
        content: content(7),
        commit: commit(7),
        name: name("x"),
        revisable: true,
        derived_from: None,
    });
    let encoded = call.encode();
    assert!(RuntimeCall::decode_all(&mut &encoded[..]).is_ok());
    // Byte 0 is the pallet, byte 1 the call, and the reference starts at 2.
    let mut unknown_tag = encoded.clone();
    unknown_tag[2] = 3;
    assert!(RuntimeCall::decode_all(&mut &unknown_tag[..]).is_err());
    assert!(RuntimeCall::decode_all(&mut &encoded[..encoded.len() - 1]).is_err());

    // Tags that name an algorithm a mint does not accept yet, and a manifest of
    // no bytes, are refused when dispatched, by mint and by revise alike.
    new_test_ext().execute_with(|| {
        for algo in [HashAlgo::Sha2_256, HashAlgo::Blake2_256] {
            let reference = ContentRef { algo, ..content(1) };
            assert_noop!(
                Drc369::mint(signed(ALICE), reference, commit(1), name("x"), true, None),
                Error::<Test>::UnsupportedAlgorithm
            );
        }
        let empty = ContentRef {
            size: 0,
            ..content(1)
        };
        assert_noop!(
            Drc369::mint(signed(ALICE), empty, commit(1), name("x"), true, None),
            Error::<Test>::EmptyContent
        );

        assert_ok!(mint(ALICE, 1));
        let sha2 = ContentRef {
            algo: HashAlgo::Sha2_256,
            ..content(2)
        };
        assert_noop!(
            Drc369::revise(signed(ALICE), 0, 0, sha2, commit(2)),
            Error::<Test>::UnsupportedAlgorithm
        );
    });
}

// ---------------------------------------------------------------------------
// Revision and the one-way switch (ADR-047, decision 10)
// ---------------------------------------------------------------------------

#[test]
fn make_permanent_is_one_way_and_revise_is_refused_after() {
    new_test_ext().execute_with(|| {
        assert_ok!(mint(ALICE, 1));
        assert_ok!(Drc369::revise(signed(ALICE), 0, 0, content(2), commit(2)));
        assert_ok!(Drc369::make_permanent(signed(ALICE), 0, 0));

        let asset = Assets::<Test>::get(0, 0).unwrap();
        assert!(!asset.revisable);
        assert_eq!(asset.origin, content(1));
        assert_eq!(asset.current, content(2));

        assert_noop!(
            Drc369::revise(signed(ALICE), 0, 0, content(3), commit(3)),
            Error::<Test>::Permanent
        );
        assert_noop!(
            Drc369::make_permanent(signed(ALICE), 0, 0),
            Error::<Test>::AlreadyPermanent
        );

        // Permanence travels with the asset: its next owner cannot revise it.
        assert_ok!(Nfts::transfer(signed(ALICE), 0, 0, account(BOB)));
        assert_noop!(
            Drc369::revise(signed(BOB), 0, 0, content(3), commit(3)),
            Error::<Test>::Permanent
        );

        // An asset minted permanent is permanent from the start.
        assert_ok!(Drc369::mint(
            signed(ALICE),
            content(4),
            commit(4),
            name("final"),
            false,
            None
        ));
        assert_noop!(
            Drc369::revise(signed(ALICE), 0, 1, content(5), commit(5)),
            Error::<Test>::Permanent
        );
    });

    // And nothing turns it back: the pallet has no call that could. The progress
    // calls (ADR-113) write `HomeGame`, `Writers`, `WriterCount` and `Records`,
    // never an `Assets` record, so none of them can touch `revisable`.
    let calls: Vec<&str> =
        <Call<Test> as frame_support::traits::GetCallName>::get_call_names().to_vec();
    assert_eq!(
        calls,
        vec![
            "mint",
            "revise",
            "make_permanent",
            "nest",
            "unnest",
            "set_home_game",
            "authorise_writer",
            "revoke_writer",
            "record_progress",
            "correct_progress",
        ]
    );
}

#[test]
fn a_revision_needs_the_owner_a_known_asset_and_a_change() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Drc369::revise(signed(ALICE), 0, 0, content(2), commit(2)),
            Error::<Test>::UnknownAsset
        );
        assert_noop!(
            Drc369::make_permanent(signed(ALICE), 0, 0),
            Error::<Test>::UnknownAsset
        );

        assert_ok!(mint(ALICE, 1));
        assert_noop!(
            Drc369::revise(signed(ALICE), 0, 0, content(1), commit(1)),
            Error::<Test>::Unchanged
        );
        // The same files from a later commit is a change: the pin moves.
        assert_ok!(Drc369::revise(signed(ALICE), 0, 0, content(1), commit(9)));
        assert_eq!(Assets::<Test>::get(0, 0).unwrap().commit, commit(9));

        // The owner is whoever holds it now.
        assert_ok!(Nfts::transfer(signed(ALICE), 0, 0, account(BOB)));
        assert_noop!(
            Drc369::revise(signed(ALICE), 0, 0, content(2), commit(2)),
            Error::<Test>::NotOwner
        );
        assert_ok!(Drc369::revise(signed(BOB), 0, 0, content(2), commit(2)));
    });
}

// ---------------------------------------------------------------------------
// Events (ADR-047, decision 12)
// ---------------------------------------------------------------------------

#[test]
fn events_carry_the_content_reference() {
    new_test_ext().execute_with(|| {
        assert_ok!(mint(ALICE, 1));
        assert_ok!(Drc369::revise(signed(ALICE), 0, 0, content(2), commit(2)));
        assert_ok!(Drc369::make_permanent(signed(ALICE), 0, 0));

        assert_eq!(
            events(),
            vec![
                Event::SinglesCollectionCreated {
                    creator: account(ALICE),
                    collection: 0,
                },
                Event::Minted {
                    collection: 0,
                    item: 0,
                    owner: account(ALICE),
                    by: account(ALICE),
                    origin: content(1),
                    commit: commit(1),
                    revisable: true,
                    derived_from: None,
                },
                Event::Revised {
                    collection: 0,
                    item: 0,
                    from: content(1),
                    to: content(2),
                    commit: commit(2),
                    by: account(ALICE),
                },
                Event::Locked {
                    collection: 0,
                    item: 0,
                    content: content(2),
                    by: account(ALICE),
                },
            ]
        );
    });
}

// ---------------------------------------------------------------------------
// One singles collection per creator (ADR-047, decision 13 row 12)
// ---------------------------------------------------------------------------

#[test]
fn the_singles_collection_is_created_exactly_once_per_creator() {
    new_test_ext().execute_with(|| {
        for seed in 1..=3 {
            assert_ok!(mint(ALICE, seed));
        }
        assert_eq!(
            Singles::<Test>::get(account(ALICE)),
            Some(SinglesCollection {
                collection: 0,
                next_item: 3
            })
        );
        for item in 0..3 {
            assert_eq!(Nfts::owner(0, item), Some(account(ALICE)));
        }
        assert_eq!(nfts_collections_created(), 1);

        assert_ok!(mint(BOB, 4));
        assert_eq!(Singles::<Test>::get(account(BOB)).unwrap().collection, 1);
        assert_eq!(nfts_collections_created(), 2);

        let created: Vec<_> = events()
            .into_iter()
            .filter_map(|e| match e {
                Event::SinglesCollectionCreated {
                    creator,
                    collection,
                } => Some((creator, collection)),
                _ => None,
            })
            .collect();
        assert_eq!(created, vec![(account(ALICE), 0), (account(BOB), 1)]);
    });
}

// ---------------------------------------------------------------------------
// Deposits, and a mint that cannot be paid for
// ---------------------------------------------------------------------------

#[test]
fn a_mint_holds_its_deposits_from_the_minter() {
    new_test_ext().execute_with(|| {
        // The first mint pays for the collection, the item and the name.
        assert_ok!(mint(ALICE, 1));
        let name_bytes = "a song".len() as Balance;
        let first = COLLECTION_DEPOSIT
            + ITEM_DEPOSIT
            + METADATA_DEPOSIT_BASE
            + name_bytes * DEPOSIT_PER_BYTE;
        assert_eq!(Balances::reserved_balance(account(ALICE)), first);

        // Later mints pay for the item alone, and nothing for an empty name.
        assert_ok!(Drc369::mint(
            signed(ALICE),
            content(2),
            commit(2),
            name(""),
            true,
            None
        ));
        assert_eq!(
            Balances::reserved_balance(account(ALICE)),
            first + ITEM_DEPOSIT
        );
    });
}

#[test]
fn a_mint_that_cannot_be_paid_for_moves_nothing() {
    new_test_ext().execute_with(|| {
        // Nothing at all.
        assert_noop!(
            mint(NOBODY, 1),
            pallet_balances::Error::<Test>::InsufficientBalance
        );

        // Enough for the collection but not for the item: the collection is not
        // left behind either.
        let short = account(NOBODY);
        let _ = Balances::deposit_creating(&short, COLLECTION_DEPOSIT + 50);
        assert_noop!(
            mint(NOBODY, 1),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
        assert_eq!(Balances::reserved_balance(&short), 0);
        assert_eq!(Singles::<Test>::get(&short), None);
    });
}

// ---------------------------------------------------------------------------
// Owner enumeration, read from storage (ADR-025)
// ---------------------------------------------------------------------------

#[test]
fn owner_enumeration_follows_the_ledger() {
    new_test_ext().execute_with(|| {
        assert_ok!(mint(ALICE, 1));
        assert_ok!(Drc369::mint(
            signed(ALICE),
            content(2),
            commit(2),
            name("second"),
            true,
            None
        ));
        assert_ok!(mint(BOB, 3));

        let held = |who: u8| {
            let mut assets: Vec<(CollectionId, ItemId)> = Drc369::assets_of(&account(who))
                .into_iter()
                .map(|owned| (owned.collection, owned.item))
                .collect();
            assets.sort();
            assets
        };
        assert_eq!(held(ALICE), vec![(0, 0), (0, 1)]);
        assert_eq!(held(BOB), vec![(1, 0)]);

        let second = Drc369::assets_of(&account(ALICE))
            .into_iter()
            .find(|owned| owned.item == 1)
            .unwrap();
        assert_eq!(second.name, b"second".to_vec());
        assert_eq!(second.asset.origin, content(2));
        assert_eq!(second.asset.commit, commit(2));
        assert!(second.asset.revisable);

        // A transfer through the ledger moves it in the enumeration.
        assert_ok!(Nfts::transfer(signed(ALICE), 0, 1, account(BOB)));
        assert_eq!(held(ALICE), vec![(0, 0)]);
        assert_eq!(held(BOB), vec![(0, 1), (1, 0)]);

        // An item the ledger holds without a DRC-369 record is not a DRC-369
        // asset, and is not listed.
        assert_ok!(Nfts::force_create(
            RuntimeOrigin::root(),
            account(ALICE),
            pallet_nfts::CollectionConfig {
                settings: pallet_nfts::CollectionSettings::all_enabled(),
                max_supply: None,
                mint_settings: pallet_nfts::MintSettings::default(),
            }
        ));
        assert_ok!(Nfts::force_mint(
            RuntimeOrigin::root(),
            2,
            0,
            account(ALICE),
            pallet_nfts::ItemConfig {
                settings: pallet_nfts::ItemSettings::all_enabled()
            }
        ));
        assert_eq!(Nfts::owner(2, 0), Some(account(ALICE)));
        assert_eq!(held(ALICE), vec![(0, 0)]);
    });
}

// ---------------------------------------------------------------------------
// Remix provenance (M4.2, ADR-047 decisions 7 and 11, ADR-061)
// ---------------------------------------------------------------------------

#[test]
fn a_remix_records_its_source_and_depth_and_anyone_may_declare_one() {
    new_test_ext().execute_with(|| {
        assert_ok!(mint(ALICE, 1));
        // Bob remixes Alice's work without owning it: declaring a source obliges
        // the remix, not the source.
        assert_ok!(remix(BOB, 2, (0, 0)));
        // And Alice remixes Bob's remix.
        assert_ok!(remix(ALICE, 3, (1, 0)));

        let original = Drc369::asset(0, 0).unwrap();
        assert_eq!((original.derived_from, original.remix_depth), (None, 0));
        let first = Drc369::asset(1, 0).unwrap();
        assert_eq!((first.derived_from, first.remix_depth), (Some((0, 0)), 1));
        let second = Drc369::asset(0, 1).unwrap();
        assert_eq!((second.derived_from, second.remix_depth), (Some((1, 0)), 2));

        // Each source counts the remixes that name it.
        assert_eq!(Drc369::remixes_of(0, 0), 1);
        assert_eq!(Drc369::remixes_of(1, 0), 1);
        assert_eq!(Drc369::remixes_of(0, 1), 0);

        assert!(events().contains(&Event::Minted {
            collection: 1,
            item: 0,
            owner: account(BOB),
            by: account(BOB),
            origin: content(2),
            commit: commit(2),
            revisable: true,
            derived_from: Some((0, 0)),
        }));

        // Nothing changes it afterwards: a revision leaves the source alone.
        assert_ok!(Drc369::revise(signed(BOB), 1, 0, content(9), commit(9)));
        assert_eq!(Drc369::asset(1, 0).unwrap().derived_from, Some((0, 0)));
    });
}

#[test]
fn a_remix_of_nothing_or_past_the_depth_bound_is_refused_at_mint() {
    new_test_ext().execute_with(|| {
        // No such asset, including the id the remix itself is about to take.
        assert_noop!(remix(ALICE, 1, (0, 0)), Error::<Test>::UnknownSource);

        assert_ok!(mint(ALICE, 1));
        let mut source = (0, 0);
        for depth in 1..=MAX_REMIX_DEPTH {
            assert_ok!(remix(ALICE, 1 + depth, source));
            source = (0, depth as ItemId);
            assert_eq!(
                Drc369::asset(0, depth as ItemId).unwrap().remix_depth,
                depth
            );
        }
        // One more would be deeper than the bound: refused, and nothing is left
        // behind, not even an item id.
        let before = Singles::<Test>::get(account(ALICE)).unwrap();
        assert_noop!(remix(ALICE, 50, source), Error::<Test>::RemixTooDeep);
        assert_eq!(Singles::<Test>::get(account(ALICE)).unwrap(), before);
    });
}

// ---------------------------------------------------------------------------
// Nesting (M4.2, M4.5, requirement R-2, ADR-025)
// ---------------------------------------------------------------------------

/// Alice's assets are `(0, 0)`, `(0, 1)`, … in the order she mints them.
fn alice_mints(count: u8) {
    for seed in 0..count {
        assert_ok!(mint(ALICE, seed + 1));
    }
}

fn nest(who: u8, child: (CollectionId, ItemId), parent: (CollectionId, ItemId)) -> DispatchResult {
    Drc369::nest(signed(who), child.0, child.1, parent)
}

fn unnest(who: u8, child: (CollectionId, ItemId)) -> DispatchResult {
    Drc369::unnest(signed(who), child.0, child.1)
}

#[test]
fn an_owner_nests_an_asset_and_takes_it_out_again() {
    new_test_ext().execute_with(|| {
        alice_mints(3);
        assert_ok!(nest(ALICE, (0, 1), (0, 0)));
        assert_ok!(nest(ALICE, (0, 2), (0, 1)));

        assert_eq!(Drc369::parent_of(0, 0), None);
        assert_eq!(Drc369::parent_of(0, 1), Some((0, 0)));
        assert_eq!(Drc369::parent_of(0, 2), Some((0, 1)));
        assert_eq!(Drc369::children_of(0, 0), 1);
        assert_eq!(Drc369::children_of(0, 1), 1);
        assert_eq!(Drc369::children_of(0, 2), 0);
        // Nesting changes where an asset is, not whose it is or what it is.
        assert_eq!(Nfts::owner(0, 2), Some(account(ALICE)));
        assert_eq!(Drc369::asset(0, 2).unwrap().origin, content(3));

        // Taking the middle one out takes what it holds with it.
        assert_ok!(unnest(ALICE, (0, 1)));
        assert_eq!(Drc369::parent_of(0, 1), None);
        assert_eq!(Drc369::parent_of(0, 2), Some((0, 1)));
        assert_eq!(Drc369::children_of(0, 0), 0);
        assert!(!Drc369::is_held_in_place(0, 0));

        assert_ok!(unnest(ALICE, (0, 2)));
        // Nothing is left behind: no parent, no count, nothing held in place.
        assert_eq!(ParentOf::<Test>::iter().count(), 0);
        assert_eq!(ChildCount::<Test>::iter().count(), 0);
        for item in 0..3 {
            assert!(!Drc369::is_held_in_place(0, item));
        }

        let nesting: Vec<_> = events()
            .into_iter()
            .filter(|e| matches!(e, Event::Nested { .. } | Event::Unnested { .. }))
            .collect();
        assert_eq!(
            nesting,
            vec![
                Event::Nested {
                    parent: (0, 0),
                    child: (0, 1),
                    by: account(ALICE),
                    depth: 1,
                },
                Event::Nested {
                    parent: (0, 1),
                    child: (0, 2),
                    by: account(ALICE),
                    depth: 2,
                },
                Event::Unnested {
                    parent: (0, 0),
                    child: (0, 1),
                    by: account(ALICE),
                },
                Event::Unnested {
                    parent: (0, 1),
                    child: (0, 2),
                    by: account(ALICE),
                },
            ]
        );

        // And it can go somewhere else afterwards.
        assert_ok!(nest(ALICE, (0, 0), (0, 2)));
        assert_eq!(Drc369::parent_of(0, 0), Some((0, 2)));
    });
}

/// The parent-owner check: the custom chain checked the child alone until
/// 14 September 2026, so a stranger could nest under somebody's asset.
#[test]
fn only_the_owner_of_both_assets_may_nest_them() {
    new_test_ext().execute_with(|| {
        alice_mints(2);
        assert_ok!(mint(BOB, 8));
        assert_ok!(mint(BOB, 9));

        // Bob's own asset, under Alice's.
        assert_noop!(nest(BOB, (1, 0), (0, 0)), Error::<Test>::NotOwner);
        // Alice's asset, under Bob's own.
        assert_noop!(nest(BOB, (0, 1), (1, 0)), Error::<Test>::NotOwner);
        // Neither his.
        assert_noop!(nest(BOB, (0, 1), (0, 0)), Error::<Test>::NotOwner);
        for origin in [RuntimeOrigin::none(), RuntimeOrigin::root()] {
            assert_noop!(Drc369::nest(origin, 0, 1, (0, 0)), DispatchError::BadOrigin);
        }
        assert_eq!(ParentOf::<Test>::iter().count(), 0);

        // Nor may anyone else take an asset out.
        assert_ok!(nest(ALICE, (0, 1), (0, 0)));
        assert_noop!(unnest(BOB, (0, 1)), Error::<Test>::NotOwner);
        assert_noop!(
            Drc369::unnest(RuntimeOrigin::root(), 0, 1),
            DispatchError::BadOrigin
        );
        assert_eq!(Drc369::parent_of(0, 1), Some((0, 0)));
    });
}

#[test]
fn an_asset_cannot_be_nested_in_itself() {
    new_test_ext().execute_with(|| {
        alice_mints(1);
        assert_noop!(nest(ALICE, (0, 0), (0, 0)), Error::<Test>::NestingCycle);
        assert!(!Drc369::is_held_in_place(0, 0));
    });
}

/// R-2's own example: A inside B, then B inside A.
#[test]
fn a_two_asset_cycle_is_refused() {
    new_test_ext().execute_with(|| {
        alice_mints(2);
        assert_ok!(nest(ALICE, (0, 0), (0, 1)));
        assert_noop!(nest(ALICE, (0, 1), (0, 0)), Error::<Test>::NestingCycle);
        assert_eq!(Drc369::parent_of(0, 1), None);
    });
}

/// A cycle through every level the bound allows is still seen as a cycle: the
/// walk reaches the child on its last step.
#[test]
fn a_longer_cycle_is_refused() {
    new_test_ext().execute_with(|| {
        alice_mints(4);
        // 0 holds 1 holds 2.
        assert_ok!(nest(ALICE, (0, 1), (0, 0)));
        assert_ok!(nest(ALICE, (0, 2), (0, 1)));
        assert_noop!(nest(ALICE, (0, 0), (0, 2)), Error::<Test>::NestingCycle);

        // 0 holds 1 holds 2 holds 3, as deep as the bound goes.
        assert_ok!(nest(ALICE, (0, 3), (0, 2)));
        assert_eq!(MAX_NESTING_DEPTH, 3);
        assert_noop!(nest(ALICE, (0, 0), (0, 3)), Error::<Test>::NestingCycle);
        assert_noop!(nest(ALICE, (0, 1), (0, 3)), Error::<Test>::AlreadyNested);
        assert_eq!(Drc369::parent_of(0, 0), None);
    });
}

#[test]
fn nesting_deeper_than_the_bound_is_refused() {
    new_test_ext().execute_with(|| {
        alice_mints(MAX_NESTING_DEPTH + 2);
        for depth in 1..=MAX_NESTING_DEPTH {
            let child = (0, depth as ItemId);
            let parent = (0, depth as ItemId - 1);
            assert_ok!(nest(ALICE, child, parent));
            assert!(events().contains(&Event::Nested {
                parent,
                child,
                by: account(ALICE),
                depth,
            }));
        }

        // One more level would be past the bound.
        let deepest = (0, MAX_NESTING_DEPTH as ItemId);
        let another = (0, MAX_NESTING_DEPTH as ItemId + 1);
        assert_noop!(nest(ALICE, another, deepest), Error::<Test>::NestedTooDeep);

        // The same asset one level up is exactly at the bound, and allowed.
        let above = (0, MAX_NESTING_DEPTH as ItemId - 1);
        assert_ok!(nest(ALICE, another, above));
    });
}

/// What keeps the depth bound exact: a tree is built from the root down, so an
/// asset never arrives carrying levels nothing counted.
#[test]
fn an_asset_holding_others_cannot_itself_be_nested() {
    new_test_ext().execute_with(|| {
        alice_mints(3);
        assert_ok!(nest(ALICE, (0, 1), (0, 0)));
        assert_noop!(nest(ALICE, (0, 0), (0, 2)), Error::<Test>::HoldsAssets);

        // Emptied, it may be.
        assert_ok!(unnest(ALICE, (0, 1)));
        assert_ok!(nest(ALICE, (0, 0), (0, 2)));
    });
}

#[test]
fn a_parent_holds_no_more_than_the_bound() {
    new_test_ext().execute_with(|| {
        assert_eq!(MAX_CHILDREN, 2);
        alice_mints(4);
        assert_ok!(nest(ALICE, (0, 1), (0, 0)));
        assert_ok!(nest(ALICE, (0, 2), (0, 0)));
        assert_noop!(nest(ALICE, (0, 3), (0, 0)), Error::<Test>::TooManyChildren);
        assert_eq!(Drc369::children_of(0, 0), MAX_CHILDREN);

        // Taking one out makes room for another.
        assert_ok!(unnest(ALICE, (0, 1)));
        assert_ok!(nest(ALICE, (0, 3), (0, 0)));
        assert_eq!(Drc369::children_of(0, 0), MAX_CHILDREN);
    });
}

/// ADR-025: a nested child is locked in `pallet-nfts`. So is the asset holding
/// it, because nothing here moves a tree. The refusal is `pallet-nfts`'s own.
#[test]
fn a_nested_asset_and_the_asset_holding_it_cannot_be_transferred() {
    new_test_ext().execute_with(|| {
        alice_mints(2);
        // An approval given beforehand does not get round it either.
        assert_ok!(Nfts::approve_transfer(
            signed(ALICE),
            0,
            1,
            account(BOB),
            None
        ));
        assert_ok!(nest(ALICE, (0, 1), (0, 0)));

        assert_noop!(
            Nfts::transfer(signed(ALICE), 0, 1, account(BOB)),
            pallet_nfts::Error::<Test>::ItemLocked
        );
        assert_noop!(
            Nfts::transfer(signed(BOB), 0, 1, account(BOB)),
            pallet_nfts::Error::<Test>::ItemLocked
        );
        assert_noop!(
            Nfts::transfer(signed(ALICE), 0, 0, account(BOB)),
            pallet_nfts::Error::<Test>::ItemLocked
        );
        assert_eq!(Nfts::owner(0, 0), Some(account(ALICE)));
        assert_eq!(Nfts::owner(0, 1), Some(account(ALICE)));

        // Taken out, both move again.
        assert_ok!(unnest(ALICE, (0, 1)));
        assert_ok!(Nfts::transfer(signed(ALICE), 0, 1, account(BOB)));
        assert_ok!(Nfts::transfer(signed(ALICE), 0, 0, account(BOB)));
        assert_eq!(Nfts::owner(0, 0), Some(account(BOB)));
        assert_eq!(Nfts::owner(0, 1), Some(account(BOB)));
    });
}

#[test]
fn an_asset_is_in_one_place_at_a_time_and_only_assets_nest() {
    new_test_ext().execute_with(|| {
        alice_mints(3);
        assert_noop!(unnest(ALICE, (0, 1)), Error::<Test>::NotNested);

        assert_ok!(nest(ALICE, (0, 1), (0, 0)));
        // Not into a second parent, and not into the same one twice.
        assert_noop!(nest(ALICE, (0, 1), (0, 2)), Error::<Test>::AlreadyNested);
        assert_noop!(nest(ALICE, (0, 1), (0, 0)), Error::<Test>::AlreadyNested);
        assert_eq!(Drc369::children_of(0, 0), 1);
        assert_eq!(Drc369::children_of(0, 2), 0);

        // Neither end may be something that is not a DRC-369 asset.
        assert_noop!(nest(ALICE, (0, 9), (0, 0)), Error::<Test>::UnknownAsset);
        assert_noop!(nest(ALICE, (0, 2), (0, 9)), Error::<Test>::UnknownParent);
        assert_noop!(unnest(ALICE, (0, 9)), Error::<Test>::NotNested);
    });
}

// ---------------------------------------------------------------------------
// Progress, kept by each game that earns it (ADR-113, M4.2 and M4.5's state and XP)
// ---------------------------------------------------------------------------

/// A game server's key: holds nothing, writes only what Alice's game allows it to.
const WRITER: u8 = 3;
/// Buys one of Bob's items.
const CAROL: u8 = 4;

/// Alice's game: the Cartridge she minted first, `(0, 0)`.
const GAME: (CollectionId, ItemId) = (0, 0);
/// Bob's singles collection, where his items for the game are.
const ITEMS: CollectionId = 1;

fn stats(pairs: &[(u16, i64)]) -> StatsOf<Test> {
    pairs.to_vec().try_into().unwrap()
}

fn record(
    who: u8,
    item: ItemId,
    game: (CollectionId, ItemId),
    add_xp: u64,
    level: Option<u32>,
    set: &[(u16, i64)],
) -> DispatchResult {
    Drc369::record_progress(signed(who), ITEMS, item, game, add_xp, level, stats(set))
}

fn correct(who: u8, item: ItemId, level: u32, xp: u64, set: &[(u16, i64)]) -> DispatchResult {
    Drc369::correct_progress(signed(who), ITEMS, item, GAME, level, xp, stats(set))
}

fn kept(item: ItemId, game: (CollectionId, ItemId)) -> Option<ProgressOf<Test>> {
    Records::<Test>::get((ITEMS, item), Issuer::game(game))
}

/// Alice publishes a game, Bob mints `items` for it and binds his collection to
/// it, and Alice lets `WRITER` write the game's records.
fn a_game_with_items(items: u8) {
    assert_ok!(mint(ALICE, 1));
    for seed in 0..items {
        assert_ok!(mint(BOB, 10 + seed));
    }
    assert_ok!(Drc369::set_home_game(signed(BOB), ITEMS, Some(GAME)));
    assert_ok!(Drc369::authorise_writer(
        signed(ALICE),
        GAME,
        account(WRITER)
    ));
}

#[test]
fn a_collection_is_static_until_its_owner_binds_a_home_game() {
    new_test_ext().execute_with(|| {
        assert_ok!(mint(ALICE, 1));
        assert_ok!(mint(BOB, 10));
        assert_ok!(Drc369::authorise_writer(
            signed(ALICE),
            GAME,
            account(WRITER)
        ));

        // Nothing is written to an asset whose collection has no home game.
        assert_noop!(
            record(WRITER, 0, GAME, 5, None, &[]),
            Error::<Test>::NoHomeGame
        );

        // Only the collection's owner binds it, and only to a DRC-369 asset.
        assert_noop!(
            Drc369::set_home_game(signed(ALICE), ITEMS, Some(GAME)),
            Error::<Test>::NotCollectionOwner
        );
        assert_noop!(
            Drc369::set_home_game(signed(BOB), ITEMS, Some((0, 9))),
            Error::<Test>::UnknownGame
        );
        assert_ok!(Drc369::set_home_game(signed(BOB), ITEMS, Some(GAME)));
        assert_eq!(Drc369::home_game(ITEMS), Some(Issuer::game(GAME)));
        assert_noop!(
            Drc369::set_home_game(signed(BOB), ITEMS, Some(GAME)),
            Error::<Test>::NothingRecorded
        );
        assert_ok!(record(WRITER, 0, GAME, 5, None, &[]));

        // Cleared, the collection is static again, and its record stays.
        assert_ok!(Drc369::set_home_game(signed(BOB), ITEMS, None));
        assert_eq!(Drc369::home_game(ITEMS), None);
        assert_noop!(
            record(WRITER, 0, GAME, 5, None, &[]),
            Error::<Test>::NoHomeGame
        );
        assert_eq!(kept(0, GAME).unwrap().xp, 5);
        assert!(events().contains(&Event::HomeGameSet {
            collection: ITEMS,
            game: None,
            by: account(BOB),
        }));
    });
}

#[test]
fn only_whoever_holds_the_cartridge_names_its_writers() {
    new_test_ext().execute_with(|| {
        assert_ok!(mint(ALICE, 1));
        let game = Issuer::game(GAME);

        assert_noop!(
            Drc369::authorise_writer(signed(BOB), GAME, account(WRITER)),
            Error::<Test>::NotGameHolder
        );
        assert_noop!(
            Drc369::authorise_writer(signed(ALICE), (0, 9), account(WRITER)),
            Error::<Test>::UnknownGame
        );
        assert_ok!(Drc369::authorise_writer(
            signed(ALICE),
            GAME,
            account(WRITER)
        ));
        assert_noop!(
            Drc369::authorise_writer(signed(ALICE), GAME, account(WRITER)),
            Error::<Test>::AlreadyWriter
        );
        assert_ok!(Drc369::authorise_writer(
            signed(ALICE),
            GAME,
            account(CAROL)
        ));
        assert_eq!(WriterCount::<Test>::get(game), MAX_WRITERS);
        assert_noop!(
            Drc369::authorise_writer(signed(ALICE), GAME, account(NOBODY)),
            Error::<Test>::TooManyWriters
        );

        // The authority goes with the Cartridge.
        assert_ok!(Nfts::transfer(signed(ALICE), GAME.0, GAME.1, account(BOB)));
        assert_noop!(
            Drc369::revoke_writer(signed(ALICE), GAME, account(WRITER)),
            Error::<Test>::NotGameHolder
        );
        assert_ok!(Drc369::revoke_writer(signed(BOB), GAME, account(WRITER)));
        assert_noop!(
            Drc369::revoke_writer(signed(BOB), GAME, account(WRITER)),
            Error::<Test>::NotWriter
        );
        assert_ok!(Drc369::revoke_writer(signed(BOB), GAME, account(CAROL)));
        assert!(!WriterCount::<Test>::contains_key(game), "dropped at zero");
        assert!(events().contains(&Event::WriterRevoked {
            game,
            writer: account(CAROL),
            by: account(BOB),
        }));
    });
}

#[test]
fn a_writer_adds_xp_raises_the_level_and_sets_stats() {
    new_test_ext().execute_with(|| {
        a_game_with_items(1);

        assert_ok!(record(WRITER, 0, GAME, 40, Some(2), &[(7, 100), (2, -5)]));
        assert_ok!(record(WRITER, 0, GAME, 10, None, &[(7, 90)]));
        assert_ok!(record(WRITER, 0, GAME, 0, Some(3), &[]));

        let progress = kept(0, GAME).unwrap();
        assert_eq!(progress.xp, 50);
        assert_eq!(progress.level, 3);
        assert_eq!(
            progress.stats.into_inner(),
            vec![(2, -5), (7, 90)],
            "sorted by number"
        );
        assert_eq!(
            events().last(),
            Some(&Event::StateChanged {
                collection: ITEMS,
                item: 0,
                game: Issuer::game(GAME),
                level: 3,
                xp: 50,
                stats: stats(&[]),
                by: account(WRITER),
            })
        );
        assert_eq!(
            Drc369::progress_of(ITEMS, 0),
            vec![ProgressRecord {
                issuer: Issuer::game(GAME),
                home: true,
                level: 3,
                xp: 50,
                stats: vec![(2, -5), (7, 90)],
            }]
        );
    });
}

#[test]
fn a_writer_can_never_take_progress_back() {
    new_test_ext().execute_with(|| {
        a_game_with_items(1);
        assert_ok!(record(WRITER, 0, GAME, u64::MAX - 1, Some(5), &[(1, 1)]));

        assert_noop!(
            record(WRITER, 0, GAME, 0, Some(4), &[]),
            Error::<Test>::LevelMustRise
        );
        assert_noop!(
            record(WRITER, 0, GAME, 2, None, &[]),
            Error::<Test>::XpOverflow
        );
        // A call that would change nothing is refused, so every event is a change.
        assert_noop!(
            record(WRITER, 0, GAME, 0, None, &[]),
            Error::<Test>::NothingRecorded
        );
        assert_noop!(
            record(WRITER, 0, GAME, 0, Some(5), &[(1, 1)]),
            Error::<Test>::NothingRecorded
        );
        // Stats move either way.
        assert_ok!(record(WRITER, 0, GAME, 0, None, &[(1, -1)]));
    });
}

#[test]
fn stats_are_bounded_and_each_named_once() {
    new_test_ext().execute_with(|| {
        a_game_with_items(1);
        assert_noop!(
            record(WRITER, 0, GAME, 1, None, &[(4, 1), (4, 2)]),
            Error::<Test>::DuplicateStat
        );
        assert_ok!(record(WRITER, 0, GAME, 0, None, &[(1, 1), (2, 2), (3, 3)]));
        assert_noop!(
            record(WRITER, 0, GAME, 0, None, &[(4, 4)]),
            Error::<Test>::TooManyStats
        );
        // Changing one already held is no new stat.
        assert_ok!(record(WRITER, 0, GAME, 0, None, &[(3, 30)]));
    });
}

#[test]
fn only_a_writer_of_the_home_game_writes_and_never_the_holder() {
    new_test_ext().execute_with(|| {
        a_game_with_items(2);
        assert_ok!(Nfts::transfer(signed(BOB), ITEMS, 0, account(CAROL)));

        // The item's own holder cannot raise it, and neither can the game's
        // holder or the collection's owner without being a writer.
        for who in [CAROL, ALICE, BOB] {
            assert_noop!(record(who, 0, GAME, 5, None, &[]), Error::<Test>::NotWriter);
        }
        // Another game's writer writes nothing here.
        assert_ok!(mint(ALICE, 2));
        let other = (0, 1);
        assert_ok!(Drc369::authorise_writer(
            signed(ALICE),
            other,
            account(NOBODY)
        ));
        assert_noop!(
            record(NOBODY, 0, other, 5, None, &[]),
            Error::<Test>::NotHomeGame
        );
        assert_noop!(
            record(WRITER, 9, GAME, 5, None, &[]),
            Error::<Test>::UnknownAsset
        );

        assert_ok!(record(WRITER, 0, GAME, 5, None, &[]));
        assert_ok!(Drc369::revoke_writer(signed(ALICE), GAME, account(WRITER)));
        assert_noop!(
            record(WRITER, 1, GAME, 5, None, &[]),
            Error::<Test>::NotWriter
        );
    });
}

#[test]
fn nothing_changes_while_an_asset_is_offered_for_sale() {
    new_test_ext().execute_with(|| {
        a_game_with_items(1);
        assert_ok!(record(WRITER, 0, GAME, 5, Some(1), &[]));

        ListedItems::set(&vec![(ITEMS, 0)]);
        assert_noop!(
            record(WRITER, 0, GAME, 5, None, &[]),
            Error::<Test>::ListedForSale
        );
        assert_noop!(correct(ALICE, 0, 0, 0, &[]), Error::<Test>::ListedForSale);

        ListedItems::set(&Vec::new());
        assert_ok!(record(WRITER, 0, GAME, 5, None, &[]));
    });
}

#[test]
fn a_correction_is_the_cartridge_holders_and_may_lower() {
    new_test_ext().execute_with(|| {
        a_game_with_items(2);
        assert_noop!(correct(ALICE, 0, 0, 0, &[]), Error::<Test>::NoRecord);
        assert_ok!(record(WRITER, 0, GAME, 900, Some(9), &[(5, 50), (1, 10)]));

        // Not the writer, and not the item's holder.
        assert_noop!(correct(WRITER, 0, 1, 1, &[]), Error::<Test>::NotGameHolder);
        assert_noop!(correct(BOB, 0, 1, 1, &[]), Error::<Test>::NotGameHolder);

        assert_ok!(correct(ALICE, 0, 1, 20, &[(5, 5), (2, 2)]));
        let progress = kept(0, GAME).unwrap();
        assert_eq!((progress.level, progress.xp), (1, 20));
        assert_eq!(
            progress.stats.into_inner(),
            vec![(2, 2), (5, 5)],
            "replaced, sorted"
        );
        assert_eq!(
            events().last(),
            Some(&Event::StateCorrected {
                collection: ITEMS,
                item: 0,
                game: Issuer::game(GAME),
                level: 1,
                xp: 20,
                stats: stats(&[(2, 2), (5, 5)]),
                by: account(ALICE),
            })
        );
        assert_noop!(
            correct(ALICE, 0, 1, 20, &[(2, 2), (5, 5)]),
            Error::<Test>::NothingRecorded
        );
        assert_noop!(
            correct(ALICE, 0, 1, 1, &[(2, 1), (2, 2)]),
            Error::<Test>::DuplicateStat
        );
    });
}

#[test]
fn records_stay_with_their_game_when_the_home_changes() {
    new_test_ext().execute_with(|| {
        a_game_with_items(1);
        assert_ok!(record(WRITER, 0, GAME, 30, Some(3), &[]));

        // Bob moves his collection to a second game of Alice's.
        assert_ok!(mint(ALICE, 2));
        let second = (0, 1);
        assert_ok!(Drc369::authorise_writer(
            signed(ALICE),
            second,
            account(NOBODY)
        ));
        assert_ok!(Drc369::set_home_game(signed(BOB), ITEMS, Some(second)));

        assert_noop!(
            record(WRITER, 0, GAME, 1, None, &[]),
            Error::<Test>::NotHomeGame
        );
        assert_ok!(record(NOBODY, 0, second, 7, None, &[]));

        let records = Drc369::progress_of(ITEMS, 0);
        assert_eq!(records.len(), 2);
        assert_eq!(
            (records[0].issuer, records[0].home, records[0].xp),
            (Issuer::game(second), true, 7)
        );
        assert_eq!(
            (records[1].issuer, records[1].home, records[1].xp),
            (Issuer::game(GAME), false, 30)
        );
    });
}

/// A record never moves, locks or sells an asset, and never touches CGT (ADR-113
/// decision 8).
#[test]
fn a_record_never_holds_an_asset_or_moves_currency() {
    new_test_ext().execute_with(|| {
        a_game_with_items(1);
        let before = (
            Balances::total_balance(&account(ALICE)),
            Balances::total_balance(&account(BOB)),
        );
        assert_ok!(record(WRITER, 0, GAME, 50, Some(2), &[(1, 1)]));
        assert_ok!(correct(ALICE, 0, 1, 10, &[]));

        assert!(!Drc369::is_held_in_place(ITEMS, 0));
        assert_eq!(
            (
                Balances::total_balance(&account(ALICE)),
                Balances::total_balance(&account(BOB))
            ),
            before
        );
        assert_eq!(
            Balances::total_balance(&account(WRITER)),
            0,
            "a writer needs no CGT"
        );
        assert_ok!(Nfts::transfer(signed(BOB), ITEMS, 0, account(CAROL)));
        assert_eq!(Nfts::owner(ITEMS, 0), Some(account(CAROL)));
        assert_eq!(
            kept(0, GAME).unwrap().xp,
            10,
            "the record goes with the asset"
        );
    });
}

/// The issuer is tagged, as the content reference is, so a later kind of issuer
/// is a new index and never a change to this one's encoding.
#[test]
fn an_issuer_is_a_tag_and_a_cartridge() {
    let encoded = Issuer::game((7, 9)).encode();
    assert_eq!(encoded, [vec![0u8], 7u32.encode(), 9u32.encode()].concat());
    assert_eq!(Issuer::max_encoded_len(), 9);
    assert!(Issuer::decode_all(&mut &[1u8, 0, 0, 0, 0, 0, 0, 0, 0][..]).is_err());
}
