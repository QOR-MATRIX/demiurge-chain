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

// ---------------------------------------------------------------------------
// Renting (ADR-119)
// ---------------------------------------------------------------------------

/// Offer `asset` from `who` at `price` a period of `period` blocks, for at most
/// `max` periods, to anyone.
fn offer(
    who: u8,
    asset: (CollectionId, ItemId),
    price: Balance,
    period: u64,
    max: u32,
) -> DispatchResult {
    Royalties::offer_rental(signed(who), asset.0, asset.1, price, period, max, None)
}

/// Rent `asset` for `periods`, agreeing to pay up to `max_price` for the content
/// minted from `seed`.
fn rent(
    who: u8,
    asset: (CollectionId, ItemId),
    periods: u32,
    max_price: Balance,
    seed: u8,
) -> DispatchResult {
    Royalties::rent(
        signed(who),
        asset.0,
        asset.1,
        periods,
        max_price,
        content(seed),
    )
}

fn transfer(who: u8, asset: (CollectionId, ItemId), to: u8) -> DispatchResult {
    Nfts::transfer(signed(who), asset.0, asset.1, account(to))
}

#[test]
fn rent_pays_the_royalties_then_the_holder_and_makes_the_renter_its_user() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(set_terms(
            ALICE,
            work,
            &[(DAVE, percent(10))],
            Permill::zero()
        ));
        assert_ok!(offer(ALICE, work, 100, 10, 5));
        assert_eq!(
            Royalties::rental_offer(work.0, work.1),
            Some(RentalOffer {
                owner: account(ALICE),
                price_per_period: 100,
                period: 10,
                max_periods: 5,
                renter: None,
            })
        );

        let before = snapshot(&[ALICE, CAROL, DAVE]);
        assert_ok!(rent(CAROL, work, 3, 300, 1));

        // Three periods of 100: Dave's 10% is paid on rent as on a sale, and the
        // holder receives the rest. The asset does not move.
        assert_eq!(
            changes(&before),
            vec![(ALICE, 270), (CAROL, -300), (DAVE, 30)]
        );
        assert_eq!(holder(work), Some(account(ALICE)));
        assert_eq!(Royalties::user_of(work.0, work.1), Some(account(CAROL)));
        assert!(Royalties::is_rented(work.0, work.1));
        // Rented at block 1 for thirty blocks.
        assert_eq!(
            Royalties::rental(work.0, work.1),
            Some(Rental {
                owner: account(ALICE),
                renter: account(CAROL),
                until: 31,
            })
        );
        assert_eq!(Royalties::rentals_of(&account(CAROL)), vec![(work, 31)]);
        assert_eq!(
            events().last(),
            Some(&Event::Rented {
                collection: work.0,
                item: work.1,
                owner: account(ALICE),
                renter: account(CAROL),
                periods: 3,
                until: 31,
                price: 300,
                source: None,
                remix: vec![],
                royalties: vec![(account(DAVE), 30)],
                owner_received: 270,
            })
        );
        // The offer stays, to serve the next rental.
        assert!(Royalties::rental_offer(work.0, work.1).is_some());
    });
}

#[test]
fn rent_of_a_remix_pays_its_source_first() {
    new_test_ext().execute_with(|| {
        // Alice's original: 30% to Alice and 10% to Dave, and a 20% remix share.
        // Bob's remix: 50% to Bob.
        let (_, remix) = a_remix_with_terms();
        assert_ok!(offer(BOB, remix, 1_000, 5, 1));
        let before = snapshot(&[ALICE, BOB, CAROL, DAVE]);
        assert_ok!(rent(CAROL, remix, 1, 1_000, 2));
        // The pool is 200, divided three to one; Bob's 50% of the 800 left is
        // 400, and Bob as holder receives the other 400.
        assert_eq!(
            changes(&before),
            vec![(ALICE, 150), (BOB, 800), (CAROL, -1_000), (DAVE, 50)]
        );
    });
}

