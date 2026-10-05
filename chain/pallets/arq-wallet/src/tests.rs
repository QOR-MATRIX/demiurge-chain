//! Tests for ARQ Wallets (ADR-070). Each rule the module documentation states
//! has a test here; the ones that pin money were shown to fail against a
//! planted fault before they were trusted (`HANDOFF.md` records which).

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok, traits::fungible::Inspect};
use pallet_drc369::{CommitId, ContentRef, HashAlgo};
use sp_core::H256;

fn signed(who: u8) -> RuntimeOrigin {
    RuntimeOrigin::signed(account(who))
}

/// Mint a Cartridge to `who`.
fn cartridge(who: u8, seed: u8) -> (CollectionId, ItemId) {
    assert_ok!(Drc369::mint(
        signed(who),
        ContentRef {
            algo: HashAlgo::Blake3_256,
            root: H256::repeat_byte(seed),
            size: 1_000
        },
        Some(CommitId::Sha1([seed; 20])),
        b"game".to_vec().try_into().unwrap(),
        true,
        None,
    ));
    let singles = pallet_drc369::Singles::<Test>::get(account(who)).unwrap();
    (singles.collection, singles.next_item - 1)
}

fn rv(s: &str) -> RuleVersionOf<Test> {
    s.as_bytes().to_vec().try_into().unwrap()
}

fn policy() -> PolicyOf<Test> {
    Policy {
        max_payout: 50,
        epoch_blocks: 100,
        epoch_budget: 500,
        per_recipient_per_epoch: 100,
        rule_versions: vec![rv("flux@1")].try_into().unwrap(),
        accrual_expiry: 50,
        loosen_delay: 10,
    }
}

/// A Cartridge held by Alice, its wallet funded with `funding`, and Server as
/// its payout authority.
fn wallet(funding: Balance) -> (CollectionId, ItemId) {
    let (c, i) = cartridge(ALICE, 1);
    assert_ok!(ArqWallet::create(signed(ALICE), c, i, policy(), funding));
    assert_ok!(ArqWallet::set_authority(
        signed(ALICE),
        c,
        i,
        Some(account(SERVER))
    ));
    (c, i)
}

fn pay(c: CollectionId, i: ItemId, outcome: u8, to: u8, amount: Balance) -> DispatchResult {
    ArqWallet::payout(
        signed(SERVER),
        c,
        i,
        [outcome; 32],
        System::block_number() as u32,
        rv("flux@1"),
        account(to),
        amount,
    )
}

fn balance(who: &AccountId) -> Balance {
    <Balances as Inspect<AccountId>>::balance(who)
}

fn run_to(n: u64) {
    System::set_block_number(n);
}

#[test]
fn only_the_cartridge_holder_creates_a_wallet_and_it_is_keyless_and_funded() {
    new_test_ext().execute_with(|| {
        let (c, i) = cartridge(ALICE, 1);
        assert_noop!(
            ArqWallet::create(signed(BOB), c, i, policy(), 1_000),
            Error::<Test>::NotGovernor
        );
        assert_noop!(
            ArqWallet::create(signed(ALICE), c, i + 9, policy(), 1_000),
            Error::<Test>::UnknownCartridge
        );
        assert_noop!(
            ArqWallet::create(signed(ALICE), c, i, policy(), EXISTENTIAL_DEPOSIT - 1),
            Error::<Test>::BelowExistentialDeposit
        );
        assert_ok!(ArqWallet::create(signed(ALICE), c, i, policy(), 1_000));
        assert_noop!(
            ArqWallet::create(signed(ALICE), c, i, policy(), 1_000),
            Error::<Test>::WalletExists
        );

        let acct = ArqWallet::account_of(c, i);
        assert_eq!(balance(&acct), 1_000);
        // Derived, not a key: different from every named account, stable, per Cartridge.
        assert_ne!(acct, account(ALICE));
        assert_eq!(acct, ArqWallet::account_of(c, i));
        assert_ne!(acct, ArqWallet::account_of(c, i + 1));
        assert!(matches!(events()[0], Event::Created { funded: 1_000, .. }));
    });
}

