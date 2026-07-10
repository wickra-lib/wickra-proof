# Wickra Proof — Node.js

Node.js bindings for the `wickra-proof` deterministic Proof-of-Backtest core
(napi-rs). Create a `Prover`, drive it with command JSON, read back response
JSON — the same protocol as the native CLI and every other binding.

## Install

```bash
npm install wickra-proof
```

## Usage

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

## Commands

| `cmd`          | Request fields          | Response                               |
|----------------|-------------------------|----------------------------------------|
| `prove`        | `spec`, `data`          | the full `Proof` JSON                  |
| `verify`       | `proof`, `spec`, `data` | `{"ok":true,"valid":<bool>}`           |
| `canonicalize` | `value`                 | `{"ok":true,"canonical":"..."}`        |
| `version`      | —                       | `{"version":...,"engine_version":...}` |

`Prover.prototype.version()` and the module-level `version()` return the library
version. Unknown commands come back in-band as `{"ok":false,"error":...}`.

## Build from source

```bash
npm install
npm run build   # regenerates index.js / index.d.ts and the .node addon
npm test
```

## License

Dual-licensed under either MIT or Apache-2.0, at your option.
