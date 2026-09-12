"""Operating-mode equivalence: the proof does not depend on how its inputs arrive.

A binding hands JSON text to the core. It can pass the committed golden bytes
through untouched, or it can hand over what the host language's JSON library
re-emits -- keys in another order, whitespace, floats re-formatted. The core
canonicalizes before hashing, so both operating modes must produce the same
proof, byte for byte, and a proof re-emitted by the host must still verify.
This is where a binding that mangles a float (``1.0`` -> ``1``,
``120.25`` -> ``120.25000000000001``) is caught.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from wickra_proof import Prover


def _golden_dir() -> Path | None:
    for parent in Path(__file__).resolve().parents:
        g = parent / "golden"
        if (g / "specs").is_dir():
            return g
    return None


GOLDEN = _golden_dir()


def _reordered(value):
    """The same value with every object's keys in reverse order."""
    if isinstance(value, dict):
        return {k: _reordered(value[k]) for k in sorted(value, reverse=True)}
    if isinstance(value, list):
        return [_reordered(v) for v in value]
    return value


@pytest.mark.skipif(GOLDEN is None, reason="golden fixtures not present")
def test_committed_and_host_serialised_inputs_prove_alike() -> None:
    data_text = (GOLDEN / "data.json").read_text()
    data = json.loads(data_text)
    prover = Prover()
    for spec_path in sorted((GOLDEN / "specs").glob("*.json")):
        spec_text = spec_path.read_text()
        expected = (GOLDEN / "expected" / spec_path.name).read_text().strip()

        # Mode 1: the committed bytes, spliced into the envelope verbatim.
        committed = prover.command(
            '{"cmd":"prove","spec":' + spec_text + ',"data":' + data_text + "}"
        )
        # Mode 2: what the host re-emits -- reversed key order, pretty-printed.
        hosted = prover.command(
            json.dumps(
                {"cmd": "prove", "spec": _reordered(json.loads(spec_text)), "data": _reordered(data)},
                indent=2,
            )
        )
        assert committed.strip() == expected, f"committed bytes: {spec_path.name}"
        assert hosted.strip() == expected, f"host-serialised bytes: {spec_path.name}"

        # A proof the host re-emitted still verifies.
        proof = _reordered(json.loads(expected))
        verdict = json.loads(
            prover.command(
                json.dumps({"cmd": "verify", "proof": proof, "spec": json.loads(spec_text), "data": data})
            )
        )
        assert verdict == {"ok": True, "valid": True}, spec_path.name