#[test]
fn the_governor_is_whoever_holds_the_cartridge_now() {
    new_test_ext().execute_with(|| {
        let (c, i) = wallet(1_000);
        assert_ok!(Nfts::transfer(signed(ALICE), c, i, account(BOB)));
        assert_noop!(
            ArqWallet::set_paused(signed(ALICE), c, i, true),
            Error::<Test>::NotGovernor
        );
        assert_ok!(ArqWallet::set_paused(signed(BOB), c, i, true));
    });
}

#[test]
fn an_unsound_policy_is_refused() {
    new_test_ext().execute_with(|| {
        let (c, i) = cartridge(ALICE, 1);
        let bad = |f: fn(&mut PolicyOf<Test>)| {
            let mut p = policy();
            f(&mut p);
            ArqWallet::create(signed(ALICE), c, i, p, 1_000)
        };
        assert_noop!(bad(|p| p.max_payout = 0), Error::<Test>::ZeroLimit);
        assert_noop!(bad(|p| p.max_payout = 101), Error::<Test>::LimitsOutOfOrder);
        assert_noop!(
            bad(|p| p.per_recipient_per_epoch = 501),
            Error::<Test>::LimitsOutOfOrder
        );
        assert_noop!(bad(|p| p.epoch_blocks = 0), Error::<Test>::ZeroEpoch);
        assert_noop!(
            bad(|p| p.accrual_expiry = MAX_ACCRUAL_EXPIRY + 1),
            Error::<Test>::AccrualExpiryOutOfBounds
        );
        assert_noop!(
            bad(|p| p.loosen_delay = MIN_LOOSEN_DELAY - 1),
            Error::<Test>::DelayTooShort
        );
        assert_noop!(
            bad(|p| p.rule_versions = Default::default()),
            Error::<Test>::BadRuleVersions
        );
        assert_noop!(
            bad(|p| p.rule_versions = vec![rv("a@1"), rv("a@1")].try_into().unwrap()),
            Error::<Test>::BadRuleVersions
        );
    });
}

#[test]
fn tightening_applies_at_once_and_loosening_waits_its_delay() {
    new_test_ext().execute_with(|| {
        let (c, i) = wallet(1_000);
        let mut tighter = policy();
        tighter.max_payout = 10;
        assert_ok!(ArqWallet::set_policy(signed(ALICE), c, i, tighter.clone()));
        assert_eq!(Wallets::<Test>::get(c, i).unwrap().policy, tighter);

        let mut looser = tighter.clone();
        looser.max_payout = 90;
        assert_ok!(ArqWallet::set_policy(signed(ALICE), c, i, looser.clone()));
        assert_eq!(Wallets::<Test>::get(c, i).unwrap().policy, tighter);
        assert_noop!(
            ArqWallet::apply_policy(signed(BOB), c, i),
            Error::<Test>::NotDue
        );
        run_to(1 + 10);
        assert_ok!(ArqWallet::apply_policy(signed(BOB), c, i));
        assert_eq!(Wallets::<Test>::get(c, i).unwrap().policy, looser);

        // And the governor can cancel during the delay.
        let mut loosest = looser.clone();
        loosest.epoch_budget = 1_000;
        assert_ok!(ArqWallet::set_policy(signed(ALICE), c, i, loosest));
        assert_ok!(ArqWallet::cancel_pending(signed(ALICE), c, i));
        run_to(100);
        assert_noop!(
            ArqWallet::apply_policy(signed(BOB), c, i),
            Error::<Test>::NothingPending
        );
    });
}

