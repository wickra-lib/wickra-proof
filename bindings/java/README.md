# Wickra Proof — Java

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

## Usage

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

## API

| Member | Description |
|--------|-------------|
| `new Prover()` | Create a stateless prover. |
| `String command(String cmdJson)` | Apply a command JSON, return the response JSON. |
| `static String version()` | The library version. |
| `close()` | Free the native handle (via `AutoCloseable`). |

## Commands

| Command | Payload | Response |
|---------|---------|----------|
| `prove` | `{spec, data}` | `{report, inputs_hash, report_hash, engine_version}` |
| `verify` | `{proof, spec, data}` | `{ok: true, valid: bool}` |
| `canonicalize` | `{value}` | `{ok: true, canonical}` |
| `version` | — | `{engine_version}` |

Domain errors are reported in-band as `{"ok":false,"error":"…"}`.

## Building from this repository (contributors)

```bash
cargo build -p wickra-proof-c
mvn -f bindings/java/pom.xml test
```

## License

`MIT OR Apache-2.0`.
