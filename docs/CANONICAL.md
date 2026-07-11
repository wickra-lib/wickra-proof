# Canonicalization (normative)

This page specifies, normatively, how `proof-core` serializes a JSON value
before hashing it. Every language binding **must** reproduce these rules exactly:
the `report_hash` and `inputs_hash` are blake3 over the canonical *string*, so a
single-byte disagreement is a divergence. The reference implementation is
[`crates/proof-core/src/canonical.rs`](../crates/proof-core/src/canonical.rs);
this document is its contract.

## The rules

Each rule is normative.

1. **Object keys are sorted ascending by Unicode code point.** The reference
   collects entries into a `BTreeMap<String, _>` before emitting. Sorting is on
   the raw key string's code points, not a locale collation.
2. **No whitespace.** Separators are exactly `,` between elements and `:`
   between a key and its value — nothing else. No spaces, no newlines, no
   indentation.
3. **Floats are quantized to `1e-8` by decimal rounding.** A number is formatted
   with `{:.8}` (eight fractional digits, round-half-to-even as Rust's
   formatter does), then:
   - trailing zeros in the fractional part are trimmed;
   - a trailing `.` is removed, so a **whole-valued float collapses to its bare
     integer token** (`1.0` → `1`, `2.50000000` → `2.5`);
   - **negative zero normalizes to `0`** — a tiny negative that rounds to zero
     at eight decimals must not keep its sign, or idempotence breaks.
4. **Large magnitudes fall back to the shortest round-trippable form.** At or
   above `2^52 × 1e-8` (`45035996.27370496`, the magnitude where the f64 ULP
   first reaches the `1e-8` grid), `{:.8}` is finer than f64 can represent and is
   unstable, so the number is emitted via `Display` (always positional for f64,
   never scientific), which re-parses to the same value under `serde_json`'s
   `float_roundtrip` parser.
5. **`NaN` and `±inf` cannot occur.** `serde_json` rejects them at parse time,
   so every number reaching canonicalization is a finite integer or float by
   construction. There is no encoding for them.
6. **Integers stay integers.** An integer JSON number is emitted as its integer
   token, matching the whole-float collapse in rule 3 so `1` and `1.0` produce
   the same bytes.
7. **Strings use standard JSON escaping** (`serde_json`'s), and **array order is
   preserved** — arrays are meaning-bearing.

## Why the integer collapse matters

JSON in most host languages cannot preserve the `.0` of a whole-valued float:
`JSON.stringify(1.0)` in JavaScript emits `1`. If canonicalization kept `1.0`,
the hash would depend on which language loaded the spec. Collapsing every whole
value to its integer token makes the representation one that **every** language
can reproduce, so the hash is language-independent.

## The fixed-point property

The load-bearing invariant is that canonicalization is a **fixed point**:

```
canonicalize(parse(canonicalize(x))) == canonicalize(x)
```

Re-parsing a canonical string and canonicalizing again must yield identical
bytes. This is what lets a proof minted in one language be re-serialized and
re-hashed in another without drift. Rule 3's pure-decimal rounding (rather than a
binary `(x*1e8).round()/1e8` grid) and rule 4's magnitude cutoff exist precisely
to keep this a fixed point — a binary grid disagrees with the decimal one right
at the `2^52 × 1e-8` boundary and used to drift on inputs like `44447444.444…`
and `5e55`, found by the `canonicalize` fuzz target.

## Testing the contract

- **Golden corpus** (`golden/`) — fixed `(spec, data)` → expected canonical
  bytes and hash, checked byte-for-byte.
- **`canonicalize` fuzz target** (`fuzz/`) — asserts the fixed-point property
  across the full finite f64 range.
- **Cross-language golden tests** — every binding canonicalizes the same inputs
  and must produce identical strings.

## See also

- [ARCHITECTURE.md](ARCHITECTURE.md) — where canonicalization sits in the prove pipeline.
- [PROOF_FORMAT.md](PROOF_FORMAT.md) — what gets canonicalized (the report and the inputs).
- [VERIFYING.md](VERIFYING.md) — recomputing and comparing a foreign proof.
