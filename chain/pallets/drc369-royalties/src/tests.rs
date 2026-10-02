//! Tests for M4.2's royalties (ADR-061). The ones that pin money were each shown
//! to fail against a planted fault before they were trusted; `HANDOFF.md` records
//! which faults.

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok, traits::fungible::Inspect, BoundedVec};
use pallet_drc369::{CommitId, ContentRef, HashAlgo};
use sp_core::H256;
use sp_runtime::{DispatchError, DispatchResult, TokenError};

fn content(seed: u8) -> ContentRef {
    ContentRef {
        algo: HashAlgo::Blake3_256,
        root: H256::repeat_byte(seed),
        size: 1_000 + seed as u64,
    }
}

fn signed(who: u8) -> RuntimeOrigin {
    RuntimeOrigin::signed(account(who))
}

/// Mint to `who`, derived from `source` if given. Returns the new asset's id.
fn mint(who: u8, seed: u8, source: Option<(CollectionId, ItemId)>) -> (CollectionId, ItemId) {
    assert_ok!(Drc369::mint(
        signed(who),
        content(seed),
        Some(CommitId::Sha1([seed; 20])),
        b"work".to_vec().try_into().unwrap(),
        true,
        source,
    ));
    let singles = pallet_drc369::Singles::<Test>::get(account(who)).unwrap();
    (singles.collection, singles.next_item - 1)
}

fn ppm(parts: u32) -> Permill {
    Permill::from_parts(parts)
}

fn percent(value: u32) -> Permill {
    Permill::from_percent(value)
}