#[test]
fn every_loosening_is_recognised_field_by_field() {
    let p = policy();
    assert!(loosened(&p, &p).is_empty());
    let check = |f: fn(&mut PolicyOf<Test>), field: &str| {
        let mut n = policy();
        f(&mut n);
        assert_eq!(loosened(&p, &n), vec![field]);
    };
    check(|n| n.max_payout = 51, "max_payout");
    check(|n| n.epoch_blocks = 99, "epoch_blocks");
    check(|n| n.epoch_budget = 501, "epoch_budget");
    check(
        |n| n.per_recipient_per_epoch = 101,
        "per_recipient_per_epoch",
    );
    check(
        |n| n.rule_versions = vec![rv("flux@1"), rv("flux@2")].try_into().unwrap(),
        "rule_versions",
    );
    check(|n| n.accrual_expiry = 51, "accrual_expiry");
    check(|n| n.loosen_delay = 9, "loosen_delay");
    let mut tighter = policy();
    tighter.max_payout = 1;
    tighter.epoch_blocks = 200;
    tighter.rule_versions = Default::default();
    assert!(loosened(&p, &tighter).is_empty());
}

#[test]
fn only_the_authority_pays_and_only_within_the_policy() {
    new_test_ext().execute_with(|| {
        let (c, i) = wallet(1_000);
        let now = System::block_number() as u32;
        assert_noop!(
            ArqWallet::payout(
                signed(ALICE),
                c,
                i,
                [1; 32],
                now,
                rv("flux@1"),
                account(PLAYER),
                5
            ),
            Error::<Test>::NotAuthority
        );
        assert_noop!(pay(c, i, 1, PLAYER, 0), Error::<Test>::ZeroAmount);
        assert_noop!(pay(c, i, 1, PLAYER, 51), Error::<Test>::AboveMaxPayout);
        assert_noop!(
            ArqWallet::payout(
                signed(SERVER),
                c,
                i,
                [1; 32],
                now,
                rv("flux@2"),
                account(PLAYER),
                5
            ),
            Error::<Test>::RuleVersionNotAllowed
        );
        assert_ok!(ArqWallet::set_paused(signed(ALICE), c, i, true));
        assert_noop!(pay(c, i, 1, PLAYER, 5), Error::<Test>::Paused);
        assert_ok!(ArqWallet::set_paused(signed(ALICE), c, i, false));

        assert_ok!(pay(c, i, 1, PLAYER, 50));
        assert_eq!(balance(&account(PLAYER)), START + 50);
        assert_eq!(balance(&ArqWallet::account_of(c, i)), 950);

        // A revoked authority pays nothing.
        assert_ok!(ArqWallet::set_authority(signed(ALICE), c, i, None));
        assert_noop!(pay(c, i, 2, PLAYER, 5), Error::<Test>::NotAuthority);
    });
}

#[test]
fn each_outcome_is_paid_once_and_only_inside_its_window() {
    new_test_ext().execute_with(|| {
        let (c, i) = wallet(1_000);
        run_to(30);
        assert_ok!(pay(c, i, 7, PLAYER, 10));
        assert_noop!(pay(c, i, 7, PLAYER, 10), Error::<Test>::OutcomePaid);
        assert_noop!(pay(c, i, 7, BOB, 1), Error::<Test>::OutcomePaid);

        let pay_at = |issued: u32| {
            ArqWallet::payout(
                signed(SERVER),
                c,
                i,
                [8; 32],
                issued,
                rv("flux@1"),
                account(PLAYER),
                1,
            )
        };
        assert_noop!(pay_at(31), Error::<Test>::OutsideOutcomeWindow);
        assert_noop!(
            pay_at(30 - OUTCOME_WINDOW - 1),
            Error::<Test>::OutsideOutcomeWindow
        );
        assert_ok!(pay_at(30 - OUTCOME_WINDOW));

        // Once the window has passed, the record can go, and the outcome still
        // cannot be paid again: its issue block is outside the window.
        assert_noop!(
            ArqWallet::prune_outcome(signed(BOB), c, i, [7; 32]),
            Error::<Test>::OutcomeStillRemembered
        );
        run_to(30 + OUTCOME_WINDOW as u64 + 1);
        assert_ok!(ArqWallet::prune_outcome(signed(BOB), c, i, [7; 32]));
        assert_noop!(
            ArqWallet::payout(
                signed(SERVER),
                c,
                i,
                [7; 32],
                30,
                rv("flux@1"),
                account(PLAYER),
                10
            ),
            Error::<Test>::OutsideOutcomeWindow
        );
    });
}

