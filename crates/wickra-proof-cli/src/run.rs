//! Load inputs, run the requested command, and render the output.

use crate::args::{Cli, Command, Format};
use proof_core::{canonicalize, prove, verify, Candle, Config, Proof, Prover};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// The rendered output plus whether the process should exit successfully.
pub struct Output {
    /// Text written to standard output.
    pub text: String,
    /// Whether the command succeeded (drives the process exit code).
    pub success: bool,
}

/// Dispatch the parsed CLI.
pub fn run(cli: &Cli) -> Result<Output, String> {
    match &cli.command {
        Command::Prove { spec, data, format } => {
            let spec = load_spec(spec)?;
            let data = load_data(data)?;
            let proof = prove(&spec, &data).map_err(|e| e.to_string())?;
            let text = match format {
                Format::Json => {
                    let mut json = serde_json::to_string(&proof).map_err(|e| e.to_string())?;
                    json.push('\n');
                    json
                }
                Format::Text => summarize(&proof),
            };
            Ok(Output {
                text,
                success: true,
            })
        }
        Command::Verify { proof, spec, data } => {
            let claim: Proof = serde_json::from_str(&read(proof)?)
                .map_err(|e| format!("parse proof {}: {e}", proof.display()))?;
            let spec = load_spec(spec)?;
            let data = load_data(data)?;
            let valid = verify(&claim, &spec, &data).map_err(|e| e.to_string())?;
            Ok(Output {
                text: if valid { "valid\n" } else { "INVALID\n" }.to_string(),
                success: valid,
            })
        }
        Command::Canonicalize { file } => {
            let value: Value = serde_json::from_str(&read(file)?)
                .map_err(|e| format!("parse {}: {e}", file.display()))?;
            let mut out = canonicalize(&value).map_err(|e| e.to_string())?;
            out.push('\n');
            Ok(Output {
                text: out,
                success: true,
            })
        }
        Command::Version => {
            let mut prover = Prover::new();
            let mut out = prover
                .command_json("{\"cmd\":\"version\"}")
                .map_err(|e| e.to_string())?;
            out.push('\n');
            Ok(Output {
                text: out,
                success: true,
            })
        }
    }
}

/// A compact human-readable summary of a proof.
fn summarize(proof: &Proof) -> String {
    let trades = proof
        .report
        .get("trades")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    format!(
        "report_hash: {}\ninputs_hash: {}\nengine_version: {}\ntrades: {trades}\n",
        proof.report_hash, proof.inputs_hash, proof.engine_version
    )
}

/// Read a file to a string with a contextual error.
fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))
}

/// Read and parse a spec file, choosing JSON or TOML by extension.
fn load_spec(path: &Path) -> Result<proof_core::ProofSpec, String> {
    let content = read(path)?;
    let is_toml = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("toml"));
    let cfg = if is_toml {
        Config::from_toml(&content)
    } else {
        Config::from_json(&content)
    };
    cfg.map(|c| c.spec).map_err(|e| e.to_string())
}

/// Load candles from a single `<SYMBOL>.csv` file or a directory of them.
fn load_data(path: &Path) -> Result<BTreeMap<String, Vec<Candle>>, String> {
    let mut data = BTreeMap::new();
    if path.is_dir() {
        let entries =
            fs::read_dir(path).map_err(|e| format!("read dir {}: {e}", path.display()))?;
        for entry in entries {
            let file = entry.map_err(|e| e.to_string())?.path();
            if file.extension().and_then(|e| e.to_str()) != Some("csv") {
                continue;
            }
            data.insert(symbol_of(&file)?, parse_csv(&read(&file)?)?);
        }
    } else {
        data.insert(symbol_of(path)?, parse_csv(&read(path)?)?);
    }
    Ok(data)
}

/// The symbol name is the file stem (`AAA.csv` -> `AAA`).
fn symbol_of(path: &Path) -> Result<String, String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(ToString::to_string)
        .ok_or_else(|| format!("bad file name: {}", path.display()))
}

/// Parse OHLCV rows (`ts,open,high,low,close,volume`) into candles; a
/// non-numeric first row is treated as a header and skipped.
fn parse_csv(content: &str) -> Result<Vec<Candle>, String> {
    let mut candles = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(str::trim).collect();
        if cols.len() < 6 {
            return Err(format!(
                "CSV line {}: expected 6 columns, got {}",
                idx + 1,
                cols.len()
            ));
        }
        let time = match cols[0].parse::<i64>() {
            Ok(t) => t,
            Err(_) if idx == 0 => continue, // header row
            Err(e) => return Err(format!("CSV line {}: bad timestamp: {e}", idx + 1)),
        };
        let field = |i: usize, name: &str| {
            cols[i]
                .parse::<f64>()
                .map_err(|e| format!("CSV line {}: {name}: {e}", idx + 1))
        };
        candles.push(Candle {
            time,
            open: field(1, "open")?,
            high: field(2, "high")?,
            low: field(3, "low")?,
            close: field(4, "close")?,
            volume: field(5, "volume")?,
        });
    }
    Ok(candles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_csv_with_a_header() {
        let csv = "ts,open,high,low,close,volume\n1,10,11,9,10.5,100\n2,10.5,12,10,11,200\n";
        let candles = parse_csv(csv).unwrap();
        assert_eq!(candles.len(), 2);
        assert_eq!(candles[0].time, 1);
        assert!((candles[1].close - 11.0).abs() < 1e-9);
    }

    #[test]
    fn parse_csv_rejects_a_short_row() {
        assert!(parse_csv("1,2,3\n").is_err());
    }

    #[test]
    fn symbol_is_the_file_stem() {
        assert_eq!(symbol_of(Path::new("AAA.csv")).unwrap(), "AAA");
    }
}