fn recipients(
    list: &[(u8, Permill)],
) -> BoundedVec<(AccountId, Permill), <Test as crate::Config>::MaxRoyaltyRecipients> {
    list.iter()
        .map(|(who, share)| (account(*who), *share))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn set_terms(
    who: u8,
    asset: (CollectionId, ItemId),
    list: &[(u8, Permill)],
    remix: Permill,
) -> DispatchResult {
    Royalties::set_terms(signed(who), asset.0, asset.1, recipients(list), remix)
}

fn free(who: u8) -> Balance {
    Balances::balance(&account(who))
}

fn holder(asset: (CollectionId, ItemId)) -> Option<AccountId> {
    Nfts::owner(asset.0, asset.1)
}

/// What each named account gained or lost since `before`.
fn changes(before: &[(u8, Balance)]) -> Vec<(u8, i128)> {
    before
        .iter()
        .map(|(who, was)| (*who, free(*who) as i128 - *was as i128))
        .collect()
}

fn snapshot(who: &[u8]) -> Vec<(u8, Balance)> {
    who.iter().map(|w| (*w, free(*w))).collect()
}

// ---------------------------------------------------------------------------
// The arithmetic
// ---------------------------------------------------------------------------

#[test]
fn a_split_always_sums_to_the_price() {
    let source = [(1u8, ppm(333_333)), (2, ppm(1)), (3, ppm(250_000))];
    let own = [(4u8, ppm(123_457)), (5, ppm(876_543))];
    for price in [0u128, 1, 7, 999, 1_000_001, 10u128.pow(20), 10u128.pow(32)] {
        for remix in [Permill::zero(), ppm(1), percent(17), Permill::one()] {
            let parts = split(price, Some((remix, &source[..])), &own);
            let total: u128 = parts
                .remix
                .iter()
                .chain(parts.royalties.iter())
                .map(|(_, amount)| *amount)
                .sum::<u128>()
                + parts.seller;
            assert_eq!(total, price, "price {price}, remix {remix:?}");
        }
    }
}

#[test]
fn the_remix_pool_is_divided_in_proportion_to_the_sources_shares() {
    // A 20% remix share of 10,000 is a pool of 2,000; shares of 30% and 10%
    // divide it three to one. The remix's own 50% is of the 8,000 left.
    let parts = split(
        10_000,
        Some((percent(20), &[(1u8, percent(30)), (2, percent(10))][..])),
        &[(3u8, percent(50))],
    );
    assert_eq!(parts.remix, vec![(1, 1_500), (2, 500)]);
    assert_eq!(parts.royalties, vec![(3, 4_000)]);
    assert_eq!(parts.seller, 4_000);
}

#[test]
fn rounding_goes_to_the_seller_and_never_above_a_share() {
    // 1% of 99 is 0.99: rounded down to nothing, and the seller keeps it.
    let parts = split(99, None, &[(1u8, percent(1))]);
    assert_eq!(parts.royalties, vec![(1, 0)]);
    assert_eq!(parts.seller, 99);

    // A pool of 10 divided three ways gets 3 each; the 1 left is the seller's.
    let thirds = [(1u8, ppm(1)), (2, ppm(1)), (3, ppm(1))];
    let parts = split(100, Some((percent(10), &thirds[..])), &[]);
    assert_eq!(parts.remix, vec![(1, 3), (2, 3), (3, 3)]);
    assert_eq!(parts.seller, 91);
}

/// AGENTS.md §5: every pallet that moves CGT pins the largest intermediate its
/// money paths form. `split` forms `share × price` inside `Permill::mul_floor`
/// and `pool × part` inside `multiply_by_rational_with_rounding`, both of which
/// widen internally; here both run at the largest price a `u128` can carry, with
/// every share at its most, and nothing overflows or panics.
#[test]
fn the_largest_intermediate_cannot_overflow() {
    let full_supply = 10u128.pow(32);
    for price in [full_supply, u128::MAX, u128::MAX - 1] {
        let parts = split(
            price,
            Some((Permill::one(), &[(1u8, ppm(999_999)), (2, ppm(1))][..])),
            &[(3u8, Permill::one())],
        );
        let upstream: u128 = parts.remix.iter().map(|(_, a)| *a).sum();
        assert!(upstream <= price);
        // A 100% remix share leaves at most the rounding for the rest.
        assert!(price - upstream < 2);
        let total = upstream + parts.royalties.iter().map(|(_, a)| *a).sum::<u128>() + parts.seller;
        assert_eq!(total, price);
    }

    // And a whole-price royalty with no remix pays the whole price.
    let parts = split(u128::MAX, None, &[(1u8, Permill::one())]);
    assert_eq!(parts.royalties, vec![(1, u128::MAX)]);
    assert_eq!(parts.seller, 0);
}

// ---------------------------------------------------------------------------
// Terms
// ---------------------------------------------------------------------------

#[test]
fn terms_change_only_while_the_creator_holds_the_asset() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);

        // Not somebody else.
        assert_noop!(
            set_terms(BOB, work, &[(BOB, percent(50))], Permill::zero()),
            Error::<Test>::NotCreator
        );

        assert_ok!(set_terms(ALICE, work, &[(DAVE, percent(10))], percent(5)));
        assert_eq!(
            events(),
            vec![Event::TermsSet {
                collection: work.0,
                item: work.1,
                recipients: vec![(account(DAVE), percent(10))],
                remix: percent(5),
                by: account(ALICE),
            }]
        );

        // The creator corrects a mistake while they still hold it.
        assert_ok!(set_terms(ALICE, work, &[(DAVE, percent(12))], percent(5)));
        assert_eq!(
            Royalties::terms(work.0, work.1)
                .unwrap()
                .recipients
                .into_inner(),
            vec![(account(DAVE), percent(12))]
        );

        // Once someone else holds it, its terms are fixed: neither the holder,
        // who is not its creator, nor the creator, who no longer holds it, may
        // change them.
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_000));
        assert_ok!(Royalties::buy(signed(CAROL), work.0, work.1, 1_000));
        assert_noop!(
            set_terms(CAROL, work, &[(CAROL, percent(10))], Permill::zero()),
            Error::<Test>::NotCreator
        );
        assert_noop!(
            set_terms(ALICE, work, &[(ALICE, percent(90))], percent(5)),
            Error::<Test>::NotCreator
        );

        // The same holds for an asset given away before any terms were set.
        let other = mint(ALICE, 2, None);
        assert_ok!(Nfts::transfer(
            signed(ALICE),
            other.0,
            other.1,
            account(BOB)
        ));
        assert_noop!(
            set_terms(BOB, other, &[(BOB, percent(10))], Permill::zero()),
            Error::<Test>::NotCreator
        );
        assert_noop!(
            set_terms(ALICE, other, &[(ALICE, percent(10))], Permill::zero()),
            Error::<Test>::NotCreator
        );
    });
}

