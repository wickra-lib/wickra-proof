#![no_main]
//! Fuzz the command dispatch surface every binding shares. Arbitrary bytes are
//! handed to `Prover::command_json` as a command envelope; any malformed input
//! must come back as an in-band error envelope, never a panic. The returned
//! string, when it parses, is always canonical (whitespace-free).

use libfuzzer_sys::fuzz_target;
use proof_core::Prover;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let mut prover = Prover::new();
    // Never panics: unknown commands and parse errors return an error envelope.
    if let Ok(response) = prover.command_json(text) {
        // A canonical response carries no raw newline: `serde_json` escapes any
        // newline inside a string literal, so a bare `\n` byte can never appear.
        // (`: ` is intentionally not checked — it can legitimately sit inside a
        // string, e.g. the error message "unknown cmd: foo".)
        assert!(!response.contains('\n'));
    }
});
