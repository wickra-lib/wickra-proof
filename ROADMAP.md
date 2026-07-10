# Roadmap

`wickra-proof` targets structural parity with the rest of the Wickra ecosystem
(`wickra-backtest`, `wickra-screener`, `wickra-exchange`, `wickra-terminal`):
same versions, same layout, same tests/fuzz/golden/examples/bindings/CI.

## 0.1.0 (first release)

- `proof-core`: `ProofSpec`, `prove`, `verify`, `canonicalize`, `command_json`.
- Pinned `wickra-backtest` engine dependency (`engine_version` embedded).
- Reference CLI: `prove` and `verify`.
- Ten language bindings over the C ABI hub, all recomputing one `report_hash`.
- Golden `(spec, data) -> (report, hash)` fixtures with cross-language checks.
- Full CI: fmt, clippy (default + no-default-features), test matrix, MSRV,
  coverage, cargo-deny, fuzz-smoke, link check, per-language binding jobs.

## After 0.1.0

- Data-commitment options (hash the input candles into the proof envelope).
- Streaming/chunked proving for very long histories.
- Integration hooks for `wickra-verify` (public anti-fraud tool) and
  `wickra-zk` (zero-knowledge proof of the same report).

Releases to registries are **user-gated** and never happen automatically.