#[test]
fn a_remixed_works_remix_share_can_fall_but_never_rise() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(set_terms(ALICE, work, &[(ALICE, percent(10))], percent(10)));
        // Before anyone remixes it, the creator may raise it.
        assert_ok!(set_terms(ALICE, work, &[(ALICE, percent(10))], percent(20)));

        let _remix = mint(BOB, 2, Some(work));
        assert_noop!(
            set_terms(ALICE, work, &[(ALICE, percent(10))], percent(21)),
            Error::<Test>::RemixShareLocked
        );
        // Lowering it, or changing who is paid, is still the creator's.
        assert_ok!(set_terms(ALICE, work, &[(DAVE, percent(10))], percent(20)));
        assert_ok!(set_terms(ALICE, work, &[(DAVE, percent(10))], percent(15)));
        assert_noop!(
            set_terms(ALICE, work, &[(DAVE, percent(10))], percent(20)),
            Error::<Test>::RemixShareLocked
        );

        // A work remixed before it had any terms had a remix share of nothing,
        // and it stays at nothing.
        let bare = mint(ALICE, 3, None);
        let _ = mint(BOB, 4, Some(bare));
        assert_noop!(
            set_terms(ALICE, bare, &[(ALICE, percent(10))], ppm(1)),
            Error::<Test>::RemixShareLocked
        );
        assert_ok!(set_terms(
            ALICE,
            bare,
            &[(ALICE, percent(10))],
            Permill::zero()
        ));
    });
}

#[test]
fn terms_must_be_well_formed() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            set_terms(ALICE, (0, 0), &[], Permill::zero()),
            Error::<Test>::UnknownAsset
        );
        let work = mint(ALICE, 1, None);

        assert_noop!(
            set_terms(
                ALICE,
                work,
                &[(BOB, percent(60)), (DAVE, ppm(400_001))],
                Permill::zero()
            ),
            Error::<Test>::SharesExceedWhole
        );
        assert_noop!(
            set_terms(ALICE, work, &[(BOB, Permill::zero())], Permill::zero()),
            Error::<Test>::ZeroShare
        );
        assert_noop!(
            set_terms(
                ALICE,
                work,
                &[(BOB, percent(1)), (DAVE, percent(1)), (BOB, percent(1))],
                Permill::zero()
            ),
            Error::<Test>::DuplicateRecipient
        );
        assert_noop!(
            set_terms(ALICE, work, &[], percent(10)),
            Error::<Test>::RemixShareWithoutRecipients
        );

        // Exactly the whole is allowed, as is the most recipients.
        let most: Vec<(u8, Permill)> = (10..10 + MAX_RECIPIENTS as u8)
            .map(|who| (who, ppm(125_000)))
            .collect();
        assert_ok!(set_terms(ALICE, work, &most, Permill::one()));
        // One more recipient does not fit the type at all.
        let too_many: Vec<(AccountId, Permill)> = (0..=MAX_RECIPIENTS as u8)
            .map(|who| (account(who), ppm(1)))
            .collect();
        assert!(
            BoundedVec::<_, <Test as crate::Config>::MaxRoyaltyRecipients>::try_from(too_many)
                .is_err()
        );
    });
}

// ---------------------------------------------------------------------------
// The settled sale
// ---------------------------------------------------------------------------

