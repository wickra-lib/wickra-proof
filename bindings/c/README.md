<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Proof — a deterministic (spec, data) → blake3 hash, byte-identical across ten languages" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/ci.svg)](https://github.com/wickra-lib/wickra-proof/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-proof)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/release.svg)](https://github.com/wickra-lib/wickra-proof/releases/latest)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-proof/license.svg)](https://github.com/wickra-lib/wickra-proof#license)

# Wickra Proof — C / C++

---

**Proof-of-Backtest. Turn a `(spec, data)` pair into a deterministic backtest report *and* a canonical blake3 hash that anyone can recompute byte-for-byte in ten languages — for C / C++. `cargo build -p wickra-proof-c --release` — a prebuilt shared/static library plus a generated `wickra_proof.h`, no system dependencies.**

The C ABI is the hub every C-capable language (C, C++, C#, Go, Java, R) links
against. It exposes `wickra-proof-core` as a tiny, JSON-shaped surface built as both a
`cdylib` (dynamic library) and a `staticlib`.

## Install

Grab the prebuilt header + library for your platform from the
[GitHub releases](https://github.com/wickra-lib/wickra-proof/releases) — each archive
has `wickra_proof.h`, the C++ wrapper where the binding ships one, and the shared/static
library — or build from source:

```bash
cargo build -p wickra-proof-c --release
# -> target/release/libwickra_proof.{so,dylib} or wickra_proof.dll (+ import lib) + a staticlib
```

Then compile against the header and link the library.

## Quick start

[`examples/c/prove.c`](https://github.com/wickra-lib/wickra-proof/blob/main/examples/c/prove.c) is the runnable example the CI smoke job executes; in full:

```c
/* A minimal C example: prove a (spec, data) pair through the wickra-proof C ABI,
 * print the report hash, then verify the proof and assert it holds.
 *
 * The prove response is itself a JSON object, so the verify command embeds it
 * verbatim as the "proof" value — no JSON parser is needed on the C side. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_proof.h"

static const char *SPEC =
    "{\"strategy\":{\"symbol\":\"AAA\",\"timeframe\":\"1h\","
    "\"indicators\":{\"ema_fast\":{\"type\":\"Ema\",\"params\":[3]},"
    "\"ema_slow\":{\"type\":\"Ema\",\"params\":[8]}},"
    "\"entry\":{\"cross_above\":[\"ema_fast\",\"ema_slow\"]},"
    "\"exit\":{\"cross_below\":[\"ema_fast\",\"ema_slow\"]},"
    "\"sizing\":{\"type\":\"fixed_fraction\",\"fraction\":0.95},"
    "\"costs\":{\"taker_bps\":5,\"slippage\":{\"type\":\"fixed_bps\",\"bps\":2}},"
    "\"risk\":{}},\"dataset_ref\":\"example/AAA/1h\"}";

/* A short V-shaped price path so the fast/slow EMA cross fires at least once. */
static const char *DATA =
    "{\"AAA\":["
    "{\"time\":1700000000,\"open\":120,\"high\":121,\"low\":119,\"close\":120,\"volume\":1000},"
    "{\"time\":1700003600,\"open\":120,\"high\":121,\"low\":117,\"close\":118,\"volume\":1000},"
    "{\"time\":1700007200,\"open\":118,\"high\":119,\"low\":115,\"close\":116,\"volume\":1000},"
    "{\"time\":1700010800,\"open\":116,\"high\":117,\"low\":113,\"close\":114,\"volume\":1000},"
    "{\"time\":1700014400,\"open\":114,\"high\":115,\"low\":111,\"close\":112,\"volume\":1000},"
    "{\"time\":1700018000,\"open\":112,\"high\":113,\"low\":109,\"close\":110,\"volume\":1000},"
    "{\"time\":1700021600,\"open\":110,\"high\":111,\"low\":107,\"close\":108,\"volume\":1000},"
    "{\"time\":1700025200,\"open\":108,\"high\":113,\"low\":107,\"close\":112,\"volume\":1000},"
    "{\"time\":1700028800,\"open\":112,\"high\":117,\"low\":111,\"close\":116,\"volume\":1000},"
    "{\"time\":1700032400,\"open\":116,\"high\":121,\"low\":115,\"close\":120,\"volume\":1000},"
    "{\"time\":1700036000,\"open\":120,\"high\":125,\"low\":119,\"close\":124,\"volume\":1000},"
    "{\"time\":1700039600,\"open\":124,\"high\":129,\"low\":123,\"close\":128,\"volume\":1000}]}";

/* Read a command response into a freshly malloc'd, NUL-terminated buffer using
 * the length-out protocol. Returns NULL on failure. */
static char *run(WickraProof *prover, const char *cmd) {
    int len = wickra_proof_command(prover, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", len);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        return NULL;
    }
    wickra_proof_command(prover, cmd, buf, (size_t)len + 1);
    return buf;
}

int main(void) {
    WickraProof *prover = wickra_proof_new();
    if (!prover) {
        fprintf(stderr, "failed to create prover\n");
        return 1;
    }

    /* Build and run the prove command. */
    size_t prove_cap = strlen(SPEC) + strlen(DATA) + 64;
    char *prove_cmd = (char *)malloc(prove_cap);
    if (!prove_cmd) {
        wickra_proof_free(prover);
        return 1;
    }
    snprintf(prove_cmd, prove_cap, "{\"cmd\":\"prove\",\"spec\":%s,\"data\":%s}", SPEC, DATA);

    char *proof = run(prover, prove_cmd);
    if (!proof) {
        free(prove_cmd);
        wickra_proof_free(prover);
        return 1;
    }

    printf("wickra-proof %s\n", wickra_proof_version());
    const char *hash = strstr(proof, "\"report_hash\":");
    if (hash) {
        printf("proof: %.*s...\n", 30, hash);
    }

    /* Verify: the prove response is valid JSON, so it drops straight in as the
     * "proof" value. Assert the round-trip holds. */
    size_t verify_cap = strlen(proof) + strlen(SPEC) + strlen(DATA) + 64;
    char *verify_cmd = (char *)malloc(verify_cap);
    if (!verify_cmd) {
        free(proof);
        free(prove_cmd);
        wickra_proof_free(prover);
        return 1;
    }
    snprintf(verify_cmd, verify_cap,
             "{\"cmd\":\"verify\",\"proof\":%s,\"spec\":%s,\"data\":%s}", proof, SPEC, DATA);

    char *verdict = run(prover, verify_cmd);
    int ok = verdict && strstr(verdict, "\"valid\":true") != NULL;
    printf("verify: %s\n", ok ? "valid" : "INVALID");

    free(verdict);
    free(verify_cmd);
    free(proof);
    free(prove_cmd);
    wickra_proof_free(prover);

    if (!ok) {
        fprintf(stderr, "verification did not hold\n");
        return 1;
    }
    return 0;
}
```

### Surface

```c
#include "wickra_proof.h"

WickraProof *wickra_proof_new(void);
void         wickra_proof_free(WickraProof *handle);
int32_t      wickra_proof_command(WickraProof *handle,
                                  const char *cmd_json,
                                  char *out, size_t cap);
const char  *wickra_proof_version(void);
```

- **`wickra_proof_new`** creates a stateless prover handle. Never fails.
- **`wickra_proof_free`** destroys a handle (null is a no-op).
- **`wickra_proof_command`** applies a command JSON and writes the response JSON
  into the caller's buffer using a length-out protocol (below).
- **`wickra_proof_version`** returns a static, NUL-terminated version string
  (do not free).

### Command / response protocol

Everything goes through `wickra_proof_command`. Commands are JSON objects with a
`"cmd"` field: `prove`, `verify`, `canonicalize`, `version`. Responses are JSON,
e.g. the full `Proof` for `prove`, `{"ok":true,"valid":true}` for `verify`.

The response is returned via a caller-owned buffer with a length-out protocol —
the callee never allocates memory the caller must free:

1. Call with `out = NULL`, `cap = 0` to learn the response length `len`
   (excluding the terminating NUL).
2. Allocate `len + 1` bytes and call again; the response plus a NUL is written.

Whenever `len < cap`, the response is written on that call, so a
sufficiently-large buffer needs only one call.

Return codes:

| Return   | Meaning                                             |
|----------|-----------------------------------------------------|
| `>= 0`   | Response length in bytes (excluding the NUL).       |
| `-1`     | A required pointer (`handle` or `cmd_json`) is null. |
| `-2`     | `cmd_json` is not valid UTF-8.                       |
| `-3`     | A panic was caught at the boundary.                 |

Domain errors (a bad spec, an unknown command) are **not** negative — they come
back in-band as `{"ok":false,"error":...}` JSON in the buffer.

### Header generation

`include/wickra_proof.h` is generated with [cbindgen] and committed; CI fails if
it drifts from the source. Regenerate after changing the ABI:

```sh
cbindgen --config cbindgen.toml --crate wickra-proof-c --output include/wickra_proof.h
```

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the C ABI itself, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-proof/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-proof>
- **Docs** (guides, spec reference, cookbook): <https://proof.wickra.org>
- **Runnable example:** [`examples/c/`](https://github.com/wickra-lib/wickra-proof/tree/main/examples/c)

Wickra Proof ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-proof/blob/main/SECURITY.md>.

## Disclaimer

`wickra-proof` is research and engineering tooling, not financial advice. A proof
attests only that a given report is the deterministic result of a given spec over
given data — it makes no claim about the quality, profitability or future
performance of any strategy. Trading carries risk; you are responsible for your
own decisions.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-proof/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-proof/blob/main/LICENSE-MIT) at your option.

[cbindgen]: https://github.com/mozilla/cbindgen
