//! The escrow, the challenges and the receipts (ADR-096; MESH_PROOFS.md §2 and §3), against real BLAKE3 proofs and real
//! Sr25519 signatures.

use frame_support::{
    assert_noop, assert_ok,
    traits::{fungible::Inspect, ConstU32},
    BoundedVec,
};
use polkadot_sdk::*;
use sp_core::Pair;
use sp_runtime::{DispatchError, MultiSignature};

use crate::mock::*;
use crate::{Error, Escrows, NextEscrowId, Receipt, Takedowns, MIB};

fn signed(id: u8) -> RuntimeOrigin {
    RuntimeOrigin::signed(account(id))
}

fn balance(id: u8) -> u128 {
    Balances::balance(&account(id))
}

fn file(len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| (i as u32).wrapping_mul(2_654_435_761).to_le_bytes()[2])
        .collect()
}

const STORAGE_PRICE: u128 = 1_000;
const SERVING_PRICE: u128 = 500;

/// An escrow for `bytes`, two copies, the test's prices, a cap of `cap` bytes a period, and `deposit`.
fn open(bytes: &[u8], cap: u64, deposit: u128) -> u64 {
    let id = NextEscrowId::<Test>::get();
    assert_ok!(Mesh::open_escrow(
        signed(UPLOADER),
        *blake3::hash(bytes).as_bytes(),
        bytes.len() as u64,
        2,
        STORAGE_PRICE,
        SERVING_PRICE,
        cap,
        deposit,
    ));
    id
}

fn seed(id: u8, escrow: u64) {
    assert_ok!(Mesh::bind_node(signed(id), [id; 32]));
    assert_ok!(Mesh::take_assignment(signed(id), escrow));
}

/// The group the chain names for `seeder` now, and its proof, from the whole file.
type Group = BoundedVec<u8, ConstU32<{ mesh_proof::GROUP as u32 }>>;
type Proof = BoundedVec<[u8; 32], ConstU32<{ mesh_proof::MAX_DEPTH as u32 }>>;

fn answer_for(bytes: &[u8], escrow: u64, seeder: u8) -> (Group, Proof) {
    let e = Escrows::<Test>::get(escrow).unwrap();
    let (_, index) = Mesh::challenge(escrow, &account(seeder), &e).expect("the window is open");
    let (start, len) = mesh_proof::group_range(bytes.len() as u64, index).unwrap();
    let group = bytes[start as usize..(start + len) as usize].to_vec();
    let proof = mesh_proof::prove(bytes, index).unwrap();
    (group.try_into().unwrap(), proof.try_into().unwrap())
}

fn receipt(escrow: u64, period: u32, seeder: u8, receiver: u8, bytes: u64) -> Receipt<AccountId> {
    Receipt {
        escrow,
        period,
        seeder: account(seeder),
        receiver: account(receiver),
        bytes,
    }
}

fn sign(r: &Receipt<AccountId>, by: u8) -> MultiSignature {
    MultiSignature::Sr25519(pair(by).sign(&Mesh::receipt_message(r)))
}

fn submit(
    seeder: u8,
    receipts: Vec<(Receipt<AccountId>, MultiSignature)>,
) -> sp_runtime::DispatchResult {
    Mesh::submit_receipts(signed(seeder), receipts.try_into().unwrap())
}

#[test]
fn an_escrow_holds_its_deposit_where_no_key_signs_and_returns_it_all_on_close() {
    new_test_ext().execute_with(|| {
        let bytes = file(50_000);
        let escrow = open(&bytes, 0, 10_000);
        assert_eq!(Balances::balance(&Mesh::account_of(escrow)), 10_000);
        assert_eq!(balance(UPLOADER), START - 10_000);
        assert_ok!(Mesh::top_up(signed(STRANGER), escrow, 500));
        assert_noop!(
            Mesh::close_escrow(signed(STRANGER), escrow),
            Error::<Test>::NotUploader
        );
        assert_ok!(Mesh::close_escrow(signed(UPLOADER), escrow));
        assert_eq!(
            balance(UPLOADER),
            START + 500,
            "everything it held, deposit and top-up, goes back"
        );
        assert_eq!(Balances::balance(&Mesh::account_of(escrow)), 0);
        assert!(Escrows::<Test>::get(escrow).is_none());
        assert_noop!(
            Mesh::top_up(signed(UPLOADER), escrow, 1),
            Error::<Test>::NoSuchEscrow
        );
    });
}

