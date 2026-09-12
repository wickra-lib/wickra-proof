using System.Text.Json;
using System.Text.Json.Nodes;
using Wickra.Proof;
using Xunit;

namespace WickraProof.Tests;

// Operating-mode equivalence: the proof does not depend on how its inputs
// arrive. A binding hands JSON text to the core. It can pass the committed
// golden bytes through untouched, or it can hand over what the host's JSON
// library re-emits -- keys in another order, whitespace, floats re-formatted.
// The core canonicalizes before hashing, so both operating modes must produce
// the same proof, byte for byte, and a proof re-emitted by the host must still
// verify. This is where a binding that mangles a float is caught.
public class ModeTests
{
    private static string? FindGolden()
    {
        string? dir = AppContext.BaseDirectory;
        for (int i = 0; i < 10 && dir is not null; i++)
        {
            string g = Path.Combine(dir, "golden");
            if (Directory.Exists(Path.Combine(g, "specs")))
            {
                return g;
            }
            dir = Path.GetDirectoryName(dir);
        }
        return null;
    }

    // The same value with every object's keys in reverse order.
    private static JsonNode? Reordered(JsonNode? value)
    {
        switch (value)
        {
            case JsonObject obj:
                var reversed = new JsonObject();
                foreach (string key in obj.Select(p => p.Key).OrderByDescending(k => k, StringComparer.Ordinal))
                {
                    reversed[key] = Reordered(obj[key]);
                }
                return reversed;
            case JsonArray arr:
                var items = new JsonArray();
                foreach (JsonNode? item in arr)
                {
                    items.Add(Reordered(item));
                }
                return items;
            default:
                return value?.DeepClone();
        }
    }

    private static readonly JsonSerializerOptions Pretty = new() { WriteIndented = true };

    [Fact]
    public void CommittedAndHostSerialisedInputs_ProveAlike()
    {
        string? golden = FindGolden();
        Assert.NotNull(golden);

        string dataText = File.ReadAllText(Path.Combine(golden, "data.json"));
        using var prover = new Prover();

        foreach (string specPath in Directory.GetFiles(Path.Combine(golden, "specs"), "*.json"))
        {
            string name = Path.GetFileName(specPath);
            string specText = File.ReadAllText(specPath);
            string expected = File.ReadAllText(Path.Combine(golden, "expected", name)).TrimEnd();

            // Mode 1: the committed bytes, spliced into the envelope verbatim.
            string committed = prover.Command("{\"cmd\":\"prove\",\"spec\":" + specText + ",\"data\":" + dataText + "}");
            // Mode 2: what the host re-emits -- reversed key order, pretty-printed.
            var hostedCmd = new JsonObject
            {
                ["cmd"] = "prove",
                ["spec"] = Reordered(JsonNode.Parse(specText)),
                ["data"] = Reordered(JsonNode.Parse(dataText)),
            };
            string hosted = prover.Command(hostedCmd.ToJsonString(Pretty));
            Assert.True(expected == committed.TrimEnd(), $"committed bytes: {name}");
            Assert.True(expected == hosted.TrimEnd(), $"host-serialised bytes: {name}");

            // A proof the host re-emitted still verifies.
            var verifyCmd = new JsonObject
            {
                ["cmd"] = "verify",
                ["proof"] = Reordered(JsonNode.Parse(expected)),
                ["spec"] = JsonNode.Parse(specText),
                ["data"] = JsonNode.Parse(dataText),
            };
            string verdict = prover.Command(verifyCmd.ToJsonString());
            Assert.True("{\"ok\":true,\"valid\":true}" == verdict.TrimEnd(), $"{name}: {verdict}");
        }
    }
}
