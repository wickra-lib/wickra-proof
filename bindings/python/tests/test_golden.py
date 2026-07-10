"""Determinism is the product: a fixed (spec, data) proves to the same hash on
every call, and a genuine proof verifies while a tampered one does not."""

import json

from wickra_proof import Prover

from test_smoke import _candles, STRATEGY

SPEC = {"strategy": STRATEGY, "dataset_ref": "BTCUSDT/1h/golden"}
DATA = {"BTCUSDT": _candles()}


def _prove(prover: Prover) -> dict:
    return json.loads(
        prover.command(json.dumps({"cmd": "prove", "spec": SPEC, "data": DATA}))
    )


def test_prove_is_reproducible() -> None:
    a = _prove(Prover())
    b = _prove(Prover())
    assert a["report_hash"] == b["report_hash"]
    assert a["inputs_hash"] == b["inputs_hash"]


def test_verify_accepts_genuine_and_rejects_tampered() -> None:
    prover = Prover()
    proof = _prove(prover)

    good = json.loads(
        prover.command(
            json.dumps({"cmd": "verify", "proof": proof, "spec": SPEC, "data": DATA})
        )
    )
    assert good == {"ok": True, "valid": True}

    tampered = dict(proof)
    tampered["report_hash"] = "0" * 64
    bad = json.loads(
        prover.command(
            json.dumps(
                {"cmd": "verify", "proof": tampered, "spec": SPEC, "data": DATA}
            )
        )
    )
    assert bad == {"ok": True, "valid": False}
