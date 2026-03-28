use crate::mock::*;
use crate::pallet::{CrossSettlementStatus, CrossSettlements, Error, NextCrossSettlementId};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

fn setup_two_operators() -> (u64, u32, u64, u32) {
    System::set_block_number(1);
    let op1_acc = 1u64;
    let op2_acc = 2u64;
    let op1_id = register_operator(op1_acc);
    let op2_id = register_operator(op2_acc);
    (op1_acc, op1_id, op2_acc, op2_id)
}

#[test]
fn propose_single_participant_auto_approves() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, _) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            10,
            bvec(b"REF"),
        ));

        let info = CrossSettlements::<Test>::get(0).unwrap();
        assert_eq!(info.status, CrossSettlementStatus::Approved);
        assert_eq!(info.initiator_id, op1_id);
        assert_eq!(info.approvals.len(), 1);
        assert_eq!(NextCrossSettlementId::<Test>::get(), 1);
    });
}

#[test]
fn propose_multi_participant_stays_pending() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, op2_id) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id, op2_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            10,
            bvec(b"REF"),
        ));

        let info = CrossSettlements::<Test>::get(0).unwrap();
        assert_eq!(info.status, CrossSettlementStatus::Pending);
        assert_eq!(info.approvals.len(), 1);
        assert!(info.approvals.contains(&op1_id));
    });
}

#[test]
fn propose_requires_registered_operator() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_noop!(
            CrossSettlement::propose_cross_settlement(
                RawOrigin::Signed(99).into(),
                participants(&[0]),
                legs(vec![]),
                10,
                bvec(b"REF"),
            ),
            Error::<Test>::OperatorNotFound
        );
    });
}

#[test]
fn propose_initiator_must_be_in_participants() {
    new_test_ext().execute_with(|| {
        let (op1_acc, _op1_id, _, op2_id) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_noop!(
            CrossSettlement::propose_cross_settlement(
                RawOrigin::Signed(op1_acc).into(),
                participants(&[op2_id]),
                legs(vec![(asset_id, 10, 20, 100)]),
                10,
                bvec(b"REF"),
            ),
            Error::<Test>::NotAParticipant
        );
    });
}

#[test]
fn propose_rejects_invalid_asset() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, _) = setup_two_operators();

        assert_noop!(
            CrossSettlement::propose_cross_settlement(
                RawOrigin::Signed(op1_acc).into(),
                participants(&[op1_id]),
                legs(vec![(99, 10, 20, 100)]),
                10,
                bvec(b"REF"),
            ),
            Error::<Test>::AssetNotFound
        );
    });
}

#[test]
fn propose_rejects_expiry_in_past() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, _) = setup_two_operators();
        let asset_id = register_asset(op1_acc);
        System::set_block_number(5);

        assert_noop!(
            CrossSettlement::propose_cross_settlement(
                RawOrigin::Signed(op1_acc).into(),
                participants(&[op1_id]),
                legs(vec![(asset_id, 10, 20, 100)]),
                4,
                bvec(b"REF"),
            ),
            Error::<Test>::ExpiryInPast
        );
    });
}

#[test]
fn approve_transitions_to_approved_when_all_approve() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, op2_acc, op2_id) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id, op2_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            10,
            bvec(b"REF"),
        ));

        assert_ok!(CrossSettlement::approve_cross_settlement(
            RawOrigin::Signed(op2_acc).into(),
            0,
        ));

        let info = CrossSettlements::<Test>::get(0).unwrap();
        assert_eq!(info.status, CrossSettlementStatus::Approved);
        assert_eq!(info.approvals.len(), 2);
    });
}

#[test]
fn approve_not_pending_fails() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, op2_acc, op2_id) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id, op2_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            10,
            bvec(b"REF"),
        ));

        assert_ok!(CrossSettlement::approve_cross_settlement(
            RawOrigin::Signed(op2_acc).into(),
            0,
        ));

        // Now status is Approved, cannot approve again
        assert_noop!(
            CrossSettlement::approve_cross_settlement(RawOrigin::Signed(op2_acc).into(), 0),
            Error::<Test>::NotPending
        );
    });
}

#[test]
fn approve_not_a_participant_fails() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, op2_acc, _op2_id) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            10,
            bvec(b"REF"),
        ));

        // op2 is not in participants
        assert_noop!(
            CrossSettlement::approve_cross_settlement(RawOrigin::Signed(op2_acc).into(), 0),
            Error::<Test>::NotPending
        );
    });
}