#[test]
fn a_sale_pays_the_royalties_then_the_seller_and_hands_the_asset_over() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(set_terms(
            ALICE,
            work,
            &[(DAVE, percent(10)), (ALICE, percent(5))],
            Permill::zero()
        ));
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_000));
        assert_eq!(
            Royalties::listing(work.0, work.1),
            Some(Listing {
                seller: account(ALICE),
                price: 1_000
            })
        );

        let before = snapshot(&[ALICE, CAROL, DAVE]);
        assert_ok!(Royalties::buy(signed(CAROL), work.0, work.1, 1_000));

        // Dave 10%, Alice 5% as recipient and the other 85% as seller.
        assert_eq!(
            changes(&before),
            vec![(ALICE, 900), (CAROL, -1_000), (DAVE, 100)]
        );
        assert_eq!(holder(work), Some(account(CAROL)));
        assert_eq!(Royalties::listing(work.0, work.1), None);
        assert_eq!(
            events().last(),
            Some(&Event::Sold {
                collection: work.0,
                item: work.1,
                from: account(ALICE),
                to: account(CAROL),
                price: 1_000,
                source: None,
                remix: vec![],
                royalties: vec![(account(DAVE), 100), (account(ALICE), 50)],
                seller_received: 850,
            })
        );

        // A resale pays the creator's recipients again: this is the point.
        assert_ok!(Royalties::list(signed(CAROL), work.0, work.1, 2_000));
        let before = snapshot(&[ALICE, BOB, CAROL, DAVE]);
        assert_ok!(Royalties::buy(signed(BOB), work.0, work.1, 2_500));
        assert_eq!(
            changes(&before),
            vec![(ALICE, 100), (BOB, -2_000), (CAROL, 1_700), (DAVE, 200)]
        );
        assert_eq!(holder(work), Some(account(BOB)));
    });
}

#[test]
fn an_asset_without_terms_sells_with_everything_to_the_seller() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 500));
        let before = snapshot(&[ALICE, CAROL]);
        assert_ok!(Royalties::buy(signed(CAROL), work.0, work.1, 500));
        assert_eq!(changes(&before), vec![(ALICE, 500), (CAROL, -500)]);
        assert_eq!(holder(work), Some(account(CAROL)));
    });
}

#[test]
fn a_remix_pays_its_source_first_then_its_own_recipients() {
    new_test_ext().execute_with(|| {
        let original = mint(ALICE, 1, None);
        assert_ok!(set_terms(
            ALICE,
            original,
            &[(ALICE, percent(30)), (DAVE, percent(10))],
            percent(20)
        ));
        let remix = mint(BOB, 2, Some(original));
        assert_ok!(set_terms(
            BOB,
            remix,
            &[(BOB, percent(50))],
            Permill::zero()
        ));

        assert_ok!(Royalties::list(signed(BOB), remix.0, remix.1, 10_000));
        let before = snapshot(&[ALICE, BOB, CAROL, DAVE]);
        assert_ok!(Royalties::buy(signed(CAROL), remix.0, remix.1, 10_000));

        // Pool 2,000 split 3:1; Bob's 50% of the 8,000 left; Bob the rest.
        assert_eq!(
            changes(&before),
            vec![(ALICE, 1_500), (BOB, 8_000), (CAROL, -10_000), (DAVE, 500)]
        );
        match events().last() {
            Some(Event::Sold {
                source,
                remix: upstream,
                royalties,
                seller_received,
                ..
            }) => {
                assert_eq!(*source, Some(original));
                assert_eq!(
                    upstream,
                    &vec![(account(ALICE), 1_500), (account(DAVE), 500)]
                );
                assert_eq!(royalties, &vec![(account(BOB), 4_000)]);
                assert_eq!(*seller_received, 4_000);
            }
            other => panic!("expected Sold, got {other:?}"),
        }
    });
}

#[test]
fn a_remix_pays_its_direct_source_only() {
    new_test_ext().execute_with(|| {
        let original = mint(ALICE, 1, None);
        assert_ok!(set_terms(
            ALICE,
            original,
            &[(ALICE, percent(10))],
            percent(50)
        ));
        let remix = mint(BOB, 2, Some(original));
        assert_ok!(set_terms(BOB, remix, &[(BOB, percent(10))], percent(10)));
        // Dave remixes the remix and sets no terms of his own.
        let second = mint(DAVE, 3, Some(remix));

        assert_ok!(Royalties::list(signed(DAVE), second.0, second.1, 1_000));
        let before = snapshot(&[ALICE, BOB, CAROL, DAVE]);
        assert_ok!(Royalties::buy(signed(CAROL), second.0, second.1, 1_000));

        // Bob's recipients get Bob's remix share, 10%; Alice, two steps up,
        // gets nothing from this sale.
        assert_eq!(
            changes(&before),
            vec![(ALICE, 0), (BOB, 100), (CAROL, -1_000), (DAVE, 900)]
        );
    });
}

