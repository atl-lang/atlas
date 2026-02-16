package api

import (
	"os"
	"path/filepath"
	"testing"
)

func TestParse(t *testing.T) {
	tests := []struct {
		name          string
		file          string
		wantTitle     string
		wantFuncCount int
		wantErr       bool
	}{
		{
			name:          "valid API doc",
			file:          "sample_api.md",
			wantTitle:     "Sample API Documentation",
			wantFuncCount: 4,
			wantErr:       false,
		},
		{
			name:          "empty API",
			file:          "empty_api.md",
			wantTitle:     "Empty API",
			wantFuncCount: 0,
			wantErr:       false,
		},
		{
			name:          "malformed API",
			file:          "malformed_api.md",
			wantTitle:     "Malformed API",
			wantFuncCount: 2,
			wantErr:       false,
		},
		{
			name:    "nonexistent file",
			file:    "nonexistent.md",
			wantErr: true,
		},
		{
			name:    "no title",
			file:    "../spec/testdata/no_title.md",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			path := filepath.Join("testdata", tt.file)
			doc, err := Parse(path)

			if tt.wantErr {
				if err == nil {
					t.Errorf("Parse() expected error, got nil")
				}
				return
			}

			if err != nil {
				t.Fatalf("Parse() unexpected error: %v", err)
			}

			if doc.Title != tt.wantTitle {
				t.Errorf("Parse() title = %q, want %q", doc.Title, tt.wantTitle)
			}

			if len(doc.Functions) != tt.wantFuncCount {
				t.Errorf("Parse() function count = %d, want %d", len(doc.Functions), tt.wantFuncCount)
			}
		})
	}
}

func TestParseDetailedFunctions(t *testing.T) {
	path := filepath.Join("testdata", "sample_api.md")
	doc, err := Parse(path)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}

	tests := []struct {
		name          string
		wantSignature string
		wantDesc      bool
	}{
		{
			name:          "initialize",
			wantSignature: "pub fn initialize(config: Config) -> Result<()>",
			wantDesc:      true,
		},
		{
			name:          "process_data",
			wantSignature: "pub fn process_data(input: &str, options: ProcessOptions) -> ProcessResult",
			wantDesc:      true,
		},
		{
			name:          "validate_input",
			wantSignature: "pub fn validate_input(data: &[u8]) -> bool",
			wantDesc:      true,
		},
		{
			name:          "compute",
			wantSignature: "pub fn compute(x: f64, y: f64) -> f64",
			wantDesc:      false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fn := doc.FindFunction(tt.name)
			if fn == nil {
				t.Fatalf("FindFunction(%q) = nil, want function", tt.name)
			}

			if fn.Name != tt.name {
				t.Errorf("Function name = %q, want %q", fn.Name, tt.name)
			}

			if fn.Signature != tt.wantSignature {
				t.Errorf("Function signature = %q, want %q", fn.Signature, tt.wantSignature)
			}

			if tt.wantDesc && fn.Description == "" {
				t.Errorf("Function description is empty, want non-empty")
			}
		})
	}
}

