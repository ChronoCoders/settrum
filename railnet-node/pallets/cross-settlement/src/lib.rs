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
        + pallet_settlement_engine::pallet::Config
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
    pub enum CrossSettlementStatus {
        Pending,
        Approved,
        Executed,
        Cancelled,
    }

    #[derive(
        Clone, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo, PartialEq, Eq,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct Leg<T: Config> {
        pub asset_id: u32,
        pub from: T::AccountId,
        pub to: T::AccountId,
        pub amount: u128,
    }

    impl<T: Config> core::fmt::Debug for Leg<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("Leg")
                .field("asset_id", &self.asset_id)
                .field("from", &self.from)
                .field("to", &self.to)
                .field("amount", &self.amount)
                .finish()
        }
    }

    #[derive(
        Clone, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo, PartialEq, Eq,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct CrossSettlementInfo<T: Config> {
        pub id: u32,
        pub initiator_id: u32,
        pub participants: BoundedVec<u32, ConstU32<10>>,
        pub legs: BoundedVec<Leg<T>, ConstU32<20>>,
        pub approvals: BoundedVec<u32, ConstU32<10>>,
        pub reference: BoundedVec<u8, ConstU32<256>>,
        pub status: CrossSettlementStatus,
        pub created_at: u32,
        pub expires_at: u32,
        pub executed_at: Option<u32>,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type CrossSettlements<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, CrossSettlementInfo<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_cross_settlement_id)]
    pub type NextCrossSettlementId<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CrossSettlementProposed(u32, u32),
        ParticipantApproved(u32, u32),
        CrossSettlementApproved(u32),
        CrossSettlementExecuted(u32, u32),
        CrossSettlementCancelled(u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        OperatorNotFound,
        OperatorNotActive,
        AssetNotFound,
        CrossSettlementNotFound,
        NotPending,
        NotApproved,
        AlreadyApproved,
        NotAParticipant,
        Expired,
        NotExecutable,
        InsufficientBalance,
        ArithmeticOverflow,
        TooManyParticipants,
        TooManyLegs,
        ExpiryInPast,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn propose_cross_settlement(
            origin: OriginFor<T>,
            participants: BoundedVec<u32, ConstU32<10>>,
            legs: BoundedVec<Leg<T>, ConstU32<20>>,
            expires_at: u32,
            reference: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            use frame_support::sp_runtime::traits::SaturatedConversion;
            let block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();
            ensure!(expires_at > block, Error::<T>::ExpiryInPast);

            let operator = pallet_operators::pallet::Operators::<T>::get(&who)
                .ok_or(Error::<T>::OperatorNotFound)?;
            ensure!(
                operator.status == pallet_operators::pallet::OperatorStatus::Active,
                Error::<T>::OperatorNotActive
            );
            let initiator_id = operator.id;

            ensure!(
                participants.contains(&initiator_id),
                Error::<T>::NotAParticipant
            );

            for &participant_id in &participants {
                let acc = pallet_operators::pallet::OperatorAccountById::<T>::get(participant_id)
                    .ok_or(Error::<T>::OperatorNotFound)?;
                let op = pallet_operators::pallet::Operators::<T>::get(&acc)
                    .ok_or(Error::<T>::OperatorNotFound)?;
                ensure!(
                    op.status == pallet_operators::pallet::OperatorStatus::Active,
                    Error::<T>::OperatorNotActive
                );
            }

            for leg in &legs {
                ensure!(
                    pallet_asset_registry::pallet::Assets::<T>::contains_key(leg.asset_id),
                    Error::<T>::AssetNotFound
                );
            }

            let id = NextCrossSettlementId::<T>::get();

            let mut approvals: BoundedVec<u32, ConstU32<10>> = BoundedVec::new();
            approvals
                .try_push(initiator_id)
                .map_err(|_| Error::<T>::TooManyParticipants)?;

            let status = if approvals.len() == participants.len() {
                CrossSettlementStatus::Approved
            } else {
                CrossSettlementStatus::Pending
            };

            let info = CrossSettlementInfo::<T> {
                id,
                initiator_id,
                participants,
                legs,
                approvals,
                reference,
                status,
                created_at: block,
                expires_at,
                executed_at: None,
            };

            CrossSettlements::<T>::insert(id, &info);
            NextCrossSettlementId::<T>::put(id.saturating_add(1));

            Self::deposit_event(Event::<T>::CrossSettlementProposed(id, initiator_id));
            if info.status == CrossSettlementStatus::Approved {
                Self::deposit_event(Event::<T>::CrossSettlementApproved(id));
            }
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn approve_cross_settlement(origin: OriginFor<T>, id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let operator = pallet_operators::pallet::Operators::<T>::get(&who)
                .ok_or(Error::<T>::OperatorNotFound)?;
            ensure!(
                operator.status == pallet_operators::pallet::OperatorStatus::Active,
                Error::<T>::OperatorNotActive
            );
            let operator_id = operator.id;

            CrossSettlements::<T>::try_mutate(id, |maybe| -> DispatchResult {
                let info = maybe.as_mut().ok_or(Error::<T>::CrossSettlementNotFound)?;

                ensure!(
                    info.status == CrossSettlementStatus::Pending,
                    Error::<T>::NotPending
                );

                use frame_support::sp_runtime::traits::SaturatedConversion;
                let block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();
                ensure!(block <= info.expires_at, Error::<T>::Expired);

                ensure!(
                    info.participants.contains(&operator_id),
                    Error::<T>::NotAParticipant
                );
                ensure!(!info.approvals.contains(&operator_id), Error::<T>::AlreadyApproved);

                info.approvals
                    .try_push(operator_id)
                    .map_err(|_| Error::<T>::TooManyParticipants)?;

                Self::deposit_event(Event::<T>::ParticipantApproved(id, operator_id));

                if info.approvals.len() == info.participants.len() {
                    info.status = CrossSettlementStatus::Approved;
                    Self::deposit_event(Event::<T>::CrossSettlementApproved(id));
                }

                Ok(())
            })
        }

        #[pallet::call_index(2)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn execute_cross_settlement(origin: OriginFor<T>, id: u32) -> DispatchResult {
            ensure_root(origin)?;

            CrossSettlements::<T>::try_mutate(id, |maybe| -> DispatchResult {
                let info = maybe.as_mut().ok_or(Error::<T>::CrossSettlementNotFound)?;

                ensure!(
                    info.status == CrossSettlementStatus::Approved,
                    Error::<T>::NotApproved
                );

                use frame_support::sp_runtime::traits::SaturatedConversion;
                let block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();
                ensure!(block <= info.expires_at, Error::<T>::Expired);

                for leg in &info.legs {
                    pallet_settlement_engine::pallet::AccountBalances::<T>::try_mutate(
                        &leg.from,
                        leg.asset_id,
                        |bal| -> DispatchResult {
                            ensure!(*bal >= leg.amount, Error::<T>::InsufficientBalance);
                            *bal = bal.saturating_sub(leg.amount);
                            Ok(())
                        },
                    )?;
                    pallet_settlement_engine::pallet::AccountBalances::<T>::try_mutate(
                        &leg.to,
                        leg.asset_id,
                        |bal| -> DispatchResult {
                            *bal = bal
                                .checked_add(leg.amount)
                                .ok_or(Error::<T>::ArithmeticOverflow)?;
                            Ok(())
                        },
                    )?;
                }

                info.status = CrossSettlementStatus::Executed;
                info.executed_at = Some(block);

                Self::deposit_event(Event::<T>::CrossSettlementExecuted(id, block));
                Ok(())
            })
        }

        #[pallet::call_index(3)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn cancel_cross_settlement(origin: OriginFor<T>, id: u32) -> DispatchResult {
            ensure_root(origin)?;

            CrossSettlements::<T>::try_mutate(id, |maybe| -> DispatchResult {
                let info = maybe.as_mut().ok_or(Error::<T>::CrossSettlementNotFound)?;

                ensure!(
                    info.status != CrossSettlementStatus::Executed
                        && info.status != CrossSettlementStatus::Cancelled,
                    Error::<T>::NotExecutable
                );

                info.status = CrossSettlementStatus::Cancelled;
                Self::deposit_event(Event::<T>::CrossSettlementCancelled(id));
                Ok(())
            })
        }
    }
}
