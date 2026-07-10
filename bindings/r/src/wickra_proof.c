/* R .Call glue for the wickra-proof C ABI hub. */
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>
#include <stddef.h>
#include "wickra_proof.h"

/* --- handle lifetime ----------------------------------------------------- */

static void wkproof_finalize(SEXP ext) {
    WickraProof *h = (WickraProof *)R_ExternalPtrAddr(ext);
    if (h) {
        wickra_proof_free(h);
    }
    R_ClearExternalPtr(ext);
}

static WickraProof *handle_of(SEXP ext) {
    WickraProof *h = (WickraProof *)R_ExternalPtrAddr(ext);
    if (!h) {
        Rf_error("wickra-proof: handle is closed");
    }
    return h;
}

/* --- exported .Call entries ---------------------------------------------- */

SEXP wkproof_version(void) {
    return Rf_mkString(wickra_proof_version());
}

SEXP wkproof_new(void) {
    WickraProof *h = wickra_proof_new();
    if (!h) {
        Rf_error("wickra-proof: failed to create a prover");
    }
    SEXP ext = PROTECT(R_MakeExternalPtr(h, R_NilValue, R_NilValue));
    R_RegisterCFinalizerEx(ext, wkproof_finalize, TRUE);
    UNPROTECT(1);
    return ext;
}

SEXP wkproof_command(SEXP ext, SEXP cmd_json) {
    WickraProof *h = handle_of(ext);
    const char *cmd = CHAR(STRING_ELT(cmd_json, 0));

    /* Length-out protocol: learn the length, then read into a caller buffer.
       Domain errors come back in-band as {"ok":false,...} JSON, not a negative
       code; only unusable arguments / a caught panic return < 0. */
    int len = wickra_proof_command(h, cmd, NULL, 0);
    if (len < 0) {
        Rf_error("wickra-proof: command failed (code %d)", len);
    }
    char *buf = (char *)R_alloc((size_t)len + 1, 1);
    wickra_proof_command(h, cmd, buf, (size_t)len + 1);
    return Rf_mkString(buf);
}

/* --- registration -------------------------------------------------------- */

static const R_CallMethodDef CallEntries[] = {
    {"wkproof_version", (DL_FUNC)&wkproof_version, 0},
    {"wkproof_new", (DL_FUNC)&wkproof_new, 0},
    {"wkproof_command", (DL_FUNC)&wkproof_command, 2},
    {NULL, NULL, 0}};

void R_init_wickraproof(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}