#[test]
fn an_escrow_needs_a_file_a_copy_at_most_the_bound_and_the_existential_deposit() {
    new_test_ext().execute_with(|| {
        let root = [9; 32];
        let open_with = |size, copies, deposit| {
            Mesh::open_escrow(signed(UPLOADER), root, size, copies, 1, 1, 1, deposit)
        };
        assert_noop!(open_with(0, 1, 100), Error::<Test>::BadEscrow);
        assert_noop!(open_with(10, 0, 100), Error::<Test>::BadEscrow);
        assert_noop!(open_with(10, MAX_COPIES + 1, 100), Error::<Test>::BadEscrow);
        assert_noop!(
            open_with(10, 1, EXISTENTIAL_DEPOSIT - 1),
            Error::<Test>::BelowExistentialDeposit
        );
        assert_ok!(open_with(10, MAX_COPIES, EXISTENTIAL_DEPOSIT));
    });
}

#[test]
fn a_seeder_binds_a_node_is_not_the_uploader_and_takes_only_the_copies_asked_for() {
    new_test_ext().execute_with(|| {
        let escrow = open(&file(1000), 0, 10_000);
        assert_noop!(
            Mesh::take_assignment(signed(SEEDER), escrow),
            Error::<Test>::NoNode
        );
        assert_ok!(Mesh::bind_node(signed(UPLOADER), [1; 32]));
        assert_noop!(
            Mesh::take_assignment(signed(UPLOADER), escrow),
            Error::<Test>::UploaderCannotSeed
        );
        seed(SEEDER, escrow);
        assert_noop!(
            Mesh::take_assignment(signed(SEEDER), escrow),
            Error::<Test>::AlreadyAssigned
        );
        seed(OTHER_SEEDER, escrow);
        assert_ok!(Mesh::bind_node(signed(STRANGER), [6; 32]));
        assert_noop!(
            Mesh::take_assignment(signed(STRANGER), escrow),
            Error::<Test>::NoCopiesLeft
        );
        assert_ok!(Mesh::leave_assignment(signed(SEEDER), escrow));
        // A copy left is a copy free.
        assert_ok!(Mesh::take_assignment(signed(STRANGER), escrow));
        assert_noop!(
            Mesh::leave_assignment(signed(SEEDER), escrow),
            Error::<Test>::NotAssigned
        );
    });
}

#[test]
fn the_group_the_chain_names_with_its_proof_is_paid_once_a_period() {
    new_test_ext().execute_with(|| {
        let bytes = file(100_000);
        let escrow = open(&bytes, 0, 100_000);
        seed(SEEDER, escrow);
        run_to(window_of(1));
        let (group, proof) = answer_for(&bytes, escrow, SEEDER);
        assert_ok!(Mesh::answer_challenge(
            signed(SEEDER),
            escrow,
            group.clone(),
            proof.clone()
        ));
        assert_eq!(balance(SEEDER), START + STORAGE_PRICE);
        assert_noop!(
            Mesh::answer_challenge(signed(SEEDER), escrow, group, proof),
            Error::<Test>::AlreadyAnswered
        );
        run_to(window_of(2));
        let (group, proof) = answer_for(&bytes, escrow, SEEDER);
        assert_ok!(Mesh::answer_challenge(signed(SEEDER), escrow, group, proof));
        assert_eq!(
            balance(SEEDER),
            START + 2 * STORAGE_PRICE,
            "and again the next period"
        );
    });
}

