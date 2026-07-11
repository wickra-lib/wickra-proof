# Architecture (internals)

The top-level [ARCHITECTURE.md](../ARCHITECTURE.md) gives the high-level shape;
this page covers how `proof-core` actually turns a `(spec, data)` pair into a
`Proof`, and where the determinism guarantees live. The whole product is **one
deterministic core** (`proof-core`) and N thin consumers — the CLI and the ten
language bindings — each of which only ships a command JSON and reads back a
response JSON.

## The prove pipeline

```
ProofSpec { strategy, dataset_ref, engine_version? }   +   data: { SYMBOL -> [Candle] }
   │
   │  engine_version guard: if the spec pins a version, it must equal the
   │  linked wickra-backtest engine_version, else EngineMismatch.
   ▼
StrategySpec  (spec.strategy parsed into the backtest strategy)
   │  candles = data[strategy.symbol]     (Data error if the symbol is absent)
   ▼
run(&strategy, candles) -> BacktestReport          (wickra-backtest, pinned)
   │
   ├─ report_value = to_value(report)
   │     report_hash = blake3_hex(canonicalize(report_value))
   │
   └─ inputs = { strategy, dataset_ref, candles, engine_version }
         inputs_hash = blake3_hex(canonicalize(inputs))
   ▼
Proof { report, inputs_hash, report_hash, engine_version }
```

Two independent commitments come out of one run:

- **`report_hash`** — blake3 over the canonical serialization of the
  `BacktestReport`. This is *the* proof: it says "this report is the
  deterministic result of this run."
- **`inputs_hash`** — blake3 over the canonical serialization of
  `{strategy, dataset_ref, candles, engine_version}`. It binds the report to the
  exact strategy, data and engine that produced it, so a verifier can detect a
  swapped input even before re-running.

## The command boundary

Every consumer speaks the same envelope. `Prover::command_json(&str) -> String`
parses a `{"cmd": ...}` envelope, dispatches, and returns a **canonical** JSON
string (the response is itself run through `canonicalize`, so even the wrapper is
byte-stable). The four commands are `prove`, `verify`, `canonicalize`,
`version`; see [PROOF_FORMAT.md](PROOF_FORMAT.md) for the request/response
shapes.

The handle (`Prover`) is a zero-sized, stateless value. It is handle-shaped only
so the ten bindings share the exact surface of `wickra-screener` /
`wickra-terminal` — there is no hidden state, no RNG, no clock.

## Where determinism is enforced

- **Canonicalization** (`crates/proof-core/src/canonical.rs`) is the single
  load-bearing contract; its rules are normative and specified in
  [CANONICAL.md](CANONICAL.md). Everything that gets hashed passes through it.
- **No RNG, no time, fixed float operation order** — `prove` is a pure function
  of `(spec, data)` and the linked engine version.
- **Engine-version pinning** — the linked `wickra-backtest` version is folded
  into both the `inputs_hash` and the `Proof`, so a different engine produces a
  different, clearly-labelled hash rather than a silent divergence. See
  [VERIFYING.md](VERIFYING.md) for what that means to a verifier.

Any divergence of `report_hash` between two languages or two runs is a bug,
caught by the byte-exact golden corpus (`golden/`), the `canonicalize` fuzz
target, and the cross-language golden tests.

## Error handling

`dispatch` never panics: a bad envelope, an unknown `cmd`, an engine mismatch,
or a missing symbol all come back in-band as
`{"ok": false, "error": "…"}`. The bindings surface that verbatim.

## See also

- [CANONICAL.md](CANONICAL.md) — the normative canonicalization rules (the moat).
- [PROOF_FORMAT.md](PROOF_FORMAT.md) — the `ProofSpec` and `Proof` schema, and the command envelope.
- [VERIFYING.md](VERIFYING.md) — recomputing a foreign proof.
- [ARCHITECTURE.md](../ARCHITECTURE.md) — the high-level layer diagram.