func TestExtractSignature(t *testing.T) {
	tests := []struct {
		name string
		code string
		want string
	}{
		{
			name: "rust function",
			code: "pub fn test() -> Result<()>",
			want: "pub fn test() -> Result<()>",
		},
		{
			name: "multiline code",
			code: "let x = 5;\npub fn process(data: &str) -> String\nlet y = 10;",
			want: "pub fn process(data: &str) -> String",
		},
		{
			name: "go function",
			code: "func Calculate(x int) int",
			want: "func Calculate(x int) int",
		},
		{
			name: "javascript function",
			code: "function doSomething() { return 42; }",
			want: "function doSomething() { return 42; }",
		},
		{
			name: "no function",
			code: "let x = 5;\nlet y = 10;",
			want: "let x = 5;\nlet y = 10;",
		},
		{
			name: "empty code",
			code: "",
			want: "",
		},
		{
			name: "whitespace only",
			code: "   \n  \t  \n",
			want: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := extractSignature(tt.code)
			if got != tt.want {
				t.Errorf("extractSignature() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestAPIDocFindFunction(t *testing.T) {
	doc := &APIDoc{
		Functions: []*Function{
			{Name: "func1", Signature: "sig1"},
			{Name: "func2", Signature: "sig2"},
			{Name: "func3", Signature: "sig3"},
		},
	}

	tests := []struct {
		name     string
		funcName string
		wantNil  bool
	}{
		{
			name:     "existing function",
			funcName: "func1",
			wantNil:  false,
		},
		{
			name:     "another existing",
			funcName: "func3",
			wantNil:  false,
		},
		{
			name:     "nonexistent function",
			funcName: "func4",
			wantNil:  true,
		},
		{
			name:     "empty name",
			funcName: "",
			wantNil:  true,
		},
		{
			name:     "case sensitive",
			funcName: "Func1",
			wantNil:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := doc.FindFunction(tt.funcName)
			if tt.wantNil && got != nil {
				t.Errorf("FindFunction(%q) = %v, want nil", tt.funcName, got)
			}
			if !tt.wantNil && got == nil {
				t.Errorf("FindFunction(%q) = nil, want non-nil", tt.funcName)
			}
			if !tt.wantNil && got != nil && got.Name != tt.funcName {
				t.Errorf("FindFunction(%q).Name = %q, want %q", tt.funcName, got.Name, tt.funcName)
			}
		})
	}
}

func TestParseCodeBlocks(t *testing.T) {
	path := filepath.Join("testdata", "sample_api.md")
	doc, err := Parse(path)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}

	// Check that process_data has examples
	fn := doc.FindFunction("process_data")
	if fn == nil {
		t.Fatal("process_data function not found")
	}

	if len(fn.Examples) == 0 {
		t.Error("process_data should have examples, got none")
	}
}

func TestParseEdgeCases(t *testing.T) {
	// Create temp file with edge cases
	tmpDir := t.TempDir()

	t.Run("multiple headings levels", func(t *testing.T) {
		content := `# API Doc
## Function1
### SubFunction
#### DeepFunction
`
		path := filepath.Join(tmpDir, "multi_level.md")
		err := os.WriteFile(path, []byte(content), 0644)
		if err != nil {
			t.Fatal(err)
		}

		doc, err := Parse(path)
		if err != nil {
			t.Fatalf("Parse() error: %v", err)
		}

		// Parser treats ## and ### as function headings, but #### might not always be parsed
		if len(doc.Functions) < 2 {
			t.Errorf("expected at least 2 functions, got %d", len(doc.Functions))
		}
	})

	t.Run("backtick wrapped function name", func(t *testing.T) {
		content := "# API\n## `wrapped_name`\n"
		path := filepath.Join(tmpDir, "backtick.md")
		err := os.WriteFile(path, []byte(content), 0644)
		if err != nil {
			t.Fatal(err)
		}

		doc, err := Parse(path)
		if err != nil {
			t.Fatalf("Parse() error: %v", err)
		}

		fn := doc.FindFunction("wrapped_name")
		if fn == nil {
			t.Error("expected to find wrapped_name function")
		}
	})

	t.Run("code block without language", func(t *testing.T) {
		content := "# API\n## func1\n```\ncode here\n```\n"
		path := filepath.Join(tmpDir, "no_lang.md")
		err := os.WriteFile(path, []byte(content), 0644)
		if err != nil {
			t.Fatal(err)
		}

		doc, err := Parse(path)
		if err != nil {
			t.Fatalf("Parse() error: %v", err)
		}

		fn := doc.FindFunction("func1")
		if fn == nil {
			t.Fatal("func1 not found")
		}
		if fn.Signature == "" {
			t.Error("expected signature to be extracted from code block")
		}
	})
}

func TestEmptyAPIDoc(t *testing.T) {
	doc := &APIDoc{
		Functions: []*Function{},
	}

	fn := doc.FindFunction("anything")
	if fn != nil {
		t.Errorf("FindFunction on empty doc should return nil, got %v", fn)
	}
}