#[test]
fn a_rental_ends_by_itself_at_its_block() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(offer(ALICE, work, 100, 10, 2));
        assert_ok!(rent(CAROL, work, 1, 100, 1));

        System::set_block_number(10);
        assert!(Royalties::is_rented(work.0, work.1));
        assert_eq!(Royalties::user_of(work.0, work.1), Some(account(CAROL)));
        assert_noop!(
            transfer(ALICE, work, BOB),
            pallet_nfts::Error::<Test>::ItemLocked
        );

        // Rented at block 1 for ten blocks: over at block 11, with nothing sent.
        System::set_block_number(11);
        assert!(!Royalties::is_rented(work.0, work.1));
        assert_eq!(Royalties::rental(work.0, work.1), None);
        assert_eq!(Royalties::user_of(work.0, work.1), Some(account(ALICE)));
        assert_eq!(Royalties::rentals_of(&account(CAROL)), vec![]);
        assert_ok!(transfer(ALICE, work, BOB));
        assert_eq!(Royalties::user_of(work.0, work.1), Some(account(BOB)));
    });
}

#[test]
fn a_rented_asset_is_held_in_place() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        let other = mint(ALICE, 2, None);
        assert_ok!(offer(ALICE, work, 100, 10, 1));
        assert_ok!(rent(CAROL, work, 1, 100, 1));

        // Not moved, by a transfer or by anyone Alice approved.
        assert_noop!(
            transfer(ALICE, work, BOB),
            pallet_nfts::Error::<Test>::ItemLocked
        );
        assert_ok!(Nfts::approve_transfer(
            signed(ALICE),
            work.0,
            work.1,
            account(BOB),
            None
        ));
        assert_noop!(
            transfer(BOB, work, BOB),
            pallet_nfts::Error::<Test>::ItemLocked
        );
        // Not sold.
        assert_noop!(
            Royalties::list(signed(ALICE), work.0, work.1, 1_000),
            Error::<Test>::Rented
        );
        // Not changed: what the renter rented is what they have.
        assert_noop!(
            Drc369::revise(signed(ALICE), work.0, work.1, content(9), None),
            pallet_drc369::Error::<Test>::Rented
        );
        assert_noop!(
            Drc369::nest(signed(ALICE), work.0, work.1, other),
            pallet_drc369::Error::<Test>::Rented
        );
        assert_noop!(
            Drc369::nest(signed(ALICE), other.0, other.1, work),
            pallet_drc369::Error::<Test>::Rented
        );
        // Not rented twice.
        assert_noop!(rent(BOB, work, 1, 100, 1), Error::<Test>::Rented);

        // Once it ends, all of it is the holder's again.
        System::set_block_number(11);
        assert_ok!(Drc369::nest(signed(ALICE), other.0, other.1, work));
        assert_ok!(Drc369::unnest(signed(ALICE), other.0, other.1));
        assert_ok!(Drc369::revise(
            signed(ALICE),
            work.0,
            work.1,
            content(9),
            None
        ));
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_000));
    });
}

#[test]
fn nothing_inside_a_rented_asset_is_taken_out() {
    new_test_ext().execute_with(|| {
        let car = mint(ALICE, 1, None);
        let engine = mint(ALICE, 2, None);
        assert_ok!(Drc369::nest(signed(ALICE), engine.0, engine.1, car));
        assert_ok!(offer(ALICE, car, 0, 10, 1));
        assert_ok!(rent(CAROL, car, 1, 0, 1));
        assert_noop!(
            Drc369::unnest(signed(ALICE), engine.0, engine.1),
            pallet_drc369::Error::<Test>::Rented
        );
        System::set_block_number(11);
        assert_ok!(Drc369::unnest(signed(ALICE), engine.0, engine.1));
    });
}

#[test]
fn listed_and_rented_never_at_once() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        // An offer and a listing may stand together.
        assert_ok!(offer(ALICE, work, 100, 10, 1));
        assert_ok!(Royalties::list(signed(ALICE), work.0, work.1, 1_000));
        assert_noop!(rent(CAROL, work, 1, 100, 1), Error::<Test>::Listed);

        assert_ok!(Royalties::unlist(signed(ALICE), work.0, work.1));
        assert_ok!(rent(CAROL, work, 1, 100, 1));
        assert_noop!(
            Royalties::list(signed(ALICE), work.0, work.1, 1_000),
            Error::<Test>::Rented
        );
    });
}

