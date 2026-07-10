# A runnable R example: prove a (spec, data) pair through the binding, print the
# report hash, then verify the proof and assert it holds.
#
#   cargo build -p wickra-proof-c --release
#   export WKPROOF_INC="$PWD/bindings/c/include"
#   export WKPROOF_LIB="$PWD/target/release"
#   export LD_LIBRARY_PATH="$WKPROOF_LIB:$LD_LIBRARY_PATH"   # PATH on Windows
#   R CMD INSTALL bindings/r
#   Rscript examples/r/prove.R

library(wickraproof)

spec <- paste0(
  '{"strategy":{"symbol":"AAA","timeframe":"1h",',
  '"indicators":{"ema_fast":{"type":"Ema","params":[3]},',
  '"ema_slow":{"type":"Ema","params":[8]}},',
  '"entry":{"cross_above":["ema_fast","ema_slow"]},',
  '"exit":{"cross_below":["ema_fast","ema_slow"]},',
  '"sizing":{"type":"fixed_fraction","fraction":0.95},',
  '"costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},',
  '"risk":{}},"dataset_ref":"example/AAA/1h"}'
)

# A short V-shaped price path so the fast/slow EMA cross fires at least once.
closes <- c(120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128)

candle <- function(i) {
  close <- closes[i]
  open <- if (i == 1) close else closes[i - 1]
  paste0(
    '{"time":', 1700000000 + (i - 1) * 3600,
    ',"open":', open,
    ',"high":', max(open, close) + 1,
    ',"low":', min(open, close) - 1,
    ',"close":', close, ',"volume":1000}'
  )
}

data <- paste0(
  '{"AAA":[',
  paste(vapply(seq_along(closes), candle, character(1)), collapse = ","),
  "]}"
)

prover <- wkproof_new()
proof <- wkproof_command(prover, paste0(
  '{"cmd":"prove","spec":', spec, ',"data":', data, "}"
))

cat("wickra-proof", wkproof_version(), "\n")
hash_at <- regexpr('"report_hash":', proof)
cat("proof:", substr(proof, hash_at, hash_at + 29), "...\n")

# The prove response is valid JSON, so it drops straight in as "proof".
verdict <- wkproof_command(prover, paste0(
  '{"cmd":"verify","proof":', proof, ',"spec":', spec, ',"data":', data, "}"
))
stopifnot(grepl('"valid":true', verdict, fixed = TRUE))
cat("verify: valid\n")