#[test]
fn another_group_a_changed_byte_or_a_wrong_proof_is_refused_and_pays_nothing() {
    new_test_ext().execute_with(|| {
        let bytes = file(100_000);
        let escrow = open(&bytes, 0, 100_000);
        seed(SEEDER, escrow);
        run_to(window_of(1));
        let e = Escrows::<Test>::get(escrow).unwrap();
        let (_, index) = Mesh::challenge(escrow, &account(SEEDER), &e).unwrap();
        // A group the chain did not name, with its own true proof.
        let other = (index + 1) % mesh_proof::groups(e.size);
        let (s, l) = mesh_proof::group_range(e.size, other).unwrap();
        let wrong_group: Group = bytes[s as usize..(s + l) as usize]
            .to_vec()
            .try_into()
            .unwrap();
        let wrong_proof: Proof = mesh_proof::prove(&bytes, other)
            .unwrap()
            .try_into()
            .unwrap();
        assert_noop!(
            Mesh::answer_challenge(signed(SEEDER), escrow, wrong_group, wrong_proof),
            Error::<Test>::BadProof
        );
        let (group, proof) = answer_for(&bytes, escrow, SEEDER);
        let mut changed = group.clone();
        changed[0] ^= 1;
        assert_noop!(
            Mesh::answer_challenge(signed(SEEDER), escrow, changed, proof.clone()),
            Error::<Test>::BadProof
        );
        let mut bent = proof.clone();
        bent[0][0] ^= 1;
        assert_noop!(
            Mesh::answer_challenge(signed(SEEDER), escrow, group, bent),
            Error::<Test>::BadProof
        );
        assert_eq!(balance(SEEDER), START);
    });
}

#[test]
fn outside_the_window_too_soon_or_unassigned_is_refused() {
    new_test_ext().execute_with(|| {
        let bytes = file(40_000);
        let escrow = open(&bytes, 0, 100_000);
        seed(SEEDER, escrow);
        // Taken in period 0: its first challenge is period 1's.
        run_to(4);
        let (group, proof) = answer_for(&bytes, escrow, SEEDER);
        assert_noop!(
            Mesh::answer_challenge(signed(SEEDER), escrow, group, proof),
            Error::<Test>::TooSoon
        );
        // Period 1's seed block itself, and the block after its window closes.
        run_to(u64::from(PERIOD) + 1);
        let e = Escrows::<Test>::get(escrow).unwrap();
        assert!(Mesh::challenge(escrow, &account(SEEDER), &e).is_none());
        run_to(u64::from(PERIOD) + 2 + u64::from(WINDOW));
        assert!(Mesh::challenge(escrow, &account(SEEDER), &e).is_none());
        assert_noop!(
            Mesh::answer_challenge(signed(SEEDER), escrow, BoundedVec::new(), BoundedVec::new()),
            Error::<Test>::OutsideWindow
        );
        assert_noop!(
            Mesh::answer_challenge(
                signed(STRANGER),
                escrow,
                BoundedVec::new(),
                BoundedVec::new()
            ),
            Error::<Test>::NotAssigned
        );
    });
}

#[test]
fn an_assignment_unanswered_past_the_limit_is_ended_by_anyone() {
    new_test_ext().execute_with(|| {
        let escrow = open(&file(1000), 0, 100_000);
        seed(SEEDER, escrow);
        run_to(u64::from(PERIOD * MAX_MISSED));
        assert_noop!(
            Mesh::end_lapsed(signed(STRANGER), escrow, account(SEEDER)),
            Error::<Test>::NotLapsed
        );
        run_to(u64::from(PERIOD * (MAX_MISSED + 1)));
        assert_ok!(Mesh::end_lapsed(signed(STRANGER), escrow, account(SEEDER)));
        assert_eq!(
            Escrows::<Test>::get(escrow).unwrap().assigned,
            0,
            "its copy is free again"
        );
    });
}

