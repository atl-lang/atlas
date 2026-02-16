package api

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestGenerate(t *testing.T) {
	sourcePath := filepath.Join("testdata", "sample_code.rs")

	t.Run("generate to string", func(t *testing.T) {
		result, err := Generate(sourcePath, "")
		if err != nil {
			t.Fatalf("Generate() error: %v", err)
		}

		if result == nil {
			t.Fatal("Generate() returned nil result")
		}

		if result.FunctionCount != 5 {
			t.Errorf("FunctionCount = %d, want 5", result.FunctionCount)
		}

		if result.Generated == "" {
			t.Error("Generated markdown should not be empty")
		}

		if result.OutputPath != "" {
			t.Errorf("OutputPath should be empty when not writing to file, got %q", result.OutputPath)
		}

		// Check that markdown contains expected elements
		if !strings.Contains(result.Generated, "# API Documentation") {
			t.Error("Generated markdown missing title")
		}

		if !strings.Contains(result.Generated, "initialize") {
			t.Error("Generated markdown missing initialize function")
		}
	})

	t.Run("generate to file", func(t *testing.T) {
		tmpDir := t.TempDir()
		outputPath := filepath.Join(tmpDir, "output.md")

		result, err := Generate(sourcePath, outputPath)
		if err != nil {
			t.Fatalf("Generate() error: %v", err)
		}

		if result.OutputPath != outputPath {
			t.Errorf("OutputPath = %q, want %q", result.OutputPath, outputPath)
		}

		if result.Generated != "" {
			t.Error("Generated should be empty when writing to file")
		}

		// Verify file was created
		content, err := os.ReadFile(outputPath)
		if err != nil {
			t.Fatalf("failed to read output file: %v", err)
		}

		contentStr := string(content)
		if !strings.Contains(contentStr, "# API Documentation") {
			t.Error("Output file missing title")
		}

		if !strings.Contains(contentStr, "process_data") {
			t.Error("Output file missing process_data function")
		}
	})
}

func TestGenerateFromDirectory(t *testing.T) {
	tmpDir := t.TempDir()

	// Create multiple .rs files
	file1 := filepath.Join(tmpDir, "mod1.rs")
	err := os.WriteFile(file1, []byte("pub fn func1() {}\npub fn func2() -> String"), 0644)
	if err != nil {
		t.Fatal(err)
	}

	subdir := filepath.Join(tmpDir, "subdir")
	err = os.MkdirAll(subdir, 0755)
	if err != nil {
		t.Fatal(err)
	}

	file2 := filepath.Join(subdir, "mod2.rs")
	err = os.WriteFile(file2, []byte("pub fn func3() -> i32"), 0644)
	if err != nil {
		t.Fatal(err)
	}

	result, err := Generate(tmpDir, "")
	if err != nil {
		t.Fatalf("Generate() error: %v", err)
	}

	if result.FunctionCount != 3 {
		t.Errorf("FunctionCount = %d, want 3", result.FunctionCount)
	}

	// Check all functions are present
	for _, funcName := range []string{"func1", "func2", "func3"} {
		if !strings.Contains(result.Generated, funcName) {
			t.Errorf("Generated markdown missing function %q", funcName)
		}
	}
}

func TestGenerateNonexistentPath(t *testing.T) {
	_, err := Generate("/nonexistent/path", "")
	// Should not error due to filepath.Walk behavior
	if err != nil {
		t.Logf("Generate() with nonexistent path: %v", err)
	}
}

func TestGenerateInvalidOutputPath(t *testing.T) {
	sourcePath := filepath.Join("testdata", "sample_code.rs")
	invalidPath := "/nonexistent/directory/output.md"

	_, err := Generate(sourcePath, invalidPath)
	if err == nil {
		t.Error("Generate() should error with invalid output path")
	}
}

func TestExtractFunctionsWithDocs(t *testing.T) {
	tmpDir := t.TempDir()

	content := `
/// Initialize the system
/// This is a doc comment
pub fn initialize() -> Result<()> {
    Ok(())
}

/// Process data
pub fn process() {}

// Regular comment, not a doc comment
pub fn without_doc() {}

/// Doc comment for private function
fn private_func() {}
`
	path := filepath.Join(tmpDir, "test.rs")
	err := os.WriteFile(path, []byte(content), 0644)
	if err != nil {
		t.Fatal(err)
	}

	functions, err := extractFunctionsWithDocs(path)
	if err != nil {
		t.Fatalf("extractFunctionsWithDocs() error: %v", err)
	}

	// Should extract only pub functions
	expectedCount := 3 // initialize, process, without_doc
	if len(functions) != expectedCount {
		t.Errorf("extractFunctionsWithDocs() count = %d, want %d", len(functions), expectedCount)
	}

	// Verify function names
	funcNames := make(map[string]bool)
	for _, fn := range functions {
		funcNames[fn.name] = true
	}

	if !funcNames["initialize"] {
		t.Error("initialize should be extracted")
	}
	if !funcNames["process"] {
		t.Error("process should be extracted")
	}
	if !funcNames["without_doc"] {
		t.Error("without_doc should be extracted")
	}
	if funcNames["private_func"] {
		t.Error("private_func should not be extracted")
	}
}

