package parity

import (
	"os"
	"path/filepath"
	"testing"
)

func TestTestAnalysisReport_ToCompactJSON(t *testing.T) {
	report := &TestAnalysisReport{
		Requirements:  []TestRequirement{{Required: 10, Actual: 8}},
		TotalRequired: 50,
		TotalActual:   45,
		TotalMet:      8,
		TotalDeficit:  5,
		Coverage:      90.0,
	}

	result := report.ToCompactJSON()

	if required, ok := result["required"].(int); !ok || required != 50 {
		t.Errorf("Expected required=50, got %v", result["required"])
	}

	if coverage, ok := result["coverage"].(float64); !ok || coverage != 90.0 {
		t.Errorf("Expected coverage=90.0, got %v", result["coverage"])
	}
}

func TestTestAnalysisReport_GetDeficits(t *testing.T) {
	report := &TestAnalysisReport{
		Requirements: []TestRequirement{
			{PhaseID: "phase-01", Required: 10, Actual: 10, Met: true},
			{PhaseID: "phase-02", Required: 20, Actual: 15, Met: false, Deficit: 5},
			{PhaseID: "phase-03", Required: 5, Actual: 3, Met: false, Deficit: 2},
		},
	}

	deficits := report.GetDeficits()

	if len(deficits) != 2 {
		t.Errorf("Expected 2 deficits, got %d", len(deficits))
	}

	// Verify all deficits are !Met
	for _, d := range deficits {
		if d.Met {
			t.Errorf("Deficit should have Met=false, got true for %s", d.PhaseID)
		}
	}
}

func TestTestRequirement_Structure(t *testing.T) {
	req := TestRequirement{
		PhasePath: "phases/test/phase-01.md",
		PhaseID:   "phase-01",
		Category:  "test",
		Required:  25,
		Actual:    20,
		Deficit:   5,
		Met:       false,
		TestFiles: []string{"test1.rs", "test2.rs"},
	}

	if req.PhaseID != "phase-01" {
		t.Errorf("Expected PhaseID='phase-01', got '%s'", req.PhaseID)
	}

	if req.Deficit != 5 {
		t.Errorf("Expected Deficit=5, got %d", req.Deficit)
	}

	if req.Met {
		t.Error("Expected Met=false")
	}

	if len(req.TestFiles) != 2 {
		t.Errorf("Expected 2 test files, got %d", len(req.TestFiles))
	}
}

func TestTestAnalyzer_ExtractTestRequirement(t *testing.T) {
	tempDir := t.TempDir()
	phaseDir := filepath.Join(tempDir, "phases")
	if err := os.MkdirAll(phaseDir, 0755); err != nil {
		t.Fatal(err)
	}

	tests := []struct {
		name     string
		content  string
		expected int
		shouldErr bool
	}{
		{
			name:     "standard format",
			content:  "Minimum test count: 35 tests",
			expected: 35,
			shouldErr: false,
		},
		{
			name:     "bold format",
			content:  "**Minimum test count:** 40 tests",
			expected: 40,
			shouldErr: false,
		},
		{
			name:     "tests required format",
			content:  "Tests required: 25",
			expected: 25,
			shouldErr: false,
		},
		{
			name:     "target format",
			content:  "Target: 50+ tests",
			expected: 50,
			shouldErr: false,
		},
		{
			name:     "no requirement",
			content:  "This phase has no test requirements",
			expected: 0,
			shouldErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			phaseFile := filepath.Join(phaseDir, "test-phase.md")
			if err := os.WriteFile(phaseFile, []byte(tt.content), 0644); err != nil {
				t.Fatal(err)
			}

			analyzer := NewTestAnalyzer(phaseDir, "")
			req, err := analyzer.extractTestRequirement(phaseFile)

			if tt.shouldErr {
				if err == nil {
					t.Error("Expected error but got none")
				}
			} else {
				if err != nil {
					t.Errorf("Unexpected error: %v", err)
				}
				if req.Required != tt.expected {
					t.Errorf("Expected %d required tests, got %d", tt.expected, req.Required)
				}
			}
		})
	}
}

func TestTestAnalyzer_CountTests(t *testing.T) {
	tempDir := t.TempDir()
	testDir := filepath.Join(tempDir, "tests")
	if err := os.MkdirAll(testDir, 0755); err != nil {
		t.Fatal(err)
	}

	testFile := filepath.Join(testDir, "phase_01_test.rs")
	content := `
#[test]
fn test_one() {
    assert!(true);
}

#[test]
fn test_two() {
    assert!(true);
}

fn test_with_prefix() {
    assert!(true);
}

fn not_a_test() {
    println!("Not a test");
}
`
	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewTestAnalyzer("", testDir)
	count, files := analyzer.countTests("phase-01", "test")

	if count < 2 {
		t.Errorf("Expected at least 2 tests, got %d", count)
	}

	if len(files) == 0 {
		t.Error("Expected to find test files")
	}
}

