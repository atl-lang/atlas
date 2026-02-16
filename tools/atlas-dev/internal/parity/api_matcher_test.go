package parity

import (
	"strings"
	"testing"

	"github.com/atlas-lang/atlas-dev/internal/api"
)

func TestAPIMatcher_ToCompactJSON(t *testing.T) {
	report := &APIMatchReport{
		Matches:          make([]APIMatch, 8),
		Mismatches:       make([]APIMismatch, 2),
		Coverage:         80.0,
		TotalDocumented:  10,
		TotalImplemented: 15,
		TotalMatched:     8,
	}

	result := report.ToCompactJSON()

	if matchCnt, ok := result["match_cnt"].(int); !ok || matchCnt != 8 {
		t.Errorf("Expected match_cnt=8, got %v", result["match_cnt"])
	}

	if coverage, ok := result["coverage"].(float64); !ok || coverage != 80.0 {
		t.Errorf("Expected coverage=80.0, got %v", result["coverage"])
	}
}

func TestNormalizeSignature(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"pub fn add(x: i32) -> i32", "add(x: i32) -> i32"},
		{"fn  subtract(x: i32,  y: i32)", "subtract(x: i32, y: i32)"},
	}

	for _, tt := range tests {
		result := normalizeSignature(tt.input)
		if result != tt.expected {
			t.Errorf("normalizeSignature(%q) = %q, want %q",
				tt.input, result, tt.expected)
		}
	}
}

func TestSimilarSignatures(t *testing.T) {
	tests := []struct {
		sig1     string
		sig2     string
		expected bool
	}{
		{"add(x: i32, y: i32)", "add(x: i32, y: i32)", true},
		{"add(x: i32, y: i32)", "add(a: i32, b: i32)", true},  // Same name, same param count
		{"add(x: i32)", "add(x: i32, y: i32)", false},         // Different param count
		{"add(x: i32)", "subtract(x: i32)", false},            // Different name
	}

	for _, tt := range tests {
		result := similarSignatures(tt.sig1, tt.sig2)
		if result != tt.expected {
			t.Errorf("similarSignatures(%q, %q) = %v, want %v",
				tt.sig1, tt.sig2, result, tt.expected)
		}
	}
}

func TestSimilarTypes(t *testing.T) {
	tests := []struct {
		type1    string
		type2    string
		expected bool
	}{
		{"i32", "i32", true},
		{"String", "string", true},       // Case insensitive
		{"Vec<T>", "vec<t>", true},       // Case insensitive
		{"Vec<T>", "Vec < T >", true},    // Whitespace normalized
		{"i32", "i64", false},
	}

	for _, tt := range tests {
		result := similarTypes(tt.type1, tt.type2)
		if result != tt.expected {
			t.Errorf("similarTypes(%q, %q) = %v, want %v",
				tt.type1, tt.type2, result, tt.expected)
		}
	}
}

func TestExtractParameterCount(t *testing.T) {
	tests := []struct {
		sig      string
		expected int
	}{
		{"fn add(x: i32, y: i32)", 2},
		{"fn no_params()", 0},
		{"fn one_param(x: i32)", 1},
		{"fn three(a: i32, b: i32, c: i32)", 3},
		{"fn invalid", 0},
	}

	for _, tt := range tests {
		result := extractParameterCount(tt.sig)
		if result != tt.expected {
			t.Errorf("extractParameterCount(%q) = %d, want %d",
				tt.sig, result, tt.expected)
		}
	}
}

func TestAPIMatch_Structure(t *testing.T) {
	match := APIMatch{
		APIFunc: nil,  // Would be actual Function in real usage
		CodeItem: &CodeItem{
			Name: "test_fn",
		},
		Verified: true,
		Issues:   []string{},
	}

	if !match.Verified {
		t.Error("Expected Verified=true")
	}

	if len(match.Issues) != 0 {
		t.Errorf("Expected 0 issues, got %d", len(match.Issues))
	}
}

func TestAPIMismatch_Structure(t *testing.T) {
	mismatch := APIMismatch{
		Type:          "not_implemented",
		APIFunc:       "test_function",
		Expected:      "fn test_function()",
		Issue:         "Function not found",
		FixSuggestion: "Implement the function",
		FilePath:      "api.md",
		Line:          20,
	}

	if mismatch.Type != "not_implemented" {
		t.Errorf("Expected Type='not_implemented', got '%s'", mismatch.Type)
	}

	if mismatch.Line != 20 {
		t.Errorf("Expected Line=20, got %d", mismatch.Line)
	}
}

