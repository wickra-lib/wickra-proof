// A minimal C++ example: prove a (spec, data) pair through the wickra-proof C
// ABI, print the report hash, then verify the proof and assert it holds.
//
// The prove response is itself a JSON object, so the verify command embeds it
// verbatim as the "proof" value — no JSON parser is needed on the C++ side.
#include <cstddef>
#include <iostream>
#include <string>
#include <vector>

#include "wickra_proof.h"

namespace {
const char *SPEC =
    R"({"strategy":{"symbol":"AAA","timeframe":"1h",)"
    R"("indicators":{"ema_fast":{"type":"Ema","params":[3]},)"
    R"("ema_slow":{"type":"Ema","params":[8]}},)"
    R"("entry":{"cross_above":["ema_fast","ema_slow"]},)"
    R"("exit":{"cross_below":["ema_fast","ema_slow"]},)"
    R"("sizing":{"type":"fixed_fraction","fraction":0.95},)"
    R"("costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},)"
    R"("risk":{}},"dataset_ref":"example/AAA/1h"})";

// A short V-shaped price path so the fast/slow EMA cross fires at least once.
const char *DATA =
    R"({"AAA":[)"
    R"({"time":1700000000,"open":120,"high":121,"low":119,"close":120,"volume":1000},)"
    R"({"time":1700003600,"open":120,"high":121,"low":117,"close":118,"volume":1000},)"
    R"({"time":1700007200,"open":118,"high":119,"low":115,"close":116,"volume":1000},)"
    R"({"time":1700010800,"open":116,"high":117,"low":113,"close":114,"volume":1000},)"
    R"({"time":1700014400,"open":114,"high":115,"low":111,"close":112,"volume":1000},)"
    R"({"time":1700018000,"open":112,"high":113,"low":109,"close":110,"volume":1000},)"
    R"({"time":1700021600,"open":110,"high":111,"low":107,"close":108,"volume":1000},)"
    R"({"time":1700025200,"open":108,"high":113,"low":107,"close":112,"volume":1000},)"
    R"({"time":1700028800,"open":112,"high":117,"low":111,"close":116,"volume":1000},)"
    R"({"time":1700032400,"open":116,"high":121,"low":115,"close":120,"volume":1000},)"
    R"({"time":1700036000,"open":120,"high":125,"low":119,"close":124,"volume":1000},)"
    R"({"time":1700039600,"open":124,"high":129,"low":123,"close":128,"volume":1000}]})";

// Run a command and return its response using the length-out protocol.
std::string run(WickraProof *prover, const std::string &cmd) {
    int len = wickra_proof_command(prover, cmd.c_str(), nullptr, 0);
    if (len < 0) {
        std::cerr << "command failed: code " << len << "\n";
        return {};
    }
    std::vector<char> buf(static_cast<std::size_t>(len) + 1);
    wickra_proof_command(prover, cmd.c_str(), buf.data(), buf.size());
    return std::string(buf.data());
}
}  // namespace

int main() {
    WickraProof *prover = wickra_proof_new();
    if (prover == nullptr) {
        std::cerr << "failed to create prover\n";
        return 1;
    }

    const std::string spec(SPEC);
    const std::string data(DATA);

    std::string proof =
        run(prover, R"({"cmd":"prove","spec":)" + spec + R"(,"data":)" + data + "}");
    if (proof.empty()) {
        wickra_proof_free(prover);
        return 1;
    }

    std::cout << "wickra-proof " << wickra_proof_version() << "\n";
    auto pos = proof.find("\"report_hash\":");
    if (pos != std::string::npos) {
        std::cout << "proof: " << proof.substr(pos, 30) << "...\n";
    }

    // The prove response is valid JSON, so it drops straight in as "proof".
    std::string verdict = run(prover, R"({"cmd":"verify","proof":)" + proof + R"(,"spec":)" +
                                           spec + R"(,"data":)" + data + "}");
    bool ok = verdict.find("\"valid\":true") != std::string::npos;
    std::cout << "verify: " << (ok ? "valid" : "INVALID") << "\n";

    wickra_proof_free(prover);
    if (!ok) {
        std::cerr << "verification did not hold\n";
        return 1;
    }
    return 0;
}
