## Plain-R tests for the wickra-proof R binding (no testthat dependency).
## Mirrors the Rust/Python/Node/Go/C#/Java tests and doubles as the completeness
## guard: it exercises the full public surface (version + new + command).

library(wickraproof)

strategy <- paste0(
  '{"symbol":"BTCUSDT","timeframe":"1h",',
  '"indicators":{"ema_fast":{"type":"Ema","params":[5]},',
  '"ema_slow":{"type":"Ema","params":[15]}},',
  '"entry":{"cross_above":["ema_fast","ema_slow"]},',
  '"exit":{"cross_below":["ema_fast","ema_slow"]},',
  '"sizing":{"type":"fixed_fraction","fraction":0.95},',
  '"costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},',
  '"risk":{"trailing_stop_pct":5.0}}'
)

candles <- function() {
  parts <- vapply(0:39, function(i) {
    b <- 100.0 + sin(i * 0.4) * 8.0
    paste0(
      '{"time":', format(1700000000 + i * 3600, scientific = FALSE),
      ',"open":', b, ',"high":', b + 1.0, ',"low":', b - 1.0,
      ',"close":', b + 0.5, ',"volume":1000.0}'
    )
  }, character(1))
  paste0("[", paste(parts, collapse = ","), "]")
}

spec <- paste0('{"strategy":', strategy, ',"dataset_ref":"BTCUSDT/1h/test"}')
data <- paste0('{"BTCUSDT":', candles(), '}')

prove <- function(prover) {
  wkproof_command(prover, paste0('{"cmd":"prove","spec":', spec, ',"data":', data, '}'))
}

hex_field <- function(json, key) {
  m <- regmatches(json, regexpr(paste0('"', key, '":"[0-9a-f]{64}"'), json))
  stopifnot(length(m) == 1)
  m
}

## version
stopifnot(nzchar(wkproof_version()))

## prove -> 64-hex report_hash + inputs_hash
prover <- wkproof_new()
proof <- prove(prover)
stopifnot(nchar(hex_field(proof, "report_hash")) == 64 + nchar('"report_hash":""'))
stopifnot(nchar(hex_field(proof, "inputs_hash")) == 64 + nchar('"inputs_hash":""'))

## prove is reproducible
stopifnot(identical(
  hex_field(prove(wkproof_new()), "report_hash"),
  hex_field(prove(wkproof_new()), "report_hash")
))

## verify accepts a genuine proof and rejects a tampered one
good <- wkproof_command(
  prover,
  paste0('{"cmd":"verify","proof":', proof, ',"spec":', spec, ',"data":', data, '}')
)
stopifnot(identical(good, '{"ok":true,"valid":true}'))

tampered <- sub(
  '"report_hash":"[0-9a-f]{64}"',
  paste0('"report_hash":"', strrep("0", 64), '"'),
  proof
)
bad <- wkproof_command(
  prover,
  paste0('{"cmd":"verify","proof":', tampered, ',"spec":', spec, ',"data":', data, '}')
)
stopifnot(identical(bad, '{"ok":true,"valid":false}'))

## an unknown command is an in-band error, not a hard error
inband <- wkproof_command(prover, '{"cmd":"nope"}')
stopifnot(grepl('"ok":false', inband, fixed = TRUE))

## cross-language golden parity: for each committed golden/specs/*.json, prove
## over the shared golden/data.json and assert the response equals
## golden/expected/<spec>.json byte-for-byte. The binding returns the core's
## canonical command output verbatim, so byte equality is the exact
## cross-language parity check. The fixtures arrive in a later phase; until then
## the golden section is skipped.
golden_dir <- function() {
  d <- normalizePath(getwd(), mustWork = FALSE)
  for (i in seq_len(8)) {
    g <- file.path(d, "golden")
    if (dir.exists(file.path(g, "specs"))) {
      return(g)
    }
    d <- dirname(d)
  }
  NULL
}

