# Proof format

This page specifies the two data types a caller sees — `ProofSpec` (the input)
and `Proof` (the output) — and the command envelope every binding speaks. The
reference types are [`crates/proof-core/src/spec.rs`](../crates/proof-core/src/spec.rs)
and [`crates/proof-core/src/proof.rs`](../crates/proof-core/src/proof.rs).

## `ProofSpec` (input)

```json
{
  "strategy": { "...": "the wickra-backtest strategy definition" },
  "dataset_ref": "BTCUSDT/1h",
  "engine_version": "0.1.0"
}
```

| Field            | Type              | Required | Meaning |
|------------------|-------------------|----------|---------|
| `strategy`       | object            | yes      | The backtest strategy (indicators, rules, parameters). Parsed into the backtest engine's `StrategySpec`; must name a `symbol`. |
| `dataset_ref`    | string            | yes      | A human-readable label for the candle series (e.g. `"BTCUSDT/1h"`). It travels into the `inputs_hash`, so it is part of what a proof commits to. |
| `engine_version` | string (optional) | no       | If present, it **must** equal the linked `wickra-backtest` engine version, else `prove` returns an `EngineMismatch` error. If absent, the linked version is used and recorded. |

A `ProofSpec` can be loaded from JSON (`ProofSpec::from_json`) or TOML
(`ProofSpec::from_toml`). The CLI's `--spec` flag takes a **Config wrapper**
(`{ "spec": <ProofSpec> }`); a bare `ProofSpec` is what the command API consumes.

## `Proof` (output)

```json
{
  "report":         { "...": "the BacktestReport" },
  "inputs_hash":    "<64-hex blake3>",
  "report_hash":    "<64-hex blake3>",
  "engine_version": "0.1.0"
}
```

| Field            | Type   | Meaning |
|------------------|--------|---------|
| `report`         | object | The `BacktestReport` (metrics, equity, trades) as produced by the engine. |
| `inputs_hash`    | string | blake3 over `canonicalize({strategy, dataset_ref, candles, engine_version})`. Binds the report to the exact inputs. |
| `report_hash`    | string | blake3 over `canonicalize(report)`. *This* is the proof — the value a verifier recomputes. |
| `engine_version` | string | The linked engine version that produced the report. |

Both hashes are 64-character lowercase hex (blake3-256). Canonicalization is
specified in [CANONICAL.md](CANONICAL.md).

## Candle shape

`data` is a map of `SYMBOL → [Candle]`. Each candle is:

```json
{ "time": 1, "open": 100.0, "high": 101.0, "low": 99.0, "close": 100.5, "volume": 12.0 }
```

## Command envelope

Every binding calls one function — `command(cmd_json) -> response_json`. The
request is a `{"cmd": ...}` envelope; the response is a **canonical** JSON string.

| `cmd`          | Request fields          | Response |
|----------------|-------------------------|----------|
| `prove`        | `spec`, `data`          | the full `Proof` |
| `verify`       | `proof`, `spec`, `data` | `{"ok": true, "valid": <bool>}` |
| `canonicalize` | `value`                 | `{"ok": true, "canonical": "<string>"}` |
| `version`      | —                       | `{"version": "...", "engine_version": "..."}` |

Unknown commands and any error (bad envelope, `EngineMismatch`, missing symbol)
come back in-band as `{"ok": false, "error": "…"}` — never a panic.

## Versioning

- The **`engine_version`** is the semantic version of the pinned
  `wickra-backtest`. A change to it changes every hash by design (see the
  engine-version section of the top-level [README](../README.md)).
- The **canonicalization rules** are the format's real versioning surface: they
  are frozen and normative ([CANONICAL.md](CANONICAL.md)). Any future change to
  them is a breaking format change and would be called out in
  [CHANGELOG.md](../CHANGELOG.md).

## See also

- [ARCHITECTURE.md](ARCHITECTURE.md) — how a `Proof` is produced.
- [CANONICAL.md](CANONICAL.md) — the byte-exact serialization rules.
- [VERIFYING.md](VERIFYING.md) — recomputing a foreign proof.
