"""A runnable Python example: prove a (spec, data) pair through the binding,
print the report hash, then verify the proof and assert it holds.

    pip install wickra-proof
    python examples/python/prove.py
"""

import json

from wickra_proof import Prover

SPEC = {
    "strategy": {
        "symbol": "AAA",
        "timeframe": "1h",
        "indicators": {
            "ema_fast": {"type": "Ema", "params": [3]},
            "ema_slow": {"type": "Ema", "params": [8]},
        },
        "entry": {"cross_above": ["ema_fast", "ema_slow"]},
        "exit": {"cross_below": ["ema_fast", "ema_slow"]},
        "sizing": {"type": "fixed_fraction", "fraction": 0.95},
        "costs": {"taker_bps": 5, "slippage": {"type": "fixed_bps", "bps": 2}},
        "risk": {},
    },
    "dataset_ref": "example/AAA/1h",
}

# A short V-shaped price path so the fast/slow EMA cross fires at least once.
CLOSES = [120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128]


def _candles() -> list[dict]:
    out = []
    for i, close in enumerate(CLOSES):
        open_ = close if i == 0 else CLOSES[i - 1]
        out.append(
            {
                "time": 1_700_000_000 + i * 3600,
                "open": open_,
                "high": max(open_, close) + 1,
                "low": min(open_, close) - 1,
                "close": close,
                "volume": 1000,
            }
        )
    return out


def main() -> None:
    prover = Prover()
    data = {"AAA": _candles()}

    proof = json.loads(
        prover.command(json.dumps({"cmd": "prove", "spec": SPEC, "data": data}))
    )
    print(f"wickra-proof {Prover.version()}")
    print(f"report_hash: {proof['report_hash']}")

    verdict = json.loads(
        prover.command(
            json.dumps({"cmd": "verify", "proof": proof, "spec": SPEC, "data": data})
        )
    )
    assert verdict == {"ok": True, "valid": True}, "proof must verify"
    print("verify: valid")


if __name__ == "__main__":
    main()
