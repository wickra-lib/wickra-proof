//! Inline tests for proof-core: canonicalization vectors, prove/verify
//! round-trips, tamper detection, and the engine-version pin.

use crate::{canonicalize, prove, verify, Config, Error, Proof, ProofSpec, Prover};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use wickra_backtest_core::Candle;

/// A small, valid EMA-cross strategy (the shape wickra-backtest accepts).
fn strategy() -> Value {
    json!({
        "symbol": "BTCUSDT",
        "timeframe": "1h",
        "indicators": {
            "ema_fast": { "type": "Ema", "params": [5] },
            "ema_slow": { "type": "Ema", "params": [15] }
        },
        "entry": { "cross_above": ["ema_fast", "ema_slow"] },
        "exit": { "cross_below": ["ema_fast", "ema_slow"] },
        "sizing": { "type": "fixed_fraction", "fraction": 0.95 },
        "costs": { "taker_bps": 5, "slippage": { "type": "fixed_bps", "bps": 2 } },
        "risk": { "trailing_stop_pct": 5.0 }
    })
}

/// A deterministic oscillating candle series long enough to warm up EMA(15) and
/// produce at least one crossing.
fn candles() -> Vec<Candle> {
    (0..40)
        .map(|i| {
            let t = f64::from(i);
            let base = 100.0 + (t * 0.4).sin() * 8.0;
            Candle {
                time: 1_700_000_000 + i64::from(i) * 3600,
                open: base,
                high: base + 1.0,
                low: base - 1.0,
                close: base + 0.5,
                volume: 1000.0,
            }
        })
        .collect()
}

fn data() -> BTreeMap<String, Vec<Candle>> {
    let mut m = BTreeMap::new();
    m.insert("BTCUSDT".to_string(), candles());
    m
}

fn spec() -> ProofSpec {
    ProofSpec {
        strategy: strategy(),
        dataset_ref: "BTCUSDT/1h/test".to_string(),
        engine_version: None,
    }
}

#[test]
fn canonicalize_sorts_keys_and_strips_whitespace() {
    let v = json!({ "z": 1, "a": 2, "m": { "y": 3, "b": 4 } });
    assert_eq!(
        canonicalize(&v).unwrap(),
        "{\"a\":2,\"m\":{\"b\":4,\"y\":3},\"z\":1}"
    );
}

#[test]
fn canonicalize_rounds_floats_and_keeps_integers() {
    // A float is quantized to 1e-8; a whole value collapses to its integer token
    // (so `1.0` and `1` are indistinguishable across languages), a fractional one
    // keeps its digits.
    let v = json!({ "b": 1.000_000_000_4, "a": 2, "c": 1.5 });
    assert_eq!(canonicalize(&v).unwrap(), "{\"a\":2,\"b\":1,\"c\":1.5}");
}

#[test]
fn canonicalize_preserves_array_order() {
    let v = json!([3, 1, 2]);
    assert_eq!(canonicalize(&v).unwrap(), "[3,1,2]");
}

#[test]
fn canonicalize_is_deterministic() {
    let v = json!({ "report": [1.5, 2.5], "hash": "x" });
    assert_eq!(canonicalize(&v).unwrap(), canonicalize(&v).unwrap());
}

#[test]
fn prove_then_verify_is_true() {
    let s = spec();
    let d = data();
    let proof = prove(&s, &d).unwrap();
    assert_eq!(proof.report_hash.len(), 64);
    assert_eq!(proof.inputs_hash.len(), 64);
    assert_eq!(proof.engine_version, wickra_backtest_core::version());
    assert!(verify(&proof, &s, &d).unwrap());
}

#[test]
fn prove_is_reproducible() {
    let s = spec();
    let d = data();
    let a = prove(&s, &d).unwrap();
    let b = prove(&s, &d).unwrap();
    assert_eq!(a.report_hash, b.report_hash);
    assert_eq!(a.inputs_hash, b.inputs_hash);
}

#[test]
fn tampered_report_fails_verification() {
    let s = spec();
    let d = data();
    let genuine = prove(&s, &d).unwrap();
    let forged = Proof {
        report: genuine.report.clone(),
        // Swap in a plausible-looking but wrong hash.
        report_hash: "0".repeat(64),
        inputs_hash: genuine.inputs_hash.clone(),
        engine_version: genuine.engine_version.clone(),
    };
    assert!(!verify(&forged, &s, &d).unwrap());
}

#[test]
fn engine_version_mismatch_is_rejected() {
    let mut s = spec();
    s.engine_version = Some("9.9.9-nonexistent".to_string());
    match prove(&s, &data()) {
        Err(Error::EngineMismatch { expected, .. }) => assert_eq!(expected, "9.9.9-nonexistent"),
        other => panic!("expected EngineMismatch, got {other:?}"),
    }
}

#[test]
fn missing_symbol_data_is_reported() {
    let s = spec();
    let empty: BTreeMap<String, Vec<Candle>> = BTreeMap::new();
    assert!(matches!(prove(&s, &empty), Err(Error::Data(_))));
}

#[test]
fn bad_spec_is_rejected() {
    // A strategy that is not a JSON object fails validation.
    let json_str = r#"{"strategy": 42, "dataset_ref": "x"}"#;
    assert!(matches!(
        ProofSpec::from_json(json_str),
        Err(Error::BadSpec(_))
    ));
}

#[test]
fn config_roundtrips_from_json() {
    let cfg_json = json!({ "spec": spec() }).to_string();
    let cfg = Config::from_json(&cfg_json).unwrap();
    assert_eq!(cfg.spec, spec());
}

#[test]
fn command_json_prove_matches_direct_prove() {
    let s = spec();
    let d = data();
    let direct = prove(&s, &d).unwrap();
    let req = json!({ "cmd": "prove", "spec": s, "data": d }).to_string();
    let mut prover = Prover::new();
    let out = prover.command_json(&req).unwrap();
    let parsed: Proof = serde_json::from_str(&out).unwrap();
    assert_eq!(parsed.report_hash, direct.report_hash);
    assert_eq!(parsed.inputs_hash, direct.inputs_hash);
}

#[test]
fn command_json_verify_reports_valid() {
    let s = spec();
    let d = data();
    let proof = prove(&s, &d).unwrap();
    let req = json!({ "cmd": "verify", "proof": proof, "spec": s, "data": d }).to_string();
    let mut prover = Prover::new();
    let out = prover.command_json(&req).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["ok"], json!(true));
    assert_eq!(v["valid"], json!(true));
}

#[test]
fn command_json_version_reports_both_versions() {
    let mut prover = Prover::new();
    let out = prover.command_json(r#"{"cmd":"version"}"#).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["version"], json!(crate::version()));
    assert_eq!(v["engine_version"], json!(wickra_backtest_core::version()));
}

#[test]
fn command_json_canonicalize_exposes_the_string() {
    let mut prover = Prover::new();
    let out = prover
        .command_json(r#"{"cmd":"canonicalize","value":{"b":1,"a":2}}"#)
        .unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["ok"], json!(true));
    assert_eq!(v["canonical"], json!("{\"a\":2,\"b\":1}"));
}

#[test]
fn command_json_unknown_cmd_returns_error_envelope() {
    let mut prover = Prover::new();
    let out = prover.command_json(r#"{"cmd":"nope"}"#).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["ok"], json!(false));
    assert!(v["error"].as_str().unwrap().contains("nope"));
}
