package wickra

// Operating-mode equivalence: the proof does not depend on how its inputs
// arrive. A binding hands JSON text to the core. It can pass the committed
// golden bytes through untouched, or it can hand over what the host's JSON
// library re-emits -- keys in another order, whitespace, floats re-formatted.
// The core canonicalizes before hashing, so both operating modes must produce
// the same proof, byte for byte, and a proof re-emitted by the host must still
// verify. This is where a binding that mangles a float is caught.
//
// encoding/json sorts map keys on output, so "another order" here is the
// sorted order, which is not the committed one: the specs are written
// strategy-first, the canonical form is alphabetical.

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// decoded parses JSON into the host's own value types -- map, slice, float64,
// string, bool, nil -- so re-emitting it is the host's serialisation, not a
// pass-through of the committed bytes.
func decoded(t *testing.T, text []byte) any {
	t.Helper()
	var v any
	if err := json.Unmarshal(text, &v); err != nil {
		t.Fatal(err)
	}
	return v
}

func TestCommittedAndHostSerialisedInputsProveAlike(t *testing.T) {
	g := goldenDir()
	if g == "" {
		t.Fatal("golden fixtures not found above the module")
	}
	dataText, err := os.ReadFile(filepath.Join(g, "data.json"))
	if err != nil {
		t.Fatal(err)
	}
	specs, err := filepath.Glob(filepath.Join(g, "specs", "*.json"))
	if err != nil {
		t.Fatal(err)
	}
	if len(specs) == 0 {
		t.Fatal("no golden specs; this would test nothing")
	}
	p := New()
	defer p.Close()
	for _, specPath := range specs {
		name := filepath.Base(specPath)
		specText, err := os.ReadFile(specPath)
		if err != nil {
			t.Fatal(err)
		}
		expectedRaw, err := os.ReadFile(filepath.Join(g, "expected", name))
		if err != nil {
			t.Fatal(err)
		}
		expected := strings.TrimSpace(string(expectedRaw))

		// Mode 1: the committed bytes, spliced into the envelope verbatim.
		committed, err := p.Command(`{"cmd":"prove","spec":` + string(specText) + `,"data":` + string(dataText) + `}`)
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		// Mode 2: what the host re-emits -- decoded to maps, re-encoded pretty.
		hostedCmd, err := json.MarshalIndent(map[string]any{
			"cmd":  "prove",
			"spec": decoded(t, specText),
			"data": decoded(t, dataText),
		}, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		hosted, err := p.Command(string(hostedCmd))
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		if strings.TrimSpace(committed) != expected {
			t.Fatalf("%s: committed bytes prove differently\n  expected: %s\n  got:      %s", name, expected, committed)
		}
		if strings.TrimSpace(hosted) != expected {
			t.Fatalf("%s: host-serialised bytes prove differently\n  expected: %s\n  got:      %s", name, expected, hosted)
		}

		// A proof the host re-emitted still verifies.
		verifyCmd, err := json.Marshal(map[string]any{
			"cmd":   "verify",
			"proof": decoded(t, []byte(expected)),
			"spec":  json.RawMessage(specText),
			"data":  json.RawMessage(dataText),
		})
		if err != nil {
			t.Fatal(err)
		}
		verdict, err := p.Command(string(verifyCmd))
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		if strings.TrimSpace(verdict) != `{"ok":true,"valid":true}` {
			t.Fatalf("%s: host-re-emitted proof does not verify: %s", name, verdict)
		}
	}
}
