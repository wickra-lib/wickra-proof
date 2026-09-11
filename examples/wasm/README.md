# wickra-proof WASM examples

Browser demos for the `wickra-proof-wasm` binding.

The WASM build carries the whole proof core: the same deterministic engine run,
the same canonicalization and the same blake3 `report_hash` the native bindings
produce. The spec and candles on the page are the bytes `examples/node/prove.js`
sends, so the hash it shows is the one every other language prints.

## Build

The module ships as a `wasm-pack` `--target web` bundle. Build it once from the
repository root:

```bash
wasm-pack build bindings/wasm --target web --release
```

That writes `bindings/wasm/pkg/` with the `.wasm` binary, the JS loader and the
TypeScript types. The demo imports the loader via
`../../bindings/wasm/pkg/wickra_proof_wasm.js`.

## Serve

ES-module imports need a real HTTP origin, not `file://`. Any static server from
the repository root works:

```bash
python -m http.server 8000
```

Then open `http://localhost:8000/examples/wasm/prove.html`.

## Demos

| File | What it does |
| --- | --- |
| `prove.html` | Proves the shared (spec, data) pair, shows the `report_hash` and the raw proof, then verifies the proof against the same inputs. The page counterpart of `examples/node/prove.js`. |

## See also

- [examples/README.md](../README.md) — the same proof in every other language.
- [bindings/wasm/README.md](../../bindings/wasm/README.md) — the module's API.