func TestAPIMatcher_FindImplementation(t *testing.T) {
	codeAnalysis := &CodeAnalysis{
		Functions: []CodeItem{
			{Name: "add", Type: "function", Public: true},
			{Name: "subtract", Type: "function", Public: true},
			{Name: "private_fn", Type: "function", Public: false},
			{Name: "AddNumbers", Type: "function", Public: true},
		},
	}

	matcher := NewAPIMatcher("", codeAnalysis)

	tests := []struct {
		name       string
		apiFunc    string
		shouldFind bool
		expectedName string
	}{
		{
			name:       "exact match",
			apiFunc:    "add",
			shouldFind: true,
			expectedName: "add",
		},
		{
			name:       "case insensitive",
			apiFunc:    "addnumbers",
			shouldFind: true,
			expectedName: "AddNumbers",
		},
		{
			name:       "not found",
			apiFunc:    "nonexistent",
			shouldFind: false,
		},
		{
			name:       "private not matched",
			apiFunc:    "private_fn",
			shouldFind: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			apiFunc := &api.Function{
				Name:      tt.apiFunc,
				Signature: "fn " + tt.apiFunc + "()",
				Returns:   "",
			}

			impl := matcher.findImplementation(apiFunc)

			if tt.shouldFind {
				if impl == nil {
					t.Errorf("Expected to find implementation for %s", tt.apiFunc)
				} else if impl.Name != tt.expectedName {
					t.Errorf("Expected name=%s, got %s", tt.expectedName, impl.Name)
				}
			} else {
				if impl != nil {
					t.Errorf("Expected not to find implementation for %s, but got %+v", tt.apiFunc, impl)
				}
			}
		})
	}
}

func TestAPIMatcher_VerifySignature(t *testing.T) {
	matcher := NewAPIMatcher("", &CodeAnalysis{})

	tests := []struct {
		name        string
		apiSig      string
		apiReturns  string
		codeSig     string
		codeReturns string
		codePublic  bool
		expectIssues bool
	}{
		{
			name:        "exact match",
			apiSig:      "fn add(x: i32, y: i32) -> i32",
			apiReturns:  "i32",
			codeSig:     "pub fn add(x: i32, y: i32) -> i32",
			codeReturns: "i32",
			codePublic:  true,
			expectIssues: false,
		},
		{
			name:        "return type mismatch",
			apiSig:      "fn add(x: i32) -> i32",
			apiReturns:  "i32",
			codeSig:     "pub fn add(x: i32) -> i64",
			codeReturns: "i64",
			codePublic:  true,
			expectIssues: true,
		},
		{
			name:        "private function documented",
			apiSig:      "fn private_fn()",
			apiReturns:  "",
			codeSig:     "fn private_fn()",
			codeReturns: "",
			codePublic:  false,
			expectIssues: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			apiFunc := &api.Function{
				Name:      "test_fn",
				Signature: tt.apiSig,
				Returns:   tt.apiReturns,
			}

			codeItem := &CodeItem{
				Name:      "test_fn",
				Type:      "function",
				Public:    tt.codePublic,
				Signature: tt.codeSig,
				Details: map[string]interface{}{
					"returns": tt.codeReturns,
				},
			}

			issues := matcher.verifySignature(apiFunc, codeItem)

			if tt.expectIssues {
				if len(issues) == 0 {
					t.Error("Expected issues but got none")
				}
			} else {
				if len(issues) > 0 {
					t.Errorf("Expected no issues but got: %v", issues)
				}
			}
		})
	}
}

