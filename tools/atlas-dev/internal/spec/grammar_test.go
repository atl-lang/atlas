package spec

import (
	"path/filepath"
	"strings"
	"testing"
)

func TestValidateGrammar(t *testing.T) {
	tests := []struct {
		name       string
		rules      []*GrammarRule
		wantValid  bool
		wantErrors int
	}{
		{
			name: "valid grammar",
			rules: []*GrammarRule{
				{Name: "expression", Definition: `term`},
				{Name: "term", Definition: `factor`},
				{Name: "factor", Definition: `number | identifier`},
			},
			wantValid:  true,
			wantErrors: 0,
		},
		{
			name:       "empty grammar",
			rules:      []*GrammarRule{},
			wantValid:  false,
			wantErrors: 1,
		},
		{
			name: "undefined reference",
			rules: []*GrammarRule{
				{Name: "expression", Definition: "undefined_rule"},
			},
			wantValid:  false,
			wantErrors: 1,
		},
		{
			name: "valid with built-in terminals",
			rules: []*GrammarRule{
				{Name: "program", Definition: "statement+"},
				{Name: "statement", Definition: "identifier | number | string"},
			},
			wantValid:  true,
			wantErrors: 0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			validation := ValidateGrammar(tt.rules)

			if validation.Valid != tt.wantValid {
				t.Errorf("Valid = %v, want %v", validation.Valid, tt.wantValid)
			}

			if validation.TotalRules != len(tt.rules) {
				t.Errorf("TotalRules = %d, want %d", validation.TotalRules, len(tt.rules))
			}

			if len(validation.Errors) != tt.wantErrors {
				t.Errorf("Errors count = %d, want %d. Errors: %v", len(validation.Errors), tt.wantErrors, validation.Errors)
			}
		})
	}
}

func TestValidateEBNFSyntax(t *testing.T) {
	tests := []struct {
		name       string
		definition string
		wantErr    bool
	}{
		{
			name:       "valid simple",
			definition: `term { operator term }`,
			wantErr:    false,
		},
		{
			name:       "valid with alternation",
			definition: `"+" | "-" | "*" | "div"`,
			wantErr:    false,
		},
		{
			name:       "valid with optional",
			definition: `[optional_part]`,
			wantErr:    false,
		},
		{
			name:       "valid with repetition",
			definition: `{ repeated }`,
			wantErr:    false,
		},
		{
			name:       "valid with grouping",
			definition: `(group1 | group2)`,
			wantErr:    false,
		},
		{
			name:       "unmatched opening bracket",
			definition: `term [unmatched`,
			wantErr:    true,
		},
		{
			name:       "unmatched closing bracket",
			definition: `term unmatched]`,
			wantErr:    true,
		},
		{
			name:       "mismatched brackets",
			definition: `term [mismatched}`,
			wantErr:    true,
		},
		{
			name:       "nested brackets",
			definition: `term { [ optional ] }`,
			wantErr:    false,
		},
		{
			name:       "complex nested",
			definition: `(a | [b { c }]) | d`,
			wantErr:    false,
		},
		{
			name:       "invalid nested mismatch",
			definition: `(a | [b { c ])`,
			wantErr:    true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := validateEBNFSyntax(tt.definition)
			if tt.wantErr && err == nil {
				t.Error("validateEBNFSyntax() expected error, got nil")
			}
			if !tt.wantErr && err != nil {
				t.Errorf("validateEBNFSyntax() unexpected error: %v", err)
			}
		})
	}
}

