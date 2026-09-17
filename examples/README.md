# Wickra Proof examples

A runnable "prove then verify" example in every language. Each one proves the
same `(spec, data)` pair — an EMA-cross strategy on a short V-shaped price path
for symbol `AAA` — prints the resulting `report_hash`, then verifies the proof
against its own inputs and asserts the verdict is `valid`.

## What every example prints

Every example prints the version, the report hash, and the verify verdict:

```text
wickra-proof 0.1.2
report_hash: b63909002621f33009f3259fa13c194e5e7c1bf5fb81a94359d53f1318399b0d
verify: valid
```

## Rust — `examples/rust/`

As the CI examples job runs it, from the repository root:

```bash
cargo run -q --manifest-path examples/rust/Cargo.toml
```

| Example | What it does |
| --- | --- |
| `src/main.rs` | A runnable Rust example: prove a (spec, data) pair with the native `prove` API, print the report hash, then verify the proof and assert it holds. |

## C / C++ — `examples/c/`

Build the library first (`cargo build -p wickra-proof-c --release`), then build and run
the examples via CMake, as the CI C ABI job does:

```bash
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

| Example | What it does |
| --- | --- |
| `prove.c` | A minimal C example: prove a (spec, data) pair through the wickra-proof C ABI, |
| `prove.cpp` | A minimal C++ example: prove a (spec, data) pair through the wickra-proof C ABI, print the report hash, then verify the proof and assert it holds. |

## C# — `examples/csharp/`

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Prove
```

| Example | What it does |
| --- | --- |
| `Prove/Program.cs` | A runnable .NET example: prove a (spec, data) pair through the binding, print the report hash, then verify the proof and assert it holds. |

## Go — `examples/go/`

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

| Example | What it does |
| --- | --- |
| `prove.go` | A runnable Go example: prove a (spec, data) pair through the binding, print the report hash, then verify the proof and assert it holds. |

## R — `examples/r/`

As the CI examples job runs it, from the repository root:

```bash
R CMD INSTALL bindings/r
Rscript examples/r/prove.R
```

| Example | What it does |
| --- | --- |
| `prove.R` | A runnable R example: prove a (spec, data) pair through the binding, print the report hash, then verify the proof and assert it holds. |

## Java — `examples/java/`

As the CI examples job runs it, from the repository root:

```bash
mvn -f bindings/java/pom.xml -q package -DskipTests
javac -cp bindings/java/target/classes examples/java/Prove.java -d examples/java/out
java --enable-native-access=ALL-UNNAMED  -Dnative.lib.dir="$PWD/target/release"  -cp "bindings/java/target/classes:examples/java/out" Prove
```

| Example | What it does |
| --- | --- |
| `Prove.java` | A runnable Java example: prove a (spec, data) pair through the binding, print the report hash, then verify the proof and assert it holds. |

## Python — `examples/python/`

As the CI examples job runs it, from the repository root:

```bash
python -m pip install --require-hashes -r .github/requirements/ci-dev-py3.txt
( cd bindings/python && maturin build --release --out dist )
python -m pip install --no-index --find-links bindings/python/dist wickra-proof
python examples/python/prove.py
```

| Example | What it does |
| --- | --- |
| `prove.py` | A runnable Python example: prove a (spec, data) pair through the binding, |

## Node.js — `examples/node/`

As the CI examples job runs it, from the repository root:

```bash
( cd bindings/node && npm install --no-audit --no-fund && npx napi build --platform --release )
( cd examples/node && npm install --no-audit --no-fund )
node examples/node/prove.js
```

| Example | What it does |
| --- | --- |
| `prove.js` | A runnable Node.js example: prove a (spec, data) pair through the binding, print the report hash, then verify the proof and assert it holds. |

## WASM — `examples/wasm/`

Build the WASM package, serve the repository root, and open the page in a browser;
the module script inside it is what runs (CI parses it with `node --check`):

```bash
wasm-pack build bindings/wasm --target web
python -m http.server 8000     # then open http://localhost:8000/examples/wasm/
```

| Example | What it does |
| --- | --- |
| `prove.html` | A runnable example against this binding. |

## Example datasets

The examples read from [`examples/data/`](data/): `config.json`, `config.proof.json`. The
cross-language golden fixtures, which every binding is checked against byte for
byte, live in [`../golden/`](../golden).

## Data

The examples above carry their spec and candles inline so each file runs on its
own. The same fixture is also written out under [`data/`](data/) for tooling and
cross-language checks:

| File | What it is |
|------|------------|
| [`data/specs/example.json`](data/specs/example.json) | the `ProofSpec` (EMA-cross on `AAA`, `1h`) |
| [`data/candles/AAA.csv`](data/candles/AAA.csv) | the 12-bar V-shaped price path (`ts,open,high,low,close,volume`) |
| [`data/proofs/example.json`](data/proofs/example.json) | the resulting canonical `Proof` (its `report_hash` matches the runs below) |
