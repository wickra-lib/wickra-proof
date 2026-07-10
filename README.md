<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Proof — a deterministic (spec, data) → blake3 hash, byte-identical across ten languages" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-proof)
[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/ci.svg)](https://github.com/wickra-lib/wickra-proof/actions/workflows/ci.yml)
[![CodeQL](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/codeql.svg)](https://github.com/wickra-lib/wickra-proof/actions/workflows/codeql.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-proof)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/license.svg)](#license)
[![OpenSSF Scorecard](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/scorecard.svg)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-proof)
[![OpenSSF Best Practices](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/best-practices.svg)](https://www.bestpractices.dev/)
[![Build provenance](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/provenance.svg)](https://github.com/wickra-lib/wickra-proof/attestations)
[![Docs](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/docs.svg)](https://wickra.org)

---

# Wickra Proof

**Proof-of-Backtest. Turn a `(spec, data)` pair into a deterministic backtest report *and* a canonical blake3 hash that anyone can recompute byte-for-byte in ten languages.**

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib):** the same deterministic core and ten-language binding surface also power [wickra-exchange](https://github.com/wickra-lib/wickra-exchange), [wickra-backtest](https://github.com/wickra-lib/wickra-backtest), [wickra-terminal](https://github.com/wickra-lib/wickra-terminal), [wickra-screener](https://github.com/wickra-lib/wickra-screener), [wickra-xray](https://github.com/wickra-lib/wickra-xray), [wickra-radar](https://github.com/wickra-lib/wickra-radar), [wickra-copilot](https://github.com/wickra-lib/wickra-copilot) and [wickra-shazam](https://github.com/wickra-lib/wickra-shazam).

`wickra-proof` is a thin, deterministic layer over the Wickra backtest engine.
Given a strategy spec and candle data it produces a `BacktestReport` and a
`report_hash` (blake3 over a canonical serialization). The same core logic is
callable from **Rust, Python, Node.js, WASM, C, C++, C#, Go, Java and R** over a
single JSON-over-C-ABI boundary, so the hash is identical everywhere — that
identity *is* the proof.

Stop trusting backtest screenshots. A screenshot proves nothing: numbers can be
typed, curves can be drawn. A `wickra-proof` claim ships the spec, the data
commitment, the report and the hash — anyone, in any supported language,
recomputes the hash and either matches it or doesn't. This is the foundation for
fund transparency, reproducible research and higher-order tools
([`wickra-verify`](https://github.com/wickra-lib/wickra-verify),
[`wickra-zk`](https://github.com/wickra-lib/wickra-zk)).

## Determinism is the product

- **Canonical JSON before hashing:** keys sorted at every depth (`BTreeMap`),
  floats quantized to `1e-8` by pure decimal rounding (`{:.8}`, trailing zeros
  trimmed, whole values collapsed to their integer token so a host language's
  `1.0`-vs-`1` ambiguity can never shift the hash), no whitespace, and no
  `NaN`/`±inf` (rejected at parse time).
- **No RNG, fixed float operation order** — the same inputs always reduce to the
  same bytes.
- **`engine_version` pinned and embedded** — a different backtest engine version
  produces a different, visibly-labelled hash by design.
- Any divergence of `report_hash` between two languages or two runs is a bug,
  caught by the byte-exact golden corpus and the canonicalize fuzz target.

## Status

**Pre-release — functionally complete, CI-verified, not yet published.** The core
([`proof-core`](crates/proof-core)), the CLI, all ten language bindings, the
byte-exact golden corpus, property + fuzz tests, benchmarks and one runnable
example per language are in place and green across the full CI matrix (10
languages × 3 OS). Not yet released to any registry — track progress in
[ROADMAP.md](ROADMAP.md).

## Documentation

- [Architecture](ARCHITECTURE.md) — the core, the canonicalization boundary, the binding surface.
- [ROADMAP.md](ROADMAP.md) · [BENCHMARKS.md](BENCHMARKS.md) · [THREAT_MODEL.md](THREAT_MODEL.md) · [SECURITY.md](SECURITY.md).

## Quickstart

```bash
# Prove: compute the report + hash for a (spec, data) pair.
# --spec takes a config file ({ "spec": <ProofSpec> }); --data a CSV or a
# directory of <SYMBOL>.csv files.
cargo run -p wickra-proof-cli -- prove \
  --spec examples/data/config.json \
  --data examples/data/candles/AAA.csv \
  --format json

# Verify: recompute and compare against a claimed proof.
cargo run -p wickra-proof-cli -- verify \
  --proof examples/data/config.proof.json \
  --spec examples/data/config.json \
  --data examples/data/candles/AAA.csv
```

`prove` prints the `BacktestReport` and its `report_hash`; `verify` re-runs the
proof against the same spec and data and prints `valid` only if the recomputed
hash matches the claimed one. Tamper with a single field of the proof and
verification fails.

## ProofSpec and Proof

A **proof** carries everything a third party needs to reproduce it:

- **`spec`** — the strategy definition (`ProofSpec`): indicators, rules and
  parameters, plus the pinned `engine_version`.
- **data commitment** — a hash of the candle series the report was computed over.
- **`report`** — the resulting `BacktestReport` (metrics, equity, trades).
- **`report_hash`** — the blake3 of the canonical serialization of the report.

Because the spec and the data commitment travel with the report, a verifier
never has to trust the prover: it recomputes and compares.

## Canonicalization and hashing

The hash is only as trustworthy as the serialization it runs over, so
canonicalization is the load-bearing contract every binding reproduces exactly
(see [`crates/proof-core/src/canonical.rs`](crates/proof-core/src/canonical.rs)):

1. Object keys sorted ascending by Unicode code point.
2. No structural whitespace.
3. Floats quantized to `1e-8` by decimal rounding, trailing zeros trimmed, whole
   values collapsed to their integer token; magnitudes at or above the point
   where the f64 ULP reaches the `1e-8` grid fall back to the shortest
   round-trippable form so canonicalization stays a fixed point.
4. `NaN`/`±inf` cannot occur.
5. Arrays keep their order; strings use the standard JSON escaping.

`blake3` over that canonical string yields the 64-hex `report_hash`. The
canonicalize fuzz target pins the fixed-point property (`canonicalize → parse →
canonicalize` yields identical bytes) across the full finite f64 range.

## Verifying a foreign proof

Any supported language can verify a proof produced by any other — that is the
whole point. Each binding exposes the same JSON-over-C-ABI command surface
(`prove` / `verify`), returns the core's canonical response verbatim, and the
cross-language golden tests assert byte-for-byte equality. A proof minted in
Python verifies in Go; a proof minted in Rust verifies in the browser over WASM.

## Engine-version pinning

`engine_version` is embedded in the spec and folded into the report, so a proof
is bound to the exact backtest semantics that produced it. Upgrade the engine and
the same `(spec, data)` produces a different, clearly-labelled hash — divergence
is surfaced, never hidden.

## Use in any language

The core is a JSON-over-C-ABI data API (`Prover::command`) exposed natively in
Rust, Python, Node.js and WASM, and over the C ABI hub in C, C++, C#, Go, Java
and R. One runnable example per language lives under [`examples/`](examples); the
per-binding quickstarts are in each `bindings/<lang>/README.md`.

## Project layout

```
crates/proof-core          the library: canonicalize + prove + verify
crates/wickra-proof-cli    reference CLI (prove / verify), binary `wickra-proof`
crates/proof-bench         Criterion benchmarks
bindings/{c,python,node,wasm,go,csharp,java,r}   ten-language surface
golden/                    fixed (spec, data) -> expected (report, hash)
examples/                  runnable per-language demos
fuzz/                      cargo-fuzz targets (spec parse, canonicalize, prove, verify)
```

## Building from source

```bash
cargo build --workspace
cargo test  --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Each binding builds from its own directory — see the per-binding READMEs under
`bindings/`.

## Requirements

- **Rust** — workspace MSRV 1.86 (the Node binding needs 1.88).
- Optional per binding: Python 3.9+, Node.js 22+, a C toolchain + CMake, .NET 8
  SDK, JDK 22+, Go 1.21+, R 4.x.

## Benchmarks

Criterion benchmarks live in [`crates/proof-bench`](crates/proof-bench) and run
nightly in CI; see [BENCHMARKS.md](BENCHMARKS.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All commits are signed and DCO-signed off;
CI must be green across every language before merge.

## Security

Report vulnerabilities per [SECURITY.md](SECURITY.md). The trust model — what a
proof does and does not guarantee — is in [THREAT_MODEL.md](THREAT_MODEL.md).

## License

Dual-licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.

## Disclaimer

`wickra-proof` is research and engineering tooling, not financial advice. A proof
attests only that a given report is the deterministic result of a given spec over
given data — it makes no claim about the quality, profitability or future
performance of any strategy. Trading carries risk; you are responsible for your
own decisions.
