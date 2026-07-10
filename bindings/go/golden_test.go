package wickra

// Cross-language golden parity: for each committed golden/specs/*.json, prove
// over the shared golden/data.json and assert the response equals
// golden/expected/<spec>.json byte-for-byte. The binding returns the core's
// canonical command_json string verbatim, so byte equality is the exact
// cross-language parity check — the same blake3 report/inputs hashes in every
// language. The fixtures arrive in a later phase; until then the test skips
// cleanly.

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func goldenDir() string {
	dir, err := os.Getwd()
	if err != nil {
		return ""
	}
	for i := 0; i < 8; i++ {
		g := filepath.Join(dir, "golden")
		if _, err := os.Stat(filepath.Join(g, "specs")); err == nil {
			return g
		}
		dir = filepath.Dir(dir)
	}
	return ""
}

func TestGoldenParity(t *testing.T) {
	g := goldenDir()
	if g == "" {
		t.Skip("golden fixtures not present yet")
	}
	dataset, err := os.ReadFile(filepath.Join(g, "data.json"))
	if err != nil {
		t.Fatal(err)
	}
	specs, err := filepath.Glob(filepath.Join(g, "specs", "*.json"))
	if err != nil {
		t.Fatal(err)
	}
	for _, specPath := range specs {
		specJSON, err := os.ReadFile(specPath)
		if err != nil {
			t.Fatal(err)
		}
		name := filepath.Base(specPath)
		expected, err := os.ReadFile(filepath.Join(g, "expected", name))
		if err != nil {
			t.Fatal(err)
		}
		p := New()
		cmd, err := json.Marshal(map[string]any{
			"cmd":  "prove",
			"spec": json.RawMessage(specJSON),
			"data": json.RawMessage(dataset),
		})
		if err != nil {
			t.Fatal(err)
		}
		raw, err := p.Command(string(cmd))
		p.Close()
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		// Trim trailing whitespace on both sides: the committed fixture ends with
		// a newline while the core's canonical command string does not. The
		// canonical form itself is whitespace-free, so trimming compares the exact
		// bytes that carry meaning — the same check every other binding makes.
		if strings.TrimSpace(raw) != strings.TrimSpace(string(expected)) {
			t.Fatalf("%s: golden mismatch", name)
		}
	}
}