#[test]
fn a_remix_of_a_source_without_terms_pays_nothing_upstream() {
    new_test_ext().execute_with(|| {
        let original = mint(ALICE, 1, None);
        let remix = mint(BOB, 2, Some(original));
        assert_ok!(Royalties::list(signed(BOB), remix.0, remix.1, 1_000));
        let before = snapshot(&[ALICE, BOB, CAROL]);
        assert_ok!(Royalties::buy(signed(CAROL), remix.0, remix.1, 1_000));
        assert_eq!(
            changes(&before),
            vec![(ALICE, 0), (BOB, 1_000), (CAROL, -1_000)]
        );
    });
}

#[test]
fn a_listing_is_void_once_its_seller_no_longer_holds_the_asset() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_000));
        // Alice gives it to Bob outside the sale.
        assert_ok!(Nfts::transfer(signed(ALICE), work.0, work.1, account(BOB)));

        assert_noop!(
            Royalties::buy(signed(CAROL), work.0, work.1, 1_000),
            Error::<Test>::ListingStale
        );
        assert_eq!(holder(work), Some(account(BOB)));

        // Anyone may clear a void listing.
        assert_ok!(Royalties::unlist(signed(DAVE), work.0, work.1));
        assert_eq!(Royalties::listing(work.0, work.1), None);
    });
}

#[test]
fn only_the_holder_lists_and_only_the_seller_or_holder_withdraws_a_live_listing() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Royalties::list(signed(ALICE), 0, 0, 1),
            Error::<Test>::UnknownAsset
        );
        let work = mint(ALICE, 1, None);
        assert_noop!(
            Royalties::list(signed(BOB), work.0, work.1, 1_000),
            Error::<Test>::NotOwner
        );
        assert_noop!(
            Royalties::list(signed(ALICE), work.0, work.1, 0),
            Error::<Test>::ZeroPrice
        );
        assert_noop!(
            Royalties::unlist(signed(ALICE), work.0, work.1),
            Error::<Test>::NotListed
        );

        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_000));
        // Listing again changes the price.
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_200));
        assert_eq!(Royalties::listing(work.0, work.1).unwrap().price, 1_200);

        assert_noop!(
            Royalties::unlist(signed(BOB), work.0, work.1),
            Error::<Test>::NotOwner
        );
        assert_ok!(Royalties::unlist(signed(ALICE), work.0, work.1));
        assert_eq!(
            events().last(),
            Some(&Event::Unlisted {
                collection: work.0,
                item: work.1,
                by: account(ALICE),
            })
        );
    });
}

#[test]
fn a_buyer_pays_no_more_than_they_agreed_and_cannot_buy_their_own_listing() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_noop!(
            Royalties::buy(signed(CAROL), work.0, work.1, 1_000),
            Error::<Test>::NotListed
        );
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_000));
        assert_noop!(
            Royalties::buy(signed(CAROL), work.0, work.1, 999),
            Error::<Test>::PriceAboveLimit
        );
        assert_noop!(
            Royalties::buy(signed(ALICE), work.0, work.1, 1_000),
            Error::<Test>::OwnListing
        );
        assert_noop!(
            Royalties::buy(RuntimeOrigin::none(), work.0, work.1, 1_000),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn a_sale_the_buyer_cannot_pay_for_moves_nothing() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(set_terms(
            ALICE,
            work,
            &[(DAVE, percent(10))],
            Permill::zero()
        ));
        // Twice what Carol holds. Dave's 10% is paid first and succeeds inside
        // the transaction; the seller's part then fails, and Dave's is undone.
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 2 * START));
        let before = snapshot(&[ALICE, CAROL, DAVE]);
        assert_noop!(
            Royalties::buy(signed(CAROL), work.0, work.1, 2 * START),
            TokenError::FundsUnavailable
        );
        assert_eq!(changes(&before), vec![(ALICE, 0), (CAROL, 0), (DAVE, 0)]);
        assert_eq!(holder(work), Some(account(ALICE)));
    });
}

