//! Serde conformance for the public wire types: a `ProofSpec` round-trips
//! through JSON and TOML, a `Proof` round-trips through JSON, and a spec whose
//! embedded strategy is not a JSON object is rejected.

mod common;

use proof_core::{canonicalize, prove, ProofSpec};

#[test]
fn proof_spec_json_round_trips() {
    let spec = common::sample_spec();
    let json = serde_json::to_string(&spec).unwrap();
    let back = ProofSpec::from_json(&json).unwrap();
    assert_eq!(spec, back);
}

#[test]
fn proof_spec_toml_round_trips() {
    // A minimal spec expressed as TOML; the embedded strategy is an inline table.
    let toml = r#"
dataset_ref = "test/TEST/1h"

[strategy]
symbol = "TEST"
timeframe = "1h"

[strategy.indicators.ema_fast]
type = "Ema"
params = [3]

[strategy.indicators.ema_slow]
type = "Ema"
params = [8]

[strategy.entry]
cross_above = ["ema_fast", "ema_slow"]

[strategy.exit]
cross_below = ["ema_fast", "ema_slow"]

[strategy.sizing]
type = "fixed_fraction"
fraction = 0.95

[strategy.costs]
taker_bps = 5

[strategy.costs.slippage]
type = "fixed_bps"
bps = 2

[strategy.risk]
trailing_stop_pct = 5.0
"#;
    let spec = ProofSpec::from_toml(toml).unwrap();
    assert_eq!(spec.dataset_ref, "test/TEST/1h");
    assert!(spec.strategy.is_object());
    // The parsed strategy proves without error against the sample data.
    prove(&spec, &common::sample_data()).unwrap();
}

#[test]
fn proof_json_round_trips() {
    let proof = prove(&common::sample_spec(), &common::sample_data()).unwrap();
    let json = serde_json::to_string(&proof).unwrap();
    let back: proof_core::Proof = serde_json::from_str(&json).unwrap();
    assert_eq!(proof.report_hash, back.report_hash);
    assert_eq!(proof.inputs_hash, back.inputs_hash);
    assert_eq!(proof.engine_version, back.engine_version);
    // The report body round-trips *canonically*: serde_json's default parser can
    // land a float one ULP off, but canonicalization quantizes to 1e-8, so the
    // canonical form — the exact bytes the hash is taken over — is identical. That
    // canonical stability, not raw f64 bit-equality, is the determinism contract.
    assert_eq!(
        canonicalize(&proof.report).unwrap(),
        canonicalize(&back.report).unwrap()
    );
}

#[test]
fn strategy_must_be_an_object() {
    // A scalar strategy is not a StrategySpec: from_json rejects it.
    assert!(ProofSpec::from_json(r#"{"strategy": 42, "dataset_ref": "x"}"#).is_err());
    assert!(ProofSpec::from_json(r#"{"strategy": [1,2], "dataset_ref": "x"}"#).is_err());
}

#[test]
fn missing_field_is_rejected() {
    // `dataset_ref` is required.
    assert!(ProofSpec::from_json(r#"{"strategy": {}}"#).is_err());
}
