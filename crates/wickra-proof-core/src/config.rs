//! [`Config`] — a `ProofSpec` loaded from a CLI file (JSON or TOML).

use crate::error::{Error, Result};
use crate::spec::ProofSpec;
use serde::{Deserialize, Serialize};

/// A thin wrapper around a [`ProofSpec`] for CLI file loading.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    /// The proof job.
    pub spec: ProofSpec,
}

impl Config {
    /// Load a `Config` from JSON.
    pub fn from_json(s: &str) -> Result<Self> {
        let cfg: Self = serde_json::from_str(s)?;
        cfg.spec.validate()?;
        Ok(cfg)
    }

    /// Load a `Config` from TOML.
    pub fn from_toml(s: &str) -> Result<Self> {
        let cfg: Self = toml::from_str(s).map_err(|e| Error::Parse(e.to_string()))?;
        cfg.spec.validate()?;
        Ok(cfg)
    }
}
