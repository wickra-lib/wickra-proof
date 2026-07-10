"""Cross-language golden parity: for each committed golden/specs/*.json, prove
over the shared golden/data.json and assert the response equals
golden/expected/<spec>.json byte-for-byte. The binding returns the core's
canonical command_json string verbatim, so byte equality is the exact
cross-language parity check — the same blake3 report/inputs hashes in every
language. The blessed proof also verifies, and a tampered one does not."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from wickra_proof import Prover


def _golden_dir() -> Path | None:
    """Walk up from this test file to the repo root that holds golden/specs."""
    for parent in Path(__file__).resolve().parents:
        g = parent / "golden"
        if (g / "specs").is_dir():
            return g
    return None


GOLDEN = _golden_dir()


@pytest.mark.skipif(GOLDEN is None, reason="golden fixtures not present")
def test_golden_proofs_are_byte_identical() -> None:
    data = json.loads((GOLDEN / "data.json").read_text())
    spec_paths = sorted((GOLDEN / "specs").glob("*.json"))
    assert spec_paths, "expected at least one golden spec"

    prover = Prover()
    for spec_path in spec_paths:
        name = spec_path.name
        spec = json.loads(spec_path.read_text())
        got = prover.command(json.dumps({"cmd": "prove", "spec": spec, "data": data}))
        expected = (GOLDEN / "expected" / name).read_text().strip()
        assert got.strip() == expected, f"golden mismatch for {name}"

        # The blessed proof verifies against its inputs; a tampered one does not.
        proof = json.loads(expected)
        good = json.loads(
            prover.command(
                json.dumps({"cmd": "verify", "proof": proof, "spec": spec, "data": data})
            )
        )
        assert good == {"ok": True, "valid": True}, f"verify(blessed) {name}"

        tampered = dict(proof)
        tampered["report_hash"] = "0" * 64
        bad = json.loads(
            prover.command(
                json.dumps(
                    {"cmd": "verify", "proof": tampered, "spec": spec, "data": data}
                )
            )
        )
        assert bad == {"ok": True, "valid": False}, f"verify(tampered) {name}"
