//! A runnable Rust example: prove a (spec, data) pair with the native `prove`
//! API, print the report hash, then verify the proof and assert it holds.
//!
//! ```bash
//! cargo run -p wickra-proof-example
//! ```

use std::collections::BTreeMap;

use proof_core::{prove, verify, Candle, ProofSpec};

const SPEC: &str = r#"{
    "strategy": {
        "symbol": "AAA",
        "timeframe": "1h",
        "indicators": {
            "ema_fast": {"type": "Ema", "params": [3]},
            "ema_slow": {"type": "Ema", "params": [8]}
        },
        "entry": {"cross_above": ["ema_fast", "ema_slow"]},
        "exit": {"cross_below": ["ema_fast", "ema_slow"]},
        "sizing": {"type": "fixed_fraction", "fraction": 0.95},
        "costs": {"taker_bps": 5, "slippage": {"type": "fixed_bps", "bps": 2}},
        "risk": {}
    },
    "dataset_ref": "example/AAA/1h"
}"#;

/// A short V-shaped price path so the fast/slow EMA cross fires at least once.
fn candles() -> Vec<Candle> {
    let closes = [
        120.0, 118.0, 116.0, 114.0, 112.0, 110.0, 108.0, 112.0, 116.0, 120.0, 124.0, 128.0,
    ];
    closes
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
        .collect()
}

fn main() {
    let spec: ProofSpec = ProofSpec::from_json(SPEC).expect("valid spec");

    let mut data = BTreeMap::new();
    data.insert("AAA".to_string(), candles());

    let proof = prove(&spec, &data).expect("prove");

    println!("wickra-proof {}", proof_core::version());
    println!("report_hash: {}", proof.report_hash);

    assert!(
        verify(&proof, &spec, &data).expect("verify"),
        "a genuine proof must verify against its own inputs"
    );
    println!("verify: valid");
}
