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
        frame_system::Config<RuntimeEvent: From<Event<Self>>>
        + pallet_operators::pallet::Config
        + pallet_asset_registry::pallet::Config
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
    pub enum SettlementOperation {
        Issue,
        Redeem,
        Transfer,
        Lock,
        Unlock,
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
    pub enum SettlementStatus {
        Pending,
        Finalized,
        Disputed,
    }

    #[derive(
        Clone, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo, PartialEq, Eq,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct SettlementInfo<T: Config> {
        pub id: u32,
        pub operator_id: u32,
        pub asset_id: u32,
        pub operation: SettlementOperation,
        pub amount: u128,
        pub from: T::AccountId,
        pub to: T::AccountId,
        pub reference: BoundedVec<u8, ConstU32<256>>,
        pub status: SettlementStatus,
        pub submitted_at: u32,
        pub finalized_at: Option<u32>,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Settlements<T: Config> = StorageMap<_, Blake2_128Concat, u32, SettlementInfo<T>>;

    #[pallet::storage]
    pub type AccountBalances<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        u128,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type LockedBalances<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        u128,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_settlement_id)]
    pub type NextSettlementId<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SettlementSubmitted(u32, u32, u32, SettlementOperation),
        SettlementFinalized(u32, u32),
        SettlementDisputed(u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        OperatorNotFound,
        OperatorNotActive,
        AssetNotFound,
        SettlementNotFound,
        SettlementNotPending,
        InsufficientBalance,
        InsufficientLockedBalance,
        ArithmeticOverflow,
        Unauthorized,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[allow(clippy::too_many_arguments)]
        #[pallet::call_index(0)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn submit_settlement(
            origin: OriginFor<T>,
            operator_id: u32,
            asset_id: u32,
            operation: SettlementOperation,
            amount: u128,
            from: T::AccountId,
            to: T::AccountId,
            reference: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let acc = pallet_operators::pallet::OperatorAccountById::<T>::get(operator_id)
                .ok_or(Error::<T>::OperatorNotFound)?;
            ensure!(acc == who, Error::<T>::Unauthorized);

            let operator = pallet_operators::pallet::Operators::<T>::get(&who)
                .ok_or(Error::<T>::OperatorNotFound)?;
            ensure!(
                operator.status == pallet_operators::pallet::OperatorStatus::Active,
                Error::<T>::OperatorNotActive
            );

            ensure!(
                pallet_asset_registry::pallet::Assets::<T>::contains_key(asset_id),
                Error::<T>::AssetNotFound
            );

            let id = NextSettlementId::<T>::get();
            use frame_support::sp_runtime::traits::SaturatedConversion;
            let block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

            let info = SettlementInfo::<T> {
                id,
                operator_id,
                asset_id,
                operation,
                amount,
                from,
                to,
                reference,
                status: SettlementStatus::Pending,
                submitted_at: block,
                finalized_at: None,
            };

            Settlements::<T>::insert(id, &info);
            NextSettlementId::<T>::put(id.saturating_add(1));
            pallet_operators::pallet::Pallet::<T>::increment_settlement_count(operator_id)?;

            Self::deposit_event(Event::<T>::SettlementSubmitted(
                id,
                operator_id,
                asset_id,
                operation,
            ));
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn finalize_settlement(origin: OriginFor<T>, settlement_id: u32) -> DispatchResult {
            ensure_root(origin)?;

            Settlements::<T>::try_mutate(settlement_id, |maybe| -> DispatchResult {
                let info = maybe.as_mut().ok_or(Error::<T>::SettlementNotFound)?;
                ensure!(
                    info.status == SettlementStatus::Pending,
                    Error::<T>::SettlementNotPending
                );

                let from = info.from.clone();
                let to = info.to.clone();
                let operation = info.operation;
                let amount = info.amount;
                let asset_id = info.asset_id;

                Self::execute_operation(asset_id, operation, amount, &from, &to)?;

                use frame_support::sp_runtime::traits::SaturatedConversion;
                let block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();
                info.status = SettlementStatus::Finalized;
                info.finalized_at = Some(block);

                Self::deposit_event(Event::<T>::SettlementFinalized(settlement_id, block));
                Ok(())
            })
        }

        #[pallet::call_index(2)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn dispute_settlement(origin: OriginFor<T>, settlement_id: u32) -> DispatchResult {
            ensure_root(origin)?;

            Settlements::<T>::try_mutate(settlement_id, |maybe| -> DispatchResult {
                let info = maybe.as_mut().ok_or(Error::<T>::SettlementNotFound)?;
                ensure!(
                    info.status == SettlementStatus::Pending,
                    Error::<T>::SettlementNotPending
                );

                info.status = SettlementStatus::Disputed;
                Self::deposit_event(Event::<T>::SettlementDisputed(settlement_id));
                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        fn execute_operation(
            asset_id: u32,
            operation: SettlementOperation,
            amount: u128,
            from: &T::AccountId,
            to: &T::AccountId,
        ) -> DispatchResult {
            match operation {
                SettlementOperation::Issue => {
                    AccountBalances::<T>::try_mutate(to, asset_id, |bal| -> DispatchResult {
                        *bal = bal
                            .checked_add(amount)
                            .ok_or(Error::<T>::ArithmeticOverflow)?;
                        Ok(())
                    })?;
                    pallet_asset_registry::pallet::Assets::<T>::try_mutate(
                        asset_id,
                        |maybe| -> DispatchResult {
                            let asset = maybe.as_mut().ok_or(Error::<T>::AssetNotFound)?;
                            asset.total_supply = asset
                                .total_supply
                                .checked_add(amount)
                                .ok_or(Error::<T>::ArithmeticOverflow)?;
                            Ok(())
                        },
                    )?;
                }
                SettlementOperation::Redeem => {
                    AccountBalances::<T>::try_mutate(from, asset_id, |bal| -> DispatchResult {
                        ensure!(*bal >= amount, Error::<T>::InsufficientBalance);
                        *bal = bal.saturating_sub(amount);
                        Ok(())
                    })?;
                    pallet_asset_registry::pallet::Assets::<T>::try_mutate(
                        asset_id,
                        |maybe| -> DispatchResult {
                            let asset = maybe.as_mut().ok_or(Error::<T>::AssetNotFound)?;
                            asset.total_supply = asset
                                .total_supply
                                .checked_sub(amount)
                                .ok_or(Error::<T>::ArithmeticOverflow)?;
                            Ok(())
                        },
                    )?;
                }
                SettlementOperation::Transfer => {
                    AccountBalances::<T>::try_mutate(from, asset_id, |bal| -> DispatchResult {
                        ensure!(*bal >= amount, Error::<T>::InsufficientBalance);
                        *bal = bal.saturating_sub(amount);
                        Ok(())
                    })?;
                    AccountBalances::<T>::try_mutate(to, asset_id, |bal| -> DispatchResult {
                        *bal = bal
                            .checked_add(amount)
                            .ok_or(Error::<T>::ArithmeticOverflow)?;
                        Ok(())
                    })?;
                }
                SettlementOperation::Lock => {
                    AccountBalances::<T>::try_mutate(from, asset_id, |bal| -> DispatchResult {
                        ensure!(*bal >= amount, Error::<T>::InsufficientBalance);
                        *bal = bal.saturating_sub(amount);
                        Ok(())
                    })?;
                    LockedBalances::<T>::try_mutate(from, asset_id, |bal| -> DispatchResult {
                        *bal = bal
                            .checked_add(amount)
                            .ok_or(Error::<T>::ArithmeticOverflow)?;
                        Ok(())
                    })?;
                }
                SettlementOperation::Unlock => {
                    LockedBalances::<T>::try_mutate(from, asset_id, |bal| -> DispatchResult {
                        ensure!(*bal >= amount, Error::<T>::InsufficientLockedBalance);
                        *bal = bal.saturating_sub(amount);
                        Ok(())
                    })?;
                    AccountBalances::<T>::try_mutate(from, asset_id, |bal| -> DispatchResult {
                        *bal = bal
                            .checked_add(amount)
                            .ok_or(Error::<T>::ArithmeticOverflow)?;
                        Ok(())
                    })?;
                }
            }
            Ok(())
        }
    }
}
