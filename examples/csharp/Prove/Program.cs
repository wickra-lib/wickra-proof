// A runnable .NET example: prove a (spec, data) pair through the binding, print
// the report hash, then verify the proof and assert it holds.
//
//   cargo build --release -p wickra-proof-c
//   dotnet run --project examples/csharp/Prove

using System.Text.Json;
using Wickra.Proof;

const string spec =
    "{\"strategy\":{\"symbol\":\"AAA\",\"timeframe\":\"1h\"," +
    "\"indicators\":{\"ema_fast\":{\"type\":\"Ema\",\"params\":[3]}," +
    "\"ema_slow\":{\"type\":\"Ema\",\"params\":[8]}}," +
    "\"entry\":{\"cross_above\":[\"ema_fast\",\"ema_slow\"]}," +
    "\"exit\":{\"cross_below\":[\"ema_fast\",\"ema_slow\"]}," +
    "\"sizing\":{\"type\":\"fixed_fraction\",\"fraction\":0.95}," +
    "\"costs\":{\"taker_bps\":5,\"slippage\":{\"type\":\"fixed_bps\",\"bps\":2}}," +
    "\"risk\":{}},\"dataset_ref\":\"example/AAA/1h\"}";

// A short V-shaped price path so the fast/slow EMA cross fires at least once.
int[] closes = [120, 118, 116, 114, 112, 110, 108, 112, 116, 120, 124, 128];

static object Candle(long time, int open, int close) => new
{
    time,
    open,
    high = Math.Max(open, close) + 1,
    low = Math.Min(open, close) - 1,
    close,
    volume = 1000,
};

var candles = closes
    .Select((close, i) => Candle(1_700_000_000L + i * 3600, i == 0 ? close : closes[i - 1], close))
    .ToArray();
var data = new Dictionary<string, object[]> { ["AAA"] = candles };

using var prover = new Prover();

string proof = prover.Command(
    JsonSerializer.Serialize(new { cmd = "prove", spec = JsonDocument.Parse(spec).RootElement, data }));
using JsonDocument proofDoc = JsonDocument.Parse(proof);

Console.WriteLine($"wickra-proof {Prover.Version()}");
Console.WriteLine($"report_hash: {proofDoc.RootElement.GetProperty("report_hash").GetString()}");

// The prove response is valid JSON, so it drops straight in as "proof".
string verdict = prover.Command(JsonSerializer.Serialize(new
{
    cmd = "verify",
    proof = proofDoc.RootElement,
    spec = JsonDocument.Parse(spec).RootElement,
    data,
}));
using JsonDocument verdictDoc = JsonDocument.Parse(verdict);
if (!verdictDoc.RootElement.GetProperty("valid").GetBoolean())
{
    throw new InvalidOperationException($"proof must verify, got: {verdict}");
}
Console.WriteLine("verify: valid");