#[test]
fn a_part_that_cannot_be_received_refuses_the_whole_sale() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        // NOBODY has no account, and 1% of 500 is 5, below the existential
        // deposit of 10: the payment cannot create one.
        assert_ok!(set_terms(
            ALICE,
            work,
            &[(NOBODY, percent(1))],
            Permill::zero()
        ));
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 500));
        assert_noop!(
            Royalties::buy(signed(CAROL), work.0, work.1, 500),
            Error::<Test>::PaymentCannotBeReceived
        );
        assert_eq!(holder(work), Some(account(ALICE)));

        // At a price where the part reaches the existential deposit, it goes
        // through and creates the account.
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_000));
        assert_ok!(Royalties::buy(signed(CAROL), work.0, work.1, 1_000));
        assert_eq!(free(NOBODY), EXISTENTIAL_DEPOSIT);
        assert_eq!(holder(work), Some(account(CAROL)));
    });
}

// ---------------------------------------------------------------------------
// The work a buyer agreed to
// ---------------------------------------------------------------------------

#[test]
fn buy_exact_refuses_an_asset_revised_after_the_buyer_looked() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(set_terms(
            ALICE,
            work,
            &[(DAVE, percent(10))],
            Permill::zero()
        ));
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_000));

        // Carol looks at content 1. Before her purchase lands, Alice points the
        // asset at something else. The price has not moved.
        assert_ok!(Drc369::revise(
            signed(ALICE),
            work.0,
            work.1,
            content(2),
            None
        ));
        let before = snapshot(&[ALICE, CAROL, DAVE]);
        assert_noop!(
            Royalties::buy_exact(signed(CAROL), work.0, work.1, 1_000, content(1)),
            Error::<Test>::ContentChanged
        );
        assert_eq!(changes(&before), vec![(ALICE, 0), (CAROL, 0), (DAVE, 0)]);
        assert_eq!(holder(work), Some(account(ALICE)));
        assert!(Royalties::listing(work.0, work.1).is_some());

        // It is the reference the asset carries now that is sold, not the one
        // it was minted with: agreeing to the revision buys it.
        assert_ok!(Royalties::buy_exact(
            signed(CAROL),
            work.0,
            work.1,
            1_000,
            content(2)
        ));
        assert_eq!(
            changes(&before),
            vec![(ALICE, 900), (CAROL, -1_000), (DAVE, 100)]
        );
        assert_eq!(holder(work), Some(account(CAROL)));
        assert_eq!(Royalties::listing(work.0, work.1), None);
    });
}

#[test]
fn buy_exact_keeps_every_rule_buy_has() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_noop!(
            Royalties::buy_exact(signed(CAROL), work.0, work.1, 1_000, content(1)),
            Error::<Test>::NotListed
        );
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_000));
        assert_noop!(
            Royalties::buy_exact(signed(CAROL), work.0, work.1, 999, content(1)),
            Error::<Test>::PriceAboveLimit
        );
        assert_noop!(
            Royalties::buy_exact(signed(ALICE), work.0, work.1, 1_000, content(1)),
            Error::<Test>::OwnListing
        );
        assert_noop!(
            Royalties::buy_exact(RuntimeOrigin::none(), work.0, work.1, 1_000, content(1)),
            DispatchError::BadOrigin
        );
        assert_ok!(Nfts::transfer(signed(ALICE), work.0, work.1, account(BOB)));
        assert_noop!(
            Royalties::buy_exact(signed(CAROL), work.0, work.1, 1_000, content(1)),
            Error::<Test>::ListingStale
        );
    });
}

// ---------------------------------------------------------------------------
// Asking before paying
// ---------------------------------------------------------------------------

/// A remix of Alice's work, minted by Bob, with terms on both. Returns the
/// original and the remix.
fn a_remix_with_terms() -> ((CollectionId, ItemId), (CollectionId, ItemId)) {
    let original = mint(ALICE, 1, None);
    assert_ok!(set_terms(
        ALICE,
        original,
        &[(ALICE, percent(30)), (DAVE, percent(10))],
        percent(20)
    ));
    let remix = mint(BOB, 2, Some(original));
    assert_ok!(set_terms(
        BOB,
        remix,
        &[(BOB, percent(50))],
        Permill::zero()
    ));
    (original, remix)
}

