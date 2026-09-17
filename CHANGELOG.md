# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **The Java binding loads the library it ships.** The jar carries the native
  library under `native/<os>-<arch>/` -- the release pipeline stages every
  platform there -- but the loader only ever looked at `-Dnative.lib.dir` and
  the working directory, so a Maven Central consumer got a jar it could not
  load without pointing the JVM at a library it had to build itself. The loader
  now resolves in wickra's order: `-Dnative.lib.dir` when set, the bundled copy
  extracted to a temporary file, every `target/release` or `target/debug` up
  the tree from the working directory and the class's own location, then the
  bare name.

### Changed

- **Every README follows wickra's shape.** A cross-repo scan compared the
  heading skeleton of each README against wickra's and this repository's
  differed throughout. The root README opens as wickra's does (banner, badges,
  the one-liner, the live-demo and ecosystem lines, no separate H1), the
  License section carries wickra's wording and its `### Contribution` clause,
  and the shared sections run in wickra's order. Each binding README is
  `Install`, `Quick start`, `Benchmark`, `Documentation`, `Security`,
  `Disclaimer`, `License` with the product's own surface and protocol notes
  as subsections; the registry pages that render them now say how to report a
  vulnerability and under which licence the package ships.
  `examples/README.md` lists every language the way wickra's does, with the
  commands the CI examples job runs; the per-language example READMEs,
  `fuzz/README.md` and the `## Editing the docs` section of
  `docs/README.md` exist as they do in wickra.

### Changed

- **wickra-backtest-core 0.1.6.** The pin moves from `=0.1.4` to the release
  the family is on; the lock follows. A cross-repo scan lined the 24 wickra-lib
  repositories up, and the rest is what this one spelled differently: the C
  example's `CMAKE_CXX_STANDARD` 14 where the family builds with 17, the fuzz
  job on a rolling nightly rather than the family's pinned `nightly-2026-07-01`,
  and the example job's `dotnet-version`, which now reads `8.0.x`.

### Changed

- **wickra-backtest-core 0.1.6.** The pin moves from `=0.1.4` to the release
  the family is on; the lock follows. The golden corpus is re-blessed against
  it: every `report_hash` is unchanged — the 0.1.6 engine produces the same
  reports for the same inputs — and only `engine_version` and, because the
  engine version is part of the hashed inputs, `inputs_hash` move. A cross-repo scan lined the 24 wickra-lib
  repositories up, and the rest is what this one spelled differently: the C
  example's `CMAKE_CXX_STANDARD` 14 where the family builds with 17, the fuzz
  job on a rolling nightly rather than the family's pinned `nightly-2026-07-01`,
  and the example job's `dotnet-version`, which now reads `8.0.x`.

### Changed

- **The Python 3.9 CI row installs no pytest.** pytest 9.x requires 3.10, so
  the 3.9 row could only pin 8.4.2, which is below the fix for
  GHSA-6w46-j5rx-g56g and has no backport. The dev requirements are locked
  twice now (`ci-dev-py3.txt`, `ci-dev-py39.txt`, both hash-pinned), the 3.9
  lock carries maturin alone, and the row runs the suite through
  `run_without_pytest.py` -- the same modules, rewritten as plain functions
  with plain asserts, which 3.10 and up still run under pytest.

- **uv 0.12.15 for the lockfile script.** `scripts/update-lockfiles.sh`
  bootstraps 0.12.15 (was 0.12.13); the pin and all four release
  checksums move together, taken from the release's `.sha256` files.

## [0.1.2] - 2026-09-13

### Fixed

- **The R package builds on Windows.** `bindings/r/.Rbuildignore` excluded
  `src/Makevars.win` from the source tarball, so a Windows build from the
  tarball (which is what r-universe builds) linked the package object without
  the C ABI import library and failed to load. The file ships with the
  package now, and the ignore list names the staged files as configure
  actually names them.
- **The exported R functions are documented.** `wkproof_new`,
  `wkproof_command` and `wkproof_version` carried roxygen comments but no
  generated `man/` pages, which `R CMD check` reports as a WARNING.

## [0.1.1] - 2026-09-13

Same library and CLI code as 0.1.0; this release exists so that the GitHub
Release, its assets and the build provenance get produced, which 0.1.0 never
got.

### Fixed

