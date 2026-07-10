# Wickra Proof — WASM

WASM bindings for the `wickra-proof` deterministic Proof-of-Backtest core,
compiled to WebAssembly with wasm-bindgen.

Create a stateless `Prover`, drive it with a command JSON (`prove`, `verify`,
`canonicalize`, `version`) and read back the response JSON — the same protocol
as every other binding, running in the browser.

The core is built with `--no-default-features`, so the backtest engine runs
**sequentially** (no rayon thread pool in the browser sandbox) and the report
and its blake3 hash are byte-identical to the native ones — the exact
cross-language golden check.

## Build

```bash
wasm-pack build --target web
```

This emits `pkg/` with the `.wasm` module and JS glue.

## Usage

```js
import init, { Prover, version } from "./pkg/wickra_proof_wasm.js";

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

## Commands

| Command        | Payload                     | Response                                        |
| -------------- | --------------------------- | ----------------------------------------------- |
| `prove`        | `{ spec, data }`            | `{ report, inputs_hash, report_hash, engine_version }` |
| `verify`       | `{ proof, spec, data }`     | `{ ok: true, valid: bool }`                     |
| `canonicalize` | `{ value }`                 | `{ ok: true, canonical }`                       |
| `version`      | —                           | `{ engine_version }`                            |

Errors are reported in-band as `{ "ok": false, "error": "…" }`.
