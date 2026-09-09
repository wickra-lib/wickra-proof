# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Maven Central would have rejected the first publish, after the job reported
  success.** Three requirements were missing from `bindings/java/pom.xml`:

  `<scm>` and `<developers>` are validated by Central and their absence is
  refused outright (*"SCM URL is not defined"*, *"Developers information is
  missing"*). The `release` profile did not exist at all, so `mvn -Prelease
  deploy` matched no profile, warned, and deployed bare -- no sources jar, no
  javadoc jar, no signatures, and no publishing plugin to send them with.
  Central requires all four.

  The profile now carries the same four plugins the rest of the organisation
  publishes with, including `waitUntil=published` so a green job means the
  artifact is on the repository rather than merely validated. The two licenses
  are also split into separate `<license>` entries with URLs, since
  `MIT OR Apache-2.0` in a single `<name>` is an SPDX expression, not a licence
  Central recognises.

  Verified locally with Maven 3.9.9 and JDK 22: `-Prelease` now activates the
  profile, and `mvn -Prelease validate` passes.

- **Two high-severity `js-yaml` advisories in the Node binding's lockfile.**
  `js-yaml` 4.3.0 is reachable from the napi tooling and carries
  [GHSA quadratic CPU consumption in `!!omap` resolution][omap] (the
  CVE-2026-59870 fix was not backported to 4.x) and a second where
  `maxTotalMergeKeys` fails to limit CPU use for empty merge sources. Both
  resolve at 4.3.2, inside the range `package.json` already declares, so this is
  a lockfile-only change. Development-only: nothing here ships to a consumer.

[omap]: https://github.com/advisories

- **The engine was pinned by name, not by revision, and the goldens had drifted
  from it.** `wickra-backtest-core` was taken from a branch with no `rev`, so it
  tracked whatever upstream had last pushed while `Cargo.lock` held a rev from
  weeks earlier. Two consequences, both load-bearing:

  A consumer that pins the engine could not use this crate at all. Cargo treats
  "this git URL, default branch" and "this git URL at rev X" as two different
  sources, so pinning downstream put **two copies of `wickra-backtest-core`** in
  one graph -- and two copies share no types. wickra-zk hit exactly that: every
  symbol resolved and the build still failed with `expected BacktestReport,
  found a different BacktestReport`. The pin is now an exact rev, and it is
  meant to move together with its consumers.

  The pin also moved the linked engine from `0.1.0` to `0.1.4`, which the
  goldens had never seen. They are re-blessed through `golden/_bless.mjs`, and
  the diff is the reassuring kind: across all three fixtures **every computed
  value is unchanged** -- equity curve, every metric, every trade, fees, capital,
  schema version. What changed is `engine_version` and two descriptive fields
  (`symbol`, `timeframe`) the newer report carries, and therefore both hashes.
  The engine's arithmetic did not move; the report's shape and version did.

### Added

- **`hash_value`, `hash_candles` and `hash_report` are public.** The two hashes
  `prove` reports were computable only by running `prove`: `canonicalize` was
  exported but `blake3_hex` was `pub(crate)`, so nothing outside the prover could
  arrive at the same 64 hex characters. A zkVM guest recomputing the report hash
  inside a circuit, or a verifier binding a proof to a candle series, had no way
  in.

  `hash_value(&Value)` is now the crate's only definition of "the hash", and
  `prove` reports both of its hashes through it rather than repeating the
  expression — the two cannot drift apart, and a test pins that they do not.
  `hash_candles(&[Candle])` is the dataset commitment; note it is *not*
  `inputs_hash`, which covers `{strategy, dataset_ref, candles, engine_version}`
  as one block. `hash_report(&BacktestReport)` is byte-identical to the
  `report_hash` `prove` publishes for the same report.

  Five tests cover it: the report hash agrees with `prove`, key order does not
  change a value's identity, candle order does, an edit of 1e-6 to one close
  changes the commitment, and the commitment is distinct from `inputs_hash`.

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