#[test]
fn a_receipt_pays_what_is_new_once() {
    new_test_ext().execute_with(|| {
        let bytes = file(8 * MIB as usize);
        let escrow = open(&bytes, u64::MAX, 1_000_000);
        seed(SEEDER, escrow);
        let period = Mesh::period_now();
        let two = receipt(escrow, period, SEEDER, RECEIVER, 2 * MIB as u64);
        assert_ok!(submit(SEEDER, vec![(two.clone(), sign(&two, RECEIVER))]));
        assert_eq!(balance(SEEDER), START + 2 * SERVING_PRICE);
        assert_noop!(
            submit(SEEDER, vec![(two.clone(), sign(&two, RECEIVER))]),
            Error::<Test>::NothingNew
        );
        let three = receipt(escrow, period, SEEDER, RECEIVER, 3 * MIB as u64);
        assert_ok!(submit(
            SEEDER,
            vec![(three.clone(), sign(&three, RECEIVER))]
        ));
        assert_eq!(
            balance(SEEDER),
            START + 3 * SERVING_PRICE,
            "only the third MiB is new"
        );
        // Part of a MiB is paid in proportion, rounded down.
        let more = receipt(
            escrow,
            period,
            SEEDER,
            RECEIVER,
            3 * MIB as u64 + MIB as u64 / 2,
        );
        assert_ok!(submit(SEEDER, vec![(more.clone(), sign(&more, RECEIVER))]));
        assert_eq!(
            balance(SEEDER),
            START + 3 * SERVING_PRICE + SERVING_PRICE / 2
        );
    });
}

#[test]
fn a_receiver_is_paid_for_at_most_the_file_and_an_escrow_for_at_most_its_cap_each_period() {
    new_test_ext().execute_with(|| {
        let bytes = file(MIB as usize);
        let escrow = open(&bytes, 3 * MIB as u64 / 2, 1_000_000);
        seed(SEEDER, escrow);
        seed(OTHER_SEEDER, escrow);
        let period = Mesh::period_now();
        // One receiver, the whole file from each of two seeders: paid once, for the file.
        let a = receipt(escrow, period, SEEDER, RECEIVER, MIB as u64);
        let b = receipt(escrow, period, OTHER_SEEDER, RECEIVER, MIB as u64);
        assert_ok!(submit(SEEDER, vec![(a.clone(), sign(&a, RECEIVER))]));
        assert_noop!(
            submit(OTHER_SEEDER, vec![(b.clone(), sign(&b, RECEIVER))]),
            Error::<Test>::NothingNew
        );
        // Another receiver: the escrow's cap leaves half a MiB.
        let c = receipt(escrow, period, OTHER_SEEDER, OTHER_RECEIVER, MIB as u64);
        assert_ok!(submit(
            OTHER_SEEDER,
            vec![(c.clone(), sign(&c, OTHER_RECEIVER))]
        ));
        assert_eq!(balance(OTHER_SEEDER), START + SERVING_PRICE / 2);
        // The next period, a fresh allowance.
        run_to(u64::from(PERIOD));
        let d = receipt(escrow, period + 1, OTHER_SEEDER, RECEIVER, MIB as u64);
        assert_ok!(submit(OTHER_SEEDER, vec![(d.clone(), sign(&d, RECEIVER))]));
        assert_eq!(
            balance(OTHER_SEEDER),
            START + SERVING_PRICE / 2 + SERVING_PRICE
        );
    });
}

