# Golden fixtures

Frozen `(spec, data)` → proof triples that pin the deterministic core across the
Rust integration tests **and** every language binding. Determinism is the
product: the same inputs must fold to the same canonical report and blake3 hashes
in Rust, Python, Node, WASM, Go, C#, Java and R.

## Layout

- `data/{AAA,BBB,CCC}.csv` — the deterministic universe, one `ts,open,high,low,close,volume`
  file per symbol, 40 bars each.
- `data.json` — the same universe as a `{symbol: [candle, …]}` map, consumed by the
  cross-language binding golden tests (byte-for-byte against `expected/`).
- `specs/{sma_cross,rsi_reversion,buy_hold}.json` — three `ProofSpec`s
  (`{strategy, dataset_ref}`), each exercising a distinct execution.
- `expected/*.json` — one blessed proof per spec: the exact canonical `prove`
  response, including the real `report_hash` / `inputs_hash`.

## Data formula

Each symbol's `close[i]` (`i = 0 … 39`) is a fixed function; `open[i]` is the
previous close, `high/low` are `max/min(open, close) ± 1`, `volume` is `1000`, and
`time[i] = 1_700_000_000 + i * 3600`. All values are integers or multiples of
`0.25`, so they round-trip identically through CSV and JSON.

| Symbol | `close[i]`                              | Shape                | Exercises |
| ------ | --------------------------------------- | -------------------- | --------- |
| AAA    | `i ≤ 10 ? 120 - 2·i : 100 + 2·(i - 10)` | V (down then up)     | `sma_cross` golden-crosses on the turn and holds (1 trade) |
| BBB    | `120 - 0.25·i`                          | slow downtrend       | `buy_hold` enters at warmup and holds (1 trade) |
| CCC    | `i ≤ 20 ? 110 - 2·i : 70 + 3·(i - 20)`  | deep dip, recovery   | `rsi_reversion` buys the oversold dip, sells the >50 recovery (6 trades) |

## Blessing

The fixtures are generated — **never edit `data*` or `expected/` by hand**. To
re-bless after an intended change, build the Node binding and run the generator:

```bash
( cd bindings/node && npm run build )
node golden/_bless.mjs
```

It writes the CSVs, `data.json`, and freezes each spec's `prove` response into
`expected/`. The `_bless.mjs` universe and the CSVs are the single source of
truth; `data.json` is derived from the same numbers.

## When a hash breaks

A golden mismatch means one of two things has changed, and both are load-bearing:

1. **Canonicalization** — the key ordering, float rounding, or whitespace of the
   canonical form drifted. This breaks byte-for-byte reproducibility across
   languages and must be treated as a breaking change.
2. **`engine_version`** — the linked `wickra-backtest` engine produced a different
   report for the same inputs. Bump and re-bless deliberately.

Never "fix" a red golden by blindly re-blessing; first confirm which of the two
moved and whether the move was intended.
