"""Smoke test: construct a prover, prove a report, parse the proof."""

import json
import math

from wickra_proof import Prover, __version__

STRATEGY = {
    "symbol": "BTCUSDT",
    "timeframe": "1h",
    "indicators": {
        "ema_fast": {"type": "Ema", "params": [5]},
        "ema_slow": {"type": "Ema", "params": [15]},
    },
    "entry": {"cross_above": ["ema_fast", "ema_slow"]},
    "exit": {"cross_below": ["ema_fast", "ema_slow"]},
    "sizing": {"type": "fixed_fraction", "fraction": 0.95},
    "costs": {"taker_bps": 5, "slippage": {"type": "fixed_bps", "bps": 2}},
    "risk": {"trailing_stop_pct": 5.0},
}


def _candles() -> list[dict]:
    out = []
    for i in range(40):
        base = 100.0 + math.sin(i * 0.4) * 8.0
        out.append(
            {
                "time": 1_700_000_000 + i * 3600,
                "open": base,
                "high": base + 1.0,
                "low": base - 1.0,
                "close": base + 0.5,
                "volume": 1000.0,
            }
        )
    return out


def _prove_request() -> str:
    return json.dumps(
        {
            "cmd": "prove",
            "spec": {"strategy": STRATEGY, "dataset_ref": "BTCUSDT/1h/test"},
            "data": {"BTCUSDT": _candles()},
        }
    )


def test_prove_roundtrip() -> None:
    prover = Prover()
    proof = json.loads(prover.command(_prove_request()))
    assert len(proof["report_hash"]) == 64
    assert len(proof["inputs_hash"]) == 64
    assert proof["engine_version"] == json.loads(prover.command('{"cmd":"version"}'))[
        "engine_version"
    ]


def test_version_matches_module() -> None:
    assert Prover.version() == __version__


def test_unknown_command_is_in_band_error() -> None:
    prover = Prover()
    response = json.loads(prover.command('{"cmd":"nope"}'))
    assert response["ok"] is False
    assert "nope" in response["error"]
