package spec

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestParse(t *testing.T) {
	tests := []struct {
		name            string
		file            string
		wantTitle       string
		wantVersion     string
		wantStatus      string
		wantSections    int
		wantCodeBlocks  int
		wantGrammarRules int
		wantErr         bool
	}{
		{
			name:            "valid spec",
			file:            "sample_spec.md",
			wantTitle:       "Language Specification",
			wantVersion:     "1.0.0",
			wantStatus:      "Draft",
			wantSections:    2, // Syntax, Semantics (Introduction is level 2 but might be different)
			wantCodeBlocks:  4,
			wantGrammarRules: 6,
			wantErr:         false,
		},
		{
			name:         "minimal spec",
			file:         "minimal_spec.md",
			wantTitle:    "Minimal Spec",
			wantSections: 0,
			wantErr:      false,
		},
		{
			name:    "no title",
			file:    "no_title.md",
			wantErr: true,
		},
		{
			name:    "nonexistent file",
			file:    "nonexistent.md",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			path := filepath.Join("testdata", tt.file)
			spec, err := Parse(path)

			if tt.wantErr {
				if err == nil {
					t.Errorf("Parse() expected error, got nil")
				}
				return
			}

			if err != nil {
				t.Fatalf("Parse() unexpected error: %v", err)
			}

			if spec.Title != tt.wantTitle {
				t.Errorf("Title = %q, want %q", spec.Title, tt.wantTitle)
			}

			if spec.Version != tt.wantVersion {
				t.Errorf("Version = %q, want %q", spec.Version, tt.wantVersion)
			}

			if spec.Status != tt.wantStatus {
				t.Errorf("Status = %q, want %q", spec.Status, tt.wantStatus)
			}

			if len(spec.Sections) < tt.wantSections {
				t.Errorf("Sections count = %d, want at least %d", len(spec.Sections), tt.wantSections)
			}

			if len(spec.CodeBlocks) < tt.wantCodeBlocks {
				t.Errorf("CodeBlocks count = %d, want at least %d", len(spec.CodeBlocks), tt.wantCodeBlocks)
			}

			if len(spec.GrammarRules) < tt.wantGrammarRules {
				t.Errorf("GrammarRules count = %d, want at least %d", len(spec.GrammarRules), tt.wantGrammarRules)
			}
		})
	}
}

func TestParseDetailedSections(t *testing.T) {
	path := filepath.Join("testdata", "sample_spec.md")
	spec, err := Parse(path)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}

	// Check that sections have proper hierarchy
	syntaxSection := spec.FindSection("Syntax")
	if syntaxSection == nil {
		t.Fatal("Syntax section not found")
	}

	if syntaxSection.Level != 2 {
		t.Errorf("Syntax section level = %d, want 2", syntaxSection.Level)
	}

	if len(syntaxSection.Subsections) < 2 {
		t.Errorf("Syntax subsections count = %d, want at least 2 (Expressions, Statements)", len(syntaxSection.Subsections))
	}

	// Check subsection
	var expressionsSection *Section
	for _, sub := range syntaxSection.Subsections {
		if sub.Title == "Expressions" {
			expressionsSection = sub
			break
		}
	}

	if expressionsSection == nil {
		t.Fatal("Expressions subsection not found")
	}

	if expressionsSection.Level != 3 {
		t.Errorf("Expressions level = %d, want 3", expressionsSection.Level)
	}

	if len(expressionsSection.CodeBlocks) == 0 {
		t.Error("Expressions section should have code blocks")
	}
}

func TestParseCodeBlocks(t *testing.T) {
	path := filepath.Join("testdata", "sample_spec.md")
	spec, err := Parse(path)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}

	// Check that code blocks are assigned to sections
	foundEBNF := false
	foundRust := false

	for _, block := range spec.CodeBlocks {
		if block.Language == "ebnf" {
			foundEBNF = true
			if block.Section == "" {
				t.Error("EBNF code block should have section assigned")
			}
			if block.Code == "" {
				t.Error("EBNF code block should have code content")
			}
		}
		if block.Language == "rust" {
			foundRust = true
		}
	}

	if !foundEBNF {
		t.Error("Should find EBNF code blocks")
	}

	if !foundRust {
		t.Error("Should find Rust code blocks")
	}
}

