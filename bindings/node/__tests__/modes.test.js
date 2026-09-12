"use strict";

// Operating-mode equivalence: the proof does not depend on how its inputs
// arrive. A binding hands JSON text to the core. It can pass the committed
// golden bytes through untouched, or it can hand over what the host's JSON
// library re-emits -- keys in another order, whitespace, floats re-formatted.
// The core canonicalizes before hashing, so both operating modes must produce
// the same proof, byte for byte, and a proof re-emitted by the host must still
// verify. This is where a binding that mangles a float is caught.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Prover } = require("../index.js");

function goldenDir() {
  let dir = __dirname;
  for (let i = 0; i < 8; i++) {
    const g = path.join(dir, "golden");
    if (fs.existsSync(path.join(g, "specs"))) {
      return g;
    }
    dir = path.dirname(dir);
  }
  return null;
}

// The same value with every object's keys in reverse order.
function reordered(value) {
  if (Array.isArray(value)) return value.map(reordered);
  if (value && typeof value === "object") {
    const out = {};
    for (const k of Object.keys(value).sort().reverse()) out[k] = reordered(value[k]);
    return out;
  }
  return value;
}

const G = goldenDir();

test(
  "committed and host-serialised inputs prove alike",
  { skip: G ? false : "golden fixtures not present" },
  () => {
    const dataText = fs.readFileSync(path.join(G, "data.json"), "utf8");
    const data = JSON.parse(dataText);
    const prover = new Prover();
    for (const name of fs.readdirSync(path.join(G, "specs")).filter((n) => n.endsWith(".json"))) {
      const specText = fs.readFileSync(path.join(G, "specs", name), "utf8");
      const expected = fs.readFileSync(path.join(G, "expected", name), "utf8").trim();

      // Mode 1: the committed bytes, spliced into the envelope verbatim.
      const committed = prover.command(`{"cmd":"prove","spec":${specText},"data":${dataText}}`);
      // Mode 2: what the host re-emits -- reversed key order, pretty-printed.
      const hosted = prover.command(
        JSON.stringify({ cmd: "prove", spec: reordered(JSON.parse(specText)), data: reordered(data) }, null, 2),
      );
      assert.strictEqual(committed.trim(), expected, `committed bytes: ${name}`);
      assert.strictEqual(hosted.trim(), expected, `host-serialised bytes: ${name}`);

      // A proof the host re-emitted still verifies.
      const verdict = JSON.parse(
        prover.command(
          JSON.stringify({ cmd: "verify", proof: reordered(JSON.parse(expected)), spec: JSON.parse(specText), data }),
        ),
      );
      assert.deepStrictEqual(verdict, { ok: true, valid: true }, name);
    }
  },
);
