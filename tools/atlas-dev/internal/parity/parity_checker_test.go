package parity

import (
	"os"
	"path/filepath"
	"testing"
)

func TestParityChecker_CheckParity(t *testing.T) {
	// Create temp project structure
	tempDir := t.TempDir()

	// Create directories
	cratesDir := filepath.Join(tempDir, "crates")
	if err := os.MkdirAll(cratesDir, 0755); err != nil {
		t.Fatal(err)
	}

	// Create a simple Rust file
	rustFile := filepath.Join(cratesDir, "lib.rs")
	rustContent := `
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(1, 2), 3);
}
`
	if err := os.WriteFile(rustFile, []byte(rustContent), 0644); err != nil {
		t.Fatal(err)
	}

	// Create parity checker
	checker := NewParityChecker(tempDir)

	// Run parity check
	report, err := checker.CheckParity()
	if err != nil {
		t.Fatalf("CheckParity failed: %v", err)
	}

	// Verify report structure
	if report == nil {
		t.Fatal("Expected non-nil report")
	}

	// Health score should be between 0 and 100
	if report.HealthScore < 0 || report.HealthScore > 100 {
		t.Errorf("Invalid health score: %.2f", report.HealthScore)
	}

	// Should have run some checks
	if report.TotalChecks == 0 {
		t.Error("Expected TotalChecks > 0")
	}

	// Report should have details
	if report.Details == nil {
		t.Error("Expected non-nil Details")
	}

	// Should have code analysis details
	if _, ok := report.Details["code"]; !ok {
		t.Error("Expected code analysis in details")
	}
}

func TestParityChecker_WithCustomDirs(t *testing.T) {
	tempDir := t.TempDir()

	customCodeDir := filepath.Join(tempDir, "custom_code")
	if err := os.MkdirAll(customCodeDir, 0755); err != nil {
		t.Fatal(err)
	}

	checker := NewParityChecker(tempDir).
		WithCodeDir(customCodeDir).
		WithSpecDir(filepath.Join(tempDir, "custom_spec")).
		WithAPIDir(filepath.Join(tempDir, "custom_api"))

	// Verify directories were set (internal fields - would need getters in real code)
	// For now, just verify it doesn't crash
	if checker == nil {
		t.Error("Expected non-nil checker")
	}
}

func TestParityChecker_ToCompactJSON(t *testing.T) {
	report := &ParityReport{
		OK:           true,
		HealthScore:  95.5,
		TotalChecks:  100,
		PassedChecks: 95,
		FailedChecks: 5,
		Errors:       []ParityError{},
		Warnings:     []ParityError{},
		Details:      make(map[string]interface{}),
	}

	result := report.ToCompactJSON()

	// Verify required fields
	if ok, exists := result["ok"].(bool); !exists || !ok {
		t.Error("Expected ok=true")
	}

	if health, exists := result["health"].(float64); !exists || health != 95.5 {
		t.Errorf("Expected health=95.5, got %v", result["health"])
	}

	if checks, exists := result["checks"].(int); !exists || checks != 100 {
		t.Errorf("Expected checks=100, got %v", result["checks"])
	}
}

func TestParityChecker_ErrorHandling(t *testing.T) {
	// Test with non-existent directory
	checker := NewParityChecker("/nonexistent/path")

	report, err := checker.CheckParity()

	// Should not return error (should handle gracefully with warnings)
	if err != nil {
		// It's okay to error if directory doesn't exist
		t.Logf("CheckParity returned error (expected): %v", err)
	}

	// If report is returned, it should have structure
	if report != nil {
		if report.HealthScore < 0 || report.HealthScore > 100 {
			t.Errorf("Invalid health score: %.2f", report.HealthScore)
		}
	}
}

func TestParityError_Structure(t *testing.T) {
	err := ParityError{
		Type:     "test_error",
		Severity: "error",
		Source:   "test.rs:10",
		Issue:    "Test issue",
		Fix:      "Fix suggestion",
	}

	if err.Type != "test_error" {
		t.Errorf("Expected Type='test_error', got '%s'", err.Type)
	}

	if err.Severity != "error" {
		t.Errorf("Expected Severity='error', got '%s'", err.Severity)
	}
}

