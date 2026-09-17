# Wickra Proof examples — Go

Runnable Go examples for the [Wickra Proof Go binding](../../bindings/go). The binding links against the
prebuilt C ABI library, so build and stage it once before running anything:

```bash
cargo build -p wickra-proof-c --release
mkdir -p bindings/go/lib/linux_amd64
cp target/release/libwickra_proof.so bindings/go/lib/linux_amd64/
```

## Run

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

## The examples

| Example | What it does |
|---------|--------------|
| `prove.go` | A runnable Go example: prove a (spec, data) pair through the binding, print the report hash, then verify the proof and assert it holds. |
