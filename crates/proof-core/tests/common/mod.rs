//! Shared fixtures for the proof-core integration tests: a valid embedded
//! strategy and a small deterministic candle universe keyed by symbol.
//!
//! Each integration-test binary pulls this module in and uses a different subset
//! of these helpers, so unused items in any single binary are expected.
#![allow(dead_code)]

use proof_core::{Candle, ProofSpec};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The symbol the sample strategy trades.
pub const SYMBOL: &str = "TEST";

/// A valid embedded `StrategySpec` (EMA cross) that trades [`SYMBOL`].
pub fn strategy_json() -> Value {
    json!({
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
        "risk": { "trailing_stop_pct": 5.0 }
    })
}

/// Build a candle series from a list of closes; `open` is the previous close,
/// `high`/`low` are `max`/`min(open, close) ± 1`, volume is constant.
pub fn candles_from(closes: &[f64]) -> Vec<Candle> {
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

/// A deterministic 40-bar V-shaped universe (down to bar 10, then up) so the EMA
/// cross fires at least once.
pub fn sample_closes() -> Vec<f64> {
    (0..40)
        .map(|i| {
            if i <= 10 {
                120.0 - 2.0 * f64::from(i)
            } else {
                100.0 + 2.0 * f64::from(i - 10)
            }
        })
        .collect()
}

/// The sample `(spec, data)` pair used by the prove/verify tests.
pub fn sample_spec() -> ProofSpec {
    ProofSpec {
        strategy: strategy_json(),
        dataset_ref: "test/TEST/1h".to_string(),
        engine_version: None,
    }
}

/// The data map for [`sample_spec`].
pub fn sample_data() -> BTreeMap<String, Vec<Candle>> {
    let mut data = BTreeMap::new();
    data.insert(SYMBOL.to_string(), candles_from(&sample_closes()));
    data
}
