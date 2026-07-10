#' The wickra-proof library version.
#' @return A version string.
#' @export
wkproof_version <- function() {
  .Call(C_wkproof_version)
}

#' Create a stateless prover.
#' @return A `wickra_proof` handle (an external pointer).
#' @export
wkproof_new <- function() {
  .Call(C_wkproof_new)
}

#' Apply a command JSON and return the resulting response JSON.
#' @param prover A prover handle from [wkproof_new()].
#' @param cmd_json A command JSON string.
#' @return The response as a JSON string.
#' @export
wkproof_command <- function(prover, cmd_json) {
  .Call(C_wkproof_command, prover, cmd_json)
}
