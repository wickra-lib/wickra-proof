# Fuzzing Wickra Proof

[`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html) harnesses for the parsing and stateful entry points of Wickra Proof. Fuzzing requires a nightly Rust toolchain; CI runs every target for 30 seconds on the family's pinned `nightly-2026-07-01`.

## Setup

```bash
cargo install cargo-fuzz
rustup toolchain install nightly-2026-07-01
```

The date is the family's fuzz nightly, pinned in `ci.yml`: a rolling `nightly`
regressed with a codegen ICE unrelated to this code, so every repository moves
the date together, on purpose.

## Targets

| Target | What it exercises |
| --- | --- |
| `spec_parse` | The spec-parsing path: arbitrary bytes are parsed as a `ProofSpec` (JSON and TOML) and as a `Config`. |
| `canonicalize` | The canonicalizer — the moat. |
| `prove` | The command dispatch surface every binding shares. |
| `verify_roundtrip` | The prove/verify contract with genuine inputs. |

## Run

```bash
# From the repository root:
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu spec_parse
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu canonicalize
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu prove
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu verify_roundtrip
```

Each run continues until a crash is found or it is interrupted. A short
time-boxed smoke run is what CI does:

```bash
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu spec_parse -- -max_total_time=30
```

The expectation for every target is that it never panics: malformed or
adversarial input must surface as an `Err` or an in-band error, never a crash.