g <- golden_dir()
if (!is.null(g)) {
  dataset <- trimws(paste(
    readLines(file.path(g, "data.json"), warn = FALSE), collapse = "\n"
  ))
  for (spec_path in list.files(file.path(g, "specs"), pattern = "\\.json$", full.names = TRUE)) {
    name <- basename(spec_path)
    spec_json <- trimws(paste(readLines(spec_path, warn = FALSE), collapse = "\n"))
    expected <- trimws(paste(
      readLines(file.path(g, "expected", name), warn = FALSE), collapse = "\n"
    ))
    gprover <- wkproof_new()
    got <- wkproof_command(
      gprover, paste0('{"cmd":"prove","spec":', spec_json, ',"data":', dataset, '}')
    )
    stopifnot(identical(trimws(got), expected))
  }
}

## operating-mode equivalence: the proof does not depend on how its inputs
## arrive. A binding hands JSON text to the core. It can pass the committed
## golden bytes through untouched, or it can hand over what the host re-emits
## -- keys in another order, whitespace, numbers re-formatted by R. The core
## canonicalizes before hashing, so both operating modes must produce the same
## proof, byte for byte, and a proof re-emitted by the host must still verify.
## This is where a host that mangles a number (1700000000 -> 1.7e+09) is
## caught. Base R has no JSON reader, so the test brings the smallest one that
## behaves like a host's: objects come back with their keys in reverse order,
## numbers go through format(), the output is indented.
json_read <- function(text) {
  chars <- strsplit(text, "")[[1]]
  pos <- 1L
  skip_ws <- function() {
    while (pos <= length(chars) && chars[pos] %in% c(" ", "\n", "\r", "\t")) pos <<- pos + 1L
  }
  expect <- function(ch) {
    skip_ws()
    stopifnot(chars[pos] == ch)
    pos <<- pos + 1L
  }
  read_string <- function() {
    expect('"')
    out <- character(0)
    repeat {
      ch <- chars[pos]
      pos <<- pos + 1L
      if (ch == '"') return(paste(out, collapse = ""))
      if (ch == "\\") {
        esc <- chars[pos]
        pos <<- pos + 1L
        ch <- switch(esc,
          n = "\n", t = "\t", r = "\r", b = "\b", f = "\f",
          u = {
            code <- strtoi(paste(chars[pos:(pos + 3L)], collapse = ""), 16L)
            pos <<- pos + 4L
            intToUtf8(code)
          },
          esc
        )
      }
      out <- c(out, ch)
    }
  }
  read_number <- function() {
    start <- pos
    while (pos <= length(chars) && grepl("[-+0-9.eE]", chars[pos])) pos <<- pos + 1L
    as.numeric(paste(chars[start:(pos - 1L)], collapse = ""))
  }
  read_value <- function() {
    skip_ws()
    ch <- chars[pos]
    if (ch == "{") return(read_object())
    if (ch == "[") return(read_array())
    if (ch == '"') return(read_string())
    rest <- paste(chars[pos:min(pos + 4L, length(chars))], collapse = "")
    if (startsWith(rest, "true")) { pos <<- pos + 4L; return(TRUE) }
    if (startsWith(rest, "false")) { pos <<- pos + 5L; return(FALSE) }
    if (startsWith(rest, "null")) { pos <<- pos + 4L; return(NULL) }
    read_number()
  }
  # Objects come back with their keys in reverse order: the opposite of the
  # canonical (sorted) form, and not the committed order either.
  read_object <- function() {
    expect("{")
    keys <- character(0)
    values <- list()
    skip_ws()
    if (chars[pos] == "}") { pos <<- pos + 1L; return(structure(list(), class = "jobj")) }
    repeat {
      skip_ws()
      key <- read_string()
      expect(":")
      keys <- c(keys, key)
      values[[length(values) + 1L]] <- list(read_value())
      skip_ws()
      if (chars[pos] == ",") { pos <<- pos + 1L; next }
      expect("}")
      break
    }
    ord <- order(keys, decreasing = TRUE, method = "radix")
    structure(setNames(lapply(values[ord], `[[`, 1L), keys[ord]), class = "jobj")
  }
  read_array <- function() {
    expect("[")
    items <- list()
    skip_ws()
    if (chars[pos] == "]") { pos <<- pos + 1L; return(list()) }
    repeat {
      items[[length(items) + 1L]] <- list(read_value())
      skip_ws()
      if (chars[pos] == ",") { pos <<- pos + 1L; next }
      expect("]")
      break
    }
    lapply(items, `[[`, 1L)
  }
  value <- read_value()
  skip_ws()
  stopifnot(pos == length(chars) + 1L)
  value
}