func TestParseGrammarRules(t *testing.T) {
	path := filepath.Join("testdata", "sample_spec.md")
	spec, err := Parse(path)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}

	if len(spec.GrammarRules) == 0 {
		t.Fatal("Should extract grammar rules from EBNF blocks")
	}

	// Check for specific rules
	ruleNames := make(map[string]*GrammarRule)
	for _, rule := range spec.GrammarRules {
		ruleNames[rule.Name] = rule
		if rule.Definition == "" {
			t.Errorf("Rule %q has empty definition", rule.Name)
		}
	}

	expectedRules := []string{"expression", "term", "factor", "statement", "assignment"}
	for _, expected := range expectedRules {
		if _, found := ruleNames[expected]; !found {
			t.Errorf("Expected to find grammar rule: %q", expected)
		}
	}
}

func TestParseReferences(t *testing.T) {
	path := filepath.Join("testdata", "sample_spec.md")
	spec, err := Parse(path)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}

	if len(spec.References) == 0 {
		t.Error("Should extract references from markdown links")
	}

	// Check for specific references
	foundInternal := false
	foundExternal := false

	for _, ref := range spec.References {
		if strings.Contains(ref, "#expressions") {
			foundInternal = true
		}
		if strings.Contains(ref, "../other.md") {
			foundExternal = true
		}
	}

	if !foundInternal {
		t.Error("Should find internal reference (#expressions)")
	}

	if !foundExternal {
		t.Error("Should find external reference (../other.md)")
	}
}

