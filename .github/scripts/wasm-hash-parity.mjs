// WASM hash-parity check (run in CI after `wasm-pack build --target nodejs`).
//
// Prove every committed golden/specs/*.json through the WebAssembly core over
// the shared golden/data.json and assert the response is byte-identical to
// golden/expected/<spec>.json — the same blake3 report/inputs hashes the native
// bindings produce. The wasm Prover returns the core's canonical command_json
// string verbatim, so byte equality is the exact cross-language parity check.
// The blessed proof also verifies through wasm, and a tampered one does not.

import { createRequire } from "node:module";
import assert from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "..", "..");

// wasm-pack --target nodejs emits a CommonJS module.
const { Prover } = require(
  path.join(repo, "bindings", "wasm", "pkg-node", "wickra_proof_wasm.js"),
);

const G = path.join(repo, "golden");
const data = JSON.parse(fs.readFileSync(path.join(G, "data.json"), "utf8"));
const specNames = fs
  .readdirSync(path.join(G, "specs"))
  .filter((n) => n.endsWith(".json"));
assert.ok(specNames.length > 0, "expected at least one golden spec");

const prover = new Prover();
for (const name of specNames) {
  const spec = JSON.parse(fs.readFileSync(path.join(G, "specs", name), "utf8"));
  const got = prover.command(JSON.stringify({ cmd: "prove", spec, data }));
  const expected = fs
    .readFileSync(path.join(G, "expected", name), "utf8")
    .trim();
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

console.log(`wasm hash parity: ${specNames.length} golden proofs match`);
