package parity

import (
	"os"
	"path/filepath"
	"testing"
)

func TestReferenceReport_ToCompactJSON(t *testing.T) {
	report := &ReferenceReport{
		TotalRefs:    100,
		ValidRefs:    95,
		BrokenRefs:   []BrokenReference{{}, {}, {}, {}, {}},
		OrphanedDocs: []string{"orphan1.md", "orphan2.md"},
	}

	result := report.ToCompactJSON()

	if total, ok := result["total"].(int); !ok || total != 100 {
		t.Errorf("Expected total=100, got %v", result["total"])
	}

	if valid, ok := result["valid"].(int); !ok || valid != 95 {
		t.Errorf("Expected valid=95, got %v", result["valid"])
	}

	if brokenCnt, ok := result["broken_cnt"].(int); !ok || brokenCnt != 5 {
		t.Errorf("Expected broken_cnt=5, got %v", result["broken_cnt"])
	}

	if orphanedCnt, ok := result["orphaned_cnt"].(int); !ok || orphanedCnt != 2 {
		t.Errorf("Expected orphaned_cnt=2, got %v", result["orphaned_cnt"])
	}
}

func TestReference_Structure(t *testing.T) {
	ref := Reference{
		SourceFile:   "README.md",
		SourceLine:   10,
		TargetPath:   "docs/guide.md",
		TargetAnchor: "getting-started",
		Text:         "Getting Started",
		Type:         "markdown",
	}

	if ref.SourceFile != "README.md" {
		t.Errorf("Expected SourceFile='README.md', got '%s'", ref.SourceFile)
	}

	if ref.SourceLine != 10 {
		t.Errorf("Expected SourceLine=10, got %d", ref.SourceLine)
	}

	if ref.TargetAnchor != "getting-started" {
		t.Errorf("Expected TargetAnchor='getting-started', got '%s'", ref.TargetAnchor)
	}

	if ref.Type != "markdown" {
		t.Errorf("Expected Type='markdown', got '%s'", ref.Type)
	}
}

func TestBrokenReference_Structure(t *testing.T) {
	broken := BrokenReference{
		Ref: Reference{
			SourceFile: "doc.md",
			TargetPath: "missing.md",
		},
		ErrorType:     "file_missing",
		FixSuggestion: "Create missing.md",
	}

	if broken.ErrorType != "file_missing" {
		t.Errorf("Expected ErrorType='file_missing', got '%s'", broken.ErrorType)
	}

	if broken.FixSuggestion == "" {
		t.Error("Expected non-empty FixSuggestion")
	}
}

