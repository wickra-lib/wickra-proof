/* Cross-language golden parity and operating-mode equivalence, from C.
 *
 * Golden: prove each committed golden/specs/*.json over the shared
 * golden/data.json and assert the response equals golden/expected/<spec>.json
 * byte-for-byte. The ABI returns the core's canonical command output verbatim,
 * so byte equality is the exact cross-language parity check -- the same one
 * Python, Node, Go, C#, Java, R and WASM make.
 *
 * Operating mode: the proof does not depend on how its inputs arrive. A host
 * can splice the committed bytes into the envelope untouched, or hand over
 * what its own serialisation produces. C has no JSON library, so "what a C
 * host produces" is what printf produces: every number re-emitted through
 * strtod and %.17g (0.95 becomes 0.94999999999999996, 5.0 becomes 5),
 * whitespace after every separator, and the spec's top-level members in the
 * other order. The core canonicalizes before hashing, so both operating modes
 * must produce the same proof, byte for byte, and a proof re-emitted the same
 * way must still verify. This is where a host that mangles a number is caught.
 *
 * Until this existed the C ABI was the only reach with no test at all: the two
 * examples beside it print a proof and exit zero. Six of the ten language
 * reaches go through this ABI, so a fault here is a fault in all of them.
 *
 * C has no directory API that is portable between POSIX and Windows, so the
 * spec list is globbed by CMake at configure time and written into
 * golden_specs.h. That keeps the property the other bindings get from a
 * runtime glob: a spec added to the corpus is covered here without editing
 * this file.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_proof.h"

#include "golden_specs.h" /* GOLDEN_DIR, GOLDEN_SPECS, GOLDEN_SPEC_COUNT */

/* Read a whole file. Caller frees. Returns NULL and reports on failure. */
static char *slurp(const char *path) {
    FILE *file = fopen(path, "rb");
    if (!file) {
        fprintf(stderr, "cannot open %s\n", path);
        return NULL;
    }
    if (fseek(file, 0, SEEK_END) != 0) {
        fclose(file);
        return NULL;
    }
    long size = ftell(file);
    if (size < 0 || fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)size + 1);
    if (!buf) {
        fclose(file);
        return NULL;
    }
    size_t got = fread(buf, 1, (size_t)size, file);
    fclose(file);
    buf[got] = '\0';
    return buf;
}

/* Trim ASCII whitespace in place and return the start of the trimmed text. */
static char *trim(char *text) {
    while (*text == ' ' || *text == '\n' || *text == '\r' || *text == '\t') {
        text++;
    }
    size_t len = strlen(text);
    while (len > 0) {
        char last = text[len - 1];
        if (last != ' ' && last != '\n' && last != '\r' && last != '\t') {
            break;
        }
        text[--len] = '\0';
    }
    return text;
}

/* A growable string. */
typedef struct {
    char *buf;
    size_t len;
    size_t cap;
} Str;

static int str_push(Str *s, const char *text, size_t n) {
    if (s->len + n + 1 > s->cap) {
        size_t cap = s->cap ? s->cap : 4096;
        while (cap < s->len + n + 1) {
            cap *= 2;
        }
        char *grown = (char *)realloc(s->buf, cap);
        if (!grown) {
            return 0;
        }
        s->buf = grown;
        s->cap = cap;
    }
    memcpy(s->buf + s->len, text, n);
    s->len += n;
    s->buf[s->len] = '\0';
    return 1;
}

static int str_puts(Str *s, const char *text) { return str_push(s, text, strlen(text)); }

/* Apply one command through the two-call length protocol. Caller frees. */
static char *run(WickraProof *prover, const char *cmd) {
    int32_t len = wickra_proof_command(prover, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed with code %d\n", (int)len);
        return NULL;
    }
    char *out = (char *)malloc((size_t)len + 1);
    if (!out) {
        return NULL;
    }
    if (wickra_proof_command(prover, cmd, out, (size_t)len + 1) < 0) {
        free(out);
        return NULL;
    }
    return out;
}

/* The [start, end) span of the value that follows `"key":` at the top level of
 * `json`: an object or array by bracket matching, a string by its closing
 * quote, a scalar by its next separator. 0 when the key is absent. */
