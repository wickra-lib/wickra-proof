// Golden bless generator (developer tool, not shipped): builds the deterministic
// universe, writes the CSVs + data.json, and freezes each spec's proof into
// golden/expected/. Run from the repo root after `cd bindings/node && npm run build`:
//   node golden/_bless.mjs
// Never edit golden/expected/*.json by hand — re-bless instead.

import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { createRequire } from "node:module";

const here = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const { Prover } = require(join(here, "..", "bindings", "node", "index.js"));

const BARS = 40;
const T0 = 1_700_000_000;

// Deterministic close formulas (all values are integers or multiples of 0.25,
// exact in f64). Each shape triggers its strategy's distinct execution:
//   AAA: V-shape (down to bar 10, then up)  -> sma_cross golden-crosses and holds
//   BBB: 120 - 0.25*i  slow downtrend       -> buy_hold enters at warmup and holds
//   CCC: deep dip to bar 20, then recovery  -> rsi_reversion buys the oversold dip,
//                                              sells into the >50 recovery
const CLOSE = {
  AAA: (i) => (i <= 10 ? 120 - 2 * i : 100 + 2 * (i - 10)),
  BBB: (i) => 120 - 0.25 * i,
  CCC: (i) => (i <= 20 ? 110 - 2 * i : 70 + 3 * (i - 20)),
};

function series(close) {
  const out = [];
  for (let i = 0; i < BARS; i++) {
    const c = close(i);
    const o = i === 0 ? c : close(i - 1);
    out.push({
      time: T0 + i * 3600,
      open: o,
      high: Math.max(o, c) + 1,
      low: Math.min(o, c) - 1,
      close: c,
      volume: 1000,
    });
  }
  return out;
}

const data = { AAA: series(CLOSE.AAA), BBB: series(CLOSE.BBB), CCC: series(CLOSE.CCC) };

// CSVs (header + rows). String(n) is the shortest exact decimal for these values.
for (const [sym, bars] of Object.entries(data)) {
  const rows = bars.map(
    (b) => `${b.time},${b.open},${b.high},${b.low},${b.close},${b.volume}`,
  );
  writeFileSync(
    join(here, "data", `${sym}.csv`),
    "ts,open,high,low,close,volume\n" + rows.join("\n") + "\n",
  );
}

// data.json (the same universe, for the cross-language binding golden tests).
writeFileSync(join(here, "data.json"), JSON.stringify(data) + "\n");

// Bless each spec's proof.
const prover = new Prover();
for (const name of ["sma_cross", "rsi_reversion", "buy_hold"]) {
  const spec = JSON.parse(readFileSync(join(here, "specs", `${name}.json`), "utf8"));
  const raw = prover.command(JSON.stringify({ cmd: "prove", spec, data }));
  const parsed = JSON.parse(raw);
  if (parsed.ok === false) {
    throw new Error(`${name}: prove failed: ${raw}`);
  }
  writeFileSync(join(here, "expected", `${name}.json`), raw + "\n");
  const trades = Array.isArray(parsed.report?.trades) ? parsed.report.trades.length : "?";
  console.log(`${name}: report_hash=${parsed.report_hash.slice(0, 12)}… trades=${trades}`);
}
console.log("blessed", Object.keys(data).length, "symbols,", 3, "specs");
