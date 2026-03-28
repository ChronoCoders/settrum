use crate as pallet_operators;
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

pub fn new_test_ext() -> TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    t.into()
}

pub fn bounded_name(s: &str) -> BoundedVec<u8, ConstU32<64>> {
    BoundedVec::try_from(s.as_bytes().to_vec()).unwrap()
}
