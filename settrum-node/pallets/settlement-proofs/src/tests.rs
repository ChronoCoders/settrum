use crate::mock::{make_hash, new_test_ext, proof_data, setup, RuntimeOrigin, Test};
use crate::pallet::{
    Error, NextProofId, ProofHashes, ProofStatus, ProofType, Proofs, SettlementToProof,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn submit_proof_works() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        let info = Proofs::<Test>::get(0).unwrap();
        assert_eq!(info.settlement_id, settlement_id);
        assert_eq!(info.proof_type, ProofType::Signature);
        assert_eq!(info.status, ProofStatus::Pending);
        assert_eq!(info.verified_at, None);
        assert_eq!(info.submitter, 1u64);
    });
}

#[test]
fn submit_proof_settlement_not_found() {
    new_test_ext().execute_with(|| {
        setup();
        assert_noop!(
            crate::pallet::Pallet::<Test>::submit_proof(
                RuntimeOrigin::signed(1),
                99,
                ProofType::Signature,
                make_hash(1),
                proof_data(),
            ),
            Error::<Test>::SettlementNotFound
        );
    });
}

#[test]
fn submit_proof_duplicate_hash_rejected() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        assert_noop!(
            crate::pallet::Pallet::<Test>::submit_proof(
                RuntimeOrigin::signed(1),
                settlement_id,
                ProofType::Oracle,
                make_hash(1),
                proof_data(),
            ),
            Error::<Test>::DuplicateProofHash
        );
    });
}

#[test]
fn all_proof_types_accepted() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        let types = [
            ProofType::Signature,
            ProofType::Oracle,
            ProofType::Multisig,
            ProofType::ZeroKnowledge,
            ProofType::Documentary,
        ];
        for (i, pt) in types.iter().enumerate() {
            assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
                RuntimeOrigin::signed(1),
                settlement_id,
                *pt,
                make_hash(i as u64 + 1),
                proof_data(),
            ));
        }
        assert_eq!(NextProofId::<Test>::get(), 5);
    });
}

#[test]
fn verify_proof_works() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::verify_proof(
            RuntimeOrigin::root(),
            0,
        ));
        let info = Proofs::<Test>::get(0).unwrap();
        assert_eq!(info.status, ProofStatus::Verified);
        assert_eq!(info.verified_at, Some(1));
    });
}

#[test]
fn verify_proof_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            crate::pallet::Pallet::<Test>::verify_proof(RuntimeOrigin::root(), 99),
            Error::<Test>::ProofNotFound
        );
    });
}

#[test]
fn verify_proof_not_pending() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::verify_proof(
            RuntimeOrigin::root(),
            0,
        ));
        assert_noop!(
            crate::pallet::Pallet::<Test>::verify_proof(RuntimeOrigin::root(), 0),
            Error::<Test>::ProofNotPending
        );
    });
}

#[test]
fn revoke_pending_proof_works() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::revoke_proof(
            RuntimeOrigin::root(),
            0,
        ));
        let info = Proofs::<Test>::get(0).unwrap();
        assert_eq!(info.status, ProofStatus::Revoked);
    });
}

#[test]
fn revoke_verified_proof_works() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::verify_proof(
            RuntimeOrigin::root(),
            0,
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::revoke_proof(
            RuntimeOrigin::root(),
            0,
        ));
        let info = Proofs::<Test>::get(0).unwrap();
        assert_eq!(info.status, ProofStatus::Revoked);
    });
}

#[test]
fn revoke_proof_already_revoked() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::revoke_proof(
            RuntimeOrigin::root(),
            0,
        ));
        assert_noop!(
            crate::pallet::Pallet::<Test>::revoke_proof(RuntimeOrigin::root(), 0),
            Error::<Test>::ProofAlreadyRevoked
        );
    });
}

#[test]
fn revoke_proof_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            crate::pallet::Pallet::<Test>::revoke_proof(RuntimeOrigin::root(), 99),
            Error::<Test>::ProofNotFound
        );
    });
}

#[test]
fn revoked_proof_cannot_be_verified() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::revoke_proof(
            RuntimeOrigin::root(),
            0,
        ));
        assert_noop!(
            crate::pallet::Pallet::<Test>::verify_proof(RuntimeOrigin::root(), 0),
            Error::<Test>::ProofNotPending
        );
    });
}

#[test]
fn settlement_to_proof_mapping() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Oracle,
            make_hash(2),
            proof_data(),
        ));
        assert!(SettlementToProof::<Test>::contains_key(settlement_id, 0));
        assert!(SettlementToProof::<Test>::contains_key(settlement_id, 1));
        assert!(!SettlementToProof::<Test>::contains_key(settlement_id, 2));
    });
}

#[test]
fn proof_hash_uniqueness_enforced() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        let hash = make_hash(42);
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Documentary,
            hash,
            proof_data(),
        ));
        assert!(ProofHashes::<Test>::contains_key(hash));
        assert_eq!(ProofHashes::<Test>::get(hash), Some(0));
    });
}

#[test]
fn next_proof_id_increments() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        assert_eq!(NextProofId::<Test>::get(), 1);
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Oracle,
            make_hash(2),
            proof_data(),
        ));
        assert_eq!(NextProofId::<Test>::get(), 2);
    });
}

#[test]
fn events_emitted() {
    new_test_ext().execute_with(|| {
        let (_, _, settlement_id) = setup();
        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Signature,
            make_hash(1),
            proof_data(),
        ));
        frame_system::Pallet::<Test>::assert_last_event(
            crate::pallet::Event::<Test>::ProofSubmitted(0, settlement_id, ProofType::Signature)
                .into(),
        );

        assert_ok!(crate::pallet::Pallet::<Test>::verify_proof(
            RuntimeOrigin::root(),
            0,
        ));
        frame_system::Pallet::<Test>::assert_last_event(
            crate::pallet::Event::<Test>::ProofVerified(0, 1).into(),
        );

        assert_ok!(crate::pallet::Pallet::<Test>::submit_proof(
            RuntimeOrigin::signed(1),
            settlement_id,
            ProofType::Oracle,
            make_hash(2),
            proof_data(),
        ));
        assert_ok!(crate::pallet::Pallet::<Test>::revoke_proof(
            RuntimeOrigin::root(),
            1,
        ));
        frame_system::Pallet::<Test>::assert_last_event(
            crate::pallet::Event::<Test>::ProofRevoked(1).into(),
        );
    });
}
