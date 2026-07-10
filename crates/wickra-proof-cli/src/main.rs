//! The `wickra-proof` reference CLI.
//!
//! Proves and verifies deterministic backtest reports over `proof-core`:
//! `prove` folds a `(spec, data)` pair into a report and canonical hashes,
//! `verify` recomputes a proof and compares it against a claim, and
//! `canonicalize` exposes the canonical JSON form directly.

mod args;
mod run;

use args::Cli;
use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run::run(&cli) {
        Ok(output) => {
            print!("{}", output.text);
            if output.success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(err) => {
            eprintln!("wickra-proof: {err}");
            ExitCode::FAILURE
        }
    }
}
