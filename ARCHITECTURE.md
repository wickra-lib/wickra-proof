# Architecture

`wickra-proof` is a library-shaped product: one small deterministic core, a
reference CLI, and ten thin language bindings that all reach the same core over
a single JSON string boundary. It mirrors the structure of `wickra-backtest`
(its runtime dependency) and `wickra-screener`.

## Layers

```
                 ┌─────────────────────────────────────────┐
   (spec, data)  │  proof-core                              │
   ────────────▶ │   spec.rs     ProofSpec { strategy, ... }│
                 │   proof.rs    prove() / verify()          │
                 │   canonical.rs canonicalize() + blake3    │──▶ (report, report_hash)
                 │   command.rs  command_json(&str) -> String│
                 └──────────────┬──────────────────────────┘
                                │ depends on
                 ┌──────────────▼──────────────┐
                 │  wickra-backtest (pinned)    │  run(spec, candles) -> BacktestReport
                 │  engine_version = <pinned>   │
                 └─────────────────────────────┘

  CLI ─┐
  C ───┤  every surface passes a command JSON string in and gets a JSON
  Py ──┤  string out — byte-identical across all of them (that is the proof).
  Node ┤
  WASM ┤   command_json({ "cmd": "prove",  "spec": ..., "candles": ... })
  Go ──┤   command_json({ "cmd": "verify", "spec": ..., "candles": ..., "claim": ... })
  C# ──┤
  Java ┤
  R ───┘
```

## Determinism chain

1. `prove(spec, candles)` runs the pinned backtest engine → `BacktestReport`.
2. `canonicalize(report)` produces a canonical byte string: keys sorted,
   `BTreeMap` throughout, floats quantized via `round_to(_, 1e-8)`, no
   whitespace, no `NaN`/`±inf`.
3. `report_hash = blake3(canonical_bytes)`, hex-encoded.
4. The report embeds `engine_version` so the hash is only meaningful against a
   known engine; a different engine version is a different, labelled hash.

`verify(spec, candles, claim)` repeats steps 1–3 and compares the recomputed
hash and report against the claim, returning a structured verdict.

## Why the C ABI hub

Rust, Python (PyO3), Node (napi), and WASM (wasm-bindgen) bind natively. C, C++,
C#, Go, Java, and R reach the core through a C ABI (`cbindgen` header) that
exposes `command_json`. Because every language forwards the same JSON string
verbatim, the cross-language golden tests assert one hash across all ten.