func TestExtractFunctionsWithDocsEdgeCases(t *testing.T) {
	tmpDir := t.TempDir()

	t.Run("multiple doc comments", func(t *testing.T) {
		content := `
/// First line
/// Second line
/// Third line
pub fn documented() {}
`
		path := filepath.Join(tmpDir, "multi_doc.rs")
		err := os.WriteFile(path, []byte(content), 0644)
		if err != nil {
			t.Fatal(err)
		}

		functions, err := extractFunctionsWithDocs(path)
		if err != nil {
			t.Fatal(err)
		}

		if len(functions) != 1 {
			t.Errorf("expected 1 function, got %d", len(functions))
		}
	})

	t.Run("doc comment followed by non-pub fn", func(t *testing.T) {
		content := `
/// Doc comment
fn private() {}

pub fn public_after() {}
`
		path := filepath.Join(tmpDir, "doc_private.rs")
		err := os.WriteFile(path, []byte(content), 0644)
		if err != nil {
			t.Fatal(err)
		}

		functions, err := extractFunctionsWithDocs(path)
		if err != nil {
			t.Fatal(err)
		}

		// Should only find public_after
		if len(functions) != 1 {
			t.Errorf("expected 1 function, got %d", len(functions))
		}
		if functions[0].name != "public_after" {
			t.Errorf("expected public_after, got %s", functions[0].name)
		}
	})

	t.Run("empty file", func(t *testing.T) {
		path := filepath.Join(tmpDir, "empty.rs")
		err := os.WriteFile(path, []byte(""), 0644)
		if err != nil {
			t.Fatal(err)
		}

		functions, err := extractFunctionsWithDocs(path)
		if err != nil {
			t.Fatal(err)
		}

		if len(functions) != 0 {
			t.Errorf("expected 0 functions from empty file, got %d", len(functions))
		}
	})
}

func TestGenerateMarkdown(t *testing.T) {
	functions := []*rustFunction{
		{
			name:      "func1",
			signature: "pub fn func1() -> Result<()>",
		},
		{
			name:      "func2",
			signature: "pub fn func2(x: i32) -> i32",
		},
	}

	markdown := generateMarkdown(functions)

	// Check structure
	if !strings.Contains(markdown, "# API Documentation") {
		t.Error("Missing title")
	}

	if !strings.Contains(markdown, "Auto-generated from Rust source code") {
		t.Error("Missing description")
	}

	// Check functions
	if !strings.Contains(markdown, "## func1") {
		t.Error("Missing func1 heading")
	}

	if !strings.Contains(markdown, "## func2") {
		t.Error("Missing func2 heading")
	}

	// Check signatures
	if !strings.Contains(markdown, "pub fn func1() -> Result<()>") {
		t.Error("Missing func1 signature")
	}

	if !strings.Contains(markdown, "pub fn func2(x: i32) -> i32") {
		t.Error("Missing func2 signature")
	}

	// Check code blocks
	if !strings.Contains(markdown, "```rust") {
		t.Error("Missing rust code block markers")
	}

	// Check separators
	if strings.Count(markdown, "---") < 2 {
		t.Error("Missing section separators")
	}
}

func TestGenerateMarkdownEmpty(t *testing.T) {
	functions := []*rustFunction{}
	markdown := generateMarkdown(functions)

	if !strings.Contains(markdown, "# API Documentation") {
		t.Error("Should still have title for empty function list")
	}

	// Should not have any function headings
	if strings.Contains(markdown, "## ") {
		t.Error("Should not have function headings for empty list")
	}
}

func TestGenerateWithNonRustFiles(t *testing.T) {
	tmpDir := t.TempDir()

	// Create a mix of files
	err := os.WriteFile(filepath.Join(tmpDir, "code.rs"), []byte("pub fn rust_func() {}"), 0644)
	if err != nil {
		t.Fatal(err)
	}

	err = os.WriteFile(filepath.Join(tmpDir, "readme.md"), []byte("# README"), 0644)
	if err != nil {
		t.Fatal(err)
	}

	err = os.WriteFile(filepath.Join(tmpDir, "data.json"), []byte("{}"), 0644)
	if err != nil {
		t.Fatal(err)
	}

	result, err := Generate(tmpDir, "")
	if err != nil {
		t.Fatal(err)
	}

	// Should only extract from .rs files
	if result.FunctionCount != 1 {
		t.Errorf("FunctionCount = %d, want 1 (only from .rs file)", result.FunctionCount)
	}

	if !strings.Contains(result.Generated, "rust_func") {
		t.Error("Should contain rust_func")
	}
}

func TestExtractFunctionsWithDocsNonexistentFile(t *testing.T) {
	_, err := extractFunctionsWithDocs("/nonexistent/file.rs")
	if err == nil {
		t.Error("extractFunctionsWithDocs() should error with nonexistent file")
	}
}
