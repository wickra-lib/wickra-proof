"use strict";

// Tests over the wasm-pack (nodejs target) output, run by the WASM job after
// `wasm-pack build --target nodejs --out-dir pkg-node`:
//
//   golden   every golden/specs/*.json proves, through the WebAssembly core
//            over the shared golden/data.json, to the byte-identical
//            golden/expected/<spec>.json the native bindings produce -- the
//            same blake3 report/inputs hashes in every language;
//   operating modes
//            the proof does not depend on how its inputs arrive: the
//            committed bytes spliced in verbatim, or what the host's JSON
//            library re-emits (keys in another order, whitespace, floats
//            re-formatted) prove alike, and a proof the host re-emitted
//            still verifies;
//   smoke    the blessed proof verifies, a tampered one does not, the version
//            matches the module export, an unknown command is an in-band
//            error.
//
// The require is hard: a missing build must fail the job, not skip it.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Prover, version } = require("../pkg-node/wickra_proof_wasm.js");

const GOLDEN = path.resolve(__dirname, "..", "..", "..", "golden");
const SPECS = path.join(GOLDEN, "specs");
const EXPECTED = path.join(GOLDEN, "expected");

const dataText = fs.readFileSync(path.join(GOLDEN, "data.json"), "utf8");
const data = JSON.parse(dataText);
const files = fs
  .readdirSync(SPECS)
  .filter((f) => f.endsWith(".json"))
  .sort();

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

test("golden fixtures are present", () => {
  assert.ok(files.length > 0, "golden/specs holds at least one spec");
});

for (const name of files) {
  const specText = fs.readFileSync(path.join(SPECS, name), "utf8");
  const spec = JSON.parse(specText);
  const expected = fs.readFileSync(path.join(EXPECTED, name), "utf8").trim();

  test(`golden ${name} matches expected`, () => {
    const got = new Prover().command(JSON.stringify({ cmd: "prove", spec, data }));
    assert.strictEqual(got.trim(), expected);
  });

  test(`${name}: committed and host-serialised inputs prove alike`, () => {
    const prover = new Prover();
    // Mode 1: the committed bytes, spliced into the envelope verbatim.
    const committed = prover.command(`{"cmd":"prove","spec":${specText},"data":${dataText}}`);
    // Mode 2: what the host re-emits -- reversed key order, pretty-printed.
    const hosted = prover.command(
      JSON.stringify({ cmd: "prove", spec: reordered(spec), data: reordered(data) }, null, 2),
    );
    assert.strictEqual(committed.trim(), expected, "committed bytes");
    assert.strictEqual(hosted.trim(), expected, "host-serialised bytes");

    // A proof the host re-emitted still verifies.
    const verdict = JSON.parse(
      prover.command(JSON.stringify({ cmd: "verify", proof: reordered(JSON.parse(expected)), spec, data })),
    );
    assert.deepStrictEqual(verdict, { ok: true, valid: true });
  });

  test(`${name}: the blessed proof verifies, a tampered one does not`, () => {
    const prover = new Prover();
    const proof = JSON.parse(expected);
    const good = JSON.parse(prover.command(JSON.stringify({ cmd: "verify", proof, spec, data })));
    assert.deepStrictEqual(good, { ok: true, valid: true });

    const tampered = { ...proof, report_hash: "0".repeat(64) };
    const bad = JSON.parse(prover.command(JSON.stringify({ cmd: "verify", proof: tampered, spec, data })));
    assert.deepStrictEqual(bad, { ok: true, valid: false });
  });
}

test("the version matches the module export", () => {
  assert.strictEqual(new Prover().version(), version());
});

test("an unknown command is an in-band error", () => {
  const response = JSON.parse(new Prover().command('{"cmd":"nope"}'));
  assert.strictEqual(response.ok, false);
  assert.match(response.error, /nope/);
});
