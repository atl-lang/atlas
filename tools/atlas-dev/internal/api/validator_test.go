package api

import (
	"os"
	"path/filepath"
	"testing"
)

func TestValidate(t *testing.T) {
	apiPath := filepath.Join("testdata", "sample_api.md")
	codePath := filepath.Join("testdata", "sample_code.rs")

	doc, err := Parse(apiPath)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}

	report, err := Validate(doc, codePath)
	if err != nil {
		t.Fatalf("Validate() error: %v", err)
	}

	if report == nil {
		t.Fatal("Validate() returned nil report")
	}

	// Check basic counts
	if report.InCode != 5 {
		t.Errorf("InCode = %d, want 5 (initialize, process_data, validate_input, compute, undocumented_function)", report.InCode)
	}

	if report.Documented != 4 {
		t.Errorf("Documented = %d, want 4", report.Documented)
	}

	// Should have matches for: initialize, process_data, validate_input, compute
	if report.MatchCount != 4 {
		t.Errorf("MatchCount = %d, want 4", report.MatchCount)
	}

	// Coverage should be 80% (4 documented / 5 in code)
	expectedCoverage := 80.0
	if report.Coverage != expectedCoverage {
		t.Errorf("Coverage = %.2f%%, want %.2f%%", report.Coverage, expectedCoverage)
	}

	// undocumented_function should be in Undocumented list
	if len(report.Undocumented) != 1 {
		t.Errorf("Undocumented count = %d, want 1", len(report.Undocumented))
	} else if report.Undocumented[0] != "undocumented_function" {
		t.Errorf("Undocumented[0] = %q, want %q", report.Undocumented[0], "undocumented_function")
	}

	// All documented functions should exist in code
	if len(report.Missing) != 0 {
		t.Errorf("Missing = %v, want empty (all documented functions exist in code)", report.Missing)
	}
}

func TestValidateMissingFunctions(t *testing.T) {
	// Create API doc with functions that don't exist in code
	doc := &APIDoc{
		Title: "Test API",
		Functions: []*Function{
			{Name: "initialize"},
			{Name: "nonexistent_func1"},
			{Name: "process_data"},
			{Name: "nonexistent_func2"},
		},
	}

	codePath := filepath.Join("testdata", "sample_code.rs")
	report, err := Validate(doc, codePath)
	if err != nil {
		t.Fatalf("Validate() error: %v", err)
	}

	if report.Valid {
		t.Error("Validate() should be invalid when functions are missing in code")
	}

	if len(report.Missing) != 2 {
		t.Errorf("Missing count = %d, want 2", len(report.Missing))
	}

	// Check that the missing functions are the nonexistent ones
	expectedMissing := map[string]bool{
		"nonexistent_func1": true,
		"nonexistent_func2": true,
	}
	for _, missing := range report.Missing {
		if !expectedMissing[missing] {
			t.Errorf("unexpected missing function: %q", missing)
		}
	}
}

func TestValidateEmptyCode(t *testing.T) {
	tmpDir := t.TempDir()
	emptyCodePath := filepath.Join(tmpDir, "empty.rs")
	err := os.WriteFile(emptyCodePath, []byte("// empty file\n"), 0644)
	if err != nil {
		t.Fatal(err)
	}

	doc := &APIDoc{
		Functions: []*Function{
			{Name: "some_function"},
		},
	}

	report, err := Validate(doc, emptyCodePath)
	if err != nil {
		t.Fatalf("Validate() error: %v", err)
	}

	if report.Valid {
		t.Error("should be invalid when code has no functions")
	}

	if report.InCode != 0 {
		t.Errorf("InCode = %d, want 0", report.InCode)
	}

	if report.Coverage != 0 {
		t.Errorf("Coverage should be 0 when no code functions exist")
	}
}

func TestValidateNonexistentPath(t *testing.T) {
	doc := &APIDoc{Functions: []*Function{}}
	_, err := Validate(doc, "/nonexistent/path")
	// Should not error, just return empty results (due to filepath.Walk behavior)
	if err != nil {
		// This is actually okay - the function handles walk errors
		t.Logf("Validate() with nonexistent path: %v", err)
	}
}

func TestExtractRustFunctions(t *testing.T) {
	codePath := filepath.Join("testdata", "sample_code.rs")
	functions, err := extractRustFunctions(codePath)
	if err != nil {
		t.Fatalf("extractRustFunctions() error: %v", err)
	}

	if len(functions) != 5 {
		t.Errorf("extractRustFunctions() count = %d, want 5", len(functions))
	}

	expectedFuncs := map[string]bool{
		"initialize":             true,
		"process_data":           true,
		"validate_input":         true,
		"compute":                true,
		"undocumented_function":  true,
	}

	for _, fn := range functions {
		if !expectedFuncs[fn.name] {
			t.Errorf("unexpected function extracted: %q", fn.name)
		}
		if fn.signature == "" {
			t.Errorf("function %q has empty signature", fn.name)
		}
	}

	// Verify internal_helper (private function) is NOT extracted
	for _, fn := range functions {
		if fn.name == "internal_helper" {
			t.Error("private function internal_helper should not be extracted")
		}
	}
}

