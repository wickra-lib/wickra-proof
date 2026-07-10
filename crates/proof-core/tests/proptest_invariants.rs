//! Property tests: over a wide range of bounded, valid candle universes the
//! prove/verify contract never panics, always round-trips, and is deterministic.

mod common;

use proof_core::{prove, verify};
use proptest::prelude::*;
use std::collections::BTreeMap;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// A genuine proof over any bounded universe verifies and reproves identically.
    #[test]
    fn prove_round_trips_and_is_deterministic(
        closes in prop::collection::vec(50.0f64..200.0f64, 16..64)
    ) {
        let spec = common::sample_spec();
        let mut data = BTreeMap::new();
        data.insert(common::SYMBOL.to_string(), common::candles_from(&closes));

        let a = prove(&spec, &data).unwrap();
        let b = prove(&spec, &data).unwrap();

        // Determinism: byte-identical hashes on repeat.
        prop_assert_eq!(&a.report_hash, &b.report_hash);
        prop_assert_eq!(&a.inputs_hash, &b.inputs_hash);
        prop_assert_eq!(&a.engine_version, &b.engine_version);

        // The proof verifies against its own inputs.
        prop_assert!(verify(&a, &spec, &data).unwrap());

        // Hashes are 64-hex blake3 digests.
        prop_assert_eq!(a.report_hash.len(), 64);
        prop_assert_eq!(a.inputs_hash.len(), 64);
    }

    /// A proof never verifies against a different dataset.
    #[test]
    fn a_proof_does_not_verify_against_other_data(
        closes in prop::collection::vec(50.0f64..200.0f64, 16..48),
        bump in 1.0f64..20.0f64,
    ) {
        let spec = common::sample_spec();
        let mut data = BTreeMap::new();
        data.insert(common::SYMBOL.to_string(), common::candles_from(&closes));
        let proof = prove(&spec, &data).unwrap();

        let shifted: Vec<f64> = closes.iter().map(|c| c + bump).collect();
        let mut other = BTreeMap::new();
        other.insert(common::SYMBOL.to_string(), common::candles_from(&shifted));

        prop_assert!(!verify(&proof, &spec, &other).unwrap());
    }
}
