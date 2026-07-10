"use strict";

// Cross-language golden parity: for each committed golden/specs/*.json, prove
// over the shared golden/data.json and assert the response equals
// golden/expected/<spec>.json byte-for-byte. The binding returns the core's
// canonical command_json string verbatim, so byte equality is the exact
// cross-language parity check — the same blake3 report/inputs hashes in every
// language. The blessed proof also verifies, and a tampered one does not.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Prover } = require("../index.js");

// Walk up from this test file to the repo root that holds golden/specs.
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

const G = goldenDir();

test(
  "golden proofs are byte-identical across languages",
  { skip: G ? false : "golden fixtures not present" },
  () => {
    const data = JSON.parse(fs.readFileSync(path.join(G, "data.json"), "utf8"));
    const specNames = fs
      .readdirSync(path.join(G, "specs"))
      .filter((n) => n.endsWith(".json"));
    assert.ok(specNames.length > 0, "expected at least one golden spec");

    const prover = new Prover();
    for (const name of specNames) {
      const spec = JSON.parse(fs.readFileSync(path.join(G, "specs", name), "utf8"));
      const got = prover.command(JSON.stringify({ cmd: "prove", spec, data }));
      const expected = fs.readFileSync(path.join(G, "expected", name), "utf8").trim();
      assert.strictEqual(got.trim(), expected, `golden mismatch for ${name}`);

      // The blessed proof verifies against its inputs; a tampered one does not.
      const proof = JSON.parse(expected);
      const good = JSON.parse(
        prover.command(JSON.stringify({ cmd: "verify", proof, spec, data })),
      );
      assert.deepStrictEqual(good, { ok: true, valid: true }, `verify(blessed) ${name}`);

      const tampered = { ...proof, report_hash: "0".repeat(64) };
      const bad = JSON.parse(
        prover.command(JSON.stringify({ cmd: "verify", proof: tampered, spec, data })),
      );
      assert.deepStrictEqual(bad, { ok: true, valid: false }, `verify(tampered) ${name}`);
    }
  },
);
