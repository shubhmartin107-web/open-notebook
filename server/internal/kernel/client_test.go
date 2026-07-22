package kernel

import (
	"encoding/json"
	"testing"
	"time"
)

func TestNew(t *testing.T) {
	c := New("/some/path", 10*time.Second)
	if c == nil {
		t.Fatal("New returned nil")
	}
	if c.kernelPath != "/some/path" {
		t.Fatalf("expected kernel path '/some/path', got %q", c.kernelPath)
	}
	if c.timeout != 10*time.Second {
		t.Fatalf("expected timeout 10s, got %v", c.timeout)
	}
}

func TestNewDefaults(t *testing.T) {
	c := New("", 0)
	if c.kernelPath != "onb-kernel" {
		t.Fatalf("expected default path 'onb-kernel', got %q", c.kernelPath)
	}
	if c.timeout != 30*time.Second {
		t.Fatalf("expected default timeout 30s, got %v", c.timeout)
	}
}

func TestStopWithoutStart(t *testing.T) {
	c := New("/nonexistent", time.Second)
	if err := c.Stop(); err != nil {
		t.Fatalf("Stop without Start should not error, got: %v", err)
	}
}

func TestExecuteResponseJSON(t *testing.T) {
	resp := ExecuteResponse{
		Results: []CellResult{
			{CellID: "cell_1", Stdout: "hello\n", Stderr: "", ExitCode: 0},
			{CellID: "cell_2", Stdout: "", Stderr: "error", ExitCode: 1},
		},
	}

	data, err := json.Marshal(resp)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}

	var decoded ExecuteResponse
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("Unmarshal failed: %v", err)
	}

	if len(decoded.Results) != 2 {
		t.Fatalf("expected 2 results, got %d", len(decoded.Results))
	}
	if decoded.Results[0].CellID != "cell_1" {
		t.Fatalf("expected cell_1, got %q", decoded.Results[0].CellID)
	}
	if decoded.Results[0].Stdout != "hello\n" {
		t.Fatalf("expected 'hello\\n', got %q", decoded.Results[0].Stdout)
	}
	if decoded.Results[1].Stderr != "error" {
		t.Fatalf("expected 'error', got %q", decoded.Results[1].Stderr)
	}
	if decoded.Results[1].ExitCode != 1 {
		t.Fatalf("expected exit_code 1, got %d", decoded.Results[1].ExitCode)
	}
}

func TestCellResultJSON(t *testing.T) {
	cr := CellResult{CellID: "cell_1", Stdout: "output", Stderr: "", ExitCode: 0}
	data, err := json.Marshal(cr)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}

	var decoded CellResult
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("Unmarshal failed: %v", err)
	}
	if decoded.CellID != "cell_1" {
		t.Fatalf("expected cell_1, got %q", decoded.CellID)
	}
	if decoded.Stdout != "output" {
		t.Fatalf("expected 'output', got %q", decoded.Stdout)
	}
}

func TestCellResultZeroValues(t *testing.T) {
	cr := CellResult{}
	data, err := json.Marshal(cr)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}

	var decoded CellResult
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("Unmarshal failed: %v", err)
	}
	if decoded.CellID != "" {
		t.Fatalf("expected empty cell_id, got %q", decoded.CellID)
	}
}