- **The Maven Central publish is idempotent, and waits as long as Central
  takes.** The 0.1.0 release run published every package, but the Maven job
  timed out inside the plugin's default 30-minute wait for Central's
  validation while the deployment went through anyway. The run was red, so
  the GitHub Release, its assets and the provenance attestation were skipped,
  and a rerun could only fail again: Central refuses a second deployment of
  a version that already exists. The release workflow now checks Central
  for the version before deploying and skips a version that is already
  there, and the plugin waits up to two hours (`waitMaxTime`) instead of 30
  minutes.

- **`SECURITY.md` names the supported release.** It still said nothing had
  been published.

## [0.1.0] - 2026-09-13

### Changed

- **The Examples job runs every example and checks what it prints.** It used
  to parse the Python, Node and R files and `cargo check` the Rust one; the
  Go, Java and C# examples were never built. Now each example is executed
  against the library it ships with and must print `verify: valid`, and the
  ones that print the `report_hash` must print the one `examples/README.md`
  promises -- an engine bump that changes the hash fails here until the
  README says the new one. A WASM demo (`examples/wasm/prove.html`) joins the
  set, and its module is parse-checked.

- **The engine pin moves from a git rev to the published release.**
  `wickra-backtest-core` is `=0.1.4` from crates.io, the same source as the
  pinned rev `d58e357` minus one Java pom. A git rev is the trap the manifest
  comment itself described: cargo treats "this git URL, default branch" and
  "this git URL at rev X" as two sources, and a consumer that pins the engine
  differently carries two copies that share no `BacktestReport` -- wickra-zk
  hit it against this crate, wickra-verify's fuzz manifest against its own
  workspace. A registry version cannot split that way, and `cargo publish`
  needs one anyway.

- **The core crate is renamed `proof-core` -> `wickra-proof-core`.** The old
  name is taken on crates.io: `proof-core` 1.0.0 was published on 2026-09-09 by
  an unrelated project, and this repository has never released, so the name went
  from under it. `cargo publish -p proof-core` would have failed with a
  permission error on the first release -- after the tag, and after the other
  registries had already accepted their half.

  It was also the last core crate in the organisation without the `wickra-`
  prefix. wickra-strategy-ci renamed `strategy-ci-core` for the same reason and
  recorded why; this is the same move, forced sooner.

  **Nothing a user types changes.** The CLI binary keeps the name
  `wickra-proof`, and the Python, npm, NuGet, Maven and R package names never
  carried the crate name. The Rust library is now
  `cargo add wickra-proof-core`, and `use proof_core::` becomes
  `use wickra_proof_core::`.

### Fixed

- **The README's own `verify` command printed INVALID.** The proof shipped
  in `examples/data/` was generated by engine 0.1.0; the pinned engine is
  0.1.4 and folds the same candles to a different report. Both fixtures are
  regenerated, the README's hash table says the hash the examples actually
  print (`b6390900…`, not `12c288bb…`), and the Examples job now runs that
  verify command so the fixture cannot go stale again.
- **CodeQL's Java analysis built `examples/java/pom.xml`, which does not
  exist.** The example is one source file compiled with javac by the Examples
  job; CodeQL builds the binding alone, the shape xray uses.

- **A dependency declaration that nothing used and could not have resolved.**
  The workspace declared `wickra-data = "0.9"` for "the CLI's data input", but
  no crate referenced it: it is absent from `Cargo.lock` and `wickra_data`
  appears nowhere in the source. The CLI parses CSV with its own `parse_csv` in
  `run.rs`.

  The pin was also unreachable. `wickra-data` is published at `1.0.x`, and a
  `"0.9"` requirement can never resolve to it -- so it would not have produced a
  Dependabot PR either. A pin that blocks an update raises nothing; it just goes
  quiet. Removed rather than bumped, and the manifest now records why: `Candle`
  comes from the engine, so adding `wickra-data` back would mean two crates
  defining the same row -- two definitions of one type being the failure this
  repository exists to rule out. wickra-strategy-ci reached the same conclusion
  and carries the same note.

- **`SECURITY.md` promised support for versions that do not exist.** It offered
  fixes for "the latest `0.x` minor line" of a repository that has never
  released. It now says so plainly, and names what happens after `0.1.0`.

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

- `wickra-proof-core`: the deterministic Proof-of-Backtest core — a serde `ProofSpec`
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

[Unreleased]: https://github.com/wickra-lib/wickra-proof/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/wickra-lib/wickra-proof/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wickra-lib/wickra-proof/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wickra-lib/wickra-proof/releases/tag/v0.1.0
