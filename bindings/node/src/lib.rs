//! Node.js bindings for `wickra-proof` (napi-rs).
//!
//! Thin glue over the proof core's command surface: create a stateless
//! `Prover`, drive it with a command JSON (`prove`, `verify`, `canonicalize`,
//! `version`) and read back the response JSON. The same command protocol
//! crosses every binding, so a Node front-end drives the exact same core as the
//! native CLI.

#![allow(missing_debug_implementations)]
// napi exposes owned `String` arguments; the bodies only need to borrow them.
#![allow(clippy::needless_pass_by_value)]

use napi::Result;
use napi_derive::napi;

use proof_core::Prover as CoreProver;

/// Build a napi error from a message.
fn err(message: impl Into<String>) -> napi::Error {
    napi::Error::from_reason(message.into())
}

/// The library version.
#[napi]
pub fn version() -> String {
    CoreProver::version().to_string()
}

/// A stateless prover driven by JSON commands.
#[napi]
pub struct Prover {
    inner: CoreProver,
}

#[napi]
impl Prover {
    /// Create a stateless prover.
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: CoreProver::new(),
        }
    }

    /// Apply a command JSON and return the resulting response JSON.
    #[napi]
    pub fn command(&mut self, cmd_json: String) -> Result<String> {
        self.inner
            .command_json(&cmd_json)
            .map_err(|e| err(e.to_string()))
    }

    /// The library version.
    #[napi]
    pub fn version(&self) -> String {
        CoreProver::version().to_string()
    }
}