static int member_span(const char *json, const char *key, size_t *start, size_t *end) {
    char quoted[128];
    snprintf(quoted, sizeof quoted, "\"%s\"", key);
    const char *at = strstr(json, quoted);
    if (!at) {
        return 0;
    }
    const char *p = at + strlen(quoted);
    while (*p == ' ' || *p == '\n' || *p == '\r' || *p == '\t' || *p == ':') {
        p++;
    }
    const char *open = p;
    if (*p == '{' || *p == '[') {
        int depth = 0;
        int in_string = 0;
        for (; *p; p++) {
            if (in_string) {
                if (*p == '\\') {
                    p++;
                } else if (*p == '"') {
                    in_string = 0;
                }
            } else if (*p == '"') {
                in_string = 1;
            } else if (*p == '{' || *p == '[') {
                depth++;
            } else if (*p == '}' || *p == ']') {
                if (--depth == 0) {
                    p++;
                    break;
                }
            }
        }
    } else if (*p == '"') {
        for (p++; *p && *p != '"'; p++) {
            if (*p == '\\') {
                p++;
            }
        }
        p++;
    } else {
        while (*p && *p != ',' && *p != '}' && *p != ' ' && *p != '\n') {
            p++;
        }
    }
    *start = (size_t)(open - json);
    *end = (size_t)(p - json);
    return 1;
}

/* What a C host re-emits: numbers through strtod and %.17g, a space and a
 * newline after every separator, strings untouched. */
static int host_reemit(Str *out, const char *json) {
    int in_string = 0;
    for (const char *p = json; *p;) {
        if (in_string) {
            if (*p == '\\' && p[1]) {
                if (!str_push(out, p, 2)) {
                    return 0;
                }
                p += 2;
                continue;
            }
            if (*p == '"') {
                in_string = 0;
            }
            if (!str_push(out, p++, 1)) {
                return 0;
            }
            continue;
        }
        if (*p == '"') {
            in_string = 1;
            if (!str_push(out, p++, 1)) {
                return 0;
            }
            continue;
        }
        if (*p == '-' || (*p >= '0' && *p <= '9')) {
            char *stop = NULL;
            double value = strtod(p, &stop);
            char text[64];
            snprintf(text, sizeof text, "%.17g", value);
            if (!str_puts(out, text)) {
                return 0;
            }
            p = stop;
            continue;
        }
        if (*p == ',' || *p == ':' || *p == '{' || *p == '[') {
            if (!str_push(out, p++, 1) || !str_puts(out, *p ? " \n " : "")) {
                return 0;
            }
            continue;
        }
        if (!str_push(out, p++, 1)) {
            return 0;
        }
    }
    return 1;
}

/* The spec with its two top-level members in the other order. The golden specs
 * are written strategy-first; the canonical form is alphabetical; this is
 * neither. */
static int swapped_spec(Str *out, const char *spec) {
    size_t s0 = 0, s1 = 0, r0 = 0, r1 = 0;
    if (!member_span(spec, "strategy", &s0, &s1) || !member_span(spec, "dataset_ref", &r0, &r1) || s1 > r0) {
        return 0;
    }
    if (!str_puts(out, "{\"dataset_ref\":") || !str_push(out, spec + r0, r1 - r0)) {
        return 0;
    }
    if (!str_puts(out, ",\"strategy\":") || !str_push(out, spec + s0, s1 - s0)) {
        return 0;
    }
    return str_puts(out, "}");
}

