# Wickra Proof — Python

Python bindings for [`wickra-proof`](https://github.com/wickra-lib/wickra-proof):
the deterministic Proof-of-Backtest core. Create a `Prover`, drive it with
command JSONs, and read back response JSONs — the same command protocol every
language binding shares, so this front-end drives the exact same core as the
native CLI.

## Install

```sh
pip install wickra-proof
```

## Usage

```python
import json
from wickra_proof import Prover

prover = Prover()

proof = json.loads(prover.command(json.dumps({
    "cmd": "prove",
    "spec": {"strategy": strategy, "dataset_ref": "BTCUSDT/1h"},
    "data": {"BTCUSDT": candles},
})))
print(proof["report_hash"])

verdict = json.loads(prover.command(json.dumps({
    "cmd": "verify", "proof": proof, "spec": spec, "data": data,
})))
assert verdict == {"ok": True, "valid": True}
```

## Commands

| `cmd`          | Request fields                         | Response                         |
|----------------|----------------------------------------|----------------------------------|
| `prove`        | `spec`, `data`                         | the full `Proof` JSON            |
| `verify`       | `proof`, `spec`, `data`                | `{"ok":true,"valid":<bool>}`     |
| `canonicalize` | `value`                                | `{"ok":true,"canonical":"..."}`  |
| `version`      | —                                      | `{"version":...,"engine_version":...}` |

`Prover.version()` returns the library version. Unknown commands come back
in-band as `{"ok":false,"error":...}`; a malformed command JSON raises
`ValueError`.

## Build from source

```sh
maturin develop --release
pytest tests -q
```

## License

Dual-licensed under either MIT or Apache-2.0, at your option.