#[test]
fn receipts_from_oneself_signed_by_another_for_another_seeder_or_stale_are_refused() {
    new_test_ext().execute_with(|| {
        let escrow = open(&file(MIB as usize), u64::MAX, 1_000_000);
        seed(SEEDER, escrow);
        let p = Mesh::period_now();
        let own = receipt(escrow, p, SEEDER, SEEDER, 1000);
        assert_noop!(
            submit(SEEDER, vec![(own.clone(), sign(&own, SEEDER))]),
            Error::<Test>::SelfDealing
        );
        let uploader = receipt(escrow, p, SEEDER, UPLOADER, 1000);
        assert_noop!(
            submit(SEEDER, vec![(uploader.clone(), sign(&uploader, UPLOADER))]),
            Error::<Test>::SelfDealing
        );
        let forged = receipt(escrow, p, SEEDER, RECEIVER, 1000);
        assert_noop!(
            submit(SEEDER, vec![(forged.clone(), sign(&forged, STRANGER))]),
            Error::<Test>::BadReceipt
        );
        let theirs = receipt(escrow, p, OTHER_SEEDER, RECEIVER, 1000);
        assert_noop!(
            submit(SEEDER, vec![(theirs.clone(), sign(&theirs, RECEIVER))]),
            Error::<Test>::BadReceipt
        );
        // A signature over the same fields for any other purpose is not a receipt.
        let r = receipt(escrow, p, SEEDER, RECEIVER, 1000);
        let elsewhere = MultiSignature::Sr25519(pair(RECEIVER).sign(&codec::Encode::encode(&r)));
        assert_noop!(
            submit(SEEDER, vec![(r.clone(), elsewhere)]),
            Error::<Test>::BadReceipt
        );
        run_to(u64::from(2 * PERIOD));
        assert_noop!(
            submit(SEEDER, vec![(r.clone(), sign(&r, RECEIVER))]),
            Error::<Test>::StaleReceipt
        );
        assert_eq!(balance(SEEDER), START);
    });
}

#[test]
fn a_root_taken_down_cannot_be_escrowed_pays_nothing_more_and_can_still_be_closed() {
    new_test_ext().execute_with(|| {
        let bytes = file(40_000);
        let escrow = open(&bytes, u64::MAX, 100_000);
        seed(SEEDER, escrow);
        let root = *blake3::hash(&bytes).as_bytes();
        assert_noop!(
            Mesh::set_takedown(signed(STRANGER), root, true),
            DispatchError::BadOrigin
        );
        assert_ok!(Mesh::set_takedown(RuntimeOrigin::root(), root, true));
        assert!(Takedowns::<Test>::contains_key(root));
        assert_noop!(
            Mesh::open_escrow(signed(UPLOADER), root, 40_000, 1, 1, 1, 1, 100),
            Error::<Test>::TakenDown
        );
        run_to(window_of(1));
        let (group, proof) = answer_for(&bytes, escrow, SEEDER);
        assert_noop!(
            Mesh::answer_challenge(signed(SEEDER), escrow, group, proof),
            Error::<Test>::TakenDown
        );
        let r = receipt(escrow, Mesh::period_now(), SEEDER, RECEIVER, 1000);
        assert_noop!(
            submit(SEEDER, vec![(r.clone(), sign(&r, RECEIVER))]),
            Error::<Test>::TakenDown
        );
        assert_ok!(Mesh::close_escrow(signed(UPLOADER), escrow));
        assert_eq!(balance(UPLOADER), START);
        assert_ok!(Mesh::set_takedown(RuntimeOrigin::root(), root, false));
        assert!(!Takedowns::<Test>::contains_key(root));
    });
}

#[test]
fn an_exhausted_escrow_pays_nothing_and_keeps_its_existential_deposit() {
    new_test_ext().execute_with(|| {
        let bytes = file(40_000);
        let escrow = open(&bytes, u64::MAX, STORAGE_PRICE + EXISTENTIAL_DEPOSIT - 1);
        seed(SEEDER, escrow);
        run_to(window_of(1));
        let (group, proof) = answer_for(&bytes, escrow, SEEDER);
        assert_noop!(
            Mesh::answer_challenge(signed(SEEDER), escrow, group, proof),
            Error::<Test>::EscrowExhausted
        );
        assert_eq!(
            Balances::balance(&Mesh::account_of(escrow)),
            STORAGE_PRICE + EXISTENTIAL_DEPOSIT - 1
        );
    });
}

