//! Deterministic JSON canonicalization — the single source of the hash's
//! stability across all ten language bindings.
//!
//! The rules (each normative):
//! 1. Object keys are sorted ascending by Unicode code point (`BTreeMap`).
//! 2. No whitespace: `,` and `:` separators, nothing else.
//! 3. Every float is quantized via `round_to(x, 1e-8)` and formatted with a
//!    fixed decimal representation (always at least one fractional digit).
//!    Integers stay integers.
//! 4. `NaN` / `±inf` cannot occur: `serde_json` rejects them at parse time, so
//!    every `Value` number is a finite integer or float by construction.
//! 5. Strings use `serde_json`'s standard escaping.
//! 6. Array order is preserved (it is meaning-bearing).

use crate::error::Result;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Quantize a float to 1e-8 so its representation is identical across languages.
fn round_to(x: f64) -> f64 {
    (x * 1e8).round() / 1e8
}

/// Format a quantized float with a fixed, cross-language-stable decimal form:
/// eight fractional digits, trailing zeros trimmed but always one digit kept,
/// and negative zero normalized to `0.0`.
fn format_f64(x: f64) -> String {
    let x = if x == 0.0 { 0.0 } else { x };
    let mut s = format!("{x:.8}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.push('0');
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
        out.push_str(&format_f64(round_to(f)));
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