#[test]
fn approve_already_approved_fails() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, op2_id) = setup_two_operators();
        // Add third operator so cross-settlement stays pending
        let op3_acc = 3u64;
        let op3_id = register_operator(op3_acc);
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id, op2_id, op3_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            10,
            bvec(b"REF"),
        ));

        // op1 already approved via propose — try again
        assert_noop!(
            CrossSettlement::approve_cross_settlement(RawOrigin::Signed(op1_acc).into(), 0),
            Error::<Test>::AlreadyApproved
        );
    });
}

#[test]
fn approve_expired_fails() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, op2_acc, op2_id) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id, op2_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            5,
            bvec(b"REF"),
        ));

        System::set_block_number(6);

        assert_noop!(
            CrossSettlement::approve_cross_settlement(RawOrigin::Signed(op2_acc).into(), 0),
            Error::<Test>::Expired
        );
    });
}

#[test]
fn execute_transfers_balances_atomically() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, _) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        issue_balance(op1_acc, op1_id, asset_id, 10, 1000);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id]),
            legs(vec![(asset_id, 10, 20, 400)]),
            100,
            bvec(b"REF"),
        ));

        assert_ok!(CrossSettlement::execute_cross_settlement(
            RawOrigin::Root.into(),
            0,
        ));

        let bal_10 = pallet_settlement_engine::pallet::AccountBalances::<Test>::get(10, asset_id);
        let bal_20 = pallet_settlement_engine::pallet::AccountBalances::<Test>::get(20, asset_id);
        assert_eq!(bal_10, 600);
        assert_eq!(bal_20, 400);

        let info = CrossSettlements::<Test>::get(0).unwrap();
        assert_eq!(info.status, CrossSettlementStatus::Executed);
        assert_eq!(info.executed_at, Some(1));
    });
}

#[test]
fn execute_multi_leg_atomic() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, op2_acc, op2_id) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        issue_balance(op1_acc, op1_id, asset_id, 10, 1000);
        issue_balance(op1_acc, op1_id, asset_id, 20, 1000);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id, op2_id]),
            legs(vec![(asset_id, 10, 20, 300), (asset_id, 20, 10, 150),]),
            100,
            bvec(b"SWAP"),
        ));

        assert_ok!(CrossSettlement::approve_cross_settlement(
            RawOrigin::Signed(op2_acc).into(),
            0,
        ));

        assert_ok!(CrossSettlement::execute_cross_settlement(
            RawOrigin::Root.into(),
            0,
        ));

        let bal_10 = pallet_settlement_engine::pallet::AccountBalances::<Test>::get(10, asset_id);
        let bal_20 = pallet_settlement_engine::pallet::AccountBalances::<Test>::get(20, asset_id);
        assert_eq!(bal_10, 850); // 1000 - 300 + 150
        assert_eq!(bal_20, 1150); // 1000 + 300 - 150
    });
}

#[test]
fn execute_not_approved_fails() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, op2_id) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id, op2_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            10,
            bvec(b"REF"),
        ));

        assert_noop!(
            CrossSettlement::execute_cross_settlement(RawOrigin::Root.into(), 0),
            Error::<Test>::NotApproved
        );
    });
}

#[test]
fn execute_expired_fails() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, _) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            5,
            bvec(b"REF"),
        ));

        System::set_block_number(6);

        assert_noop!(
            CrossSettlement::execute_cross_settlement(RawOrigin::Root.into(), 0),
            Error::<Test>::Expired
        );
    });
}

#[test]
fn execute_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, _) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        // account 10 has 0 balance
        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            100,
            bvec(b"REF"),
        ));

        assert_noop!(
            CrossSettlement::execute_cross_settlement(RawOrigin::Root.into(), 0),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn cancel_pending_works() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, op2_id) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id, op2_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            10,
            bvec(b"REF"),
        ));

        assert_ok!(CrossSettlement::cancel_cross_settlement(
            RawOrigin::Root.into(),
            0,
        ));

        let info = CrossSettlements::<Test>::get(0).unwrap();
        assert_eq!(info.status, CrossSettlementStatus::Cancelled);
    });
}

#[test]
fn cancel_approved_works() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, _) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            10,
            bvec(b"REF"),
        ));

        // Status is auto-Approved
        assert_ok!(CrossSettlement::cancel_cross_settlement(
            RawOrigin::Root.into(),
            0,
        ));

        let info = CrossSettlements::<Test>::get(0).unwrap();
        assert_eq!(info.status, CrossSettlementStatus::Cancelled);
    });
}

