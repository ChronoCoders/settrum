use crate::mock::{
    new_test_ext, reference, register_asset, register_operator, RuntimeOrigin, Test,
};
use crate::pallet::{
    AccountBalances, Error, LockedBalances, NextSettlementId, SettlementOperation,
    SettlementStatus, Settlements,
};
use frame_support::{assert_err, assert_noop, assert_ok};

fn setup() -> (u64, u32) {
    frame_system::Pallet::<Test>::set_block_number(1);
    register_operator(1);
    let asset_id = register_asset(1);
    (1u64, asset_id)
}

fn issue(operator: u64, asset_id: u32, to: u64, amount: u128) {
    assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
        RuntimeOrigin::signed(operator),
        0,
        asset_id,
        SettlementOperation::Issue,
        amount,
        operator,
        to,
        reference(),
    ));
    let id = NextSettlementId::<Test>::get().saturating_sub(1);
    assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
        RuntimeOrigin::root(),
        id,
    ));
}

#[test]
fn submit_settlement_works() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            op,
            2u64,
            reference(),
        ));
        let info = Settlements::<Test>::get(0).unwrap();
        assert_eq!(info.operator_id, 0);
        assert_eq!(info.asset_id, asset_id);
        assert_eq!(info.operation, SettlementOperation::Issue);
        assert_eq!(info.amount, 1_000);
        assert_eq!(info.status, SettlementStatus::Pending);
        assert_eq!(info.finalized_at, None);
    });
}

#[test]
fn submit_settlement_operator_not_found() {
    new_test_ext().execute_with(|| {
        let (_, asset_id) = setup();
        assert_noop!(
            crate::pallet::Pallet::<Test>::submit_settlement(
                RuntimeOrigin::signed(1),
                99,
                asset_id,
                SettlementOperation::Issue,
                1_000,
                1u64,
                2u64,
                reference(),
            ),
            Error::<Test>::OperatorNotFound
        );
    });
}

#[test]
fn submit_settlement_unauthorized() {
    new_test_ext().execute_with(|| {
        let (_, asset_id) = setup();
        assert_noop!(
            crate::pallet::Pallet::<Test>::submit_settlement(
                RuntimeOrigin::signed(99),
                0,
                asset_id,
                SettlementOperation::Issue,
                1_000,
                99u64,
                2u64,
                reference(),
            ),
            Error::<Test>::Unauthorized
        );
    });
}

#[test]
fn submit_settlement_operator_not_active() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(
            pallet_operators::pallet::Pallet::<Test>::update_operator_status(
                RuntimeOrigin::root(),
                0,
                pallet_operators::pallet::OperatorStatus::Suspended,
            )
        );
        assert_noop!(
            crate::pallet::Pallet::<Test>::submit_settlement(
                RuntimeOrigin::signed(op),
                0,
                asset_id,
                SettlementOperation::Issue,
                1_000,
                op,
                2u64,
                reference(),
            ),
            Error::<Test>::OperatorNotActive
        );
    });
}

#[test]
fn submit_settlement_asset_not_found() {
    new_test_ext().execute_with(|| {
        let (op, _) = setup();
        assert_noop!(
            crate::pallet::Pallet::<Test>::submit_settlement(
                RuntimeOrigin::signed(op),
                0,
                99,
                SettlementOperation::Issue,
                1_000,
                op,
                2u64,
                reference(),
            ),
            Error::<Test>::AssetNotFound
        );
    });
}

#[test]
fn finalize_issue_updates_balance_and_supply() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Issue,
            5_000,
            op,
            2u64,
            reference(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            0,
        ));
        assert_eq!(AccountBalances::<Test>::get(2u64, asset_id), 5_000);
        let asset = pallet_asset_registry::pallet::Assets::<Test>::get(asset_id).unwrap();
        assert_eq!(asset.total_supply, 5_000);
        let info = Settlements::<Test>::get(0).unwrap();
        assert_eq!(info.status, SettlementStatus::Finalized);
        assert_eq!(info.finalized_at, Some(1));
    });
}

#[test]
fn finalize_redeem_updates_balance_and_supply() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        issue(op, asset_id, 2u64, 5_000);
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Redeem,
            2_000,
            2u64,
            op,
            reference(),
        ));
        let id = NextSettlementId::<Test>::get().saturating_sub(1);
        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            id,
        ));
        assert_eq!(AccountBalances::<Test>::get(2u64, asset_id), 3_000);
        let asset = pallet_asset_registry::pallet::Assets::<Test>::get(asset_id).unwrap();
        assert_eq!(asset.total_supply, 3_000);
    });
}