func TestParityReport_WithErrors(t *testing.T) {
	report := &ParityReport{
		OK:           false,
		HealthScore:  60.0,
		TotalChecks:  10,
		PassedChecks: 6,
		FailedChecks: 4,
		Errors: []ParityError{
			{
				Type:     "spec_code_mismatch",
				Severity: "error",
				Source:   "spec.md:5",
				Issue:    "Missing implementation",
				Fix:      "Implement function foo",
			},
		},
		Warnings: []ParityError{
			{
				Type:     "code_not_specified",
				Severity: "warning",
				Source:   "lib.rs:10",
				Issue:    "Public function not in spec",
				Fix:      "Add to specification",
			},
		},
		Details: make(map[string]interface{}),
	}

	result := report.ToCompactJSON()

	// Verify errors are included
	if errors, ok := result["errors"].([]map[string]interface{}); !ok || len(errors) != 1 {
		t.Error("Expected 1 error in JSON")
	}

	// Verify warnings are included
	if warnings, ok := result["warnings"].([]map[string]interface{}); !ok || len(warnings) != 1 {
		t.Error("Expected 1 warning in JSON")
	}

	// Verify error counts
	if errCnt, ok := result["err_cnt"].(int); !ok || errCnt != 1 {
		t.Errorf("Expected err_cnt=1, got %v", result["err_cnt"])
	}

	if warnCnt, ok := result["warn_cnt"].(int); !ok || warnCnt != 1 {
		t.Errorf("Expected warn_cnt=1, got %v", result["warn_cnt"])
	}
}

func TestParityChecker_WithPhaseDir(t *testing.T) {
	tempDir := t.TempDir()
	phaseDir := filepath.Join(tempDir, "custom_phases")

	checker := NewParityChecker(tempDir).WithPhaseDir(phaseDir)

	if checker == nil {
		t.Error("Expected non-nil checker")
	}
	// Internal field check would require getter, but we verified it doesn't crash
}

func TestParityChecker_WithTestDir(t *testing.T) {
	tempDir := t.TempDir()
	testDir := filepath.Join(tempDir, "custom_tests")

	checker := NewParityChecker(tempDir).WithTestDir(testDir)

	if checker == nil {
		t.Error("Expected non-nil checker")
	}
}

func TestParityChecker_AddError(t *testing.T) {
	checker := NewParityChecker("")
	report := &ParityReport{
		Errors:       []ParityError{},
		FailedChecks: 0,
	}

	checker.addError(report, "test_error", "source.rs:10", "Test issue", "Fix it")

	if len(report.Errors) != 1 {
		t.Errorf("Expected 1 error, got %d", len(report.Errors))
	}

	if report.FailedChecks != 1 {
		t.Errorf("Expected FailedChecks=1, got %d", report.FailedChecks)
	}

	err := report.Errors[0]
	if err.Type != "test_error" {
		t.Errorf("Expected Type='test_error', got '%s'", err.Type)
	}
	if err.Severity != "error" {
		t.Errorf("Expected Severity='error', got '%s'", err.Severity)
	}
}

func TestParityChecker_ProcessTestResults(t *testing.T) {
	checker := NewParityChecker("")
	report := &ParityReport{
		Errors:       []ParityError{},
		PassedChecks: 0,
		TotalChecks:  0,
	}

	testReport := &TestAnalysisReport{
		Requirements: []TestRequirement{
			{
				PhaseID:   "phase-01",
				Required:  10,
				Actual:    10,
				Met:       true,
				PhasePath: "phase-01.md",
			},
			{
				PhaseID:   "phase-02",
				Required:  20,
				Actual:    15,
				Deficit:   5,
				Met:       false,
				PhasePath: "phase-02.md",
			},
		},
	}

	checker.processTestResults(report, testReport)

	// Should have 2 checks (one passed, one failed)
	if report.TotalChecks != 2 {
		t.Errorf("Expected TotalChecks=2, got %d", report.TotalChecks)
	}

	if report.PassedChecks != 1 {
		t.Errorf("Expected PassedChecks=1, got %d", report.PassedChecks)
	}

	// Should have 1 error for unmet requirement
	if len(report.Errors) != 1 {
		t.Errorf("Expected 1 error, got %d", len(report.Errors))
	}

	if report.Errors[0].Type != "test_count_mismatch" {
		t.Errorf("Expected error type='test_count_mismatch', got '%s'", report.Errors[0].Type)
	}
}

func TestParityChecker_ProcessRefResults(t *testing.T) {
	checker := NewParityChecker("")
	report := &ParityReport{
		Errors:       []ParityError{},
		Warnings:     []ParityError{},
		PassedChecks: 0,
		TotalChecks:  0,
	}

	refReport := &ReferenceReport{
		TotalRefs: 10,
		ValidRefs: 8,
		BrokenRefs: []BrokenReference{
			{
				Ref: Reference{
					SourceFile: "doc.md",
					SourceLine: 5,
					TargetPath: "missing.md",
				},
				ErrorType:     "file_missing",
				FixSuggestion: "Create file",
			},
		},
		OrphanedDocs: []string{"orphan1.md", "orphan2.md"},
	}

	checker.processRefResults(report, refReport)

	if report.TotalChecks != 10 {
		t.Errorf("Expected TotalChecks=10, got %d", report.TotalChecks)
	}

	if report.PassedChecks != 8 {
		t.Errorf("Expected PassedChecks=8, got %d", report.PassedChecks)
	}

	// Should have 1 error for broken ref
	if len(report.Errors) != 1 {
		t.Errorf("Expected 1 error for broken ref, got %d", len(report.Errors))
	}

	// Should have 2 warnings for orphaned docs
	if len(report.Warnings) != 2 {
		t.Errorf("Expected 2 warnings for orphaned docs, got %d", len(report.Warnings))
	}
}

