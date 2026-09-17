"use strict";

// Parity guard: the Node binding must expose the full public surface of the
// prover, so an export dropped in a refactor fails loudly here (mirrors the
// completeness checks in the Python and R bindings).

const { test } = require("node:test");
const assert = require("node:assert");
const wickra = require("../index.js");

// The napi-rs loader adds its own diagnostics to the module object
// (`__napiBindingTarget` since @napi-rs/cli 3.10); they belong to the loader,
// not to the binding's surface, and are left out of the comparison.
const exported = () =>
  Object.keys(wickra)
    .filter((name) => !name.startsWith("__"))
    .sort();

test("module exposes Prover and version", () => {
  assert.strictEqual(typeof wickra.Prover, "function");
  assert.strictEqual(typeof wickra.version, "function");
});

test("Prover exposes command and version", () => {
  for (const name of ["command", "version"]) {
    assert.strictEqual(
      typeof wickra.Prover.prototype[name],
      "function",
      `Prover is missing ${name}`,
    );
  }
});

test("module surface is exactly {Prover, version}", () => {
  assert.deepStrictEqual(exported(), ["Prover", "version"]);
});

test("Prover surface is exactly {command, version}", () => {
  const methods = Object.getOwnPropertyNames(wickra.Prover.prototype)
    .filter((name) => name !== "constructor")
    .sort();
  assert.deepStrictEqual(methods, ["command", "version"]);
});