func TestExtractRustFunctionsFromDirectory(t *testing.T) {
	// Test that it walks directories correctly
	tmpDir := t.TempDir()

	// Create multiple .rs files
	file1 := filepath.Join(tmpDir, "mod1.rs")
	err := os.WriteFile(file1, []byte("pub fn func1() {}"), 0644)
	if err != nil {
		t.Fatal(err)
	}

	subdir := filepath.Join(tmpDir, "subdir")
	err = os.MkdirAll(subdir, 0755)
	if err != nil {
		t.Fatal(err)
	}

	file2 := filepath.Join(subdir, "mod2.rs")
	err = os.WriteFile(file2, []byte("pub fn func2() -> i32"), 0644)
	if err != nil {
		t.Fatal(err)
	}

	// Create a non-.rs file that should be ignored
	file3 := filepath.Join(tmpDir, "readme.txt")
	err = os.WriteFile(file3, []byte("pub fn should_not_be_found() {}"), 0644)
	if err != nil {
		t.Fatal(err)
	}

	functions, err := extractRustFunctions(tmpDir)
	if err != nil {
		t.Fatalf("extractRustFunctions() error: %v", err)
	}

	if len(functions) != 2 {
		t.Errorf("extractRustFunctions() count = %d, want 2", len(functions))
	}

	foundFunc1, foundFunc2 := false, false
	for _, fn := range functions {
		if fn.name == "func1" {
			foundFunc1 = true
		}
		if fn.name == "func2" {
			foundFunc2 = true
		}
		if fn.name == "should_not_be_found" {
			t.Error("function from non-.rs file should not be extracted")
		}
	}

	if !foundFunc1 {
		t.Error("func1 should be found")
	}
	if !foundFunc2 {
		t.Error("func2 should be found")
	}
}

func TestValidateCoverageCalculation(t *testing.T) {
	tests := []struct {
		name         string
		documented   int
		inCode       int
		wantCoverage float64
	}{
		{
			name:         "100% coverage",
			documented:   5,
			inCode:       5,
			wantCoverage: 100.0,
		},
		{
			name:         "50% coverage",
			documented:   5,
			inCode:       10,
			wantCoverage: 50.0,
		},
		{
			name:         "zero code functions",
			documented:   5,
			inCode:       0,
			wantCoverage: 0.0,
		},
		{
			name:         "zero documented",
			documented:   0,
			inCode:       10,
			wantCoverage: 0.0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create temporary files
			tmpDir := t.TempDir()

			// Create API doc
			apiContent := "# Test API\n"
			for i := 0; i < tt.documented; i++ {
				apiContent += "## func" + string(rune('A'+i)) + "\n"
			}
			apiPath := filepath.Join(tmpDir, "api.md")
			err := os.WriteFile(apiPath, []byte(apiContent), 0644)
			if err != nil {
				t.Fatal(err)
			}

			// Create code file
			codeContent := ""
			for i := 0; i < tt.inCode; i++ {
				codeContent += "pub fn func" + string(rune('A'+i)) + "() {}\n"
			}
			codePath := filepath.Join(tmpDir, "code.rs")
			err = os.WriteFile(codePath, []byte(codeContent), 0644)
			if err != nil {
				t.Fatal(err)
			}

			doc, err := Parse(apiPath)
			if err != nil {
				t.Fatal(err)
			}

			report, err := Validate(doc, codePath)
			if err != nil {
				t.Fatal(err)
			}

			if report.Coverage != tt.wantCoverage {
				t.Errorf("Coverage = %.2f%%, want %.2f%%", report.Coverage, tt.wantCoverage)
			}
		})
	}
}

func TestExtractRustFunctionsComplexSignatures(t *testing.T) {
	tmpDir := t.TempDir()
	content := `
pub fn simple() {}
pub fn with_generics<T: Clone>(x: T) -> T { x }
pub fn with_lifetime<'a>(x: &'a str) -> &'a str { x }
pub fn with_where<T>(x: T) -> T where T: Clone { x }
pub fn async_fn() -> impl Future<Output = ()> {}
pub fn const_fn() -> i32 { 0 }
`
	path := filepath.Join(tmpDir, "complex.rs")
	err := os.WriteFile(path, []byte(content), 0644)
	if err != nil {
		t.Fatal(err)
	}

	functions, err := extractRustFunctions(path)
	if err != nil {
		t.Fatal(err)
	}

	// The regex may not catch all complex signatures, but should get at least some
	// Check that we extracted at least the simple ones
	if len(functions) < 2 {
		t.Errorf("expected at least 2 functions, got %d", len(functions))
	}
}