#[test]
fn the_holder_cannot_end_a_rental_early_and_the_renter_can_with_no_refund() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(offer(ALICE, work, 100, 10, 3));
        assert_ok!(rent(CAROL, work, 3, 300, 1));

        assert_noop!(
            Royalties::end_rental(signed(ALICE), work.0, work.1),
            Error::<Test>::NotRenter
        );
        assert_noop!(
            Royalties::end_rental(signed(BOB), work.0, work.1),
            Error::<Test>::NotRenter
        );

        let before = snapshot(&[ALICE, CAROL]);
        assert_ok!(Royalties::end_rental(signed(CAROL), work.0, work.1));
        assert_eq!(changes(&before), vec![(ALICE, 0), (CAROL, 0)]);
        assert!(!Royalties::is_rented(work.0, work.1));
        assert_eq!(Royalties::rentals_of(&account(CAROL)), vec![]);
        assert_eq!(
            events().last(),
            Some(&Event::RentalEnded {
                collection: work.0,
                item: work.1,
                renter: account(CAROL),
                by: account(CAROL),
                early: true,
            })
        );
        assert_ok!(transfer(ALICE, work, BOB));
        assert_noop!(
            Royalties::end_rental(signed(CAROL), work.0, work.1),
            Error::<Test>::NotRented
        );
    });
}

#[test]
fn anyone_clears_a_rental_that_has_ended() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(offer(ALICE, work, 100, 10, 1));
        assert_ok!(rent(CAROL, work, 1, 100, 1));
        System::set_block_number(11);
        assert_ok!(Royalties::end_rental(signed(DAVE), work.0, work.1));
        assert_eq!(Rentals::<Test>::get(work.0, work.1), None);
        assert!(!RentalsOf::<Test>::contains_key(account(CAROL), work));
        assert_eq!(
            events().last(),
            Some(&Event::RentalEnded {
                collection: work.0,
                item: work.1,
                renter: account(CAROL),
                by: account(DAVE),
                early: false,
            })
        );
    });
}

#[test]
fn a_new_rental_after_one_ends_replaces_its_record() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(offer(ALICE, work, 100, 10, 1));
        assert_ok!(rent(CAROL, work, 1, 100, 1));
        System::set_block_number(11);
        assert_ok!(rent(BOB, work, 1, 100, 1));
        assert_eq!(Royalties::user_of(work.0, work.1), Some(account(BOB)));
        assert_eq!(Royalties::rentals_of(&account(BOB)), vec![(work, 21)]);
        // Carol's ended rental leaves nothing behind in the index.
        assert!(!RentalsOf::<Test>::contains_key(account(CAROL), work));
    });
}

#[test]
fn an_offer_to_one_account_is_rented_by_it_alone() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(Royalties::offer_rental(
            signed(ALICE),
            work.0,
            work.1,
            100,
            10,
            1,
            Some(account(CAROL))
        ));
        assert_noop!(rent(BOB, work, 1, 100, 1), Error::<Test>::NotOfferedToYou);
        assert_ok!(rent(CAROL, work, 1, 100, 1));
    });
}

#[test]
fn a_renter_pays_no_more_than_they_agreed_for_the_work_they_saw() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(offer(ALICE, work, 100, 10, 3));
        assert_noop!(rent(CAROL, work, 3, 299, 1), Error::<Test>::PriceAboveLimit);
        assert_noop!(
            rent(CAROL, work, 0, 300, 1),
            Error::<Test>::PeriodsOutOfRange
        );
        assert_noop!(
            rent(CAROL, work, 4, 400, 1),
            Error::<Test>::PeriodsOutOfRange
        );
        // Revised after Carol looked: refused whole.
        assert_ok!(Drc369::revise(
            signed(ALICE),
            work.0,
            work.1,
            content(9),
            None
        ));
        assert_noop!(rent(CAROL, work, 1, 100, 1), Error::<Test>::ContentChanged);
        assert_ok!(rent(CAROL, work, 1, 100, 9));
        // The holder cannot rent their own asset.
        let mine = mint(BOB, 2, None);
        assert_ok!(offer(BOB, mine, 100, 10, 1));
        assert_noop!(rent(BOB, mine, 1, 100, 2), Error::<Test>::OwnAsset);
    });
}

