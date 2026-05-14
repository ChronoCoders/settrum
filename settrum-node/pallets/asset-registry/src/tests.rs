use crate::mock::{
    name, new_test_ext, rules, symbol, AssetRegistry, Operators, RuntimeEvent, RuntimeOrigin,
    System, Test,
};
use crate::pallet::{AssetType, Error};
use crate::pallet::{Assets, NextAssetId, OperatorAssets};
use frame_support::{assert_err, assert_noop, assert_ok};

#[test]
fn register_asset_works() {
    new_test_ext().execute_with(|| {
        frame_system::Pallet::<Test>::set_block_number(1);

        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            b"Operator".to_vec().try_into().unwrap(),
            1_000_000
        ));

        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Fiat,
            b"USD".to_vec().try_into().unwrap(),
            b"USD".to_vec().try_into().unwrap(),
            2,
            b"rules".to_vec().try_into().unwrap()
        ));
    });
}

#[test]
fn register_asset_by_operator() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            frame_support::BoundedVec::try_from(b"Op".to_vec()).unwrap(),
            1_000_000
        ));
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Fiat,
            name("USD"),
            symbol("USD"),
            2u8,
            rules("x")
        ));
        let info = Assets::<Test>::get(0).unwrap();
        assert_eq!(info.issuer, 1);
        assert_eq!(info.asset_type, AssetType::Fiat);
    });
}

#[test]
fn register_asset_non_operator_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AssetRegistry::register_asset(
                RuntimeOrigin::signed(2),
                AssetType::Commodity,
                name("GOLD"),
                symbol("XAU"),
                0u8,
                rules("y")
            ),
            Error::<Test>::NotAnOperator
        );
    });
}

#[test]
fn update_supply_by_issuer() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            frame_support::BoundedVec::try_from(b"Op".to_vec()).unwrap(),
            1_000_000
        ));
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Security,
            name("BOND"),
            symbol("BND"),
            0u8,
            rules("z")
        ));
        assert_ok!(AssetRegistry::update_supply(
            RuntimeOrigin::signed(1),
            0,
            10_000
        ));
        let info = Assets::<Test>::get(0).unwrap();
        assert_eq!(info.total_supply, 10_000);
        System::assert_last_event(RuntimeEvent::AssetRegistry(
            crate::pallet::Event::<Test>::SupplyUpdated(0, 0, 10_000),
        ));
    });
}

#[test]
fn update_supply_non_issuer_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            frame_support::BoundedVec::try_from(b"Op".to_vec()).unwrap(),
            1_000_000
        ));
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Security,
            name("BOND"),
            symbol("BND"),
            0u8,
            rules("z")
        ));
        assert_err!(
            AssetRegistry::update_supply(RuntimeOrigin::signed(2), 0, 10_000),
            Error::<Test>::Unauthorized
        );
    });
}

#[test]
fn multiple_asset_types() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            frame_support::BoundedVec::try_from(b"Op".to_vec()).unwrap(),
            1_000_000
        ));
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Fiat,
            name("EUR"),
            symbol("EUR"),
            2u8,
            rules("r")
        ));
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Commodity,
            name("SILV"),
            symbol("XAG"),
            0u8,
            rules("s")
        ));
        let a0 = Assets::<Test>::get(0).unwrap();
        let a1 = Assets::<Test>::get(1).unwrap();
        assert_eq!(a0.asset_type, AssetType::Fiat);
        assert_eq!(a1.asset_type, AssetType::Commodity);
    });
}

#[test]
fn update_rules_by_issuer() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            frame_support::BoundedVec::try_from(b"Op".to_vec()).unwrap(),
            1_000_000
        ));
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::InternalLedger,
            name("IL"),
            symbol("IL"),
            0u8,
            rules("a")
        ));
        assert_ok!(AssetRegistry::update_rules(
            RuntimeOrigin::signed(1),
            0,
            rules("b")
        ));
        let info = Assets::<Test>::get(0).unwrap();
        assert_eq!(info.settlement_rules, rules("b"));
    });
}

#[test]
fn update_rules_non_issuer_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            frame_support::BoundedVec::try_from(b"Op".to_vec()).unwrap(),
            1_000_000
        ));
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::InternalLedger,
            name("IL"),
            symbol("IL"),
            0u8,
            rules("a")
        ));
        assert_err!(
            AssetRegistry::update_rules(RuntimeOrigin::signed(2), 0, rules("b")),
            Error::<Test>::Unauthorized
        );
    });
}

#[test]
fn operator_assets_tracking() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            frame_support::BoundedVec::try_from(b"Op".to_vec()).unwrap(),
            1_000_000u64
        ));
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Fiat,
            name("USD"),
            symbol("USD"),
            2u8,
            rules("r")
        ));
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Security,
            name("STK"),
            symbol("STK"),
            0u8,
            rules("s")
        ));
        assert!(OperatorAssets::<Test>::contains_key(1, 0));
        assert!(OperatorAssets::<Test>::contains_key(1, 1));
    });
}

#[test]
fn next_asset_id_increments() {
    new_test_ext().execute_with(|| {
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            frame_support::BoundedVec::try_from(b"Op".to_vec()).unwrap(),
            1_000_000
        ));
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Fiat,
            name("USD"),
            symbol("USD"),
            2u8,
            rules("r")
        ));
        assert_eq!(NextAssetId::<Test>::get(), 1);
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Commodity,
            name("OIL"),
            symbol("OIL"),
            0u8,
            rules("s")
        ));
        assert_eq!(NextAssetId::<Test>::get(), 2);
    });
}

#[test]
fn asset_not_found_update_supply_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AssetRegistry::update_supply(RuntimeOrigin::signed(1), 99, 1),
            Error::<Test>::AssetNotFound
        );
    });
}

#[test]
fn asset_not_found_update_rules_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AssetRegistry::update_rules(RuntimeOrigin::signed(1), 99, rules("x")),
            Error::<Test>::AssetNotFound
        );
    });
}

#[test]
fn bounded_lengths() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Operators::register_operator(
            RuntimeOrigin::signed(1),
            frame_support::BoundedVec::try_from(b"Op".to_vec()).unwrap(),
            1_000_000
        ));
        let n = name(&String::from_utf8(vec![b'a'; 64]).unwrap());
        let s = symbol(&String::from_utf8(vec![b'b'; 12]).unwrap());
        let r = rules(&String::from_utf8(vec![b'c'; 256]).unwrap());
        assert_ok!(AssetRegistry::register_asset(
            RuntimeOrigin::signed(1),
            AssetType::Fiat,
            n.clone(),
            s.clone(),
            6u8,
            r.clone()
        ));
        let info = Assets::<Test>::get(0).unwrap();
        assert_eq!(info.name.len(), 64);
        assert_eq!(info.symbol.len(), 12);
        assert_eq!(info.settlement_rules.len(), 256);
        System::assert_last_event(RuntimeEvent::AssetRegistry(
            crate::pallet::Event::<Test>::AssetRegistered(0, 1, n, s),
        ));
    });
}
