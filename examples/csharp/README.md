# Wickra Proof examples — C#

Runnable C# examples for the [Wickra Proof C# binding](../../bindings/csharp). The binding consumes the C ABI
library through P/Invoke, so build it once before running anything:

```bash
cargo build -p wickra-proof-c --release
```

## Run

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Prove
```

## The examples

| Example | What it does |
|---------|--------------|
| `Prove/Program.cs` | A runnable .NET example: prove a (spec, data) pair through the binding, print the report hash, then verify the proof and assert it holds. |