func TestExtractValue(t *testing.T) {
	tests := []struct {
		name  string
		line  string
		label string
		want  string
	}{
		{
			name:  "version",
			line:  "**Version:** 1.0.0",
			label: "**Version:**",
			want:  "1.0.0",
		},
		{
			name:  "status",
			line:  "**Status:** Draft",
			label: "**Status:**",
			want:  "Draft",
		},
		{
			name:  "with extra whitespace",
			line:  "**Label:**   value  ",
			label: "**Label:**",
			want:  "value",
		},
		{
			name:  "label not found",
			line:  "Some text",
			label: "**Missing:**",
			want:  "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := extractValue(tt.line, tt.label)
			if got != tt.want {
				t.Errorf("extractValue() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestExtractReferences(t *testing.T) {
	tests := []struct {
		name  string
		line  string
		count int
		want  []string
	}{
		{
			name:  "single reference",
			line:  "See [link](path.md) for details",
			count: 1,
			want:  []string{"path.md"},
		},
		{
			name:  "multiple references",
			line:  "Check [one](a.md) and [two](b.md)",
			count: 2,
			want:  []string{"a.md", "b.md"},
		},
		{
			name:  "reference with anchor",
			line:  "See [section](#anchor)",
			count: 1,
			want:  []string{"#anchor"},
		},
		{
			name:  "no references",
			line:  "Just plain text",
			count: 0,
			want:  []string{},
		},
		{
			name:  "URL reference",
			line:  "Visit [site](https://example.com)",
			count: 1,
			want:  []string{"https://example.com"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := extractReferences(tt.line)
			if len(got) != tt.count {
				t.Errorf("extractReferences() count = %d, want %d", len(got), tt.count)
			}
			for i, want := range tt.want {
				if i < len(got) && got[i] != want {
					t.Errorf("extractReferences()[%d] = %q, want %q", i, got[i], want)
				}
			}
		})
	}
}

func TestParseGrammarRulesFunction(t *testing.T) {
	tests := []struct {
		name      string
		code      string
		wantCount int
		wantRules []string
	}{
		{
			name: "simple rules",
			code: `expression = term
term = factor`,
			wantCount: 2,
			wantRules: []string{"expression", "term"},
		},
		{
			name: "complex definition",
			code: `statement = "if" expression "{" statement "}"
assignment = identifier "=" expression ";"`,
			wantCount: 2,
			wantRules: []string{"statement", "assignment"},
		},
		{
			name:      "no rules",
			code:      "just some text\nno valid rules here",
			wantCount: 0,
		},
		{
			name: "rules with alternation",
			code: `operator = "+" | "-" | "*" | "/"`,
			wantCount: 1,
			wantRules: []string{"operator"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			rules := parseGrammarRules(tt.code, 1)
			if len(rules) != tt.wantCount {
				t.Errorf("parseGrammarRules() count = %d, want %d", len(rules), tt.wantCount)
			}

			for _, wantRule := range tt.wantRules {
				found := false
				for _, rule := range rules {
					if rule.Name == wantRule {
						found = true
						break
					}
				}
				if !found {
					t.Errorf("Expected rule %q not found", wantRule)
				}
			}
		})
	}
}

func TestFindSection(t *testing.T) {
	path := filepath.Join("testdata", "sample_spec.md")
	spec, err := Parse(path)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}

	tests := []struct {
		name    string
		title   string
		wantNil bool
	}{
		{
			name:    "existing section",
			title:   "Syntax",
			wantNil: false,
		},
		{
			name:    "case insensitive",
			title:   "syntax",
			wantNil: false,
		},
		{
			name:    "subsection",
			title:   "Expressions",
			wantNil: false,
		},
		{
			name:    "nonexistent",
			title:   "Nonexistent Section",
			wantNil: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			section := spec.FindSection(tt.title)
			if tt.wantNil && section != nil {
				t.Errorf("FindSection(%q) = %v, want nil", tt.title, section)
			}
			if !tt.wantNil && section == nil {
				t.Errorf("FindSection(%q) = nil, want non-nil", tt.title)
			}
		})
	}
}

func TestGetOutline(t *testing.T) {
	path := filepath.Join("testdata", "sample_spec.md")
	spec, err := Parse(path)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}

	outline := spec.GetOutline()
	if len(outline) == 0 {
		t.Error("GetOutline() should return non-empty outline")
	}

	// Check that outline contains expected sections
	outlineStr := strings.Join(outline, "\n")
	if !strings.Contains(outlineStr, "Syntax") {
		t.Error("Outline should contain Syntax section")
	}

	if !strings.Contains(outlineStr, "Expressions") {
		t.Error("Outline should contain Expressions subsection")
	}
}

func TestParseSectionHierarchy(t *testing.T) {
	tmpDir := t.TempDir()
	content := `# Test Spec
## Level 2A
### Level 3A
#### Level 4A
### Level 3B
## Level 2B
### Level 3C
`
	path := filepath.Join(tmpDir, "hierarchy.md")
	err := os.WriteFile(path, []byte(content), 0644)
	if err != nil {
		t.Fatal(err)
	}

	spec, err := Parse(path)
	if err != nil {
		t.Fatal(err)
	}

	// Should have 2 top-level sections (Level 2A, Level 2B)
	if len(spec.Sections) != 2 {
		t.Errorf("top-level sections count = %d, want 2", len(spec.Sections))
	}

	// Level 2A should have 2 subsections (3A, 3B)
	level2A := spec.FindSection("Level 2A")
	if level2A == nil {
		t.Fatal("Level 2A not found")
	}

	if len(level2A.Subsections) != 2 {
		t.Errorf("Level 2A subsections = %d, want 2", len(level2A.Subsections))
	}

	// Level 3A should have 1 subsection (4A)
	level3A := spec.FindSection("Level 3A")
	if level3A == nil {
		t.Fatal("Level 3A not found")
	}

	if len(level3A.Subsections) != 1 {
		t.Errorf("Level 3A subsections = %d, want 1", len(level3A.Subsections))
	}
}

func TestParseEdgeCases(t *testing.T) {
	tmpDir := t.TempDir()

	t.Run("multiple code blocks in section", func(t *testing.T) {
		content := `# Spec
## Section
` + "```rust\ncode1\n```\n```ebnf\nrule = x\n```\n"
		path := filepath.Join(tmpDir, "multi_code.md")
		err := os.WriteFile(path, []byte(content), 0644)
		if err != nil {
			t.Fatal(err)
		}

		spec, err := Parse(path)
		if err != nil {
			t.Fatal(err)
		}

		section := spec.FindSection("Section")
		if section == nil {
			t.Fatal("Section not found")
		}

		if len(section.CodeBlocks) != 2 {
			t.Errorf("section code blocks = %d, want 2", len(section.CodeBlocks))
		}
	})

	t.Run("code block without language tag", func(t *testing.T) {
		content := "# Spec\n## Section\n```\ncode\n```\n"
		path := filepath.Join(tmpDir, "no_lang.md")
		err := os.WriteFile(path, []byte(content), 0644)
		if err != nil {
			t.Fatal(err)
		}

		spec, err := Parse(path)
		if err != nil {
			t.Fatal(err)
		}

		if len(spec.CodeBlocks) != 1 {
			t.Fatalf("code blocks = %d, want 1", len(spec.CodeBlocks))
		}

		if spec.CodeBlocks[0].Language != "" {
			t.Errorf("code block language = %q, want empty", spec.CodeBlocks[0].Language)
		}
	})

	t.Run("empty lines in section", func(t *testing.T) {
		content := `# Spec
## Section

First paragraph.

Second paragraph.
`
		path := filepath.Join(tmpDir, "empty_lines.md")
		err := os.WriteFile(path, []byte(content), 0644)
		if err != nil {
			t.Fatal(err)
		}

		spec, err := Parse(path)
		if err != nil {
			t.Fatal(err)
		}

		section := spec.FindSection("Section")
		if section == nil {
			t.Fatal("Section not found")
		}

		if !strings.Contains(section.Content, "First paragraph") {
			t.Error("Section should contain first paragraph")
		}
		if !strings.Contains(section.Content, "Second paragraph") {
			t.Error("Section should contain second paragraph")
		}
	})
}

func TestParseMetadata(t *testing.T) {
	tmpDir := t.TempDir()
	content := `# Test Spec
**Version:** 2.0.0
**Status:** Final
---
## Section 1
Content here.
`
	path := filepath.Join(tmpDir, "metadata.md")
	err := os.WriteFile(path, []byte(content), 0644)
	if err != nil {
		t.Fatal(err)
	}

	spec, err := Parse(path)
	if err != nil {
		t.Fatal(err)
	}

	if spec.Version != "2.0.0" {
		t.Errorf("Version = %q, want %q", spec.Version, "2.0.0")
	}

	if spec.Status != "Final" {
		t.Errorf("Status = %q, want %q", spec.Status, "Final")
	}
}

func TestFindSectionRecursive(t *testing.T) {
	sections := []*Section{
		{
			Title: "Parent",
			Subsections: []*Section{
				{Title: "Child1"},
				{
					Title: "Child2",
					Subsections: []*Section{
						{Title: "GrandChild"},
					},
				},
			},
		},
	}

	tests := []struct {
		name    string
		title   string
		wantNil bool
	}{
		{"find parent", "parent", false},
		{"find child", "child1", false},
		{"find grandchild", "grandchild", false},
		{"not found", "missing", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := findSectionRecursive(sections, tt.title)
			if tt.wantNil && result != nil {
				t.Errorf("findSectionRecursive(%q) = %v, want nil", tt.title, result)
			}
			if !tt.wantNil && result == nil {
				t.Errorf("findSectionRecursive(%q) = nil, want non-nil", tt.title)
			}
		})
	}
}
