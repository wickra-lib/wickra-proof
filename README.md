# wickra-proof

> **Proof-of-Backtest.** Turn a `(spec, data)` pair into a deterministic
> backtest report **and** a canonical hash that anyone can recompute
> byte-for-byte in ten languages. Stop trusting backtest screenshots — verify
> the report against the spec and the data and get the same hash, or you don't.

`wickra-proof` is a thin, deterministic layer over the Wickra backtest engine.
Given a strategy spec and candle data it produces a `BacktestReport` and a
`report_hash` (blake3 over a canonical serialization). The same core logic is
callable from **Rust, Python, Node.js, WASM, C, C++, C#, Go, Java, and R** over
a single JSON-over-C-ABI boundary, so the hash is identical everywhere — that
identity *is* the proof.

## Why

A backtest screenshot proves nothing: numbers can be typed, curves can be drawn.
A `wickra-proof` claim is different — it ships the spec, the data commitment,
the report, and the hash. Anyone, in any supported language, recomputes the hash
and either matches it or doesn't. This is the foundation for fund transparency,
reproducible research, and higher-order tools (`wickra-verify`, `wickra-zk`).

## Determinism is the product

- Canonical JSON before hashing: sorted keys, `round_to(_, 1e-8)` floats, no
  whitespace, no `NaN`/`±inf`, `BTreeMap` on every hashed path.
- No RNG, fixed float operation order.
- `engine_version` pinned and embedded — a different backtest engine version
  produces a different, visibly-labelled hash by design.
- Any divergence of `report_hash` between two languages or two runs is a bug.

## Quick start

```bash
# Prove: compute the report + hash for a (spec, data) pair
cargo run -p wickra-proof-cli -- prove \
  --spec examples/data/specs/sma_cross.json \
  --data examples/data/candles/AAA.csv \
  --format json

# Verify: recompute and compare against a claimed proof
cargo run -p wickra-proof-cli -- verify \
  --proof examples/data/proofs/sma_cross.json \
  --spec examples/data/specs/sma_cross.json \
  --data examples/data/candles/AAA.csv
```

## Layout

```
crates/proof-core     the library: canonicalize + prove + verify
crates/wickra-proof-cli   reference CLI (prove / verify)
bindings/{c,python,node,wasm,go,csharp,java,r}   ten-language surface
golden/               fixed (spec, data) -> expected (report, hash)
examples/             runnable per-language demos
```

## License

Dual-licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE),
at your option.