int main(void) {
    if (GOLDEN_SPEC_COUNT == 0) {
        fprintf(stderr, "no golden specs were configured; this would test nothing\n");
        return 1;
    }

    char path[1024];
    snprintf(path, sizeof path, "%s/data.json", GOLDEN_DIR);
    char *data_raw = slurp(path);
    if (!data_raw) {
        return 1;
    }
    const char *data = trim(data_raw);
    Str hosted_data = {0};
    if (!host_reemit(&hosted_data, data)) {
        fprintf(stderr, "could not re-emit golden data\n");
        return 1;
    }

    WickraProof *prover = wickra_proof_new();
    if (!prover) {
        fprintf(stderr, "failed to create prover\n");
        return 1;
    }

    int failures = 0;
    for (size_t i = 0; i < GOLDEN_SPEC_COUNT; i++) {
        const char *name = GOLDEN_SPECS[i];

        snprintf(path, sizeof path, "%s/specs/%s", GOLDEN_DIR, name);
        char *spec_raw = slurp(path);
        snprintf(path, sizeof path, "%s/expected/%s", GOLDEN_DIR, name);
        char *expected_raw = slurp(path);
        if (!spec_raw || !expected_raw) {
            free(spec_raw);
            free(expected_raw);
            failures++;
            continue;
        }
        const char *spec = trim(spec_raw);
        const char *expected = trim(expected_raw);

        /* Mode 1: the committed bytes, spliced into the envelope verbatim. */
        Str committed_cmd = {0};
        str_puts(&committed_cmd, "{\"cmd\":\"prove\",\"spec\":");
        str_puts(&committed_cmd, spec);
        str_puts(&committed_cmd, ",\"data\":");
        str_puts(&committed_cmd, data);
        str_puts(&committed_cmd, "}");
        char *committed = run(prover, committed_cmd.buf);
        free(committed_cmd.buf);
        if (!committed) {
            fprintf(stderr, "%s: no proof from the committed bytes\n", name);
            failures++;
        } else if (strcmp(trim(committed), expected) != 0) {
            fprintf(stderr, "%s: golden mismatch\n  expected: %s\n  got:      %s\n", name, expected, committed);
            failures++;
        }
        free(committed);

        /* Mode 2: what the host re-emits. */
        Str swapped = {0};
        Str hosted_spec = {0};
        if (!swapped_spec(&swapped, spec) || !host_reemit(&hosted_spec, swapped.buf)) {
            fprintf(stderr, "%s: could not re-emit the spec\n", name);
            failures++;
            free(swapped.buf);
            free(hosted_spec.buf);
            free(spec_raw);
            free(expected_raw);
            continue;
        }
        free(swapped.buf);
        Str hosted_cmd = {0};
        str_puts(&hosted_cmd, "{ \"cmd\" : \"prove\" ,\n \"spec\" : ");
        str_puts(&hosted_cmd, hosted_spec.buf);
        str_puts(&hosted_cmd, " ,\n \"data\" : ");
        str_puts(&hosted_cmd, hosted_data.buf);
        str_puts(&hosted_cmd, " }");
        char *hosted = run(prover, hosted_cmd.buf);
        free(hosted_cmd.buf);
        if (!hosted) {
            fprintf(stderr, "%s: no proof from the host-serialised bytes\n", name);
            failures++;
        } else if (strcmp(trim(hosted), expected) != 0) {
            fprintf(stderr, "%s: host-serialised bytes prove differently\n  expected: %s\n  got:      %s\n", name,
                    expected, hosted);
            failures++;
        }
        free(hosted);

        /* A proof the host re-emitted still verifies. */
        Str hosted_proof = {0};
        if (!host_reemit(&hosted_proof, expected)) {
            fprintf(stderr, "%s: could not re-emit the proof\n", name);
            failures++;
        } else {
            Str verify_cmd = {0};
            str_puts(&verify_cmd, "{\"cmd\":\"verify\",\"proof\":");
            str_puts(&verify_cmd, hosted_proof.buf);
            str_puts(&verify_cmd, ",\"spec\":");
            str_puts(&verify_cmd, hosted_spec.buf);
            str_puts(&verify_cmd, ",\"data\":");
            str_puts(&verify_cmd, hosted_data.buf);
            str_puts(&verify_cmd, "}");
            char *verdict = run(prover, verify_cmd.buf);
            free(verify_cmd.buf);
            if (!verdict) {
                fprintf(stderr, "%s: no verdict for the re-emitted proof\n", name);
                failures++;
            } else if (strcmp(trim(verdict), "{\"ok\":true,\"valid\":true}") != 0) {
                fprintf(stderr, "%s: the re-emitted proof does not verify: %s\n", name, verdict);
                failures++;
            }
            free(verdict);
        }
        free(hosted_proof.buf);
        free(hosted_spec.buf);
        free(spec_raw);
        free(expected_raw);
    }

    wickra_proof_free(prover);
    free(hosted_data.buf);
    free(data_raw);

    if (failures > 0) {
        fprintf(stderr, "%d failure(s) across %zu golden specs\n", failures, GOLDEN_SPEC_COUNT);
        return 1;
    }
    printf("all %zu golden proofs are byte-identical from C, in both operating modes\n", GOLDEN_SPEC_COUNT);
    return 0;
}
