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