#[test]
fn epoch_budgets_recipient_caps_and_payout_counts_hold_and_reset() {
    new_test_ext().execute_with(|| {
        let (c, i) = wallet(10_000);
        assert_ok!(pay(c, i, 1, PLAYER, 50));
        assert_ok!(pay(c, i, 2, PLAYER, 50));
        assert_noop!(pay(c, i, 3, PLAYER, 1), Error::<Test>::RecipientCapExceeded);
        assert_ok!(pay(c, i, 3, BOB, 50));
        assert_noop!(pay(c, i, 4, ALICE, 1), Error::<Test>::TooManyPayouts);

        run_to(100);
        assert_ok!(pay(c, i, 5, PLAYER, 50));

        // The budget, with the payout count out of the way.
        let mut p = policy();
        p.epoch_budget = 120;
        assert_ok!(ArqWallet::set_policy(signed(ALICE), c, i, p));
        run_to(200);
        assert_ok!(pay(c, i, 6, PLAYER, 50));
        assert_ok!(pay(c, i, 7, BOB, 50));
        assert_noop!(pay(c, i, 8, ALICE, 21), Error::<Test>::EpochBudgetExceeded);
        assert_ok!(pay(c, i, 8, ALICE, 20));
    });
}

#[test]
fn a_payout_never_spends_the_deposit_or_what_is_owed() {
    new_test_ext().execute_with(|| {
        let (c, i) = wallet(EXISTENTIAL_DEPOSIT + 30);
        assert_noop!(pay(c, i, 1, PLAYER, 31), Error::<Test>::InsufficientFunds);
        // 8 cannot open NOBODY's account, so it is owed instead: 22 left to pay.
        assert_ok!(pay(c, i, 1, NOBODY, 8));
        assert_eq!(Wallets::<Test>::get(c, i).unwrap().owed, 8);
        assert_noop!(pay(c, i, 2, PLAYER, 23), Error::<Test>::InsufficientFunds);
        assert_ok!(pay(c, i, 2, PLAYER, 22));
        assert_eq!(
            balance(&ArqWallet::account_of(c, i)),
            EXISTENTIAL_DEPOSIT + 8
        );
    });
}

#[test]
fn a_payout_too_small_to_open_an_account_accrues_and_is_claimed_once_it_can_be() {
    new_test_ext().execute_with(|| {
        let (c, i) = wallet(1_000);
        assert_ok!(pay(c, i, 1, NOBODY, 6));
        assert!(matches!(
            events().last(),
            Some(Event::Accrued { amount: 6, .. })
        ));
        assert_eq!(balance(&account(NOBODY)), 0);
        assert_noop!(
            ArqWallet::claim(signed(NOBODY), c, i),
            Error::<Test>::CannotReceiveYet
        );
        assert_noop!(
            ArqWallet::claim(signed(PLAYER), c, i),
            Error::<Test>::NothingOwed
        );

        assert_ok!(pay(c, i, 2, NOBODY, 6));
        assert_eq!(
            Accruals::<Test>::get((c, i), account(NOBODY)).unwrap().0,
            12
        );
        assert_ok!(ArqWallet::claim(signed(NOBODY), c, i));
        assert_eq!(balance(&account(NOBODY)), 12);
        assert_eq!(Wallets::<Test>::get(c, i).unwrap().owed, 0);
        assert_noop!(
            ArqWallet::claim(signed(NOBODY), c, i),
            Error::<Test>::NothingOwed
        );
    });
}

#[test]
fn an_unclaimed_amount_expires_back_to_the_wallet() {
    new_test_ext().execute_with(|| {
        let (c, i) = wallet(1_000);
        assert_ok!(pay(c, i, 1, NOBODY, 6));
        let expires = Accruals::<Test>::get((c, i), account(NOBODY)).unwrap().1;
        assert_noop!(
            ArqWallet::expire_accrual(signed(BOB), c, i, account(NOBODY)),
            Error::<Test>::AccrualNotExpired
        );
        run_to(expires as u64 + 1);
        assert_noop!(
            ArqWallet::claim(signed(NOBODY), c, i),
            Error::<Test>::AccrualExpired
        );
        assert_ok!(ArqWallet::expire_accrual(
            signed(BOB),
            c,
            i,
            account(NOBODY)
        ));
        assert_eq!(Wallets::<Test>::get(c, i).unwrap().owed, 0);
    });
}