#[test]
fn finalize_transfer_moves_balance() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        issue(op, asset_id, 2u64, 5_000);
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Transfer,
            1_000,
            2u64,
            3u64,
            reference(),
        ));
        let id = NextSettlementId::<Test>::get().saturating_sub(1);
        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            id,
        ));
        assert_eq!(AccountBalances::<Test>::get(2u64, asset_id), 4_000);
        assert_eq!(AccountBalances::<Test>::get(3u64, asset_id), 1_000);
        let asset = pallet_asset_registry::pallet::Assets::<Test>::get(asset_id).unwrap();
        assert_eq!(asset.total_supply, 5_000);
    });
}

#[test]
fn finalize_lock_moves_to_locked() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        issue(op, asset_id, 2u64, 5_000);
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Lock,
            2_000,
            2u64,
            2u64,
            reference(),
        ));
        let id = NextSettlementId::<Test>::get().saturating_sub(1);
        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            id,
        ));
        assert_eq!(AccountBalances::<Test>::get(2u64, asset_id), 3_000);
        assert_eq!(LockedBalances::<Test>::get(2u64, asset_id), 2_000);
        let asset = pallet_asset_registry::pallet::Assets::<Test>::get(asset_id).unwrap();
        assert_eq!(asset.total_supply, 5_000);
    });
}

#[test]
fn finalize_unlock_moves_from_locked() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        issue(op, asset_id, 2u64, 5_000);
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Lock,
            2_000,
            2u64,
            2u64,
            reference(),
        ));
        let lock_id = NextSettlementId::<Test>::get().saturating_sub(1);
        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            lock_id,
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Unlock,
            1_000,
            2u64,
            2u64,
            reference(),
        ));
        let unlock_id = NextSettlementId::<Test>::get().saturating_sub(1);
        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            unlock_id,
        ));
        assert_eq!(AccountBalances::<Test>::get(2u64, asset_id), 4_000);
        assert_eq!(LockedBalances::<Test>::get(2u64, asset_id), 1_000);
    });
}

#[test]
fn finalize_settlement_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            crate::pallet::Pallet::<Test>::finalize_settlement(RuntimeOrigin::root(), 99),
            Error::<Test>::SettlementNotFound
        );
    });
}

#[test]
fn finalize_settlement_not_pending() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            op,
            2u64,
            reference(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            0,
        ));
        assert_noop!(
            crate::pallet::Pallet::<Test>::finalize_settlement(RuntimeOrigin::root(), 0),
            Error::<Test>::SettlementNotPending
        );
    });
}

#[test]
fn dispute_settlement_works() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            op,
            2u64,
            reference(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::dispute_settlement(
            RuntimeOrigin::root(),
            0,
        ));
        let info = Settlements::<Test>::get(0).unwrap();
        assert_eq!(info.status, SettlementStatus::Disputed);
    });
}

#[test]
fn dispute_settlement_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            crate::pallet::Pallet::<Test>::dispute_settlement(RuntimeOrigin::root(), 99),
            Error::<Test>::SettlementNotFound
        );
    });
}

#[test]
fn dispute_settlement_not_pending() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            op,
            2u64,
            reference(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::dispute_settlement(
            RuntimeOrigin::root(),
            0,
        ));
        assert_noop!(
            crate::pallet::Pallet::<Test>::dispute_settlement(RuntimeOrigin::root(), 0),
            Error::<Test>::SettlementNotPending
        );
    });
}

#[test]
fn disputed_settlement_cannot_finalize() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            op,
            2u64,
            reference(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::dispute_settlement(
            RuntimeOrigin::root(),
            0,
        ));
        assert_noop!(
            crate::pallet::Pallet::<Test>::finalize_settlement(RuntimeOrigin::root(), 0),
            Error::<Test>::SettlementNotPending
        );
    });
}

