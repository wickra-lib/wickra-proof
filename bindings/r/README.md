<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Proof — a deterministic (spec, data) → blake3 hash, byte-identical across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/ci.svg)](https://github.com/wickra-lib/wickra-proof/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-proof)
[![r-universe](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/r-universe.svg)](https://wickra-lib.r-universe.dev)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/license.svg)](https://github.com/wickra-lib/wickra-proof#license)

# Wickra Proof — R

---

**Proof-of-Backtest. Turn a `(spec, data)` pair into a deterministic backtest report *and* a canonical blake3 hash that anyone can recompute byte-for-byte in ten languages — for R. `install.packages("wickraproof", repos = "https://wickra-lib.r-universe.dev")` — over the C ABI via `.Call`, prebuilt library fetched on install.**

R bindings for the `wickra-proof` deterministic Proof-of-Backtest core, over its
C ABI hub (`.Call`). Create a stateless prover, drive it with command JSON
(`prove`, `verify`, `canonicalize`, `version`), read back the response JSON — the
same protocol as the CLI and every other binding.

## Install

From r-universe:

```r
install.packages("wickraproof", repos = "https://wickra-lib.r-universe.dev")
```

The package's `configure` downloads the prebuilt C ABI library for this exact
version from the GitHub release and bundles it, so an ordinary install needs
nothing but a C toolchain (Rtools on Windows) for the thin `.Call` glue layer. To
build against a local checkout instead, point it at the header and library with
the environment variables below.

### Building from this repository (contributors)

The package links the `wickra_proof` C ABI, located out-of-tree via two
environment variables:

```bash
cargo build -p wickra-proof-c --release
export WKPROOF_INC="$PWD/bindings/c/include"
export WKPROOF_LIB="$PWD/target/release"
# ensure the shared library is on the loader path at run time
export LD_LIBRARY_PATH="$WKPROOF_LIB:$LD_LIBRARY_PATH"   # Linux
R CMD INSTALL bindings/r
Rscript bindings/r/tests/run_tests.R
```

On Windows put `wickra_proof.dll` on `PATH`; on macOS use `DYLD_LIBRARY_PATH`.

## Quick start

```r
library(wickraproof)

prover <- wkproof_new()
cmd <- paste0(
  '{"cmd":"prove","spec":{"strategy":{...},"dataset_ref":"BTCUSDT/1h"},',
  '"data":{"BTCUSDT":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}}'
)
cat(wkproof_command(prover, cmd), "\n")
# {"engine_version":"…","inputs_hash":"…","report":…,"report_hash":"…"}
cat(wkproof_version(), "\n")
```

### Commands

| Command | Payload | Response |
|---------|---------|----------|
| `prove` | `{spec, data}` | `{report, inputs_hash, report_hash, engine_version}` |
| `verify` | `{proof, spec, data}` | `{ok: true, valid: bool}` |
| `canonicalize` | `{value}` | `{ok: true, canonical}` |
| `version` | — | `{engine_version}` |

Domain errors are reported in-band as `{"ok":false,"error":"…"}`.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of R's native `.Call` interface over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-proof/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-proof>
- **Docs** (guides, spec reference, cookbook): <https://proof.wickra.org>
- **Runnable example:** [`examples/r/`](https://github.com/wickra-lib/wickra-proof/tree/main/examples/r)

Wickra Proof ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-proof/blob/main/SECURITY.md>.

## Disclaimer

`wickra-proof` is research and engineering tooling, not financial advice. A proof
attests only that a given report is the deterministic result of a given spec over
given data — it makes no claim about the quality, profitability or future
performance of any strategy. Trading carries risk; you are responsible for your
own decisions.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-proof/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-proof/blob/main/LICENSE-MIT) at your option.