#[test]
fn an_offer_is_void_once_its_maker_no_longer_holds_the_asset() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(offer(ALICE, work, 100, 10, 1));
        // A live offer is withdrawn by its maker or the holder, nobody else.
        assert_noop!(
            Royalties::withdraw_rental_offer(signed(BOB), work.0, work.1),
            Error::<Test>::NotOwner
        );
        assert_ok!(transfer(ALICE, work, BOB));
        assert_noop!(rent(CAROL, work, 1, 100, 1), Error::<Test>::OfferStale);
        // Void, it is anyone's to clear.
        assert_ok!(Royalties::withdraw_rental_offer(
            signed(DAVE),
            work.0,
            work.1
        ));
        assert_eq!(Royalties::rental_offer(work.0, work.1), None);
        assert_noop!(rent(CAROL, work, 1, 100, 1), Error::<Test>::NotOffered);
    });
}

#[test]
fn offers_must_be_well_formed() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_noop!(offer(BOB, work, 100, 10, 1), Error::<Test>::NotOwner);
        assert_noop!(
            offer(ALICE, (99, 0), 100, 10, 1),
            Error::<Test>::UnknownAsset
        );
        assert_noop!(offer(ALICE, work, 100, 0, 1), Error::<Test>::NoPeriods);
        assert_noop!(offer(ALICE, work, 100, 10, 0), Error::<Test>::NoPeriods);
        assert_noop!(
            offer(ALICE, work, 100, u64::MAX, 2),
            Error::<Test>::RentalTooLong
        );
        assert_noop!(
            Royalties::offer_rental(
                signed(ALICE),
                work.0,
                work.1,
                100,
                10,
                1,
                Some(account(ALICE))
            ),
            Error::<Test>::OwnAsset
        );
    });
}

#[test]
fn a_loan_moves_no_cgt_and_holds_the_asset_as_a_rental_does() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(set_terms(
            ALICE,
            work,
            &[(DAVE, percent(10))],
            Permill::zero()
        ));
        assert_ok!(offer(ALICE, work, 0, 100, 1));
        let before = snapshot(&[ALICE, CAROL, DAVE]);
        assert_ok!(rent(CAROL, work, 1, 0, 1));
        assert_eq!(changes(&before), vec![(ALICE, 0), (CAROL, 0), (DAVE, 0)]);
        assert_eq!(Royalties::user_of(work.0, work.1), Some(account(CAROL)));
        assert_noop!(
            Royalties::end_rental(signed(ALICE), work.0, work.1),
            Error::<Test>::NotRenter
        );
    });
}

#[test]
fn a_part_that_cannot_be_received_refuses_the_whole_rental() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        // 1% of 500 is 5, below the existential deposit of 10.
        assert_ok!(set_terms(
            ALICE,
            work,
            &[(NOBODY, percent(1))],
            Permill::zero()
        ));
        assert_ok!(offer(ALICE, work, 500, 10, 2));
        assert_noop!(
            rent(CAROL, work, 1, 500, 1),
            Error::<Test>::PaymentCannotBeReceived
        );
        assert!(!Royalties::is_rented(work.0, work.1));
        // Two periods make the part 10, enough to open the account.
        assert_ok!(rent(CAROL, work, 2, 1_000, 1));
        assert_eq!(free(NOBODY), EXISTENTIAL_DEPOSIT);
    });
}