#[test]
fn insufficient_balance_on_redeem() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Redeem,
            1_000,
            2u64,
            op,
            reference(),
        ));
        assert_err!(
            crate::pallet::Pallet::<Test>::finalize_settlement(RuntimeOrigin::root(), 0),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn insufficient_balance_on_transfer() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Transfer,
            1_000,
            2u64,
            3u64,
            reference(),
        ));
        assert_err!(
            crate::pallet::Pallet::<Test>::finalize_settlement(RuntimeOrigin::root(), 0),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn insufficient_balance_on_lock() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Lock,
            1_000,
            2u64,
            2u64,
            reference(),
        ));
        assert_err!(
            crate::pallet::Pallet::<Test>::finalize_settlement(RuntimeOrigin::root(), 0),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn insufficient_locked_balance_on_unlock() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        issue(op, asset_id, 2u64, 5_000);
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Unlock,
            1_000,
            2u64,
            2u64,
            reference(),
        ));
        let id = NextSettlementId::<Test>::get().saturating_sub(1);
        assert_err!(
            crate::pallet::Pallet::<Test>::finalize_settlement(RuntimeOrigin::root(), id),
            Error::<Test>::InsufficientLockedBalance
        );
    });
}

#[test]
fn next_settlement_id_increments() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            op,
            2u64,
            reference(),
        ));
        assert_eq!(NextSettlementId::<Test>::get(), 1);
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            op,
            2u64,
            reference(),
        ));
        assert_eq!(NextSettlementId::<Test>::get(), 2);
    });
}

#[test]
fn operator_settlement_count_increments() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            op,
            2u64,
            reference(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            op,
            2u64,
            reference(),
        ));
        let info = pallet_operators::pallet::Operators::<Test>::get(op).unwrap();
        assert_eq!(info.settlement_count, 2);
    });
}

#[test]
fn supply_unchanged_on_transfer() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        issue(op, asset_id, 2u64, 5_000);
        let supply_before = pallet_asset_registry::pallet::Assets::<Test>::get(asset_id)
            .unwrap()
            .total_supply;
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Transfer,
            1_000,
            2u64,
            3u64,
            reference(),
        ));
        let id = NextSettlementId::<Test>::get().saturating_sub(1);
        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            id,
        ));
        let supply_after = pallet_asset_registry::pallet::Assets::<Test>::get(asset_id)
            .unwrap()
            .total_supply;
        assert_eq!(supply_before, supply_after);
    });
}

#[test]
fn supply_unchanged_on_lock_unlock() {
    new_test_ext().execute_with(|| {
        let (op, asset_id) = setup();
        issue(op, asset_id, 2u64, 5_000);
        let supply_before = pallet_asset_registry::pallet::Assets::<Test>::get(asset_id)
            .unwrap()
            .total_supply;
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Lock,
            2_000,
            2u64,
            2u64,
            reference(),
        ));
        let lock_id = NextSettlementId::<Test>::get().saturating_sub(1);
        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            lock_id,
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(op),
            0,
            asset_id,
            SettlementOperation::Unlock,
            2_000,
            2u64,
            2u64,
            reference(),
        ));
        let unlock_id = NextSettlementId::<Test>::get().saturating_sub(1);
        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            unlock_id,
        ));
        let supply_after = pallet_asset_registry::pallet::Assets::<Test>::get(asset_id)
            .unwrap()
            .total_supply;
        assert_eq!(supply_before, supply_after);
    });
}

#[test]
fn events_emitted_on_submit_finalize_dispute() {
    new_test_ext().execute_with(|| {
        frame_system::Pallet::<Test>::set_block_number(1);
        register_operator(1);
        let asset_id = register_asset(1);

        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(1),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            1u64,
            2u64,
            reference(),
        ));
        frame_system::Pallet::<Test>::assert_last_event(
            crate::pallet::Event::<Test>::SettlementSubmitted(
                0,
                0,
                asset_id,
                SettlementOperation::Issue,
            )
            .into(),
        );

        assert_ok!(crate::pallet::Pallet::<Test>::finalize_settlement(
            RuntimeOrigin::root(),
            0,
        ));
        frame_system::Pallet::<Test>::assert_last_event(
            crate::pallet::Event::<Test>::SettlementFinalized(0, 1).into(),
        );

        assert_ok!(crate::pallet::Pallet::<Test>::submit_settlement(
            RuntimeOrigin::signed(1),
            0,
            asset_id,
            SettlementOperation::Issue,
            1_000,
            1u64,
            2u64,
            reference(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::dispute_settlement(
            RuntimeOrigin::root(),
            1,
        ));
        frame_system::Pallet::<Test>::assert_last_event(
            crate::pallet::Event::<Test>::SettlementDisputed(1).into(),
        );
    });
}