func TestReferenceValidator_ExtractReferences(t *testing.T) {
	tempDir := t.TempDir()

	mdFile := filepath.Join(tempDir, "test.md")
	content := `# Test Document

This is a [link to guide](guide.md) and another [link](other.md#section).

External [link](https://example.com) should be ignored.

Reference to [specification](docs/specification/spec.md).
`
	if err := os.WriteFile(mdFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	validator := NewReferenceValidator(tempDir, "")
	refs, err := validator.extractReferences(mdFile)
	if err != nil {
		t.Fatalf("extractReferences failed: %v", err)
	}

	// Should find 3 internal references (excluding external URL)
	if len(refs) < 3 {
		t.Errorf("Expected at least 3 internal references, got %d", len(refs))
	}

	// Verify external URLs are skipped
	for _, ref := range refs {
		if ref.TargetPath == "https://example.com" {
			t.Error("External URL should not be included in references")
		}
	}

	// Check anchor parsing
	foundAnchor := false
	for _, ref := range refs {
		if ref.TargetAnchor == "section" {
			foundAnchor = true
			break
		}
	}
	if !foundAnchor {
		t.Error("Expected to find reference with anchor 'section'")
	}
}

func TestReferenceValidator_ValidateReference(t *testing.T) {
	tempDir := t.TempDir()

	// Create target file
	targetFile := filepath.Join(tempDir, "target.md")
	targetContent := `# Target Document

## Getting Started

Some content here.
`
	if err := os.WriteFile(targetFile, []byte(targetContent), 0644); err != nil {
		t.Fatal(err)
	}

	validator := NewReferenceValidator(tempDir, "")

	tests := []struct {
		name      string
		ref       Reference
		shouldErr bool
	}{
		{
			name: "valid reference",
			ref: Reference{
				SourceFile: filepath.Join(tempDir, "source.md"),
				TargetPath: "target.md",
			},
			shouldErr: false,
		},
		{
			name: "valid reference with anchor",
			ref: Reference{
				SourceFile:   filepath.Join(tempDir, "source.md"),
				TargetPath:   "target.md",
				TargetAnchor: "getting-started",
			},
			shouldErr: false,
		},
		{
			name: "missing file",
			ref: Reference{
				SourceFile: filepath.Join(tempDir, "source.md"),
				TargetPath: "missing.md",
			},
			shouldErr: true,
		},
		{
			name: "missing anchor",
			ref: Reference{
				SourceFile:   filepath.Join(tempDir, "source.md"),
				TargetPath:   "target.md",
				TargetAnchor: "nonexistent-section",
			},
			shouldErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := validator.validateReference(&tt.ref)
			if tt.shouldErr {
				if err == nil {
					t.Error("Expected error but got none")
				}
			} else {
				if err != nil {
					t.Errorf("Unexpected error: %v", err)
				}
			}
		})
	}
}

func TestReferenceValidator_ValidateAnchor(t *testing.T) {
	tempDir := t.TempDir()

	mdFile := filepath.Join(tempDir, "doc.md")
	content := `# Main Heading

## Getting Started

Some content.

### Sub Section

More content.
`
	if err := os.WriteFile(mdFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	validator := NewReferenceValidator(tempDir, "")

	tests := []struct {
		anchor    string
		shouldErr bool
	}{
		{"getting-started", false},
		{"sub-section", false},
		{"main-heading", false},
		{"nonexistent", true},
	}

	for _, tt := range tests {
		t.Run(tt.anchor, func(t *testing.T) {
			err := validator.validateAnchor(mdFile, tt.anchor)
			if tt.shouldErr {
				if err == nil {
					t.Errorf("Expected error for anchor '%s'", tt.anchor)
				}
			} else {
				if err != nil {
					t.Errorf("Unexpected error for anchor '%s': %v", tt.anchor, err)
				}
			}
		})
	}
}

func TestReferenceValidator_ClassifyError(t *testing.T) {
	validator := NewReferenceValidator("", "")

	tests := []struct {
		errMsg   string
		expected string
	}{
		{"target file not found", "file_missing"},
		{"anchor not found", "section_missing"},
		{"invalid format", "invalid_format"},
		{"some other error", "invalid_format"},
	}

	for _, tt := range tests {
		t.Run(tt.errMsg, func(t *testing.T) {
			err := &struct{ error }{error: os.ErrNotExist}
			if tt.errMsg != "" {
				err = &struct{ error }{error: &customError{msg: tt.errMsg}}
			}
			result := validator.classifyError(err.error)
			if result != tt.expected {
				t.Errorf("classifyError(%q) = %q, want %q", tt.errMsg, result, tt.expected)
			}
		})
	}
}

type customError struct {
	msg string
}

func (e *customError) Error() string {
	return e.msg
}

func TestReferenceValidator_GenerateFixSuggestion(t *testing.T) {
	validator := NewReferenceValidator("", "")

	ref := Reference{
		SourceFile:   "source.md",
		SourceLine:   10,
		TargetPath:   "target.md",
		TargetAnchor: "section",
	}

	tests := []struct {
		name     string
		errType  string
		contains string
	}{
		{
			name:     "file missing",
			errType:  "file_missing",
			contains: "Create missing file",
		},
		{
			name:     "section missing",
			errType:  "section_missing",
			contains: "Add section",
		},
		{
			name:     "other error",
			errType:  "invalid_format",
			contains: "Fix reference",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := &customError{msg: tt.errType + " error"}
			suggestion := validator.generateFixSuggestion(&ref, err)
			if suggestion == "" {
				t.Error("Expected non-empty suggestion")
			}
		})
	}
}

