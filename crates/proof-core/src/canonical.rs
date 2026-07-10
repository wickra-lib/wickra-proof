//! Deterministic JSON canonicalization — the single source of the hash's
//! stability across all ten language bindings.
//!
//! The rules (each normative):
//! 1. Object keys are sorted ascending by Unicode code point (`BTreeMap`).
//! 2. No whitespace: `,` and `:` separators, nothing else.
//! 3. Every float is quantized to 1e-8 by decimal rounding (`{:.8}`), trailing
//!    zeros trimmed. A whole-valued float collapses to its integer token
//!    (`1.0` -> `"1"`), because JSON in most host languages (JavaScript above
//!    all) cannot preserve the `.0`: `JSON.stringify` emits `1`, and the hash
//!    must be byte-identical regardless of which language loaded the spec.
//!    Integers stay integers by the same token. Magnitudes at or above
//!    `2^52 * 1e-8` (where the f64 ULP reaches the grid) instead use the
//!    shortest round-trippable form, keeping canonicalization a fixed point.
//! 4. `NaN` / `±inf` cannot occur: `serde_json` rejects them at parse time, so
//!    every `Value` number is a finite integer or float by construction.
//! 5. Strings use `serde_json`'s standard escaping.
//! 6. Array order is preserved (it is meaning-bearing).

use crate::error::Result;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Format a float in a fixed, cross-language-stable, idempotent decimal form:
/// eight fractional digits, trailing zeros trimmed, and a whole value collapsed
/// to its bare integer token (`1.0` -> `"1"`). The integer collapse is essential:
/// a host language cannot distinguish `1.0` from `1` in JSON — `JSON.stringify`
/// emits `1` — so the only representation every language can reproduce for a
/// whole number is the integer one. Negative zero normalizes to `0`.
///
/// Quantization to 1e-8 is done purely by the `{:.8}` decimal rounding — no
/// separate binary-grid step. That matters for the moat's load-bearing property:
/// canonicalization must be a fixed point (canonicalize -> parse -> canonicalize
/// yields the same bytes). Rounding to eight decimals is a fixed point only while
/// the 1e-8 grid is coarser than the f64 ULP, i.e. `|x| < 2^52 * 1e-8`; a binary
/// `(x*1e8).round()/1e8` grid disagrees with the decimal one right at that
/// boundary and used to drift (found by the canonicalize fuzz target on inputs
/// like `44447444.444...` and `5e55`).
///
/// At or above that magnitude the 1e-8 grid is finer than f64 can represent, so
/// `{:.8}` is meaningless and unstable; emit the shortest round-trippable form
/// (`Display`, always positional for f64, never scientific) instead, which
/// re-parses to the same value under `serde_json`'s `float_roundtrip` parser.
fn format_f64(x: f64) -> String {
    // 2^52 * 1e-8: the magnitude at which the f64 ULP first reaches the 1e-8
    // grid. Below it, decimal rounding to eight places is a fixed point.
    const GRID_RESOLUTION_LIMIT: f64 = 45_035_996.273_704_96;
    let x = if x == 0.0 { 0.0 } else { x };
    if x.abs() >= GRID_RESOLUTION_LIMIT {
        return format!("{x}");
    }
    let mut s = format!("{x:.8}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

fn write_number(out: &mut String, n: &serde_json::Number) {
    if let Some(i) = n.as_i64() {
        write!(out, "{i}").expect("writing to a String is infallible");
    } else if let Some(u) = n.as_u64() {
        write!(out, "{u}").expect("writing to a String is infallible");
    } else {
        // A JSON number that is neither i64 nor u64 is a finite f64: serde_json
        // rejects NaN/inf at parse time and never yields an unrepresentable
        // number here.
        let f = n.as_f64().unwrap_or(0.0);
        out.push_str(&format_f64(f));
    }
}

fn write_string(out: &mut String, s: &str) {
    // serde_json's string serializer is the reference escaping; reuse it.
    let encoded = Value::String(s.to_string()).to_string();
    out.push_str(&encoded);
}

fn write_value(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => write_number(out, n),
        Value::String(s) => write_string(out, s),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(out, item);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let sorted: BTreeMap<&String, &Value> = map.iter().collect();
            out.push('{');
            for (i, (key, val)) in sorted.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(out, key);
                out.push(':');
                write_value(out, val);
            }
            out.push('}');
        }
    }
}

/// Produce the canonical, whitespace-free, key-sorted string form of `value`.
/// Byte-identical across languages; the input to the blake3 hash.
///
/// The `Result` return keeps a uniform signature with the rest of the API and
/// the language bindings; canonicalization of a `serde_json::Value` is total.
pub fn canonicalize(value: &Value) -> Result<String> {
    let mut out = String::new();
    write_value(&mut out, value);
    Ok(out)
}

/// The lowercase 64-hex blake3 of a canonical string (no prefix).
pub(crate) fn blake3_hex(canonical: &str) -> String {
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}
