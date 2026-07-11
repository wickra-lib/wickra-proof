# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `proof-core`: the deterministic Proof-of-Backtest core — a serde `ProofSpec`
  (`{strategy, dataset_ref, engine_version?}`) folded through the pinned
  `wickra-backtest` engine into a `Proof` (`{report, inputs_hash, report_hash,
  engine_version}`). Both hashes are blake3 over a canonical JSON serialization
  (`canonicalize`): keys sorted at every depth, floats quantized to `1e-8` by
  pure decimal rounding with the whole-value integer collapse, no whitespace, and
  no `NaN`/`±inf` — byte-identical across every binding. `verify` recomputes the
  proof from `(spec, data)` rather than trusting the supplied hash, so a forged
  report cannot pass, and pins the `engine_version` so an engine change surfaces
  as a visible mismatch.
- `wickra-proof` CLI: `prove` and `verify` a `(spec, data)` pair from a config
  file plus a CSV or a directory of `<SYMBOL>.csv` candle files, with text or
  JSON output.
- Language bindings exposing the same JSON-over-C-ABI command API
  (`prove` / `verify` / `canonicalize` / `version`) in ten languages — native
  Rust, Python (PyO3), Node.js (napi) and WASM (wasm-bindgen), plus a C ABI hub
  for C, C++, C#, Go, Java and R.
- Byte-exact golden corpus, conformance / canonicalization / prove-verify /
  property tests, cargo-fuzz targets (spec parse, canonicalize fixed-point,
  prove, verify), criterion benchmarks, and one runnable example per language.
- CI across all ten languages on three OSes, CodeQL, OpenSSF Scorecard, zizmor
  workflow auditing, a tag-triggered release pipeline, and the `docs/` guides
  (architecture, canonicalization, proof format, verifying).
- Repository scaffolding: Cargo workspace, supply-chain configuration
  (`deny.toml`, `osv-scanner.toml`, `lychee.toml`), lint configuration
  (`clippy.toml`), `repo-metadata.toml`, and dual `MIT OR Apache-2.0` licensing.

[Unreleased]: https://github.com/wickra-lib/wickra-proof/commits/main
