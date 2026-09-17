<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Proof — a deterministic (spec, data) → blake3 hash, byte-identical across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/ci.svg)](https://github.com/wickra-lib/wickra-proof/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-proof)
[![NuGet](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/nuget.svg)](https://www.nuget.org/packages/Wickra.Proof)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/license.svg)](https://github.com/wickra-lib/wickra-proof#license)

# Wickra Proof — C#

---

**Proof-of-Backtest. Turn a `(spec, data)` pair into a deterministic backtest report *and* a canonical blake3 hash that anyone can recompute byte-for-byte in ten languages — for C#. `dotnet add package Wickra.Proof` — prebuilt native library, no system dependencies.**

The wickra-proof core for .NET, over the C ABI via P/Invoke. The native library ships
inside the NuGet package for every supported runtime identifier, so there is
nothing to install alongside it.

## Install

```bash
dotnet add package Wickra.Proof
```

The native library ships prebuilt per platform under `runtimes/<rid>/native/`,
selected automatically. There is nothing to compile. Targets .NET 8 and later.

```sh
dotnet add package WickraProof
```

### Building from this repository (contributors)

Requires the .NET SDK and a Rust toolchain:

```sh
cargo build -p wickra-proof-c --release
dotnet test bindings/csharp
```

The test project resolves the freshly built native library from `target/release`.

## Quick start

The binding is a thin, faithful surface over the same command boundary every
other binding drives, so a request built here produces the same canonical bytes
it would in Rust, Python or Go.

```csharp
using WickraProof;

using var handle = new Prover();
string response = handle.Command("""{"cmd":"version"}""");
Console.WriteLine(response);
```

`Prover` owns a native handle and implements `IDisposable`; the `using` above is
what releases it. Dropping the reference without disposing leaks the handle
until the finalizer runs.

### What travels with the package

The managed assembly, the native C ABI library for each supported runtime
identifier, and both licence texts. The package declares `MIT OR Apache-2.0`
and carries `LICENSE-MIT` and `LICENSE-APACHE` to match.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of `[LibraryImport]` P/Invoke over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-proof/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-proof>
- **Docs** (guides, spec reference, cookbook): <https://proof.wickra.org>
- **Runnable example:** [`examples/csharp/`](https://github.com/wickra-lib/wickra-proof/tree/main/examples/csharp)

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