#[test]
fn a_preview_names_the_parts_a_sale_then_pays() {
    new_test_ext().execute_with(|| {
        assert_eq!(Royalties::sale_preview(0, 0, 1_000, None), None);

        let (original, remix) = a_remix_with_terms();
        let expected = SalePreview {
            source: Some(original),
            remix: vec![(account(ALICE), 1_500), (account(DAVE), 500)],
            royalties: vec![(account(BOB), 4_000)],
            seller: account(BOB),
            seller_receives: 4_000,
            refusal: None,
        };
        // Asked before it is listed, with and without a buyer.
        assert_eq!(
            Royalties::sale_preview(remix.0, remix.1, 10_000, None),
            Some(expected.clone())
        );
        assert_eq!(
            Royalties::sale_preview(remix.0, remix.1, 10_000, Some(account(CAROL))),
            Some(expected.clone())
        );

        // The sale pays exactly what the preview said.
        assert_ok!(Royalties::list(signed(BOB), remix.0, remix.1, 10_000));
        assert_ok!(Royalties::buy(signed(CAROL), remix.0, remix.1, 10_000));
        match events().last() {
            Some(Event::Sold {
                source,
                remix: upstream,
                royalties,
                seller_received,
                from,
                ..
            }) => {
                assert_eq!(*source, expected.source);
                assert_eq!(*upstream, expected.remix);
                assert_eq!(*royalties, expected.royalties);
                assert_eq!(*seller_received, expected.seller_receives);
                assert_eq!(*from, expected.seller);
            }
            other => panic!("expected Sold, got {other:?}"),
        }
    });
}

#[test]
fn a_preview_with_a_buyer_moves_nothing() {
    new_test_ext().execute_with(|| {
        let (_, remix) = a_remix_with_terms();
        assert_ok!(Royalties::list(signed(BOB), remix.0, remix.1, 10_000));
        let before = snapshot(&[ALICE, BOB, CAROL, DAVE]);
        let events_before = System::events().len();

        let preview = Royalties::sale_preview(remix.0, remix.1, 10_000, Some(account(CAROL)));
        assert_eq!(preview.unwrap().refusal, None);

        assert_eq!(
            changes(&before),
            vec![(ALICE, 0), (BOB, 0), (CAROL, 0), (DAVE, 0)]
        );
        assert_eq!(holder(remix), Some(account(BOB)));
        assert!(Royalties::listing(remix.0, remix.1).is_some());
        assert_eq!(System::events().len(), events_before);
    });
}

#[test]
fn a_preview_says_why_a_sale_could_not_settle() {
    new_test_ext().execute_with(|| {
        let refusal = |asset: (CollectionId, ItemId), price, buyer: Option<u8>| {
            Royalties::sale_preview(asset.0, asset.1, price, buyer.map(account))
                .unwrap()
                .refusal
        };
        let work = mint(ALICE, 1, None);
        // NOBODY has no account, and 1% of 500 is 5, below the existential
        // deposit of 10: the part cannot be received, whoever buys.
        assert_ok!(set_terms(
            ALICE,
            work,
            &[(NOBODY, percent(1))],
            Permill::zero()
        ));
        let unreceivable: DispatchError = Error::<Test>::PaymentCannotBeReceived.into();
        assert_eq!(refusal(work, 500, None), Some(unreceivable));
        assert_eq!(refusal(work, 500, Some(CAROL)), Some(unreceivable));
        // And the sale itself agrees.
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 500));
        assert_noop!(
            Royalties::buy(signed(CAROL), work.0, work.1, 500),
            Error::<Test>::PaymentCannotBeReceived
        );
        // At a price where the part reaches the existential deposit, nothing
        // stops it.
        assert_eq!(refusal(work, 1_000, None), None);
        assert_eq!(refusal(work, 1_000, Some(CAROL)), None);

        // A price of nothing, the holder buying from themselves, and a buyer
        // without the money.
        assert_eq!(
            refusal(work, 0, None),
            Some(Error::<Test>::ZeroPrice.into())
        );
        assert_eq!(
            refusal(work, 0, Some(CAROL)),
            Some(Error::<Test>::ZeroPrice.into())
        );
        assert_eq!(
            refusal(work, 1_000, Some(ALICE)),
            Some(Error::<Test>::OwnListing.into())
        );
        assert_eq!(
            refusal(work, 2 * START, Some(CAROL)),
            Some(TokenError::FundsUnavailable.into())
        );
        // Without a buyer, nobody's funds are in question.
        assert_eq!(refusal(work, 2 * START, None), None);
    });
}