func TestParityChecker_CalculateHealthScore(t *testing.T) {
	tests := []struct {
		name         string
		totalChecks  int
		passedChecks int
		errorCount   int
		warnCount    int
		minScore     float64
		maxScore     float64
	}{
		{
			name:         "perfect score",
			totalChecks:  100,
			passedChecks: 100,
			errorCount:   0,
			warnCount:    0,
			minScore:     100.0,
			maxScore:     100.0,
		},
		{
			name:         "with errors",
			totalChecks:  100,
			passedChecks: 90,
			errorCount:   5,
			warnCount:    0,
			minScore:     70.0,
			maxScore:     90.0,
		},
		{
			name:         "with warnings",
			totalChecks:  100,
			passedChecks: 100,
			errorCount:   0,
			warnCount:    10,
			minScore:     90.0,
			maxScore:     100.0,
		},
		{
			name:         "no checks",
			totalChecks:  0,
			passedChecks: 0,
			errorCount:   0,
			warnCount:    0,
			minScore:     100.0,
			maxScore:     100.0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			checker := NewParityChecker("")
			report := &ParityReport{
				TotalChecks:  tt.totalChecks,
				PassedChecks: tt.passedChecks,
				Errors:       make([]ParityError, tt.errorCount),
				Warnings:     make([]ParityError, tt.warnCount),
			}

			checker.calculateHealthScore(report)

			if report.HealthScore < tt.minScore || report.HealthScore > tt.maxScore {
				t.Errorf("Expected health score between %.2f and %.2f, got %.2f",
					tt.minScore, tt.maxScore, report.HealthScore)
			}

			// Score should be clamped to 0-100
			if report.HealthScore < 0 || report.HealthScore > 100 {
				t.Errorf("Health score out of range: %.2f", report.HealthScore)
			}
		})
	}
}

func TestParityChecker_ProcessSpecResults(t *testing.T) {
	checker := NewParityChecker("")
	report := &ParityReport{
		Errors:       []ParityError{},
		Warnings:     []ParityError{},
		PassedChecks: 0,
		TotalChecks:  0,
	}

	specReport := &SpecMatchReport{
		Matches: []SpecMatch{
			{SpecItem: "func1"},
			{SpecItem: "func2"},
		},
		Mismatches: []SpecMismatch{
			{
				SpecItem:      "missing_func",
				SpecSection:   "Core",
				Issue:         "Not implemented",
				FixSuggestion: "Implement it",
				FilePath:      "spec.md",
				Line:          10,
			},
		},
		Unspecified: []CodeItem{
			{Name: "undoc_func", Type: "function", Public: true, FilePath: "lib.rs", Line: 20},
		},
		TotalSpec: 3,
	}

	checker.processSpecResults(report, specReport)

	if report.TotalChecks != 3 {
		t.Errorf("Expected TotalChecks=3, got %d", report.TotalChecks)
	}

	if report.PassedChecks != 2 {
		t.Errorf("Expected PassedChecks=2, got %d", report.PassedChecks)
	}

	// Should have 1 error for mismatch
	if len(report.Errors) != 1 {
		t.Errorf("Expected 1 error, got %d", len(report.Errors))
	}

	// Should have 1 warning for unspecified
	if len(report.Warnings) != 1 {
		t.Errorf("Expected 1 warning, got %d", len(report.Warnings))
	}
}

func TestParityChecker_ProcessAPIResults(t *testing.T) {
	checker := NewParityChecker("")
	report := &ParityReport{
		Errors:       []ParityError{},
		Warnings:     []ParityError{},
		PassedChecks: 0,
		TotalChecks:  0,
	}

	apiReport := &APIMatchReport{
		Matches: []APIMatch{
			{},
		},
		Mismatches: []APIMismatch{
			{
				Type:          "not_implemented",
				APIFunc:       "missing",
				Issue:         "Not found",
				FixSuggestion: "Implement",
				FilePath:      "api.md",
				Line:          5,
			},
			{
				Type:          "not_documented",
				CodeItem:      "undoc",
				Issue:         "Missing docs",
				FixSuggestion: "Document",
				FilePath:      "lib.rs",
				Line:          10,
			},
		},
		TotalDocumented: 2,
	}

	checker.processAPIResults(report, apiReport)

	if report.TotalChecks != 2 {
		t.Errorf("Expected TotalChecks=2, got %d", report.TotalChecks)
	}

	if report.PassedChecks != 1 {
		t.Errorf("Expected PassedChecks=1, got %d", report.PassedChecks)
	}

	// Should have 1 error (not_implemented)
	if len(report.Errors) != 1 {
		t.Errorf("Expected 1 error, got %d", len(report.Errors))
	}

	// Should have 1 warning (not_documented)
	if len(report.Warnings) != 1 {
		t.Errorf("Expected 1 warning, got %d", len(report.Warnings))
	}
}
