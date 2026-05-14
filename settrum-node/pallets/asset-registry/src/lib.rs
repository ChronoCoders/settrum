#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, BoundedVec};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config<RuntimeEvent: From<Event<Self>>> + pallet_operators::pallet::Config
    {
    }

    #[derive(
        Clone,
        Copy,
        Encode,
        Decode,
        DecodeWithMemTracking,
        MaxEncodedLen,
        TypeInfo,
        PartialEq,
        Eq,
        Debug,
    )]
    pub enum AssetType {
        Fiat,
        Commodity,
        Security,
        InternalLedger,
    }

    #[derive(
        Clone, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo, PartialEq, Eq,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct AssetInfo<T: Config> {
        pub asset_type: AssetType,
        pub issuer: T::AccountId,
        pub name: BoundedVec<u8, ConstU32<64>>,
        pub symbol: BoundedVec<u8, ConstU32<12>>,
        pub decimals: u8,
        pub total_supply: u128,
        pub settlement_rules: BoundedVec<u8, ConstU32<256>>,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Assets<T: Config> = StorageMap<_, Blake2_128Concat, u32, AssetInfo<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_asset_id)]
    pub type NextAssetId<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    pub type OperatorAssets<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, u32, ()>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AssetRegistered(
            u32,
            T::AccountId,
            BoundedVec<u8, ConstU32<64>>,
            BoundedVec<u8, ConstU32<12>>,
        ),
        SupplyUpdated(u32, u128, u128),
    }

    #[pallet::error]
    pub enum Error<T> {
        NotAnOperator,
        AssetNotFound,
        Unauthorized,
        InvalidAssetType,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn register_asset(
            origin: OriginFor<T>,
            asset_type: AssetType,
            name: BoundedVec<u8, ConstU32<64>>,
            symbol: BoundedVec<u8, ConstU32<12>>,
            decimals: u8,
            rules: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let is_operator = pallet_operators::pallet::Operators::<T>::get(&who).is_some();
            ensure!(is_operator, Error::<T>::NotAnOperator);

            let id = NextAssetId::<T>::get();
            let info = AssetInfo::<T> {
                asset_type,
                issuer: who.clone(),
                name: name.clone(),
                symbol: symbol.clone(),
                decimals,
                total_supply: 0u128,
                settlement_rules: rules,
            };

            Assets::<T>::insert(id, &info);
            OperatorAssets::<T>::insert(&who, id, ());
            NextAssetId::<T>::put(id.saturating_add(1));

            Self::deposit_event(Event::<T>::AssetRegistered(id, who, name, symbol));
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn update_supply(
            origin: OriginFor<T>,
            asset_id: u32,
            new_supply: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Assets::<T>::try_mutate(asset_id, |maybe| -> DispatchResult {
                let Some(info) = maybe else {
                    return Err(Error::<T>::AssetNotFound.into());
                };
                ensure!(info.issuer == who, Error::<T>::Unauthorized);
                let old = info.total_supply;
                info.total_supply = new_supply;
                Self::deposit_event(Event::<T>::SupplyUpdated(asset_id, old, new_supply));
                Ok(())
            })
        }

        #[pallet::call_index(2)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn update_rules(
            origin: OriginFor<T>,
            asset_id: u32,
            new_rules: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Assets::<T>::try_mutate(asset_id, |maybe| -> DispatchResult {
                let Some(info) = maybe else {
                    return Err(Error::<T>::AssetNotFound.into());
                };
                ensure!(info.issuer == who, Error::<T>::Unauthorized);
                info.settlement_rules = new_rules;
                Ok(())
            })
        }
    }
}