#[test]
fn a_preview_knows_an_earlier_part_opens_the_account_a_later_one_reaches() {
    new_test_ext().execute_with(|| {
        // NOBODY is owed twice by one sale: 500 as the source's recipient,
        // which opens their account, then 5 as the remix's own, which alone
        // would be below the existential deposit.
        let original = mint(ALICE, 1, None);
        assert_ok!(set_terms(
            ALICE,
            original,
            &[(NOBODY, percent(10))],
            percent(50)
        ));
        let remix = mint(BOB, 2, Some(original));
        assert_ok!(set_terms(
            BOB,
            remix,
            &[(NOBODY, percent(1))],
            Permill::zero()
        ));

        let preview = Royalties::sale_preview(remix.0, remix.1, 1_000, None).unwrap();
        assert_eq!(preview.remix, vec![(account(NOBODY), 500)]);
        assert_eq!(preview.royalties, vec![(account(NOBODY), 5)]);
        assert_eq!(preview.refusal, None);
        assert_eq!(
            Royalties::sale_preview(remix.0, remix.1, 1_000, Some(account(CAROL)))
                .unwrap()
                .refusal,
            None
        );

        // The sale agrees: it settles, and pays both parts.
        assert_ok!(Royalties::list(signed(BOB), remix.0, remix.1, 1_000));
        assert_ok!(Royalties::buy(signed(CAROL), remix.0, remix.1, 1_000));
        assert_eq!(free(NOBODY), 505);
    });
}

/// AGENTS.md §5 for the preview: it forms no intermediate of its own. At the
/// largest price a `u128` carries, with every share at its most, the parts come
/// from `split` unchanged and still sum to the price, and trying the sale for a
/// buyer is refused as the sale itself is, not ended by an overflow.
#[test]
fn a_preview_at_the_largest_price_cannot_overflow() {
    new_test_ext().execute_with(|| {
        let original = mint(ALICE, 1, None);
        assert_ok!(set_terms(
            ALICE,
            original,
            &[(ALICE, ppm(999_999)), (DAVE, ppm(1))],
            Permill::one()
        ));
        let remix = mint(BOB, 2, Some(original));
        assert_ok!(set_terms(
            BOB,
            remix,
            &[(DAVE, Permill::one())],
            Permill::zero()
        ));

        for price in [10u128.pow(32), u128::MAX - 1, u128::MAX] {
            let preview = Royalties::sale_preview(remix.0, remix.1, price, None).unwrap();
            let expected = split(
                price,
                Some((
                    Permill::one(),
                    &[(account(ALICE), ppm(999_999)), (account(DAVE), ppm(1))][..],
                )),
                &[(account(DAVE), Permill::one())],
            );
            assert_eq!(preview.remix, expected.remix);
            assert_eq!(preview.royalties, expected.royalties);
            assert_eq!(preview.seller_receives, expected.seller);
            let total = preview
                .remix
                .iter()
                .chain(preview.royalties.iter())
                .map(|(_, amount)| *amount)
                .sum::<u128>()
                + preview.seller_receives;
            assert_eq!(total, price);

            // Tried for a buyer, it is refused in the words the sale itself
            // uses, whatever those are at this size, and moves nothing.
            let before = snapshot(&[ALICE, BOB, CAROL, DAVE]);
            let tried = Royalties::sale_preview(remix.0, remix.1, price, Some(account(CAROL)))
                .unwrap()
                .refusal;
            assert_ok!(Royalties::list(signed(BOB), remix.0, remix.1, price));
            let sale = Royalties::buy(signed(CAROL), remix.0, remix.1, price);
            assert_eq!(tried, sale.err());
            assert!(tried.is_some());
            assert_eq!(
                changes(&before),
                vec![(ALICE, 0), (BOB, 0), (CAROL, 0), (DAVE, 0)]
            );
        }
    });
}