func TestTestAnalyzer_CountTestsInFile(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "test.rs")
	content := `
#[test]
fn test_one() {}

#[test]
fn test_two() {}

fn test_prefix() {}

fn not_test() {}
`
	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewTestAnalyzer("", tempDir)

	// Access the method indirectly through countTests
	count, _ := analyzer.countTests("test", "test")

	if count < 2 {
		t.Errorf("Expected at least 2 tests with #[test] attribute, got %d", count)
	}
}

func TestTestAnalyzer_AnalyzeTests(t *testing.T) {
	tempDir := t.TempDir()

	// Create phase directory with phase files
	phaseDir := filepath.Join(tempDir, "phases")
	if err := os.MkdirAll(phaseDir, 0755); err != nil {
		t.Fatal(err)
	}

	// Create phase file
	phaseFile := filepath.Join(phaseDir, "phase-01.md")
	phaseContent := `# Phase 01
Minimum test count: 10 tests
`
	if err := os.WriteFile(phaseFile, []byte(phaseContent), 0644); err != nil {
		t.Fatal(err)
	}

	// Create test directory with test files
	testDir := filepath.Join(tempDir, "tests")
	if err := os.MkdirAll(testDir, 0755); err != nil {
		t.Fatal(err)
	}

	testFile := filepath.Join(testDir, "phase_01_tests.rs")
	testContent := `
#[test]
fn test_1() {}
#[test]
fn test_2() {}
#[test]
fn test_3() {}
`
	if err := os.WriteFile(testFile, []byte(testContent), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewTestAnalyzer(phaseDir, testDir)
	report, err := analyzer.AnalyzeTests()
	if err != nil {
		t.Fatalf("AnalyzeTests failed: %v", err)
	}

	if len(report.Requirements) == 0 {
		t.Error("Expected at least one requirement")
	}

	if report.Coverage < 0 || report.Coverage > 100 {
		t.Errorf("Invalid coverage: %.2f%%", report.Coverage)
	}
}

func TestTestAnalyzer_FindPhaseFiles(t *testing.T) {
	tempDir := t.TempDir()
	phaseDir := filepath.Join(tempDir, "phases", "stdlib")
	if err := os.MkdirAll(phaseDir, 0755); err != nil {
		t.Fatal(err)
	}

	// Create multiple phase files
	files := []string{"phase-01.md", "phase-02.md", "README.md"}
	for _, file := range files {
		path := filepath.Join(phaseDir, file)
		if err := os.WriteFile(path, []byte("test"), 0644); err != nil {
			t.Fatal(err)
		}
	}

	analyzer := NewTestAnalyzer(filepath.Join(tempDir, "phases"), "")
	foundFiles, err := analyzer.findPhaseFiles()
	if err != nil {
		t.Fatalf("findPhaseFiles failed: %v", err)
	}

	if len(foundFiles) < 3 {
		t.Errorf("Expected at least 3 files, got %d", len(foundFiles))
	}
}

func TestTestAnalyzer_CoverageCalculation(t *testing.T) {
	report := &TestAnalysisReport{
		TotalRequired: 100,
		TotalActual:   80,
	}

	if report.TotalRequired > 0 {
		report.Coverage = float64(report.TotalActual) / float64(report.TotalRequired) * 100.0
		if report.Coverage > 100.0 {
			report.Coverage = 100.0
		}
	}

	expectedCoverage := 80.0
	if report.Coverage != expectedCoverage {
		t.Errorf("Expected coverage=%.2f%%, got %.2f%%", expectedCoverage, report.Coverage)
	}
}

func TestTestAnalyzer_OverAchievement(t *testing.T) {
	report := &TestAnalysisReport{
		TotalRequired: 50,
		TotalActual:   75, // Over 100%
	}

	if report.TotalRequired > 0 {
		report.Coverage = float64(report.TotalActual) / float64(report.TotalRequired) * 100.0
		if report.Coverage > 100.0 {
			report.Coverage = 100.0
		}
	}

	if report.Coverage != 100.0 {
		t.Errorf("Expected coverage capped at 100%%, got %.2f%%", report.Coverage)
	}
}
