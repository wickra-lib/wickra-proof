//! WebAssembly bindings for `wickra-proof` (wasm-bindgen).
//!
//! The deterministic Proof-of-Backtest core, compiled to WebAssembly for the
//! browser: create a stateless `Prover`, drive it with a command JSON
//! (`prove`, `verify`, `canonicalize`, `version`) and read back the response
//! JSON. The same command protocol crosses every binding, so a browser
//! front-end proves against the exact same core as the native CLI.
//!
//! The backtest engine runs sequentially here (no rayon thread pool in a
//! browser sandbox), which is byte-identical to the native run — the exact
//! cross-language golden check.

use wasm_bindgen::prelude::*;

use proof_core::Prover as CoreProver;

/// A stateless prover driven by JSON commands.
#[wasm_bindgen]
pub struct Prover {
    inner: CoreProver,
}

#[wasm_bindgen]
impl Prover {
    /// Create a stateless prover.
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Prover {
        Self {
            inner: CoreProver::new(),
        }
    }

    /// Apply a command JSON and return the resulting response JSON.
    pub fn command(&mut self, cmd_json: &str) -> Result<String, JsError> {
        self.inner
            .command_json(cmd_json)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// The library version.
    #[wasm_bindgen(js_name = version)]
    pub fn instance_version(&self) -> String {
        CoreProver::version().to_string()
    }
}

/// The library version.
#[wasm_bindgen]
pub fn version() -> String {
    CoreProver::version().to_string()
}
