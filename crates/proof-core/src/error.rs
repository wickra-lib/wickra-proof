//! Error type for `proof-core`.

/// Errors returned by proof-core operations. Every fallible boundary (parsing,
/// engine invocation, canonicalization) maps into one of these; no operation
/// panics on caller-supplied input.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A JSON or TOML value failed to parse.
    #[error("parse: {0}")]
    Parse(String),
    /// The embedded strategy JSON is not a valid `StrategySpec`.
    #[error("bad spec: {0}")]
    BadSpec(String),
    /// The pinned `engine_version` differs from the linked backtest engine.
    #[error("engine mismatch: expected {expected}, linked {linked}")]
    EngineMismatch {
        /// The version pinned in the `ProofSpec`.
        expected: String,
        /// The version of the linked `wickra-backtest` engine.
        linked: String,
    },
    /// The backtest engine returned an error.
    #[error("backtest: {0}")]
    Backtest(String),
    /// The data map lacked the candles required by the strategy.
    #[error("data: {0}")]
    Data(String),
}

/// Result alias for proof-core.
pub type Result<T> = core::result::Result<T, Error>;

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Parse(e.to_string())
    }
}
