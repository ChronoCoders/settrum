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
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
        type Balance: Parameter
            + Default
            + Copy
            + MaxEncodedLen
            + TypeInfo
            + PartialOrd
            + core::ops::Add<Output = Self::Balance>;
        type MinCollateral: Get<Self::Balance>;
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
    pub enum OperatorStatus {
        Active,
        Suspended,
        Terminated,
    }

    #[derive(
        Clone, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo, PartialEq, Eq,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct OperatorInfo<T: Config> {
        pub id: u32,
        pub account: T::AccountId,
        pub name: BoundedVec<u8, ConstU32<64>>,
        pub collateral: T::Balance,
        pub status: OperatorStatus,
        pub settlement_count: u64,
        pub registered_at: u32,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Operators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, OperatorInfo<T>>;

    #[pallet::storage]
    pub type OperatorAccountById<T: Config> = StorageMap<_, Blake2_128Concat, u32, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn next_operator_id)]
    pub type NextOperatorId<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        OperatorRegistered(u32, T::AccountId, BoundedVec<u8, ConstU32<64>>),
        OperatorStatusChanged(u32, OperatorStatus, OperatorStatus),
        CollateralIncreased(u32, T::Balance, T::Balance),
    }

    #[pallet::error]
    pub enum Error<T> {
        InsufficientCollateral,
        OperatorNotFound,
        Unauthorized,
        InvalidStatus,
        AlreadyRegistered,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn register_operator(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<64>>,
            collateral: T::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !Operators::<T>::contains_key(&who),
                Error::<T>::AlreadyRegistered
            );

            let min_collateral: T::Balance = T::MinCollateral::get();
            ensure!(
                collateral >= min_collateral,
                Error::<T>::InsufficientCollateral
            );

            let id = NextOperatorId::<T>::get();
            use frame_support::sp_runtime::SaturatedConversion;
            let block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

            let info = OperatorInfo::<T> {
                id,
                account: who.clone(),
                name: name.clone(),
                collateral,
                status: OperatorStatus::Active,
                settlement_count: 0u64,
                registered_at: block,
            };

            Operators::<T>::insert(&who, &info);
            OperatorAccountById::<T>::insert(id, &who);
            NextOperatorId::<T>::put(id.saturating_add(1));

            Self::deposit_event(Event::<T>::OperatorRegistered(id, who, name));
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn update_operator_status(
            origin: OriginFor<T>,
            operator_id: u32,
            status: OperatorStatus,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let Some(acc) = OperatorAccountById::<T>::get(operator_id) else {
                return Err(Error::<T>::OperatorNotFound.into());
            };

            Operators::<T>::try_mutate(&acc, |maybe| -> DispatchResult {
                let Some(info) = maybe else {
                    return Err(Error::<T>::OperatorNotFound.into());
                };

                let old = info.status;
                match status {
                    OperatorStatus::Active
                    | OperatorStatus::Suspended
                    | OperatorStatus::Terminated => {
                        info.status = status;
                    }
                }
                Self::deposit_event(Event::<T>::OperatorStatusChanged(operator_id, old, status));
                Ok(())
            })
        }

        #[pallet::call_index(2)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn increase_collateral(origin: OriginFor<T>, amount: T::Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Operators::<T>::try_mutate(&who, |maybe| -> DispatchResult {
                let Some(info) = maybe else {
                    return Err(Error::<T>::OperatorNotFound.into());
                };

                let new_total = info.collateral + amount;
                info.collateral = new_total;
                Self::deposit_event(Event::<T>::CollateralIncreased(info.id, amount, new_total));
                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn increment_settlement_count(operator_id: u32) -> DispatchResult {
            let Some(acc) = OperatorAccountById::<T>::get(operator_id) else {
                return Err(Error::<T>::OperatorNotFound.into());
            };
            Operators::<T>::try_mutate(acc, |maybe| -> DispatchResult {
                let Some(info) = maybe else {
                    return Err(Error::<T>::OperatorNotFound.into());
                };
                info.settlement_count = info.settlement_count.saturating_add(1);
                Ok(())
            })
        }
    }
}