#[test]
fn a_periods_records_are_forgotten_only_once_no_receipt_can_name_it() {
    new_test_ext().execute_with(|| {
        let escrow = open(&file(MIB as usize), u64::MAX, 1_000_000);
        seed(SEEDER, escrow);
        let r = receipt(escrow, 0, SEEDER, RECEIVER, 1000);
        assert_ok!(submit(SEEDER, vec![(r.clone(), sign(&r, RECEIVER))]));
        run_to(u64::from(PERIOD));
        assert_noop!(
            Mesh::forget_served(signed(STRANGER), escrow, 0),
            Error::<Test>::PeriodOpen
        );
        run_to(u64::from(2 * PERIOD));
        assert_ok!(Mesh::forget_served(signed(STRANGER), escrow, 0));
        assert_eq!(crate::PeriodServed::<Test>::get(escrow, 0), 0);
    });
}

#[test]
fn the_largest_amounts_cannot_overflow() {
    new_test_ext().execute_with(|| {
        // The largest intermediate: a price per MiB times the bytes served, before dividing by a MiB.
        assert_eq!(
            Mesh::serving_cost(u128::MAX, u64::MAX),
            None,
            "refused, not wrapped"
        );
        // The whole base supply (10^14 CGT at 18 decimals, 10^32 embers) per MiB, for a TiB: fits, exactly.
        let supply = 10u128.pow(32);
        assert_eq!(
            Mesh::serving_cost(supply, 1 << 40),
            Some(supply * (1 << 20))
        );
        assert_eq!(
            Mesh::serving_cost(supply, 1),
            Some(supply / MIB),
            "rounded down"
        );
        // An escrow number at its limit is refused, not wrapped.
        NextEscrowId::<Test>::put(u64::MAX);
        assert_noop!(
            Mesh::open_escrow(signed(UPLOADER), [3; 32], 10, 1, 1, 1, 1, 100),
            Error::<Test>::Overflow
        );
        // A receipt whose cost would not fit moves nothing.
        NextEscrowId::<Test>::put(0);
        assert_ok!(Mesh::open_escrow(
            signed(UPLOADER),
            [4; 32],
            u64::MAX,
            1,
            1,
            u128::MAX,
            u64::MAX,
            100
        ));
        seed(SEEDER, 0);
        let r = receipt(0, Mesh::period_now(), SEEDER, RECEIVER, u64::MAX);
        assert_noop!(
            submit(SEEDER, vec![(r.clone(), sign(&r, RECEIVER))]),
            Error::<Test>::Overflow
        );
    });
}

/// A development chain's genesis may shorten the period and its window; every other chain keeps the runtime's, and a
/// pair that cannot work is refused at genesis rather than met at the first challenge.
#[test]
fn genesis_may_shorten_the_period_and_never_to_one_that_cannot_work() {
    use crate::{timing_holds, AnswerWindowLength, GenesisConfig, PeriodLength};
    use sp_runtime::BuildStorage;

    new_test_ext().execute_with(|| {
        assert_eq!(PeriodLength::<Test>::get(), PERIOD);
        assert_eq!(AnswerWindowLength::<Test>::get(), WINDOW);
    });

    let storage = GenesisConfig::<Test> {
        period: Some(10),
        answer_window: Some(3),
        ..Default::default()
    }
    .build_storage()
    .unwrap();
    sp_io::TestExternalities::new(storage).execute_with(|| {
        assert_eq!(PeriodLength::<Test>::get(), 10);
        assert_eq!(AnswerWindowLength::<Test>::get(), 3);
        System::set_block_number(25);
        assert_eq!(Mesh::period_now(), 2);
        assert_eq!(Mesh::seed_block(2), 21);
    });

    for (period, window) in [(10, 0), (10, 9), (10, 10), (300, 256)] {
        assert!(!timing_holds(period, window));
        let refused = std::panic::catch_unwind(|| {
            GenesisConfig::<Test> {
                period: Some(period),
                answer_window: Some(window),
                ..Default::default()
            }
            .build_storage()
        });
        assert!(
            refused.is_err(),
            "a period of {period} with a window of {window}"
        );
    }
    // Only the period given: the window is the runtime's, and must still fit.
    assert!(std::panic::catch_unwind(|| GenesisConfig::<Test> {
        period: Some(WINDOW + 1),
        ..Default::default()
    }
    .build_storage())
    .is_err());
}
