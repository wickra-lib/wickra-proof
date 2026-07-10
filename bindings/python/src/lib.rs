//! Python bindings for `wickra-proof`, exposed under the `wickra_proof`
//! package.
//!
//! Thin glue over the proof core's command surface: create a stateless
//! [`Prover`], drive it with a command JSON (`prove`, `verify`, `canonicalize`,
//! `version`) and read back the response JSON. The same command protocol
//! crosses every binding, so a Python front-end drives the exact same core as
//! the native CLI.

// PyO3 protocol methods take `self` by value/ref regardless of use.
#![allow(clippy::needless_pass_by_value)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use proof_core::Prover;

/// A stateless prover driven by JSON commands.
#[pyclass(name = "Prover")]
struct PyProver {
    inner: Prover,
}

#[pymethods]
impl PyProver {
    /// Create a stateless prover.
    #[new]
    fn new() -> Self {
        Self {
            inner: Prover::new(),
        }
    }

    /// Apply a command JSON and return the resulting response JSON.
    fn command(&mut self, cmd_json: &str) -> PyResult<String> {
        self.inner
            .command_json(cmd_json)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// The library version.
    #[staticmethod]
    fn version() -> &'static str {
        Prover::version()
    }
}

/// The native module (`wickra_proof._wickra_proof`).
#[pymodule]
fn _wickra_proof(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<PyProver>()?;
    Ok(())
}
