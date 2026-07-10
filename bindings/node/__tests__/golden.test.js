"use strict";

// Determinism is the product: a fixed (spec, data) proves to the same hash on
// every call, and a genuine proof verifies while a tampered one does not.
// Mirrors the Python binding's test_golden.py so every language pins the same
// contract.

const { test } = require("node:test");
const assert = require("node:assert");
const { Prover } = require("../index.js");

const STRATEGY = {
  symbol: "BTCUSDT",
  timeframe: "1h",
  indicators: {
    ema_fast: { type: "Ema", params: [5] },
    ema_slow: { type: "Ema", params: [15] },
  },
  entry: { cross_above: ["ema_fast", "ema_slow"] },
  exit: { cross_below: ["ema_fast", "ema_slow"] },
  sizing: { type: "fixed_fraction", fraction: 0.95 },
  costs: { taker_bps: 5, slippage: { type: "fixed_bps", bps: 2 } },
  risk: { trailing_stop_pct: 5.0 },
};

function candles() {
  const out = [];
  for (let i = 0; i < 40; i++) {
    const base = 100.0 + Math.sin(i * 0.4) * 8.0;
    out.push({
      time: 1_700_000_000 + i * 3600,
      open: base,
      high: base + 1.0,
      low: base - 1.0,
      close: base + 0.5,
      volume: 1000.0,
    });
  }
  return out;
}

const SPEC = { strategy: STRATEGY, dataset_ref: "BTCUSDT/1h/golden" };
const DATA = { BTCUSDT: candles() };

function prove(prover) {
  return JSON.parse(prover.command(JSON.stringify({ cmd: "prove", spec: SPEC, data: DATA })));
}

test("prove is reproducible", () => {
  const a = prove(new Prover());
  const b = prove(new Prover());
  assert.strictEqual(a.report_hash, b.report_hash);
  assert.strictEqual(a.inputs_hash, b.inputs_hash);
});

test("verify accepts a genuine proof and rejects a tampered one", () => {
  const prover = new Prover();
  const proof = prove(prover);

  const good = JSON.parse(
    prover.command(JSON.stringify({ cmd: "verify", proof, spec: SPEC, data: DATA })),
  );
  assert.deepStrictEqual(good, { ok: true, valid: true });

  const tampered = { ...proof, report_hash: "0".repeat(64) };
  const bad = JSON.parse(
    prover.command(JSON.stringify({ cmd: "verify", proof: tampered, spec: SPEC, data: DATA })),
  );
  assert.deepStrictEqual(bad, { ok: true, valid: false });
});