func TestReferenceValidator_FindOrphanedDocs(t *testing.T) {
	tempDir := t.TempDir()
	docsDir := filepath.Join(tempDir, "docs")
	if err := os.MkdirAll(docsDir, 0755); err != nil {
		t.Fatal(err)
	}

	// Create some docs
	referenced := filepath.Join(docsDir, "referenced.md")
	orphaned := filepath.Join(docsDir, "orphaned.md")
	readme := filepath.Join(docsDir, "README.md")

	for _, file := range []string{referenced, orphaned, readme} {
		if err := os.WriteFile(file, []byte("content"), 0644); err != nil {
			t.Fatal(err)
		}
	}

	validator := NewReferenceValidator(tempDir, docsDir)

	refs := []Reference{
		{TargetPath: "referenced.md", SourceFile: tempDir},
	}

	orphanedDocs := validator.findOrphanedDocs(refs)

	// Should find orphaned.md (README.md is typically an entry point)
	foundOrphaned := false
	foundReadme := false
	for _, doc := range orphanedDocs {
		if filepath.Base(doc) == "orphaned.md" {
			foundOrphaned = true
		}
		if filepath.Base(doc) == "README.md" || filepath.Base(doc) == "README.MD" {
			foundReadme = true
		}
	}

	if !foundOrphaned {
		t.Error("Expected to find orphaned.md in orphaned docs")
	}
	if foundReadme {
		t.Error("README.md should not be marked as orphaned")
	}
}

func TestReferenceValidator_ValidateReferences(t *testing.T) {
	tempDir := t.TempDir()

	// Create source file with references
	sourceFile := filepath.Join(tempDir, "source.md")
	sourceContent := `# Source
[Valid](target.md)
[Broken](missing.md)
`
	if err := os.WriteFile(sourceFile, []byte(sourceContent), 0644); err != nil {
		t.Fatal(err)
	}

	// Create target file
	targetFile := filepath.Join(tempDir, "target.md")
	if err := os.WriteFile(targetFile, []byte("# Target"), 0644); err != nil {
		t.Fatal(err)
	}

	validator := NewReferenceValidator(tempDir, "")
	report, err := validator.ValidateReferences()
	if err != nil {
		t.Fatalf("ValidateReferences failed: %v", err)
	}

	if report.TotalRefs < 2 {
		t.Errorf("Expected at least 2 references, got %d", report.TotalRefs)
	}

	if len(report.BrokenRefs) == 0 {
		t.Error("Expected to find broken references")
	}

	if report.ValidRefs == 0 {
		t.Error("Expected to find valid references")
	}
}

func TestReferenceValidator_ReferenceTypes(t *testing.T) {
	tempDir := t.TempDir()

	mdFile := filepath.Join(tempDir, "test.md")
	content := `
[Spec](specification/spec.md)
[API](api/functions.md)
[Phase](phase/phase-01.md)
[Other](other.md)
`
	if err := os.WriteFile(mdFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	validator := NewReferenceValidator(tempDir, "")
	refs, err := validator.extractReferences(mdFile)
	if err != nil {
		t.Fatal(err)
	}

	types := make(map[string]int)
	for _, ref := range refs {
		types[ref.Type]++
	}

	if types["spec"] == 0 {
		t.Error("Expected to find spec type reference")
	}
	if types["api"] == 0 {
		t.Error("Expected to find api type reference")
	}
	if types["phase"] == 0 {
		t.Error("Expected to find phase type reference")
	}
}
