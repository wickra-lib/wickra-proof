//! [`ProofSpec`] — the job to be proven.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The input to `prove`: an embedded backtest strategy, an opaque dataset
/// reference, and an optional engine-version pin.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProofSpec {
    /// The embedded wickra-backtest `StrategySpec`, kept as raw JSON so
    /// proof-core stays decoupled from backtest struct internals across the FFI
    /// boundary.
    pub strategy: Value,
    /// Opaque, caller-chosen identifier of the dataset (e.g. content hash, URL,
    /// git ref). It is hashed into `inputs_hash` but proof-core does NOT fetch
    /// it — data is passed explicitly.
    pub dataset_ref: String,
    /// Expected backtest engine version. If present and it differs from the
    /// linked `wickra-backtest` `version()`, `prove` returns
    /// [`Error::EngineMismatch`] (no silent drift).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_version: Option<String>,
}

impl ProofSpec {
    /// Parse a `ProofSpec` from JSON.
    pub fn from_json(s: &str) -> Result<Self> {
        let spec: Self = serde_json::from_str(s)?;
        spec.validate()?;
        Ok(spec)
    }

    /// Parse a `ProofSpec` from TOML.
    pub fn from_toml(s: &str) -> Result<Self> {
        let spec: Self = toml::from_str(s).map_err(|e| Error::Parse(e.to_string()))?;
        spec.validate()?;
        Ok(spec)
    }

    /// Validate structural invariants: the embedded strategy must be a JSON
    /// object (a `StrategySpec`), not a scalar or array.
    pub(crate) fn validate(&self) -> Result<()> {
        if !self.strategy.is_object() {
            return Err(Error::BadSpec(
                "strategy must be a JSON object (a StrategySpec)".to_string(),
            ));
        }
        Ok(())
    }
}
