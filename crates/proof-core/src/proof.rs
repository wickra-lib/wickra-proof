//! `prove` / `verify` and the [`Prover`] command-JSON handle.

use crate::canonical::{blake3_hex, canonicalize};
use crate::error::{Error, Result};
use crate::spec::ProofSpec;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use wickra_backtest_core::{run, version as engine_version, Candle, StrategySpec};

/// The proof: the full backtest report plus the two canonical hashes and the
/// exact engine version that produced it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Proof {
    /// The full deterministic backtest report, embedded as JSON.
    pub report: Value,
    /// blake3 hex of `canonicalize(inputs)` where
    /// `inputs = {strategy, dataset_ref, candles, engine_version}`.
    pub inputs_hash: String,
    /// blake3 hex of `canonicalize(report)`.
    pub report_hash: String,
    /// The exact backtest engine version that produced `report`.
    pub engine_version: String,
}

/// Fold `(spec, data)` into a deterministic report and its canonical hashes.
pub fn prove(spec: &ProofSpec, data: &BTreeMap<String, Vec<Candle>>) -> Result<Proof> {
    let linked = engine_version().to_string();
    if let Some(expected) = &spec.engine_version {
        if expected != &linked {
            return Err(Error::EngineMismatch {
                expected: expected.clone(),
                linked,
            });
        }
    }

    let strategy: StrategySpec =
        serde_json::from_value(spec.strategy.clone()).map_err(|e| Error::BadSpec(e.to_string()))?;
    let candles = data
        .get(&strategy.symbol)
        .ok_or_else(|| Error::Data(format!("no candles for symbol {}", strategy.symbol)))?;

    let report = run(&strategy, candles).map_err(|e| Error::Backtest(e.to_string()))?;
    let report_value = serde_json::to_value(&report)?;
    let report_hash = blake3_hex(&canonicalize(&report_value)?);

    let inputs = json!({
        "strategy": spec.strategy,
        "dataset_ref": spec.dataset_ref,
        "candles": serde_json::to_value(data)?,
        "engine_version": linked,
    });
    let inputs_hash = blake3_hex(&canonicalize(&inputs)?);

    Ok(Proof {
        report: report_value,
        inputs_hash,
        report_hash,
        engine_version: linked,
    })
}

/// Verify a proof by recomputing it from `(spec, data)` and comparing the
/// canonical hashes and engine version. This is recomputation, not blind trust
/// of a supplied hash, so a forged `report`+`hash` cannot pass.
pub fn verify(
    proof: &Proof,
    spec: &ProofSpec,
    data: &BTreeMap<String, Vec<Candle>>,
) -> Result<bool> {
    let fresh = prove(spec, data)?;
    Ok(fresh.report_hash == proof.report_hash
        && fresh.inputs_hash == proof.inputs_hash
        && fresh.engine_version == proof.engine_version)
}

/// Stateless command-JSON handle. It holds nothing, but is handle-shaped so the
/// ten language bindings share the same surface as screener/terminal.
#[derive(Debug, Default, Clone, Copy)]
pub struct Prover;

#[derive(Deserialize)]
struct ProveReq {
    spec: ProofSpec,
    data: BTreeMap<String, Vec<Candle>>,
}

#[derive(Deserialize)]
struct VerifyReq {
    proof: Proof,
    spec: ProofSpec,
    data: BTreeMap<String, Vec<Candle>>,
}

impl Prover {
    /// Create a new (stateless) handle.
    #[must_use]
    pub fn new() -> Self {
        Prover
    }

    /// The proof-core crate version.
    #[must_use]
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Dispatch a command envelope `{"cmd": ...}` and return a canonical JSON
    /// string. Unknown commands and errors return an error envelope, never a
    /// panic.
    pub fn command_json(&mut self, cmd_json: &str) -> Result<String> {
        let value = dispatch(cmd_json);
        canonicalize(&value)
    }
}

fn dispatch(cmd_json: &str) -> Value {
    match dispatch_inner(cmd_json) {
        Ok(v) => v,
        Err(e) => json!({ "ok": false, "error": e.to_string() }),
    }
}

fn dispatch_inner(cmd_json: &str) -> Result<Value> {
    let env: Value = serde_json::from_str(cmd_json)?;
    let cmd = env.get("cmd").and_then(Value::as_str).unwrap_or("");
    match cmd {
        "prove" => {
            let req: ProveReq = serde_json::from_value(env)?;
            Ok(serde_json::to_value(prove(&req.spec, &req.data)?)?)
        }
        "verify" => {
            let req: VerifyReq = serde_json::from_value(env)?;
            let valid = verify(&req.proof, &req.spec, &req.data)?;
            Ok(json!({ "ok": true, "valid": valid }))
        }
        "canonicalize" => {
            let value = env.get("value").cloned().unwrap_or(Value::Null);
            Ok(json!({ "ok": true, "canonical": canonicalize(&value)? }))
        }
        "version" => Ok(json!({
            "version": Prover::version(),
            "engine_version": engine_version(),
        })),
        other => Err(Error::Parse(format!("unknown cmd: {other}"))),
    }
}
