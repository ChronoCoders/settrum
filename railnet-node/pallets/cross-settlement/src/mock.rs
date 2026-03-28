use crate as pallet_cross_settlement;
use frame_support::{construct_runtime, parameter_types, traits::ConstU32, BoundedVec};
use frame_system;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

construct_runtime!(
    pub struct Test {
        System: frame_system,
        Operators: pallet_operators::pallet::{Pallet, Call, Storage, Event<T>},
        AssetRegistry: pallet_asset_registry::pallet::{Pallet, Call, Storage, Event<T>},
        SettlementEngine: pallet_settlement_engine::pallet::{Pallet, Call, Storage, Event<T>},
        CrossSettlement: pallet_cross_settlement::pallet::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = frame_system::mocking::MockBlock<Test>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type ExtensionsWeightInfo = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

parameter_types! {
    pub const MinCollateral: u64 = 1_000_000;
}

impl pallet_operators::pallet::Config for Test {
    type Balance = u64;
    type MinCollateral = MinCollateral;
}

impl pallet_asset_registry::pallet::Config for Test {}

impl pallet_settlement_engine::pallet::Config for Test {}

impl pallet_cross_settlement::pallet::Config for Test {}

pub fn new_test_ext() -> TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    t.into()
}

/// Register a single operator with account `account` and return its operator_id.
pub fn register_operator(account: u64) -> u32 {
    let id = pallet_operators::pallet::NextOperatorId::<Test>::get();
    pallet_operators::pallet::Pallet::<Test>::register_operator(
        frame_system::RawOrigin::Signed(account).into(),
        BoundedVec::try_from(format!("Op{account}").as_bytes().to_vec()).unwrap(),
        1_000_000u64,
    )
    .unwrap();
    id
}

/// Register a Fiat asset as `account` (operator) and return its asset_id.
pub fn register_asset(account: u64) -> u32 {
    let id = pallet_asset_registry::pallet::NextAssetId::<Test>::get();
    pallet_asset_registry::pallet::Pallet::<Test>::register_asset(
        frame_system::RawOrigin::Signed(account).into(),
        pallet_asset_registry::pallet::AssetType::Fiat,
        BoundedVec::try_from(b"USD".to_vec()).unwrap(),
        BoundedVec::try_from(b"USD".to_vec()).unwrap(),
        2u8,
        BoundedVec::try_from(b"rules".to_vec()).unwrap(),
    )
    .unwrap();
    id
}

/// Issue `amount` of `asset_id` to `to_account` via `operator_account`.
pub fn issue_balance(
    operator_account: u64,
    operator_id: u32,
    asset_id: u32,
    to: u64,
    amount: u128,
) {
    pallet_settlement_engine::pallet::Pallet::<Test>::submit_settlement(
        frame_system::RawOrigin::Signed(operator_account).into(),
        operator_id,
        asset_id,
        pallet_settlement_engine::pallet::SettlementOperation::Issue,
        amount,
        to,
        to,
        BoundedVec::try_from(b"ISSUE".to_vec()).unwrap(),
    )
    .unwrap();
    let s_id = pallet_settlement_engine::pallet::NextSettlementId::<Test>::get().saturating_sub(1);
    pallet_settlement_engine::pallet::Pallet::<Test>::finalize_settlement(
        frame_system::RawOrigin::Root.into(),
        s_id,
    )
    .unwrap();
}

pub fn bvec(s: &[u8]) -> BoundedVec<u8, ConstU32<256>> {
    BoundedVec::try_from(s.to_vec()).unwrap()
}

pub fn participants(ids: &[u32]) -> BoundedVec<u32, ConstU32<10>> {
    BoundedVec::try_from(ids.to_vec()).unwrap()
}

pub fn legs(v: Vec<(u32, u64, u64, u128)>) -> BoundedVec<crate::pallet::Leg<Test>, ConstU32<20>> {
    BoundedVec::try_from(
        v.into_iter()
            .map(|(asset_id, from, to, amount)| crate::pallet::Leg::<Test> {
                asset_id,
                from,
                to,
                amount,
            })
            .collect::<Vec<_>>(),
    )
    .unwrap()
}
