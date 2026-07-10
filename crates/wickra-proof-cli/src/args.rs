//! CLI argument parsing.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Prove and verify deterministic backtest reports.
#[derive(Parser, Debug)]
#[command(name = "wickra-proof", version, about)]
pub struct Cli {
    /// The subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The `wickra-proof` subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Fold a `(spec, data)` pair into a proof (report + canonical hashes).
    Prove {
        /// Path to the proof spec (JSON or TOML, chosen by extension).
        #[arg(long)]
        spec: PathBuf,
        /// A CSV candle file (`<SYMBOL>.csv`) or a directory of them.
        #[arg(long)]
        data: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Json)]
        format: Format,
    },
    /// Recompute a proof from `(spec, data)` and compare it against a claim.
    /// Exits 0 when the claim is valid, 1 when it is not.
    Verify {
        /// Path to the claimed proof JSON.
        #[arg(long)]
        proof: PathBuf,
        /// Path to the proof spec (JSON or TOML).
        #[arg(long)]
        spec: PathBuf,
        /// A CSV candle file or a directory of them.
        #[arg(long)]
        data: PathBuf,
    },
    /// Print the canonical form of a JSON file to standard output.
    Canonicalize {
        /// Path to the JSON file to canonicalize.
        #[arg(long)]
        file: PathBuf,
    },
    /// Print the proof-core and pinned engine versions.
    Version,
}

/// The `prove` output format.
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum Format {
    /// The full `Proof` as JSON.
    Json,
    /// A compact human-readable summary.
    Text,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn arg_config_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn prove_parses_with_defaults() {
        let cli = Cli::try_parse_from([
            "wickra-proof",
            "prove",
            "--spec",
            "s.json",
            "--data",
            "d.csv",
        ])
        .unwrap();
        match cli.command {
            Command::Prove { format, .. } => assert_eq!(format, Format::Json),
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn verify_requires_all_three_paths() {
        assert!(Cli::try_parse_from(["wickra-proof", "verify", "--proof", "p.json"]).is_err());
    }

    #[test]
    fn version_subcommand_parses() {
        let cli = Cli::try_parse_from(["wickra-proof", "version"]).unwrap();
        assert!(matches!(cli.command, Command::Version));
    }
}