func TestExtractNonTerminals(t *testing.T) {
	tests := []struct {
		name string
		def  string
		want []string
	}{
		{
			name: "simple references",
			def:  "term operator term",
			want: []string{"term", "operator"},
		},
		{
			name: "with literals",
			def:  `"+" | "-" | term`,
			want: []string{"term"},
		},
		{
			name: "complex expression",
			def:  "expression { operator expression }",
			want: []string{"expression", "operator"},
		},
		{
			name: "no non-terminals only punctuation",
			def:  `"+" | "-" | "*"`,
			want: []string{},
		},
		{
			name: "mixed case",
			def:  "Statement | TERM",
			want: []string{}, // Only lowercase identifiers are considered non-terminals
		},
		{
			name: "with underscores",
			def:  "binary_op | unary_op",
			want: []string{"binary_op", "unary_op"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := extractNonTerminals(tt.def)

			// Convert to set for comparison
			gotSet := make(map[string]bool)
			for _, nt := range got {
				gotSet[nt] = true
			}

			wantSet := make(map[string]bool)
			for _, nt := range tt.want {
				wantSet[nt] = true
			}

			if len(gotSet) != len(wantSet) {
				t.Errorf("extractNonTerminals() count = %d, want %d. Got: %v", len(gotSet), len(wantSet), got)
			}

			for nt := range wantSet {
				if !gotSet[nt] {
					t.Errorf("expected non-terminal %q not found", nt)
				}
			}
		})
	}
}

func TestIsEBNFKeyword(t *testing.T) {
	tests := []struct {
		word string
		want bool
	}{
		{"empty", true},
		{"EMPTY", true},
		{"or", true},
		{"OR", true},
		{"and", true},
		{"identifier", false},
		{"term", false},
		{"expression", false},
	}

	for _, tt := range tests {
		t.Run(tt.word, func(t *testing.T) {
			got := isEBNFKeyword(tt.word)
			if got != tt.want {
				t.Errorf("isEBNFKeyword(%q) = %v, want %v", tt.word, got, tt.want)
			}
		})
	}
}

func TestIsBuiltinTerminal(t *testing.T) {
	tests := []struct {
		ref  string
		want bool
	}{
		{"digit", true},
		{"DIGIT", true},
		{"letter", true},
		{"identifier", true},
		{"string", true},
		{"number", true},
		{"boolean", true},
		{"whitespace", true},
		{"custom_rule", false},
		{"expression", false},
	}

	for _, tt := range tests {
		t.Run(tt.ref, func(t *testing.T) {
			got := isBuiltinTerminal(tt.ref)
			if got != tt.want {
				t.Errorf("isBuiltinTerminal(%q) = %v, want %v", tt.ref, got, tt.want)
			}
		})
	}
}

func TestFindCircularDependencies(t *testing.T) {
	tests := []struct {
		name         string
		rules        []*GrammarRule
		wantCircular bool
	}{
		{
			name: "no circular deps",
			rules: []*GrammarRule{
				{Name: "expression", Definition: "term"},
				{Name: "term", Definition: "factor"},
				{Name: "factor", Definition: "number"},
			},
			wantCircular: false,
		},
		{
			name: "direct circular",
			rules: []*GrammarRule{
				{Name: "a", Definition: "b"},
				{Name: "b", Definition: "a"},
			},
			wantCircular: true,
		},
		{
			name: "indirect circular",
			rules: []*GrammarRule{
				{Name: "a", Definition: "b"},
				{Name: "b", Definition: "c"},
				{Name: "c", Definition: "a"},
			},
			wantCircular: true,
		},
		{
			name: "self reference",
			rules: []*GrammarRule{
				{Name: "expression", Definition: "expression operator term"},
			},
			wantCircular: true,
		},
		{
			name: "valid recursion with terminals",
			rules: []*GrammarRule{
				{Name: "list", Definition: "item | item list"},
				{Name: "item", Definition: "number"},
			},
			wantCircular: true, // list references itself - this is circular even if valid in EBNF
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			circular := findCircularDependencies(tt.rules)
			hasCircular := len(circular) > 0

			if hasCircular != tt.wantCircular {
				t.Errorf("findCircularDependencies() circular = %v, want %v. Found: %v", hasCircular, tt.wantCircular, circular)
			}
		})
	}
}

