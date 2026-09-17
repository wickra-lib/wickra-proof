# Wickra Proof examples — R

Runnable R examples for the [Wickra Proof R binding](../../bindings/r). The package compiles a thin
`.Call` glue layer against the C ABI library, so build the library and install
the package first (the CI examples job does exactly this):

```bash
cargo build -p wickra-proof-c --release
R CMD INSTALL bindings/r
```

## Run

```bash
Rscript examples/r/prove.R
```

## The examples

| Example | What it does |
|---------|--------------|
| `prove.R` | A runnable R example: prove a (spec, data) pair through the binding, print the report hash, then verify the proof and assert it holds. |
