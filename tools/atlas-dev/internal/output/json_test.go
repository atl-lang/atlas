package output

import (
	"bytes"
	"encoding/json"
	"io"
	"os"
	"testing"
)

func TestSuccess(t *testing.T) {
	// Capture stdout
	old := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	data := map[string]interface{}{
		"phase": "test",
		"count": 42,
	}

	err := Success(data)
	if err != nil {
		t.Fatalf("Success() error: %v", err)
	}

	// Restore stdout
	w.Close()
	os.Stdout = old

	// Read captured output
	var buf bytes.Buffer
	io.Copy(&buf, r)

	// Parse JSON
	var result map[string]interface{}
	if err := json.Unmarshal(buf.Bytes(), &result); err != nil {
		t.Fatalf("failed to parse JSON: %v", err)
	}

	// Verify ok=true
	if ok, exists := result["ok"]; !exists || ok != true {
		t.Error("expected ok=true")
	}

	// Verify fields preserved
	if result["phase"] != "test" {
		t.Errorf("expected phase='test', got %v", result["phase"])
	}
	if result["count"] != float64(42) {
		t.Errorf("expected count=42, got %v", result["count"])
	}
}

func TestError(t *testing.T) {
	// Capture stdout
	old := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	testErr := &testError{msg: "test error"}
	details := map[string]interface{}{
		"code": 42,
	}

	err := Error(testErr, details)
	if err != nil {
		t.Fatalf("Error() error: %v", err)
	}

	// Restore stdout
	w.Close()
	os.Stdout = old

	// Read captured output
	var buf bytes.Buffer
	io.Copy(&buf, r)

	// Parse JSON
	var result map[string]interface{}
	if err := json.Unmarshal(buf.Bytes(), &result); err != nil {
		t.Fatalf("failed to parse JSON: %v", err)
	}

	// Verify ok=false
	if ok, exists := result["ok"]; !exists || ok != false {
		t.Error("expected ok=false")
	}

	// Verify error message
	if result["err"] != "test error" {
		t.Errorf("expected err='test error', got %v", result["err"])
	}

	// Verify details
	if result["code"] != float64(42) {
		t.Errorf("expected code=42, got %v", result["code"])
	}
}

func TestRemoveEmpty(t *testing.T) {
	tests := []struct {
		name  string
		input map[string]interface{}
		want  int
	}{
		{
			"remove nil",
			map[string]interface{}{
				"a": "value",
				"b": nil,
			},
			1,
		},
		{
			"remove empty string",
			map[string]interface{}{
				"a": "value",
				"b": "",
			},
			1,
		},
		{
			"remove empty array",
			map[string]interface{}{
				"a": "value",
				"b": []interface{}{},
			},
			1,
		},
		{
			"remove empty map",
			map[string]interface{}{
				"a": "value",
				"b": map[string]interface{}{},
			},
			1,
		},
		{
			"keep zero",
			map[string]interface{}{
				"a": 0,
				"b": false,
			},
			2,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := removeEmpty(tt.input)
			if len(result) != tt.want {
				t.Errorf("removeEmpty() len = %d, want %d", len(result), tt.want)
			}
		})
	}
}

func TestIsEmpty(t *testing.T) {
	tests := []struct {
		name  string
		value interface{}
		want  bool
	}{
		{"nil", nil, true},
		{"empty string", "", true},
		{"empty array", []interface{}{}, true},
		{"empty map", map[string]interface{}{}, true},
		{"zero", 0, false},
		{"false", false, false},
		{"non-empty string", "text", false},
		{"non-empty array", []interface{}{1}, false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := isEmpty(tt.value)
			if got != tt.want {
				t.Errorf("isEmpty(%v) = %v, want %v", tt.value, got, tt.want)
			}
		})
	}
}

type testError struct {
	msg string
}

func (e *testError) Error() string {
	return e.msg
}
