<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Proof — a deterministic (spec, data) → blake3 hash, byte-identical across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/ci.svg)](https://github.com/wickra-lib/wickra-proof/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-proof)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/npm.svg)](https://www.npmjs.com/package/wickra-proof-wasm)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/license.svg)](https://github.com/wickra-lib/wickra-proof#license)

# Wickra Proof — WASM

---

**Proof-of-Backtest. Turn a `(spec, data)` pair into a deterministic backtest report *and* a canonical blake3 hash that anyone can recompute byte-for-byte in ten languages — for WASM. `npm install wickra-proof-wasm` — pure WebAssembly, runs anywhere a modern JS engine does.**

WASM bindings for the `wickra-proof` deterministic Proof-of-Backtest core,
compiled to WebAssembly with wasm-bindgen.

Create a stateless `Prover`, drive it with a command JSON (`prove`, `verify`,
`canonicalize`, `version`) and read back the response JSON — the same protocol
as every other binding, running in the browser.

The core is built with `--no-default-features`, so the backtest engine runs
**sequentially** (no rayon thread pool in the browser sandbox) and the report
and its blake3 hash are byte-identical to the native ones — the exact
cross-language golden check.

## Install

```bash
npm install wickra-proof-wasm
```

### Building from this repository (contributors)

```bash
wasm-pack build --target web
```

This emits `pkg/` with the `.wasm` module and JS glue.

## Quick start

```js
import init, { Prover, version } from "wickra-proof-wasm";

await init();

const prover = new Prover();
const proof = JSON.parse(
  prover.command(JSON.stringify({ cmd: "prove", spec, data })),
);
// proof.report_hash / proof.inputs_hash are 64-hex blake3 digests, identical
// to the native CLI for the same (spec, data).

const verdict = JSON.parse(
  prover.command(JSON.stringify({ cmd: "verify", proof, spec, data })),
);
// { ok: true, valid: true }

console.log(version()); // the library version
```

### Commands

| Command        | Payload                     | Response                                        |
| -------------- | --------------------------- | ----------------------------------------------- |
| `prove`        | `{ spec, data }`            | `{ report, inputs_hash, report_hash, engine_version }` |
| `verify`       | `{ proof, spec, data }`     | `{ ok: true, valid: bool }`                     |
| `canonicalize` | `{ value }`                 | `{ ok: true, canonical }`                       |
| `version`      | —                           | `{ engine_version }`                            |

Errors are reported in-band as `{ "ok": false, "error": "…" }`.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of wasm-bindgen, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-proof/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-proof>
- **Docs** (guides, spec reference, cookbook): <https://proof.wickra.org>
- **Runnable example:** [`examples/wasm/`](https://github.com/wickra-lib/wickra-proof/tree/main/examples/wasm)

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
