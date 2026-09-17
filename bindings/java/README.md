<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Proof — a deterministic (spec, data) → blake3 hash, byte-identical across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/ci.svg)](https://github.com/wickra-lib/wickra-proof/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-proof)
[![Maven Central](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/maven.svg)](https://central.sonatype.com/artifact/org.wickra/wickra-proof)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/license.svg)](https://github.com/wickra-lib/wickra-proof#license)

# Wickra Proof — Java

---

**Proof-of-Backtest. Turn a `(spec, data)` pair into a deterministic backtest report *and* a canonical blake3 hash that anyone can recompute byte-for-byte in ten languages — for Java. `org.wickra:wickra-proof` — prebuilt native library inside the jar, no JNI, no system dependencies.**

JVM bindings for the `wickra-proof` deterministic Proof-of-Backtest core over its
C ABI hub (FFM / Panama, `java.lang.foreign`). Create a stateless `Prover`, drive
it with command JSON (`prove`, `verify`, `canonicalize`, `version`) and read back
the response JSON — the same protocol as every other binding.

## Requirements

- Java 22+ (the Foreign Function & Memory API is stable since 22).
- Run with `--enable-native-access=ALL-UNNAMED`.
- The native library (`wickra_proof`) must be resolvable — either on the library
  path or via the `native.lib.dir` system property pointing at the directory that
  holds `libwickra_proof.{so,dylib}` / `wickra_proof.dll`.

## Install

Maven:

```xml
<dependency>
  <groupId>org.wickra</groupId>
  <artifactId>wickra-proof</artifactId>
  <version>0.1.2</version>
</dependency>
```

Gradle:

```kotlin
implementation("org.wickra:wickra-proof:0.1.2")
```

The native library ships prebuilt per platform inside the jar and is
extracted automatically on first use. There is nothing to compile.

### Building from this repository (contributors)

```bash
cargo build -p wickra-proof-c
mvn -f bindings/java/pom.xml test
```

## Quick start

```java
import org.wickra.proof.Prover;

try (Prover prover = new Prover()) {
    String cmd = """
        {"cmd":"prove","spec":{"strategy":{...},"dataset_ref":"BTCUSDT/1h"},
        "data":{"BTCUSDT":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}}""";
    System.out.println(prover.command(cmd));
    // {"engine_version":"…","inputs_hash":"…","report":…,"report_hash":"…"}
}
System.out.println(Prover.version());
```

### API

| Member | Description |
|--------|-------------|
| `new Prover()` | Create a stateless prover. |
| `String command(String cmdJson)` | Apply a command JSON, return the response JSON. |
| `static String version()` | The library version. |
| `close()` | Free the native handle (via `AutoCloseable`). |

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
the call overhead of the Java Foreign Function & Memory API over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-proof/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-proof>
- **Docs** (guides, spec reference, cookbook): <https://proof.wickra.org>
- **Runnable example:** [`examples/java/`](https://github.com/wickra-lib/wickra-proof/tree/main/examples/java)

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
