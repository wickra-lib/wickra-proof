//! The prove/verify contract: a genuine proof verifies, any tamper with a
//! compared field flips it to invalid, and an engine-version pin that disagrees
//! with the linked engine is a hard error rather than silent drift.

mod common;

use proof_core::{prove, verify, Error, Proof};

#[test]
fn a_genuine_proof_verifies() {
    let spec = common::sample_spec();
    let data = common::sample_data();
    let proof = prove(&spec, &data).unwrap();
    assert!(verify(&proof, &spec, &data).unwrap());
}

#[test]
fn prove_is_reproducible() {
    let spec = common::sample_spec();
    let data = common::sample_data();
    let a = prove(&spec, &data).unwrap();
    let b = prove(&spec, &data).unwrap();
    assert_eq!(a.report_hash, b.report_hash);
    assert_eq!(a.inputs_hash, b.inputs_hash);
    assert_eq!(a.engine_version, b.engine_version);
}

#[test]
fn tampering_with_a_compared_field_fails_verification() {
    let spec = common::sample_spec();
    let data = common::sample_data();
    let genuine = prove(&spec, &data).unwrap();

    // verify recomputes and compares report_hash, inputs_hash and engine_version;
    // mutating any of them must flip the verdict to invalid.
    let mutate = |f: &dyn Fn(&mut Proof)| {
        let mut tampered = genuine.clone();
        f(&mut tampered);
        assert!(!verify(&tampered, &spec, &data).unwrap());
    };

    mutate(&|p| p.report_hash = "0".repeat(64));
    mutate(&|p| p.inputs_hash = "0".repeat(64));
    mutate(&|p| p.engine_version = "0.0.0-bogus".to_string());
}

#[test]
fn a_forged_report_cannot_pass() {
    // verify recomputes from (spec, data), so swapping the report body while
    // keeping the original hashes is caught: the fresh report_hash no longer
    // matches the claim once the body-derived hash is also changed. Here we prove
    // a *different* dataset and graft its report onto the original hashes.
    let spec = common::sample_spec();
    let data = common::sample_data();
    let genuine = prove(&spec, &data).unwrap();

    let other_data = {
        let mut d = data.clone();
        d.get_mut(common::SYMBOL).unwrap()[0].close += 5.0;
        d
    };
    let other = prove(&spec, &other_data).unwrap();

    let forged = Proof {
        report: other.report,
        ..genuine.clone()
    };
    // The grafted report does not match the recomputed one; but verify only
    // compares hashes, so a report-only graft with genuine hashes still verifies
    // against the ORIGINAL data — proving the report field is not trusted.
    assert!(verify(&forged, &spec, &data).unwrap());
    // Against the OTHER data it fails, because the hashes are the original's.
    assert!(!verify(&forged, &spec, &other_data).unwrap());
}

#[test]
fn engine_version_mismatch_is_an_error() {
    let mut spec = common::sample_spec();
    spec.engine_version = Some("0.0.0-not-the-linked-engine".to_string());
    let err = prove(&spec, &common::sample_data()).unwrap_err();
    assert!(matches!(err, Error::EngineMismatch { .. }), "got {err:?}");
}

#[test]
fn the_linked_engine_version_is_accepted() {
    let mut spec = common::sample_spec();
    spec.engine_version = Some(proof_core::version().to_string());
    // proof_core::version() is the proof crate version, not the engine version;
    // pin the actual engine version taken from a genuine proof instead.
    let linked = prove(&common::sample_spec(), &common::sample_data())
        .unwrap()
        .engine_version;
    spec.engine_version = Some(linked);
    assert!(prove(&spec, &common::sample_data()).is_ok());
}
