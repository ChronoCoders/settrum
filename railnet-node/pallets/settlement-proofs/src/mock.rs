use crate as pallet_settlement_proofs;
use frame_support::{construct_runtime, parameter_types, traits::ConstU32};
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
        SettlementProofs: pallet_settlement_proofs::pallet::{Pallet, Call, Storage, Event<T>},
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

impl pallet_settlement_proofs::pallet::Config for Test {}

pub fn new_test_ext() -> TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    t.into()
}

pub fn setup() -> (u64, u32, u32) {
    frame_system::Pallet::<Test>::set_block_number(1);

    assert!(pallet_operators::pallet::Pallet::<Test>::register_operator(
        frame_system::RawOrigin::Signed(1).into(),
        frame_support::BoundedVec::try_from(b"Op".to_vec()).unwrap(),
        1_000_000u64,
    )
    .is_ok());

    assert!(
        pallet_asset_registry::pallet::Pallet::<Test>::register_asset(
            frame_system::RawOrigin::Signed(1).into(),
            pallet_asset_registry::pallet::AssetType::Fiat,
            frame_support::BoundedVec::try_from(b"USD".to_vec()).unwrap(),
            frame_support::BoundedVec::try_from(b"USD".to_vec()).unwrap(),
            2u8,
            frame_support::BoundedVec::try_from(b"rules".to_vec()).unwrap(),
        )
        .is_ok()
    );

    assert!(
        pallet_settlement_engine::pallet::Pallet::<Test>::submit_settlement(
            frame_system::RawOrigin::Signed(1).into(),
            0,
            0,
            pallet_settlement_engine::pallet::SettlementOperation::Issue,
            1_000,
            1u64,
            2u64,
            frame_support::BoundedVec::try_from(b"REF".to_vec()).unwrap(),
        )
        .is_ok()
    );

    (1u64, 0u32, 0u32)
}

pub fn make_hash(n: u64) -> sp_core::H256 {
    sp_core::H256::from_low_u64_be(n)
}

pub fn proof_data() -> frame_support::BoundedVec<u8, ConstU32<1024>> {
    frame_support::BoundedVec::try_from(b"proof_bytes".to_vec()).unwrap()
}
