#![no_main]
//! Fuzz the canonicalizer — the moat. Arbitrary bytes are parsed as a JSON value
//! and canonicalized. The canonical form must never panic and must never leak a
//! non-finite token: `serde_json` rejects `NaN`/`inf` at parse time, so every
//! value reaching `canonicalize` is finite and its output stays finite too.

use libfuzzer_sys::fuzz_target;
use proof_core::canonicalize;
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return;
    };
    let canonical = canonicalize(&value).expect("canonicalization of a parsed value is total");
    // No non-finite token can appear: quantization keeps everything finite, and a
    // string containing "NaN"/"inf" would have to come from the input verbatim,
    // which is fine inside a quoted string but never from a number.
    for token in ["NaN", "Infinity", "-Infinity"] {
        assert!(
            !canonical.contains(token) || value.to_string().contains(token),
            "canonical form leaked a non-finite token not present in the input"
        );
    }
    // Canonicalization is idempotent: re-parsing and re-canonicalizing is stable.
    if let Ok(reparsed) = serde_json::from_str::<Value>(&canonical) {
        assert_eq!(canonicalize(&reparsed).unwrap(), canonical);
    }
});
