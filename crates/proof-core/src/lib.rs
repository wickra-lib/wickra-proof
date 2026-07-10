//! # proof-core
//!
//! Proof-of-Backtest: fold a `(spec, data)` pair into a deterministic
//! `wickra-backtest` report and a canonical blake3 hash that anyone recomputes
//! byte-for-byte in ten languages. Verification recomputes the report and
//! compares hashes, so a forged report cannot pass.
//!
//! [`canonicalize`] is the single source of the hash's stability; [`prove`] and
//! [`verify`] are the core operations; [`Prover`] exposes the same
//! `command_json` boundary the ten language bindings forward verbatim.

mod canonical;
mod config;
mod error;
mod proof;
mod spec;

pub use canonical::canonicalize;
pub use config::Config;
pub use error::{Error, Result};
pub use proof::{prove, verify, Proof, Prover};
pub use spec::ProofSpec;

/// The proof-core crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests;
