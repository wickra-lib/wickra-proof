<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Proof — a deterministic (spec, data) → blake3 hash, byte-identical across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/ci.svg)](https://github.com/wickra-lib/wickra-proof/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-proof)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-proof-go)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/license.svg)](https://github.com/wickra-lib/wickra-proof#license)

# Wickra Proof — Go

---

**Proof-of-Backtest. Turn a `(spec, data)` pair into a deterministic backtest report *and* a canonical blake3 hash that anyone can recompute byte-for-byte in ten languages — for Go. `go get github.com/wickra-lib/wickra-proof-go` — over the C ABI via cgo, prebuilt library bundled in the module.**

[Wickra Proof](https://github.com/wickra-lib/wickra-proof) folds a `(spec, data)` pair into a deterministic `wickra-backtest` report and a canonical blake3 hash that anyone recomputes byte-for-byte in ten languages. This package is the Go binding: it consumes the C ABI hub through cgo and exposes the stateless `Prover` handle with the same JSON protocol as every other binding.

## Install

Use the published **`wickra-proof-go`** module, which bundles the prebuilt C ABI
library for every platform, so `go get` + `go build` works with no extra steps
(a C compiler is still required, as the binding uses cgo):

```bash
go get github.com/wickra-lib/wickra-proof-go
```

`wickra-proof-go` is generated from this directory by the release pipeline: it mirrors
the Go sources, the vendored C ABI header (`include/wickra_proof.h`) and the prebuilt
libraries under `lib/<goos>_<goarch>/`. On Linux/macOS the library path is baked
in via rpath; on Windows the DLL must be discoverable at run time (next to the
executable or on `PATH`).

### Building from this repository (contributors)

This `bindings/go` directory is the development source. To build it directly,
compile the C ABI hub and stage the library into the per-platform directory cgo
links against:

```bash
cargo build -p wickra-proof-c --release
mkdir -p bindings/go/lib/linux_amd64                 # match your GOOS_GOARCH
cp target/release/libwickra_proof.so    bindings/go/lib/linux_amd64/   # Linux
cp target/release/libwickra_proof.dylib bindings/go/lib/darwin_arm64/  # macOS (arm64)
cp target/release/wickra_proof.dll      bindings/go/lib/windows_amd64/ # Windows
```

Then, with the library on the loader path, run `go test ./...` from this directory.

## Quick start

```go
package main

import (
	"fmt"

	wickra "github.com/wickra-lib/wickra-proof-go"
)

func main() {
	p := wickra.New()
	defer p.Close()

	cmd := `{"cmd":"prove","spec":{"strategy":{...},"dataset_ref":"BTCUSDT/1h"},` +
		`"data":{"BTCUSDT":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}}`

	proof, err := p.Command(cmd)
	if err != nil {
		panic(err)
	}
	fmt.Println(proof) // {"engine_version":"…","inputs_hash":"…","report":…,"report_hash":"…"}
	fmt.Println(wickra.Version())
}
```

### Commands

| Command        | Payload                 | Response                                                |
| -------------- | ----------------------- | ------------------------------------------------------ |
| `prove`        | `{spec, data}`          | `{report, inputs_hash, report_hash, engine_version}`   |
| `verify`       | `{proof, spec, data}`   | `{ok: true, valid: bool}`                              |
| `canonicalize` | `{value}`               | `{ok: true, canonical}`                                |
| `version`      | —                       | `{engine_version}`                                     |

Errors are reported in-band as `{"ok":false,"error":"…"}`.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of cgo over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-proof/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-proof>
- **Docs** (guides, spec reference, cookbook): <https://proof.wickra.org>
- **Runnable example:** [`examples/go/`](https://github.com/wickra-lib/wickra-proof/tree/main/examples/go)

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
