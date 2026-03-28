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
    pub enum ProofType {
        Signature,
        Oracle,
        Multisig,
        ZeroKnowledge,
        Documentary,
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
    pub enum ProofStatus {
        Pending,
        Verified,
        Revoked,
    }

    #[derive(
        Clone, Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo, PartialEq, Eq,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct ProofInfo<T: Config> {
        pub id: u32,
        pub settlement_id: u32,
        pub proof_type: ProofType,
        pub hash: T::Hash,
        pub submitter: T::AccountId,
        pub data: BoundedVec<u8, ConstU32<1024>>,
        pub status: ProofStatus,
        pub submitted_at: u32,
        pub verified_at: Option<u32>,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, u32, ProofInfo<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_proof_id)]
    pub type NextProofId<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    pub type SettlementToProof<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, u32, ()>;

    #[pallet::storage]
    pub type ProofHashes<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, u32>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProofSubmitted(u32, u32, ProofType),
        ProofVerified(u32, u32),
        ProofRevoked(u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        SettlementNotFound,
        ProofNotFound,
        DuplicateProofHash,
        ProofNotPending,
        ProofAlreadyRevoked,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn submit_proof(
            origin: OriginFor<T>,
            settlement_id: u32,
            proof_type: ProofType,
            hash: T::Hash,
            data: BoundedVec<u8, ConstU32<1024>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                pallet_settlement_engine::pallet::Settlements::<T>::contains_key(settlement_id),
                Error::<T>::SettlementNotFound
            );
            ensure!(
                !ProofHashes::<T>::contains_key(hash),
                Error::<T>::DuplicateProofHash
            );

            let id = NextProofId::<T>::get();
            use frame_support::sp_runtime::traits::SaturatedConversion;
            let block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

            let info = ProofInfo::<T> {
                id,
                settlement_id,
                proof_type,
                hash,
                submitter: who,
                data,
                status: ProofStatus::Pending,
                submitted_at: block,
                verified_at: None,
            };

            Proofs::<T>::insert(id, &info);
            NextProofId::<T>::put(id.saturating_add(1));
            SettlementToProof::<T>::insert(settlement_id, id, ());
            ProofHashes::<T>::insert(hash, id);

            Self::deposit_event(Event::<T>::ProofSubmitted(id, settlement_id, proof_type));
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn verify_proof(origin: OriginFor<T>, proof_id: u32) -> DispatchResult {
            ensure_root(origin)?;

            Proofs::<T>::try_mutate(proof_id, |maybe| -> DispatchResult {
                let info = maybe.as_mut().ok_or(Error::<T>::ProofNotFound)?;
                ensure!(
                    info.status == ProofStatus::Pending,
                    Error::<T>::ProofNotPending
                );

                use frame_support::sp_runtime::traits::SaturatedConversion;
                let block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();
                info.status = ProofStatus::Verified;
                info.verified_at = Some(block);

                Self::deposit_event(Event::<T>::ProofVerified(proof_id, block));
                Ok(())
            })
        }

        #[pallet::call_index(2)]
        #[pallet::weight(frame_support::weights::Weight::from_parts(10_000, 0))]
        pub fn revoke_proof(origin: OriginFor<T>, proof_id: u32) -> DispatchResult {
            ensure_root(origin)?;

            Proofs::<T>::try_mutate(proof_id, |maybe| -> DispatchResult {
                let info = maybe.as_mut().ok_or(Error::<T>::ProofNotFound)?;
                ensure!(
                    info.status != ProofStatus::Revoked,
                    Error::<T>::ProofAlreadyRevoked
                );

                info.status = ProofStatus::Revoked;
                Self::deposit_event(Event::<T>::ProofRevoked(proof_id));
                Ok(())
            })
        }
    }
}
