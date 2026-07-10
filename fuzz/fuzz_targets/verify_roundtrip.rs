#![no_main]
//! Fuzz the prove/verify contract with genuine inputs. The fuzz bytes drive a
//! bounded, always-valid candle universe under a fixed strategy; proving then
//! verifying must always hold, and reproving must be byte-identical. This pins
//! the core invariant — a proof always verifies against its own inputs and is
//! deterministic — across an unbounded range of price paths.

use libfuzzer_sys::fuzz_target;
use proof_core::{prove, verify, Candle, ProofSpec};
use serde_json::json;
use std::collections::BTreeMap;

const SYMBOL: &str = "F";

fn spec() -> ProofSpec {
    ProofSpec {
        strategy: json!({
            "symbol": SYMBOL,
            "timeframe": "1h",
            "indicators": {
                "ema_fast": { "type": "Ema", "params": [3] },
                "ema_slow": { "type": "Ema", "params": [8] }
            },
            "entry": { "cross_above": ["ema_fast", "ema_slow"] },
            "exit": { "cross_below": ["ema_fast", "ema_slow"] },
            "sizing": { "type": "fixed_fraction", "fraction": 0.95 },
            "costs": { "taker_bps": 5, "slippage": { "type": "fixed_bps", "bps": 2 } },
            "risk": {}
        }),
        dataset_ref: "fuzz/F/1h".to_string(),
        engine_version: None,
    }
}

fuzz_target!(|data: &[u8]| {
    // Need enough bars to warm the slow EMA; pad the fuzz bytes deterministically.
    let mut closes: Vec<f64> = data.iter().map(|&b| 50.0 + f64::from(b)).collect();
    while closes.len() < 16 {
        closes.push(100.0);
    }

    let candles: Vec<Candle> = closes
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let o = if i == 0 { c } else { closes[i - 1] };
            Candle {
                time: 1_700_000_000 + i64::try_from(i).unwrap() * 3600,
                open: o,
                high: o.max(c) + 1.0,
                low: o.min(c) - 1.0,
                close: c,
                volume: 1000.0,
            }
        })
        .collect();

    let mut universe = BTreeMap::new();
    universe.insert(SYMBOL.to_string(), candles);

    let spec = spec();
    let proof = prove(&spec, &universe).expect("a bounded, valid universe always proves");
    assert!(
        verify(&proof, &spec, &universe).expect("verify recomputes without error"),
        "a genuine proof must verify against its own inputs"
    );

    let again = prove(&spec, &universe).unwrap();
    assert_eq!(
        proof.report_hash, again.report_hash,
        "prove is deterministic"
    );
    assert_eq!(
        proof.inputs_hash, again.inputs_hash,
        "prove is deterministic"
    );
});
