use crate::mock::{
    bounded_name, new_test_ext, Operators, RuntimeEvent, RuntimeOrigin, System, Test,
};
use crate::pallet::NextOperatorId;
use crate::pallet::{Error, OperatorStatus};
use frame_support::pallet_prelude::{ConstU32, DispatchError};
use frame_support::{assert_err, assert_noop, assert_ok, BoundedVec};

#[test]
fn register_operator_works() {
    new_test_ext().execute_with(|| {
        frame_system::Pallet::<Test>::set_block_number(1);

        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            b"Test Operator".to_vec().try_into().unwrap(),
            1_000_000
        ));
    });
}

#[test]
fn register_operator_success() {
    new_test_ext().execute_with(|| {
        let collateral = 1_000_000u64;
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            collateral
        ));
        let info = crate::pallet::Operators::<Test>::get(1).unwrap();
        assert_eq!(info.id, 0);
        assert_eq!(info.account, 1);
        assert_eq!(info.name, bounded_name("A"));
        assert_eq!(info.collateral, 1_000_000);
        assert!(matches!(info.status, OperatorStatus::Active));
    });
}

#[test]
fn register_insufficient_collateral() {
    new_test_ext().execute_with(|| {
        let collateral = 999_999u64;
        assert_noop!(
            Operators::register_operator(RuntimeOrigin::signed(1), bounded_name("A"), collateral),
            Error::<Test>::InsufficientCollateral
        );
    });
}

#[test]
fn update_status_unauthorized() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            1_000_000
        ));
        assert_err!(
            Operators::update_operator_status(
                RuntimeOrigin::signed(1),
                0,
                OperatorStatus::Suspended
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn increment_settlement_count() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            1_000_000
        ));
        assert_ok!(Operators::increment_settlement_count(0));
        let info = crate::pallet::Operators::<Test>::get(1).unwrap();
        assert_eq!(info.settlement_count, 1);
    });
}

#[test]
fn multiple_operators_isolation() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            1_000_000
        ));
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(2),
            bounded_name("B"),
            1_000_000
        ));
        assert_ok!(Operators::increment_settlement_count(1));
        let a = crate::pallet::Operators::<Test>::get(1).unwrap();
        let b = crate::pallet::Operators::<Test>::get(2).unwrap();
        assert_eq!(a.id, 0);
        assert_eq!(b.id, 1);
        assert_eq!(a.settlement_count, 0);
        assert_eq!(b.settlement_count, 1);
    });
}

#[test]
fn register_twice_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            1_000_000
        ));
        assert_err!(
            Operators::register_operator(RuntimeOrigin::signed(1), bounded_name("A"), 1_000_000),
            Error::<Test>::AlreadyRegistered
        );
    });
}

#[test]
fn increase_collateral_updates_total() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            1_000_000
        ));
        assert_ok!(Operators::increase_collateral(
            RuntimeOrigin::signed(1),
            500
        ));
        let info = crate::pallet::Operators::<Test>::get(1).unwrap();
        assert_eq!(info.collateral, 1_000_500);
    });
}

#[test]
fn increase_collateral_unknown_operator() {
    new_test_ext().execute_with(|| {
        assert_err!(
            Operators::increase_collateral(RuntimeOrigin::signed(1), 500),
            Error::<Test>::OperatorNotFound
        );
    });
}

#[test]
fn update_status_root_succeeds() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            1_000_000
        ));
        assert_ok!(Operators::update_operator_status(
            RuntimeOrigin::root(),
            0,
            OperatorStatus::Suspended
        ));
        let info = crate::pallet::Operators::<Test>::get(1).unwrap();
        assert!(matches!(info.status, OperatorStatus::Suspended));
    });
}

#[test]
fn update_status_operator_not_found() {
    new_test_ext().execute_with(|| {
        assert_err!(
            Operators::update_operator_status(
                RuntimeOrigin::root(),
                123,
                OperatorStatus::Suspended
            ),
            Error::<Test>::OperatorNotFound
        );
    });
}

#[test]
fn name_bounded_length() {
    new_test_ext().execute_with(|| {
        let long = vec![b'a'; 64];
        let name: BoundedVec<u8, ConstU32<64>> = BoundedVec::try_from(long).unwrap();
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            name,
            1_000_000
        ));
        let info = crate::pallet::Operators::<Test>::get(1).unwrap();
        assert_eq!(info.name.len(), 64);
    });
}

#[test]
fn settlement_count_saturates() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            1_000_000
        ));
        for _ in 0..5 {
            assert_ok!(Operators::increment_settlement_count(0));
        }
        let info = crate::pallet::Operators::<Test>::get(1).unwrap();
        assert_eq!(info.settlement_count, 5);
    });
}

#[test]
fn next_operator_id_increments() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            1_000_000
        ));
        assert_eq!(NextOperatorId::<Test>::get(), 1);
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(2),
            bounded_name("B"),
            1_000_000
        ));
        assert_eq!(NextOperatorId::<Test>::get(), 2);
    });
}

#[test]
fn registered_at_is_block_number() {
    new_test_ext().execute_with(|| {
        System::set_block_number(10);
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            1_000_000
        ));
        let info = crate::pallet::Operators::<Test>::get(1).unwrap();
        assert_eq!(info.registered_at, 10);
    });
}

#[test]
fn events_emitted_on_register_and_update() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            bounded_name("A"),
            1_000_000
        ));
        System::assert_last_event(RuntimeEvent::Operators(
            crate::pallet::Event::<Test>::OperatorRegistered(0, 1, bounded_name("A")),
        ));
        assert_ok!(Operators::update_operator_status(
            RuntimeOrigin::root(),
            0,
            OperatorStatus::Suspended
        ));
        System::assert_last_event(RuntimeEvent::Operators(
            crate::pallet::Event::<Test>::OperatorStatusChanged(
                0,
                OperatorStatus::Active,
                OperatorStatus::Suspended,
            ),
        ));
        assert_ok!(Operators::increase_collateral(RuntimeOrigin::signed(1), 1));
        System::assert_last_event(RuntimeEvent::Operators(
            crate::pallet::Event::<Test>::CollateralIncreased(0, 1, 1_000_001),
        ));
    });
}
