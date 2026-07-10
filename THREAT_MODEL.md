# Threat Model

`wickra-proof` recomputes a deterministic backtest report and a canonical
blake3 hash from a `(spec, data)` pair and compares it against a claimed report.
It runs offline, holds no credentials, and places no orders. This document
records the assets, trust boundaries, and the threats that matter for a
verification tool whose entire value is determinism.

## Assets

- **The `report_hash` guarantee.** The single most important asset: for a fixed
  `(spec, data)` the report and its hash must be bit-identical across every
  language binding and every run. Any divergence destroys the product.
- **Caller-supplied input:** `ProofSpec` JSON (embedding a `StrategySpec`),
  candle data, and — on the verify path — a claimed proof JSON.
- **Process integrity:** no memory-safety faults reachable from untrusted input
  crossing the C ABI / WASM boundary.

## Trust boundaries

1. **FFI / WASM boundary.** Every binding passes a command JSON string into
   `command_json` and receives a JSON string back. Untrusted bytes are parsed
   here; all parsing is fallible and returns a typed error, never panics.
2. **Determinism boundary.** Every hashed report passes through a single
   `canonicalize` function (sorted keys, fixed `round_to(_, 1e-8)` f64
   representation, no whitespace, no `NaN`/`±inf`, `BTreeMap` everywhere) before
   blake3. The `engine_version` of the pinned backtest engine is embedded in the
   report, so a different engine version yields a different — and visibly
   different — hash by design.

## Threats and mitigations

| Threat | Mitigation |
|--------|-----------|
| Cross-language hash divergence | One shared canonicalization; cross-language golden tests recompute the same hash in every binding in CI. |
| Non-determinism (RNG, map order, float order) | No RNG; `BTreeMap` on every hashed path; fixed operation order; `round_to` quantization before hashing. |
| Forged "verified" claim | Verify recomputes the report from `(spec, data)` and compares the canonical hash; a claimed report that does not recompute is reported as a mismatch. |
| Panic / UB from malformed input | All boundary parsing is `Result`-typed; no `unwrap`/`expect` outside tests; fuzz targets cover `ProofSpec` and claimed-proof parsing. |
| Silent engine drift | `engine_version` is pinned and embedded in every report and hash. |

## Out of scope

The economic soundness of a strategy; the trustworthiness of the data a caller
supplies; denial of service from pathologically large caller-supplied inputs;
third-party registry compromise.
