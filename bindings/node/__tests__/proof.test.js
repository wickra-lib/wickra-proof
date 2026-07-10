"use strict";

// Smoke + golden: a stateless prover proves a report to a stable hash, verifies
// a genuine proof, and rejects a tampered one.

const { test } = require("node:test");
const assert = require("node:assert");
const { Prover, version } = require("../index.js");

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

const SPEC = { strategy: STRATEGY, dataset_ref: "BTCUSDT/1h/test" };
const DATA = { BTCUSDT: candles() };

function prove(prover) {
  return JSON.parse(prover.command(JSON.stringify({ cmd: "prove", spec: SPEC, data: DATA })));
}

test("prove yields 64-hex hashes and the pinned engine version", () => {
  const proof = prove(new Prover());
  assert.strictEqual(proof.report_hash.length, 64);
  assert.strictEqual(proof.inputs_hash.length, 64);
  const v = JSON.parse(new Prover().command('{"cmd":"version"}'));
  assert.strictEqual(proof.engine_version, v.engine_version);
});

test("version matches the module export", () => {
  assert.strictEqual(new Prover().version(), version());
});

test("prove is reproducible", () => {
  assert.strictEqual(prove(new Prover()).report_hash, prove(new Prover()).report_hash);
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

test("unknown command is an in-band error", () => {
  const response = JSON.parse(new Prover().command('{"cmd":"nope"}'));
  assert.strictEqual(response.ok, false);
  assert.match(response.error, /nope/);
});