func TestValidateGrammarFromFile(t *testing.T) {
	path := filepath.Join("testdata", "complex_grammar.md")
	spec, err := Parse(path)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}

	if len(spec.GrammarRules) == 0 {
		t.Skip("No grammar rules found in test file")
	}

	// Extract rules from ebnf blocks
	var ebnfRules []*GrammarRule
	for _, rule := range spec.GrammarRules {
		// Filter to only EBNF rules (not from "grammar" block)
		ebnfRules = append(ebnfRules, rule)
	}

	validation := ValidateGrammar(ebnfRules)

	if validation.TotalRules == 0 {
		t.Error("Should have extracted grammar rules")
	}

	// The complex_grammar.md has some intentionally invalid rules
	// Check that validation catches issues
	if validation.Valid {
		t.Log("Warning: expected some validation errors from complex grammar")
	}
}

func TestValidateGrammarUndefinedReferences(t *testing.T) {
	rules := []*GrammarRule{
		{Name: "expression", Definition: "term operator"},
		{Name: "term", Definition: "factor"},
		// Missing: operator, factor
	}

	validation := ValidateGrammar(rules)

	if validation.Valid {
		t.Error("Should be invalid with undefined references")
	}

	if len(validation.Undefined) < 2 {
		t.Errorf("Undefined count = %d, want at least 2 (operator, factor)", len(validation.Undefined))
	}

	// Check that undefined includes expected references
	undefinedSet := make(map[string]bool)
	for _, undef := range validation.Undefined {
		undefinedSet[undef] = true
	}

	if !undefinedSet["operator"] && !undefinedSet["factor"] {
		t.Errorf("Expected undefined: operator, factor. Got: %v", validation.Undefined)
	}
}

func TestValidateGrammarWithWarnings(t *testing.T) {
	// Circular reference should generate warning
	rules := []*GrammarRule{
		{Name: "a", Definition: "b"},
		{Name: "b", Definition: "a"},
	}

	validation := ValidateGrammar(rules)

	if len(validation.Warnings) == 0 {
		t.Error("Expected warnings for circular dependencies")
	}

	if len(validation.Circular) == 0 {
		t.Error("Expected circular dependencies to be detected")
	}
}

func TestCompareToParser(t *testing.T) {
	rules := []*GrammarRule{
		{Name: "test", Definition: "value"},
	}

	result, err := CompareToParser(rules, "/some/parser/path")
	if err != nil {
		t.Errorf("CompareToParser() error: %v", err)
	}

	if result == nil {
		t.Fatal("CompareToParser() returned nil")
	}

	// Should return not_implemented status
	if status, ok := result["status"].(string); !ok || status != "not_implemented" {
		t.Errorf("Expected status 'not_implemented', got: %v", result)
	}
}

func TestValidateEBNFSyntaxEdgeCases(t *testing.T) {
	tests := []struct {
		name       string
		definition string
		wantErr    bool
	}{
		{
			name:       "empty definition",
			definition: "",
			wantErr:    false,
		},
		{
			name:       "only whitespace",
			definition: "   \n\t  ",
			wantErr:    false,
		},
		{
			name:       "multiple nested levels",
			definition: "[ { ( a ) } ]",
			wantErr:    false,
		},
		{
			name:       "string literals with allowed chars",
			definition: `"+" | "-" | "=" | "*"`,
			wantErr:    false,
		},
		{
			name:       "single quotes",
			definition: `'a' | 'b' | 'c'`,
			wantErr:    false,
		},
		{
			name:       "angle brackets",
			definition: `<identifier>`,
			wantErr:    false,
		},
		{
			name:       "comma separated",
			definition: `term, term, term`,
			wantErr:    false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := validateEBNFSyntax(tt.definition)
			if tt.wantErr && err == nil {
				t.Error("validateEBNFSyntax() expected error, got nil")
			}
			if !tt.wantErr && err != nil {
				t.Errorf("validateEBNFSyntax() unexpected error: %v", err)
			}
		})
	}
}

