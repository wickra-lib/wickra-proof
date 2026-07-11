# Benchmarks

A proof's cost is dominated by the backtest run itself; canonicalization and the
blake3 hash are a thin, near-constant tail on top. The benchmarks here measure
`prove` end-to-end (backtest + canonicalize + hash) and `canonicalize` in
isolation, so you can see how the determinism layer compares to the work it wraps.

## What is measured

The `proof-bench` crate (criterion) covers:

- **`prove`** across a matrix of **candle count** (200, 1 000, 5 000 bars) ×
  **indicator count** (specs referencing ~2 and ~10 indicators). Throughput is
  reported in bars per second.
- **`canonicalize`** on a small and a large `BacktestReport`, both taken from
  genuine proofs so the JSON shapes match production output exactly.

## Methodology

Run against fixed, in-process synthetic universes so the numbers are reproducible
and contain no I/O variance:

```bash
cargo bench -p proof-bench
```

## Results

Measured with `cargo bench -p proof-bench` (criterion) on a Windows x86-64
laptop. Figures are the median estimate; treat them as orders of magnitude, not
guarantees — they vary with CPU, toolchain and the linked `wickra-backtest`
version.

| Benchmark | Bars × indicators | Median | Throughput |
|-----------|-------------------|--------|------------|
| `prove/200bars_2ind`    | 200 × ~2    | 1.13 ms  | ~177 K bars/s |
| `prove/200bars_10ind`   | 200 × ~10   | 1.30 ms  | ~154 K bars/s |
| `prove/1000bars_2ind`   | 1 000 × ~2  | 6.02 ms  | ~166 K bars/s |
| `prove/1000bars_10ind`  | 1 000 × ~10 | 6.49 ms  | ~154 K bars/s |
| `prove/5000bars_2ind`   | 5 000 × ~2  | 30.3 ms  | ~165 K bars/s |
| `prove/5000bars_10ind`  | 5 000 × ~10 | 32.2 ms  | ~155 K bars/s |

| Benchmark | Report | Median |
|-----------|--------|--------|
| `canonicalize/small_report` | 200-bar proof   | 168 µs  |
| `canonicalize/large_report` | 5 000-bar proof | 4.75 ms |

The takeaway: per-bar throughput stays roughly constant as history grows
(~150–177 K bars/s), so prove cost scales linearly with candle count and only
mildly with the number of indicators a spec references. Canonicalization of even
a large report is a few milliseconds — a small fraction of the prove that
produced it — so the determinism guarantee costs little over the backtest it
commits to. The nightly `bench.yml` workflow reruns this on a clean Linux runner
for tracking over time.

## Caveats

These figures bound the prove / canonicalize work only. End-to-end time in a real
run also depends on loading the candle series from disk, which these in-process
benchmarks do not capture. `verify` recomputes `prove`, so its cost tracks the
`prove` figures above plus a second canonicalize.
