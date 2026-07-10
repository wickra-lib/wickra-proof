//! Golden parity: load the committed `golden/{specs,data}`, prove through the
//! same canonical command surface every binding uses, and assert the response is
//! byte-for-byte identical to `golden/expected/*.json`. This is the Rust anchor
//! of the cross-language determinism guarantee; the eight bindings assert the
//! same bytes.

use proof_core::{verify, Candle, Proof, Prover};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../golden")
}

/// Parse an OHLCV CSV (`ts,open,high,low,close,volume`, header row skipped).
fn parse_csv(content: &str) -> Vec<Candle> {
    content
        .lines()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split(',').map(str::trim).collect();
            let time = cols.first()?.parse::<i64>().ok()?; // header row fails and is skipped
            Some(Candle {
                time,
                open: cols[1].parse().unwrap(),
                high: cols[2].parse().unwrap(),
                low: cols[3].parse().unwrap(),
                close: cols[4].parse().unwrap(),
                volume: cols[5].parse().unwrap(),
            })
        })
        .collect()
}

/// Load every `golden/data/<SYMBOL>.csv` into a symbol-keyed map.
fn load_data(dir: &Path) -> BTreeMap<String, Vec<Candle>> {
    let mut data = BTreeMap::new();
    for entry in fs::read_dir(dir.join("data")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("csv") {
            let symbol = path.file_stem().unwrap().to_string_lossy().into_owned();
            data.insert(symbol, parse_csv(&fs::read_to_string(&path).unwrap()));
        }
    }
    data
}

#[test]
fn golden_proofs_are_byte_identical() {
    let dir = golden_dir();
    let data = load_data(&dir);
    let data_value: Value = serde_json::to_value(&data).unwrap();

    let mut prover = Prover::new();
    let mut count = 0;
    for entry in fs::read_dir(dir.join("specs")).unwrap() {
        let spec_path = entry.unwrap().path();
        if spec_path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = spec_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let spec_value: Value =
            serde_json::from_str(&fs::read_to_string(&spec_path).unwrap()).unwrap();

        // Prove through the exact canonical command surface the bindings use.
        let cmd = json!({ "cmd": "prove", "spec": spec_value, "data": data_value }).to_string();
        let got = prover.command_json(&cmd).unwrap();

        let expected = fs::read_to_string(dir.join("expected").join(&name)).unwrap();
        assert_eq!(
            got.trim(),
            expected.trim(),
            "golden proof mismatch for {name}"
        );

        // The blessed proof also verifies against the same (spec, data).
        let proof: Proof = serde_json::from_str(&expected).unwrap();
        let spec: proof_core::ProofSpec = serde_json::from_value(
            serde_json::from_str::<Value>(&fs::read_to_string(&spec_path).unwrap()).unwrap(),
        )
        .unwrap();
        assert!(
            verify(&proof, &spec, &data).unwrap(),
            "blessed proof for {name} fails verification"
        );
        count += 1;
    }
    assert_eq!(count, 3, "expected exactly three golden specs");
}