#[test]
fn cancel_executed_fails() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, _) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        issue_balance(op1_acc, op1_id, asset_id, 10, 1000);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            100,
            bvec(b"REF"),
        ));

        assert_ok!(CrossSettlement::execute_cross_settlement(
            RawOrigin::Root.into(),
            0,
        ));

        assert_noop!(
            CrossSettlement::cancel_cross_settlement(RawOrigin::Root.into(), 0),
            Error::<Test>::NotExecutable
        );
    });
}

#[test]
fn cancel_already_cancelled_fails() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, _) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            100,
            bvec(b"REF"),
        ));

        assert_ok!(CrossSettlement::cancel_cross_settlement(
            RawOrigin::Root.into(),
            0,
        ));

        assert_noop!(
            CrossSettlement::cancel_cross_settlement(RawOrigin::Root.into(), 0),
            Error::<Test>::NotExecutable
        );
    });
}

#[test]
fn not_found_errors() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_noop!(
            CrossSettlement::approve_cross_settlement(RawOrigin::Signed(1).into(), 99),
            Error::<Test>::OperatorNotFound
        );
        assert_noop!(
            CrossSettlement::execute_cross_settlement(RawOrigin::Root.into(), 99),
            Error::<Test>::CrossSettlementNotFound
        );
        assert_noop!(
            CrossSettlement::cancel_cross_settlement(RawOrigin::Root.into(), 99),
            Error::<Test>::CrossSettlementNotFound
        );
    });
}

#[test]
fn next_id_increments() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, _, _) = setup_two_operators();
        let asset_id = register_asset(op1_acc);

        assert_eq!(NextCrossSettlementId::<Test>::get(), 0);

        for _ in 0..3 {
            assert_ok!(CrossSettlement::propose_cross_settlement(
                RawOrigin::Signed(op1_acc).into(),
                participants(&[op1_id]),
                legs(vec![(asset_id, 10, 20, 1)]),
                100,
                bvec(b"REF"),
            ));
        }

        assert_eq!(NextCrossSettlementId::<Test>::get(), 3);
    });
}

#[test]
fn events_emitted() {
    new_test_ext().execute_with(|| {
        let (op1_acc, op1_id, op2_acc, op2_id) = setup_two_operators();
        let asset_id = register_asset(op1_acc);
        issue_balance(op1_acc, op1_id, asset_id, 10, 500);

        // Propose — single participant → auto-Approved
        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id]),
            legs(vec![(asset_id, 10, 20, 100)]),
            100,
            bvec(b"REF"),
        ));

        let evts = System::events();
        let types: Vec<_> = evts
            .iter()
            .filter_map(|e| {
                if let RuntimeEvent::CrossSettlement(e) = &e.event {
                    Some(e.clone())
                } else {
                    None
                }
            })
            .collect();

        assert!(types
            .iter()
            .any(|e| matches!(e, crate::pallet::Event::CrossSettlementProposed(0, _))));
        assert!(types
            .iter()
            .any(|e| matches!(e, crate::pallet::Event::CrossSettlementApproved(0))));

        // Execute
        assert_ok!(CrossSettlement::execute_cross_settlement(
            RawOrigin::Root.into(),
            0,
        ));

        let evts = System::events();
        let types: Vec<_> = evts
            .iter()
            .filter_map(|e| {
                if let RuntimeEvent::CrossSettlement(e) = &e.event {
                    Some(e.clone())
                } else {
                    None
                }
            })
            .collect();
        assert!(types
            .iter()
            .any(|e| matches!(e, crate::pallet::Event::CrossSettlementExecuted(0, _))));

        // Second cross-settlement: 2 participants, approve + cancel
        assert_ok!(CrossSettlement::propose_cross_settlement(
            RawOrigin::Signed(op1_acc).into(),
            participants(&[op1_id, op2_id]),
            legs(vec![(asset_id, 10, 20, 50)]),
            100,
            bvec(b"REF2"),
        ));

        assert_ok!(CrossSettlement::approve_cross_settlement(
            RawOrigin::Signed(op2_acc).into(),
            1,
        ));

        assert_ok!(CrossSettlement::cancel_cross_settlement(
            RawOrigin::Root.into(),
            1,
        ));

        let evts = System::events();
        let types: Vec<_> = evts
            .iter()
            .filter_map(|e| {
                if let RuntimeEvent::CrossSettlement(e) = &e.event {
                    Some(e.clone())
                } else {
                    None
                }
            })
            .collect();
        assert!(types
            .iter()
            .any(|e| matches!(e, crate::pallet::Event::ParticipantApproved(1, _))));
        assert!(types
            .iter()
            .any(|e| matches!(e, crate::pallet::Event::CrossSettlementCancelled(1))));
    });
}
