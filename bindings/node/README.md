<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Proof — a deterministic (spec, data) → blake3 hash, byte-identical across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/ci.svg)](https://github.com/wickra-lib/wickra-proof/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-proof)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/npm.svg)](https://www.npmjs.com/package/wickra-proof)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/license.svg)](https://github.com/wickra-lib/wickra-proof#license)

# Wickra Proof — Node.js

---

**Proof-of-Backtest. Turn a `(spec, data)` pair into a deterministic backtest report *and* a canonical blake3 hash that anyone can recompute byte-for-byte in ten languages — for Node.js. `npm install wickra-proof` — prebuilt native binary, no system dependencies.**

Node.js bindings for the `wickra-proof` deterministic Proof-of-Backtest core
(napi-rs). Create a `Prover`, drive it with command JSON, read back response
JSON — the same protocol as the native CLI and every other binding.

## Install

```bash
npm install wickra-proof
```

The native addon ships as a prebuilt binary per platform (Linux, macOS,
Windows — x64 and arm64), selected automatically through optional
dependencies. There is nothing to compile.

### Building from this repository (contributors)

```bash
npm install
npm run build   # regenerates index.js / index.d.ts and the .node addon
npm test
```

## Quick start

```js
const { Prover } = require("wickra-proof");

const prover = new Prover();

const proof = JSON.parse(prover.command(JSON.stringify({
  cmd: "prove",
  spec: { strategy, dataset_ref: "BTCUSDT/1h" },
  data: { BTCUSDT: candles },
})));
console.log(proof.report_hash);

const verdict = JSON.parse(prover.command(JSON.stringify({
  cmd: "verify", proof, spec, data,
})));
// { ok: true, valid: true }
```

### Commands

| `cmd`          | Request fields          | Response                               |
|----------------|-------------------------|----------------------------------------|
| `prove`        | `spec`, `data`          | the full `Proof` JSON                  |
| `verify`       | `proof`, `spec`, `data` | `{"ok":true,"valid":<bool>}`           |
| `canonicalize` | `value`                 | `{"ok":true,"canonical":"..."}`        |
| `version`      | —                       | `{"version":...,"engine_version":...}` |

`Prover.prototype.version()` and the module-level `version()` return the library
version. Unknown commands come back in-band as `{"ok":false,"error":...}`.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of napi-rs, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-proof/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-proof>
- **Docs** (guides, spec reference, cookbook): <https://proof.wickra.org>
- **Runnable example:** [`examples/node/`](https://github.com/wickra-lib/wickra-proof/tree/main/examples/node)

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
