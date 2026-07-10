#![no_main]
//! Fuzz the spec-parsing path: arbitrary bytes are parsed as a `ProofSpec` (JSON
//! and TOML) and as a `Config`. None must panic; malformed input must surface as
//! a clean `Err`.

use libfuzzer_sys::fuzz_target;
use proof_core::{Config, ProofSpec};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let _ = ProofSpec::from_json(text);
    let _ = ProofSpec::from_toml(text);
    let _ = Config::from_json(text);
    let _ = Config::from_toml(text);
});
