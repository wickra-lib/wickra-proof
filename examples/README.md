# Examples

A runnable "prove then verify" example in every language. Each one proves the
same `(spec, data)` pair — an EMA-cross strategy on a short V-shaped price path
for symbol `AAA` — prints the resulting `report_hash`, then verifies the proof
against its own inputs and asserts the verdict is `valid`.

Because the report hash is a canonical, cross-language digest of the backtest
report, **every language prints the exact same hash**. That byte-for-byte
agreement is the whole point of `wickra-proof`: a proof produced in one language
verifies in any other.

| Language | Path | Run |
|----------|------|-----|
| Rust | [`rust/`](rust/) | `cargo run -p wickra-proof-example` |
| Python | [`python/prove.py`](python/prove.py) | `pip install wickra-proof && python examples/python/prove.py` |
| Node.js | [`node/`](node/) | `cd examples/node && npm install && node prove.js` |
| C / C++ | [`c/`](c/) | see below |
| Go | [`go/`](go/) | `cd examples/go && go run .` |
| .NET | [`csharp/Prove/`](csharp/Prove/) | `dotnet run --project examples/csharp/Prove` |
| Java | [`java/Prove.java`](java/Prove.java) | see the header comment |
| R | [`r/prove.R`](r/prove.R) | `Rscript examples/r/prove.R` |

The native bindings (Python, Node.js) load their own compiled library. The
bindings that go through the C ABI (Go, .NET, Java, R, and the C/C++ example
itself) need the C ABI library built first:

```bash
cargo build --release -p wickra-proof-c
```

## C / C++

The C and C++ examples build with CMake and run under ctest:

```bash
cargo build --release -p wickra-proof-c
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

On Windows the build copies `wickra_proof.dll` next to each executable, since
there is no rpath.

## Data

The examples above carry their spec and candles inline so each file runs on its
own. The same fixture is also written out under [`data/`](data/) for tooling and
cross-language checks:

| File | What it is |
|------|------------|
| [`data/specs/example.json`](data/specs/example.json) | the `ProofSpec` (EMA-cross on `AAA`, `1h`) |
| [`data/candles/AAA.csv`](data/candles/AAA.csv) | the 12-bar V-shaped price path (`ts,open,high,low,close,volume`) |
| [`data/proofs/example.json`](data/proofs/example.json) | the resulting canonical `Proof` (its `report_hash` matches the runs below) |

## Expected output

Every example prints the version, the report hash, and the verify verdict:

```text
wickra-proof 0.1.0
report_hash: 12c288bb2fed3d3db887a2b09f375db9b7c84bc5f0fb2195e3af8900be2b334f
verify: valid
```

The hash is identical in every language — that is the guarantee:

| Language | `report_hash` |
|----------|---------------|
| Rust | `12c288bb2fed3d3db887a2b09f375db9b7c84bc5f0fb2195e3af8900be2b334f` |
| Python | `12c288bb2fed3d3db887a2b09f375db9b7c84bc5f0fb2195e3af8900be2b334f` |
| Node.js | `12c288bb2fed3d3db887a2b09f375db9b7c84bc5f0fb2195e3af8900be2b334f` |
| C / C++ | `12c288bb2fed3d3db887a2b09f375db9b7c84bc5f0fb2195e3af8900be2b334f` |
| Go | `12c288bb2fed3d3db887a2b09f375db9b7c84bc5f0fb2195e3af8900be2b334f` |
| .NET | `12c288bb2fed3d3db887a2b09f375db9b7c84bc5f0fb2195e3af8900be2b334f` |
| Java | `12c288bb2fed3d3db887a2b09f375db9b7c84bc5f0fb2195e3af8900be2b334f` |
| R | `12c288bb2fed3d3db887a2b09f375db9b7c84bc5f0fb2195e3af8900be2b334f` |