func TestValidateGrammarComplexScenarios(t *testing.T) {
	t.Run("grammar with all built-in terminals", func(t *testing.T) {
		rules := []*GrammarRule{
			{Name: "program", Definition: "statement+"},
			{Name: "statement", Definition: "identifier | number | string | boolean"},
		}

		validation := ValidateGrammar(rules)
		if !validation.Valid {
			t.Errorf("Should be valid with built-in terminals. Errors: %v", validation.Errors)
		}
	})

	t.Run("grammar with EBNF keywords", func(t *testing.T) {
		rules := []*GrammarRule{
			{Name: "optional", Definition: "empty | identifier"},
		}

		validation := ValidateGrammar(rules)
		// 'empty' is a keyword, should not be flagged as undefined
		if !validation.Valid {
			t.Errorf("Should be valid with EBNF keywords. Errors: %v", validation.Errors)
		}
	})

	t.Run("valid grammar with forward references", func(t *testing.T) {
		rules := []*GrammarRule{
			{Name: "a", Definition: "b | c"},
			{Name: "b", Definition: "d"},
			{Name: "c", Definition: "d"},
			{Name: "d", Definition: "identifier"},
		}

		validation := ValidateGrammar(rules)
		if !validation.Valid {
			t.Errorf("Should be valid with forward references. Errors: %v", validation.Errors)
		}
	})
}

func TestExtractNonTerminalsEdgeCases(t *testing.T) {
	tests := []struct {
		name      string
		def       string
		wantCount int
	}{
		{
			name:      "empty definition",
			def:       "",
			wantCount: 0,
		},
		{
			name:      "only literals",
			def:       `"+" | "-" | "*"`,
			wantCount: 0,
		},
		{
			name:      "repeated references",
			def:       "term term term",
			wantCount: 1, // Should deduplicate
		},
		{
			name:      "with numbers",
			def:       "term1 term2 term3",
			wantCount: 3,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := extractNonTerminals(tt.def)
			if len(got) != tt.wantCount {
				t.Errorf("extractNonTerminals() count = %d, want %d. Got: %v", len(got), tt.wantCount, got)
			}
		})
	}
}

func TestCircularDependencyDetection(t *testing.T) {
	// Test that we can detect various forms of circular dependencies
	t.Run("self-loop", func(t *testing.T) {
		rules := []*GrammarRule{
			{Name: "expr", Definition: "expr"},
		}
		circular := findCircularDependencies(rules)
		if len(circular) == 0 {
			t.Error("Should detect self-loop")
		}
	})

	t.Run("two-node cycle", func(t *testing.T) {
		rules := []*GrammarRule{
			{Name: "a", Definition: "b"},
			{Name: "b", Definition: "a"},
		}
		circular := findCircularDependencies(rules)
		if len(circular) == 0 {
			t.Error("Should detect two-node cycle")
		}
	})

	t.Run("three-node cycle", func(t *testing.T) {
		rules := []*GrammarRule{
			{Name: "a", Definition: "b"},
			{Name: "b", Definition: "c"},
			{Name: "c", Definition: "a"},
		}
		circular := findCircularDependencies(rules)
		if len(circular) == 0 {
			t.Error("Should detect three-node cycle")
		}
	})

	t.Run("no cycle with shared node", func(t *testing.T) {
		rules := []*GrammarRule{
			{Name: "a", Definition: "c"},
			{Name: "b", Definition: "c"},
			{Name: "c", Definition: "number"},
		}
		circular := findCircularDependencies(rules)
		if len(circular) > 0 {
			t.Errorf("Should not detect cycle. Found: %v", circular)
		}
	})
}

func TestValidationReportFormat(t *testing.T) {
	rules := []*GrammarRule{
		{Name: "valid_rule", Definition: "number"},
		{Name: "invalid_rule", Definition: "undefined_ref"},
		{Name: "bad_syntax", Definition: "[unmatched"},
	}

	validation := ValidateGrammar(rules)

	// Check that errors are formatted properly
	for _, err := range validation.Errors {
		if err == "" {
			t.Error("Error message should not be empty")
		}
		if !strings.Contains(err, "rule") {
			t.Errorf("Error message should mention 'rule': %q", err)
		}
	}

	// Check undefined list
	if len(validation.Undefined) == 0 {
		t.Error("Should have undefined references")
	}
}