json_emit <- function(value, indent = 0L) {
  pad <- function(n) strrep("  ", n)
  if (is.null(value)) return("null")
  if (inherits(value, "jobj")) {
    if (length(value) == 0L) return("{}")
    fields <- vapply(names(value), function(key) {
      paste0(pad(indent + 1L), json_emit(key), ": ", json_emit(value[[key]], indent + 1L))
    }, character(1))
    return(paste0("{\n", paste(fields, collapse = ",\n"), "\n", pad(indent), "}"))
  }
  if (is.list(value)) {
    if (length(value) == 0L) return("[]")
    items <- vapply(value, function(item) paste0(pad(indent + 1L), json_emit(item, indent + 1L)), character(1))
    return(paste0("[\n", paste(items, collapse = ",\n"), "\n", pad(indent), "]"))
  }
  if (is.character(value)) {
    escaped <- gsub("\\", "\\\\", value, fixed = TRUE)
    escaped <- gsub('"', '\\"', escaped, fixed = TRUE)
    escaped <- gsub("\n", "\\n", escaped, fixed = TRUE)
    escaped <- gsub("\t", "\\t", escaped, fixed = TRUE)
    escaped <- gsub("\r", "\\r", escaped, fixed = TRUE)
    return(paste0('"', escaped, '"'))
  }
  if (is.logical(value)) return(if (value) "true" else "false")
  # R's own number formatting, kept out of scientific notation: the mistake
  # the example once made (1.7e+09 for a timestamp) must not be repeated here.
  format(value, digits = 15, scientific = FALSE)
}

jobj <- function(...) structure(list(...), class = "jobj")

stopifnot(!is.null(g))
dataset <- trimws(paste(readLines(file.path(g, "data.json"), warn = FALSE), collapse = "\n"))
spec_paths <- list.files(file.path(g, "specs"), pattern = "\\.json$", full.names = TRUE)
stopifnot(length(spec_paths) > 0)
mprover <- wkproof_new()
for (spec_path in spec_paths) {
  name <- basename(spec_path)
  spec_json <- trimws(paste(readLines(spec_path, warn = FALSE), collapse = "\n"))
  expected <- trimws(paste(readLines(file.path(g, "expected", name), warn = FALSE), collapse = "\n"))

  # Mode 1: the committed bytes, spliced into the envelope verbatim.
  committed <- wkproof_command(mprover, paste0('{"cmd":"prove","spec":', spec_json, ',"data":', dataset, "}"))
  # Mode 2: what the host re-emits -- reversed key order, indented, R-formatted numbers.
  hosted <- wkproof_command(mprover, json_emit(jobj(
    cmd = "prove", spec = json_read(spec_json), data = json_read(dataset)
  )))
  stopifnot(identical(trimws(committed), expected))
  stopifnot(identical(trimws(hosted), expected))

  # A proof the host re-emitted still verifies.
  verdict <- wkproof_command(mprover, json_emit(jobj(
    cmd = "verify", proof = json_read(expected), spec = json_read(spec_json), data = json_read(dataset)
  )))
  stopifnot(identical(trimws(verdict), '{"ok":true,"valid":true}'))
}

cat("wickra-proof R tests passed\n")