#[test]
fn a_withdrawal_waits_its_delay_and_never_takes_what_is_owed_or_the_deposit() {
    new_test_ext().execute_with(|| {
        let (c, i) = wallet(EXISTENTIAL_DEPOSIT + 100);
        assert_noop!(
            ArqWallet::schedule_withdrawal(signed(BOB), c, i, account(BOB), 10),
            Error::<Test>::NotGovernor
        );
        assert_noop!(
            ArqWallet::schedule_withdrawal(signed(ALICE), c, i, account(ALICE), 101),
            Error::<Test>::InsufficientFunds
        );
        assert_ok!(ArqWallet::schedule_withdrawal(
            signed(ALICE),
            c,
            i,
            account(ALICE),
            100
        ));
        assert_noop!(
            ArqWallet::schedule_withdrawal(signed(ALICE), c, i, account(ALICE), 1),
            Error::<Test>::WithdrawalPending
        );
        assert_noop!(
            ArqWallet::execute_withdrawal(signed(BOB), c, i),
            Error::<Test>::NotDue
        );

        // During the delay a player becomes owed 8; the withdrawal can no longer take all 100.
        assert_ok!(pay(c, i, 1, NOBODY, 8));
        run_to(1 + 10);
        assert_noop!(
            ArqWallet::execute_withdrawal(signed(BOB), c, i),
            Error::<Test>::InsufficientFunds
        );

        assert_ok!(ArqWallet::cancel_pending(signed(ALICE), c, i));
        assert_ok!(ArqWallet::schedule_withdrawal(
            signed(ALICE),
            c,
            i,
            account(ALICE),
            92
        ));
        run_to(1 + 10 + 10);
        let before = balance(&account(ALICE));
        assert_ok!(ArqWallet::execute_withdrawal(signed(BOB), c, i));
        assert_eq!(balance(&account(ALICE)), before + 92);
        // The deposit and what is owed stay.
        assert_eq!(
            balance(&ArqWallet::account_of(c, i)),
            EXISTENTIAL_DEPOSIT + 8
        );
    });
}

#[test]
fn the_largest_amounts_cannot_overflow() {
    new_test_ext().execute_with(|| {
        let (c, i) = cartridge(ALICE, 1);
        let mut p = policy();
        p.max_payout = Balance::MAX;
        p.per_recipient_per_epoch = Balance::MAX;
        p.epoch_budget = Balance::MAX;
        assert_ok!(ArqWallet::create(signed(ALICE), c, i, p, 1_000));
        assert_ok!(ArqWallet::set_authority(
            signed(ALICE),
            c,
            i,
            Some(account(SERVER))
        ));
        // Spent nearly everything in the epoch's books, then ask for more:
        // refused as an overflow, not wrapped, and nothing moves.
        Wallets::<Test>::mutate(c, i, |w| w.as_mut().unwrap().epoch_paid = Balance::MAX - 1);
        assert_noop!(pay(c, i, 1, PLAYER, 2), Error::<Test>::Overflow);
        RecipientPaid::<Test>::insert(
            (c, i),
            account(PLAYER),
            (Wallets::<Test>::get(c, i).unwrap().epoch, Balance::MAX),
        );
        Wallets::<Test>::mutate(c, i, |w| w.as_mut().unwrap().epoch_paid = 0);
        assert_noop!(pay(c, i, 1, PLAYER, 1), Error::<Test>::Overflow);
        // An owed total at the limit leaves nothing available, and does not underflow.
        Wallets::<Test>::mutate(c, i, |w| w.as_mut().unwrap().owed = Balance::MAX);
        assert_eq!(ArqWallet::available(c, i, Balance::MAX), 0);
    });
}
