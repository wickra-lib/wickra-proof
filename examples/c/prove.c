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
