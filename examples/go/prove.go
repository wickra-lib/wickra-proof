// A runnable Go example: prove a (spec, data) pair through the binding, print
// the report hash, then verify the proof and assert it holds.
//
//	cargo build --release -p wickra-proof-c
//	# stage the library under bindings/go/lib/<goos>_<goarch>/ (CI does this)
//	cd examples/go && go run .
package main

import (
	"encoding/json"
	"fmt"

	wickra "github.com/wickra-lib/wickra-proof-go"
)

const spec = `{"strategy":{"symbol":"AAA","timeframe":"1h",` +
	`"indicators":{"ema_fast":{"type":"Ema","params":[3]},` +
	`"ema_slow":{"type":"Ema","params":[8]}},` +
	`"entry":{"cross_above":["ema_fast","ema_slow"]},` +
	`"exit":{"cross_below":["ema_fast","ema_slow"]},` +
	`"sizing":{"type":"fixed_fraction","fraction":0.95},` +
	`"costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},` +
	`"risk":{}},"dataset_ref":"example/AAA/1h"}`

// A short V-shaped price path so the fast/slow EMA cross fires at least once.
const data = `{"AAA":[` +
	`{"time":1700000000,"open":120,"high":121,"low":119,"close":120,"volume":1000},` +
	`{"time":1700003600,"open":120,"high":121,"low":117,"close":118,"volume":1000},` +
	`{"time":1700007200,"open":118,"high":119,"low":115,"close":116,"volume":1000},` +
	`{"time":1700010800,"open":116,"high":117,"low":113,"close":114,"volume":1000},` +
	`{"time":1700014400,"open":114,"high":115,"low":111,"close":112,"volume":1000},` +
	`{"time":1700018000,"open":112,"high":113,"low":109,"close":110,"volume":1000},` +
	`{"time":1700021600,"open":110,"high":111,"low":107,"close":108,"volume":1000},` +
	`{"time":1700025200,"open":108,"high":113,"low":107,"close":112,"volume":1000},` +
	`{"time":1700028800,"open":112,"high":117,"low":111,"close":116,"volume":1000},` +
	`{"time":1700032400,"open":116,"high":121,"low":115,"close":120,"volume":1000},` +
	`{"time":1700036000,"open":120,"high":125,"low":119,"close":124,"volume":1000},` +
	`{"time":1700039600,"open":124,"high":129,"low":123,"close":128,"volume":1000}]}`

func main() {
	prover := wickra.New()
	defer prover.Close()

	proveCmd := fmt.Sprintf(`{"cmd":"prove","spec":%s,"data":%s}`, spec, data)
	proof, err := prover.Command(proveCmd)
	if err != nil {
		panic(err)
	}

	var parsed struct {
		ReportHash string `json:"report_hash"`
	}
	if err := json.Unmarshal([]byte(proof), &parsed); err != nil {
		panic(err)
	}

	fmt.Println("wickra-proof", wickra.Version())
	fmt.Println("report_hash:", parsed.ReportHash)

	// The prove response is valid JSON, so it drops straight in as "proof".
	verifyCmd := fmt.Sprintf(`{"cmd":"verify","proof":%s,"spec":%s,"data":%s}`, proof, spec, data)
	verdict, err := prover.Command(verifyCmd)
	if err != nil {
		panic(err)
	}
	if verdict != `{"ok":true,"valid":true}` {
		panic("proof must verify, got: " + verdict)
	}
	fmt.Println("verify: valid")
}