#[test]
fn rent_too_large_to_count_or_ending_too_late_is_refused() {
    new_test_ext().execute_with(|| {
        let work = mint(ALICE, 1, None);
        assert_ok!(offer(ALICE, work, u128::MAX, 10, 2));
        assert_noop!(
            rent(CAROL, work, 2, u128::MAX, 1),
            Error::<Test>::RentTooLarge
        );

        let late = mint(ALICE, 2, None);
        assert_ok!(offer(ALICE, late, 1, u64::MAX / 2, 2));
        System::set_block_number(u64::MAX / 2);
        assert_noop!(rent(CAROL, late, 2, 2, 2), Error::<Test>::RentalTooLong);
    });
}

#[test]
fn the_largest_rent_cannot_overflow() {
    new_test_ext().execute_with(|| {
        // A whole-price remix share, a whole-price royalty, and the largest rent:
        // the intermediate `split` forms is pinned at `u128::MAX` here as for a
        // sale, and the multiplication by the periods is checked (ADR-035).
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
        assert_ok!(offer(BOB, remix, u128::MAX, 10, 1));

        let preview = Royalties::rent_preview(remix.0, remix.1, 1, None).unwrap();
        assert_eq!(preview.price, u128::MAX);
        let total = preview
            .remix
            .iter()
            .chain(preview.royalties.iter())
            .map(|(_, amount)| *amount)
            .sum::<u128>()
            + preview.owner_receives;
        assert_eq!(total, u128::MAX);

        // Tried for Carol, it is refused in the rental's own words and moves
        // nothing.
        let before = snapshot(&[ALICE, BOB, CAROL, DAVE]);
        let tried = Royalties::rent_preview(remix.0, remix.1, 1, Some(account(CAROL)))
            .unwrap()
            .refusal;
        let rental = rent(CAROL, remix, 1, u128::MAX, 2);
        assert!(tried.is_some());
        assert_eq!(tried, rental.err());
        assert_eq!(
            changes(&before),
            vec![(ALICE, 0), (BOB, 0), (CAROL, 0), (DAVE, 0)]
        );
    });
}

#[test]
fn a_rent_preview_names_the_parts_a_rental_then_pays_and_why_it_could_not() {
    new_test_ext().execute_with(|| {
        let (_, remix) = a_remix_with_terms();
        assert_eq!(Royalties::rent_preview(remix.0, remix.1, 1, None), None);
        assert_ok!(offer(BOB, remix, 1_000, 5, 2));

        let preview = Royalties::rent_preview(remix.0, remix.1, 2, None).unwrap();
        assert_eq!(preview.price, 2_000);
        assert_eq!(preview.until, 11);
        assert_eq!(preview.owner, account(BOB));
        assert_eq!(preview.refusal, None);

        // With a renter, it is the transaction run and undone.
        let before = snapshot(&[ALICE, BOB, CAROL, DAVE]);
        let tried = Royalties::rent_preview(remix.0, remix.1, 2, Some(account(CAROL))).unwrap();
        assert_eq!(tried.refusal, None);
        assert_eq!(
            changes(&before),
            vec![(ALICE, 0), (BOB, 0), (CAROL, 0), (DAVE, 0)]
        );
        assert!(!Royalties::is_rented(remix.0, remix.1));

        assert_ok!(rent(CAROL, remix, 2, 2_000, 2));
        assert_eq!(
            changes(&before),
            vec![
                (ALICE, preview.remix[0].1 as i128),
                (
                    BOB,
                    (preview.royalties[0].1 + preview.owner_receives) as i128
                ),
                (CAROL, -2_000),
                (DAVE, preview.remix[1].1 as i128)
            ]
        );

        // Rented now: every renter would be refused, and says so.
        assert_eq!(
            Royalties::rent_preview(remix.0, remix.1, 1, None)
                .unwrap()
                .refusal,
            Some(Error::<Test>::Rented.into())
        );
        assert_eq!(
            Royalties::rent_preview(remix.0, remix.1, 3, None)
                .unwrap()
                .refusal,
            Some(Error::<Test>::Rented.into())
        );
        System::set_block_number(11);
        assert_eq!(
            Royalties::rent_preview(remix.0, remix.1, 3, None)
                .unwrap()
                .refusal,
            Some(Error::<Test>::PeriodsOutOfRange.into())
        );
    });
}
