// A runnable Node.js example: prove a (spec, data) pair through the binding,
// print the report hash, then verify the proof and assert it holds.
//
//   ( cd bindings/node && npm install && npm run build )
//   ( cd examples/node && npm install && node prove.js )

"use strict";

const assert = require("node:assert");
const { Prover, version } = require("wickra-proof");

const SPEC = {
  strategy: {
    symbol: "AAA",
    timeframe: "1h",
    indicators: {
      ema_fast: { type: "Ema", params: [3] },
      ema_slow: { type: "Ema", params: [8] },
    },
    entry: { cross_above: ["ema_fast", "ema_slow"] },
    exit: { cross_below: ["ema_fast", "ema_slow"] },
    sizing: { type: "fixed_fraction", fraction: 0.95 },
    costs: { taker_bps: 5, slippage: { type: "fixed_bps", bps: 2 } },
    risk: {},
  },
  dataset_ref: "example/AAA/1h",
};

// A short V-shaped price path so the fast/slow EMA cross fires at least once.
const CLOSES = [120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128];

const candles = () =>
  CLOSES.map((close, i) => {
    const open = i === 0 ? close : CLOSES[i - 1];
    return {
      time: 1_700_000_000 + i * 3600,
      open,
      high: Math.max(open, close) + 1,
      low: Math.min(open, close) - 1,
      close,
      volume: 1000,
    };
  });

const prover = new Prover();
const data = { AAA: candles() };

const proof = JSON.parse(
  prover.command(JSON.stringify({ cmd: "prove", spec: SPEC, data })),
);
console.log("wickra-proof", version());
console.log(`report_hash: ${proof.report_hash}`);

const verdict = JSON.parse(
  prover.command(JSON.stringify({ cmd: "verify", proof, spec: SPEC, data })),
);
assert.deepStrictEqual(verdict, { ok: true, valid: true }, "proof must verify");
console.log("verify: valid");
