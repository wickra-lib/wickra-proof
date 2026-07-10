//! Criterion benchmarks for the proof core.
//!
//! `prove` is measured across the cross-product of bar counts {200, 1k, 5k} and
//! indicator counts {2, 10}, so the report captures how folding a backtest into a
//! canonical hash scales with both dataset length and strategy width.
//! `canonicalize` — the moat — is measured on a small and a large report to show
//! the hashing surface's own cost independently of the backtest.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use proof_core::{canonicalize, prove, Candle, ProofSpec};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const SYMBOL: &str = "BENCH";

/// A strategy trading [`SYMBOL`] with `n` EMA indicators (n >= 2). The entry and
/// exit cross the two fastest; the rest are computed but unreferenced, so the
/// indicator-count axis reflects real per-bar indicator work.
fn strategy(n: usize) -> Value {
    let mut indicators = serde_json::Map::new();
    for i in 0..n {
        indicators.insert(
            format!("ema{i}"),
            json!({ "type": "Ema", "params": [3 + i] }),
        );
    }
    json!({
        "symbol": SYMBOL,
        "timeframe": "1h",
        "indicators": indicators,
        "entry": { "cross_above": ["ema0", "ema1"] },
        "exit": { "cross_below": ["ema0", "ema1"] },
        "sizing": { "type": "fixed_fraction", "fraction": 0.95 },
        "costs": { "taker_bps": 5, "slippage": { "type": "fixed_bps", "bps": 2 } },
        "risk": {}
    })
}

/// A deterministic, non-degenerate `bars`-long candle universe.
fn universe(bars: usize) -> BTreeMap<String, Vec<Candle>> {
    let closes: Vec<f64> = (0..bars)
        .map(|i| 100.0 + 10.0 * ((i as f64) * 0.1).sin())
        .collect();
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
    let mut data = BTreeMap::new();
    data.insert(SYMBOL.to_string(), candles);
    data
}

fn spec(n: usize) -> ProofSpec {
    ProofSpec {
        strategy: strategy(n),
        dataset_ref: "bench/BENCH/1h".to_string(),
        engine_version: None,
    }
}

fn bench_prove(c: &mut Criterion) {
    let mut group = c.benchmark_group("prove");
    for &bars in &[200usize, 1_000, 5_000] {
        let data = universe(bars);
        group.throughput(Throughput::Elements(bars as u64));
        for &indicators in &[2usize, 10] {
            let spec = spec(indicators);
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("{bars}bars_{indicators}ind")),
                &(spec, &data),
                |b, (spec, data)| b.iter(|| prove(spec, data).unwrap()),
            );
        }
    }
    group.finish();
}

fn bench_canonicalize(c: &mut Criterion) {
    // A small and a large report, taken from genuine proofs so the shapes match
    // production output exactly.
    let small = prove(&spec(2), &universe(200)).unwrap().report;
    let large = prove(&spec(2), &universe(5_000)).unwrap().report;
    let mut group = c.benchmark_group("canonicalize");
    for (name, report) in [("small_report", &small), ("large_report", &large)] {
        group.bench_function(name, |b| b.iter(|| canonicalize(report).unwrap()));
    }
    group.finish();
}

criterion_group!(benches, bench_prove, bench_canonicalize);
criterion_main!(benches);
