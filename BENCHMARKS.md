# Benchmarks

`wickra-proof` ships a Criterion bench crate (`crates/proof-bench`) measuring the
cost of `prove` (backtest + canonicalize + hash) and `verify` over the golden
fixtures.

Numbers are filled in from the `bench.yml` nightly run once the core and bench
crate land; until then this file is a placeholder so the layout matches the rest
of the ecosystem.

| Operation | Input | Time | Notes |
|-----------|-------|------|-------|
| `prove`   | golden `sma_cross` | _TBD_ | backtest + canonicalize + blake3 |
| `verify`  | golden `sma_cross` | _TBD_ | recompute + compare |

Run locally:

```bash
cargo bench -p proof-bench
```