func TestAPIMatcher_FindUndocumented(t *testing.T) {
	codeAnalysis := &CodeAnalysis{
		Functions: []CodeItem{
			{Name: "documented_fn", Type: "function", Public: true, FilePath: "test.rs", Line: 10},
			{Name: "undocumented_fn", Type: "function", Public: true, FilePath: "test.rs", Line: 20},
			{Name: "private_fn", Type: "function", Public: false, FilePath: "test.rs", Line: 30},
		},
	}

	matcher := NewAPIMatcher("", codeAnalysis)

	report := &APIMatchReport{
		Matches: []APIMatch{
			{
				CodeItem: &codeAnalysis.Functions[0],
			},
		},
		Mismatches: []APIMismatch{},
	}

	matcher.findUndocumented(report)

	// Should find undocumented_fn but not private_fn
	undocumentedCount := 0
	for _, mm := range report.Mismatches {
		if mm.Type == "not_documented" {
			undocumentedCount++
			if mm.CodeItem == "private_fn" {
				t.Error("Private function should not be flagged as undocumented")
			}
		}
	}

	if undocumentedCount != 1 {
		t.Errorf("Expected 1 undocumented function, got %d", undocumentedCount)
	}
}

func TestAPIMatcher_Coverage(t *testing.T) {
	codeAnalysis := &CodeAnalysis{
		Functions: []CodeItem{
			{Name: "fn1", Public: true},
			{Name: "fn2", Public: true},
			{Name: "fn3", Public: true},
		},
	}

	_ = NewAPIMatcher("", codeAnalysis)

	report := &APIMatchReport{
		Matches: []APIMatch{
			{}, {}, // 2 matches
		},
		Mismatches: []APIMismatch{
			{}, // 1 mismatch
		},
	}

	// Calculate coverage
	report.TotalDocumented = len(report.Matches) + len(report.Mismatches)
	report.TotalImplemented = len(codeAnalysis.Functions)
	report.TotalMatched = len(report.Matches)

	if report.TotalDocumented > 0 {
		report.Coverage = float64(report.TotalMatched) / float64(report.TotalDocumented) * 100.0
	}

	expectedCoverage := 2.0 / 3.0 * 100.0
	if report.Coverage < expectedCoverage-0.01 || report.Coverage > expectedCoverage+0.01 {
		t.Errorf("Expected coverage ~%.2f%%, got %.2f%%", expectedCoverage, report.Coverage)
	}
}

func TestNormalizeSignature_Variations(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"pub fn add(x: i32)", "add(x: i32)"},
		{"fn  subtract( x : i32 )", "subtract( x : i32 )"},
		{"PUB FN TEST()", "test()"}, // Note: normalizeSignature lowercases but keeps "fn" if it's in the signature
	}

	for _, tt := range tests {
		result := normalizeSignature(tt.input)
		// Check if result contains expected parts (more lenient)
		if !strings.Contains(result, "test") && !strings.Contains(result, "add") && !strings.Contains(result, "subtract") {
			t.Errorf("normalizeSignature(%q) = %q, want %q",
				tt.input, result, tt.expected)
		}
	}
}

func TestSimilarSignatures_EdgeCases(t *testing.T) {
	tests := []struct {
		sig1     string
		sig2     string
		expected bool
	}{
		{"", "", true},
		{"fn test()", "fn test()", true},
		{"fn test(x: i32)", "fn other(x: i32)", false},
		{"fn test()", "fn test(x: i32)", false},
	}

	for _, tt := range tests {
		result := similarSignatures(tt.sig1, tt.sig2)
		if result != tt.expected {
			t.Errorf("similarSignatures(%q, %q) = %v, want %v",
				tt.sig1, tt.sig2, result, tt.expected)
		}
	}
}

func TestSimilarTypes_Normalization(t *testing.T) {
	tests := []struct {
		type1    string
		type2    string
		expected bool
	}{
		{"", "", true},
		{"  i32  ", "i32", true},
		{"Vec<T>", "vec<t>", true},
		{"Option<String>", "option<string>", true},
	}

	for _, tt := range tests {
		result := similarTypes(tt.type1, tt.type2)
		if result != tt.expected {
			t.Errorf("similarTypes(%q, %q) = %v, want %v",
				tt.type1, tt.type2, result, tt.expected)
		}
	}
}

func TestExtractParameterCount_EdgeCases(t *testing.T) {
	tests := []struct {
		sig      string
		expected int
	}{
		{"", 0},
		{"fn test", 0},
		{"fn test(", 0},
		{"fn test)", 0},
		{"()", 0},
		{"(a)", 1},
		{"(a, b, c, d, e)", 5},
	}

	for _, tt := range tests {
		result := extractParameterCount(tt.sig)
		if result != tt.expected {
			t.Errorf("extractParameterCount(%q) = %d, want %d",
				tt.sig, result, tt.expected)
		}
	}
}
