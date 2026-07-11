# Verifying a foreign proof

The whole point of `wickra-proof` is that a proof produced in one language can be
independently checked in any other. This page describes what verification does,
why it cannot be fooled, and how to run it.

## Verification is recomputation, not trust

`verify(proof, spec, data)` does **not** trust the `report_hash` shipped in the
proof. It re-runs the entire prove pipeline from the supplied `(spec, data)`,
producing a *fresh* proof, and compares:

```
verify == (fresh.report_hash  == proof.report_hash)
       && (fresh.inputs_hash  == proof.inputs_hash)
       && (fresh.engine_version == proof.engine_version)
```

A forged proof — a hand-edited `report` with a matching hand-computed hash —
cannot pass, because the fresh report is computed from the spec and data, not
copied from the proof. Tamper with a single field of the `report`, the
`strategy`, the `dataset_ref`, or a single candle, and the recomputed hash
diverges. The verdict is a plain boolean; nothing about the prover is taken on
faith.

## Why it is language-independent

Every binding runs the *same* `proof-core` over the *same* canonicalization
([CANONICAL.md](CANONICAL.md)), so the fresh `report_hash` is byte-identical
regardless of language. A proof minted in Python verifies in Go; a proof minted
in Rust verifies in the browser over WASM. The cross-language golden tests assert
exactly this byte-for-byte equality.

## The engine-version check

`verify` also compares `engine_version`. If the verifier's linked
`wickra-backtest` differs from the one that minted the proof, `prove` (called
inside `verify`) either:

- returns an `EngineMismatch` error if the spec **pins** a version that does not
  match the linked one; or
- recomputes under the verifier's engine, whose `report_hash` will not match the
  claimed one — so `verify` returns `false`.

Either way, a proof is bound to the exact backtest semantics that produced it;
an engine upgrade surfaces as a visible mismatch, never a silent pass. See
[PROOF_FORMAT.md](PROOF_FORMAT.md) for the `engine_version` field.

## Running verification

### CLI

```bash
cargo run -p wickra-proof-cli -- verify \
  --proof examples/data/config.proof.json \
  --spec  examples/data/config.json \
  --data  examples/data/candles/AAA.csv
```

Prints `valid` only if the recomputed hashes and engine version all match.

### Any binding

Send a `verify` command and read the boolean back:

```json
{ "cmd": "verify", "proof": <Proof>, "spec": <ProofSpec>, "data": { "AAA": [ ... ] } }
```

→ `{ "ok": true, "valid": true }`

## What verification does and does not attest

- **It does** attest that a given `report` is the deterministic result of a given
  `(spec, data)` under a given engine version.
- **It does not** attest anything about the *quality* of the strategy, nor that
  the candle data is genuine market data — only that the report follows
  deterministically from the inputs the verifier was handed. Provenance of the
  data itself is out of scope; see [THREAT_MODEL.md](../THREAT_MODEL.md).

## See also

- [ARCHITECTURE.md](ARCHITECTURE.md) — the prove/verify pipeline.
- [CANONICAL.md](CANONICAL.md) — why the hash is byte-stable across languages.
- [PROOF_FORMAT.md](PROOF_FORMAT.md) — the `Proof` and `ProofSpec` schema.
