//! Canonicalization vectors — the moat. The canonical form is the load-bearing
//! contract every language binding must reproduce byte-for-byte, so these pin
//! its exact shape: sorted keys at every depth, `1e-8`-rounded floats with
//! trailing zeros trimmed (whole values collapsed to integer tokens so no host
//! language's `1.0`-vs-`1` ambiguity can shift the hash), and no whitespace.

use proof_core::canonicalize;
use serde_json::json;

#[test]
fn keys_are_sorted_at_every_depth() {
    let value = json!({ "b": 1, "a": { "d": 4, "c": 3 }, "z": [] });
    assert_eq!(
        canonicalize(&value).unwrap(),
        r#"{"a":{"c":3,"d":4},"b":1,"z":[]}"#
    );
}

#[test]
fn key_order_does_not_affect_the_canonical_form() {
    let a: serde_json::Value = serde_json::from_str(r#"{"b":1,"a":2}"#).unwrap();
    let b: serde_json::Value = serde_json::from_str(r#"{"a":2,"b":1}"#).unwrap();
    assert_eq!(canonicalize(&a).unwrap(), canonicalize(&b).unwrap());
}

#[test]
fn floats_round_to_1e8_and_trim_trailing_zeros() {
    // 0.1 + 0.2 = 0.30000000000000004 collapses to "0.3".
    assert_eq!(canonicalize(&json!(0.1 + 0.2)).unwrap(), "0.3");
    // A whole-valued float collapses to its integer token: no host language can
    // preserve the `.0` in JSON, so `1.0` and `1` MUST hash identically.
    assert_eq!(canonicalize(&json!(3.0)).unwrap(), "3");
    assert_eq!(canonicalize(&json!(1.0)).unwrap(), "1");
    // Negative zero normalizes to a plain zero.
    assert_eq!(canonicalize(&json!(-0.0f64)).unwrap(), "0");
}

#[test]
fn a_whole_float_and_its_integer_are_indistinguishable() {
    // The moat's cross-language guarantee: a spec that carries `1.0` (loaded by a
    // language that keeps floats) and one that carries `1` (loaded by JavaScript,
    // which collapses it) canonicalize to the exact same bytes.
    assert_eq!(
        canonicalize(&json!(1.0)).unwrap(),
        canonicalize(&json!(1)).unwrap()
    );
    assert_eq!(
        canonicalize(&json!(10000.0)).unwrap(),
        canonicalize(&json!(10000)).unwrap()
    );
}

#[test]
fn canonicalization_is_idempotent_for_extreme_magnitudes() {
    // Canonicalization must be a fixed point: re-parsing a canonical form and
    // canonicalizing it again yields the identical bytes. Huge magnitudes are
    // the trap — quantizing `5e55` via `x * 1e8` overflows f64's exact-integer
    // range and used to drift on the second pass (found by the canonicalize
    // fuzz target, input `5e55`). Values this large are already integral in
    // f64, so they must pass through and round-trip stably.
    for value in [
        json!(5e55),
        json!(-5e55),
        json!(1e300),
        json!(9.007_199_254_740_993e15),
        json!(1e16),
        json!(-1e16),
        // Mid-range values near the 1e-8 grid resolution limit (2^52 * 1e-8 ~
        // 4.5e7), where a binary quantization grid used to disagree with the
        // decimal one. `44447444.444...` is the second input the fuzz target
        // found; the two straddling values pin both sides of the cutoff.
        json!(44_447_444.444_444_74_f64),
        json!(-44_447_444.444_444_74_f64),
        json!(45_035_995.0_f64),
        json!(45_035_997.0_f64),
        json!(9_999_999.999_999_99_f64),
    ] {
        let once = canonicalize(&value).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&once).unwrap();
        let twice = canonicalize(&reparsed).unwrap();
        assert_eq!(once, twice, "canonicalization not idempotent for {value}");
        assert!(
            !once.contains('e') && !once.contains('E'),
            "canonical form must not use scientific notation: {once}"
        );
    }
}

#[test]
fn a_difference_below_1e8_canonicalizes_identically() {
    // Two floats that differ only past the 8th fractional digit round together.
    let a = canonicalize(&json!(1.000_000_001_f64)).unwrap();
    let b = canonicalize(&json!(1.000_000_002_f64)).unwrap();
    assert_eq!(a, b);
}

#[test]
fn is_whitespace_free() {
    let value = json!({ "a": [1, 2, { "b": 3 }], "c": "x y" });
    let out = canonicalize(&value).unwrap();
    // No structural whitespace; a space inside a string literal is preserved.
    assert!(!out.contains('\n'));
    assert!(!out.contains(": "));
    assert!(!out.contains(", "));
    assert!(out.contains("x y"));
}

#[test]
fn empty_containers_and_null() {
    assert_eq!(canonicalize(&json!({})).unwrap(), "{}");
    assert_eq!(canonicalize(&json!([])).unwrap(), "[]");
    assert_eq!(canonicalize(&json!(null)).unwrap(), "null");
}

#[test]
fn is_deterministic() {
    let value = json!({ "nested": { "arr": [3, 1, 2], "s": "wickra" }, "n": 42 });
    assert_eq!(canonicalize(&value).unwrap(), canonicalize(&value).unwrap());
}
