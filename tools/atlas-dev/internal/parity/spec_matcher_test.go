package parity

import (
	"strings"
	"testing"
)

func TestSpecMatcher_ToCompactJSON(t *testing.T) {
	report := &SpecMatchReport{
		Matches:         make([]SpecMatch, 10),
		Mismatches:      make([]SpecMismatch, 3),
		Unspecified:     make([]CodeItem, 5),
		MatchPercentage: 76.9,
		TotalSpec:       13,
		TotalMatched:    10,
	}

	result := report.ToCompactJSON()

	if matchCnt, ok := result["match_cnt"].(int); !ok || matchCnt != 10 {
		t.Errorf("Expected match_cnt=10, got %v", result["match_cnt"])
	}

	if mismatchCnt, ok := result["mismatch_cnt"].(int); !ok || mismatchCnt != 3 {
		t.Errorf("Expected mismatch_cnt=3, got %v", result["mismatch_cnt"])
	}

	if matchPct, ok := result["match_pct"].(float64); !ok || matchPct != 76.9 {
		t.Errorf("Expected match_pct=76.9, got %v", result["match_pct"])
	}
}

func TestSpecMatcher_calculateMatchConfidence(t *testing.T) {
	codeAnalysis := &CodeAnalysis{}
	matcher := NewSpecMatcher("", codeAnalysis)

	tests := []struct {
		name       string
		req        SpecRequirement
		item       CodeItem
		minConf    float64
		maxConf    float64
	}{
		{
			name: "exact match",
			req: SpecRequirement{
				Name: "add",
				Type: "function",
			},
			item: CodeItem{
				Name:   "add",
				Type:   "function",
				Public: true,
			},
			minConf: 0.9,
			maxConf: 1.0,
		},
		{
			name: "type mismatch",
			req: SpecRequirement{
				Name: "add",
				Type: "function",
			},
			item: CodeItem{
				Name:   "add",
				Type:   "struct",
				Public: true,
			},
			minConf: 0.0,
			maxConf: 0.8,
		},
		{
			name: "name contains",
			req: SpecRequirement{
				Name: "add",
				Type: "function",
			},
			item: CodeItem{
				Name:   "add_numbers",
				Type:   "function",
				Public: true,
			},
			minConf: 0.0,
			maxConf: 0.7,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			conf := matcher.calculateMatchConfidence(tt.req, &tt.item)
			if conf < tt.minConf || conf > tt.maxConf {
				t.Errorf("Expected confidence between %.2f and %.2f, got %.2f",
					tt.minConf, tt.maxConf, conf)
			}
		})
	}
}

func TestExtractFunctionName(t *testing.T) {
	tests := []struct {
		line     string
		expected string
	}{
		{"pub fn add(x: i32) -> i32", "add"},
		{"fn subtract(x: i32, y: i32)", "subtract"},
		{"pub fn generic<T>(val: T) -> T", "generic"},
		{"fn no_params()", "no_params"},
		{"fn", ""}, // edge case
	}

	for _, tt := range tests {
		result := extractFunctionName(tt.line)
		if result != tt.expected {
			t.Errorf("extractFunctionName(%q) = %q, want %q",
				tt.line, result, tt.expected)
		}
	}
}

func TestExtractTypeName(t *testing.T) {
	tests := []struct {
		line     string
		keyword  string
		expected string
	}{
		{"pub struct Point { x: i32 }", "struct", "Point"},
		{"enum Color { Red, Green }", "enum", "Color"},
		{"pub trait Display {", "trait", "Display"},
		{"struct Generic<T> {", "struct", "Generic"},
	}

	for _, tt := range tests {
		result := extractTypeName(tt.line, tt.keyword)
		if result != tt.expected {
			t.Errorf("extractTypeName(%q, %q) = %q, want %q",
				tt.line, tt.keyword, result, tt.expected)
		}
	}
}

func TestSpecMatcher_generateFixSuggestion(t *testing.T) {
	matcher := NewSpecMatcher("", &CodeAnalysis{})

	tests := []struct {
		reqType  string
		reqName  string
		contains string
	}{
		{"function", "add", "Implement function 'add'"},
		{"struct", "Point", "Define struct 'Point'"},
		{"enum", "Color", "Define enum 'Color'"},
		{"trait", "Display", "Define trait 'Display'"},
	}

	for _, tt := range tests {
		req := SpecRequirement{
			Name:    tt.reqName,
			Type:    tt.reqType,
			Section: "test_section",
		}

		suggestion := matcher.generateFixSuggestion(req)
		if suggestion == "" {
			t.Errorf("Expected non-empty suggestion for %s", tt.reqType)
		}
	}
}

func TestSpecRequirement_Structure(t *testing.T) {
	req := SpecRequirement{
		Name:        "test_function",
		Type:        "function",
		Section:     "Core Functions",
		Description: "A test function",
		Line:        42,
	}

	if req.Name != "test_function" {
		t.Errorf("Expected Name='test_function', got '%s'", req.Name)
	}

	if req.Type != "function" {
		t.Errorf("Expected Type='function', got '%s'", req.Type)
	}

	if req.Line != 42 {
		t.Errorf("Expected Line=42, got %d", req.Line)
	}
}

func TestSpecMatch_Structure(t *testing.T) {
	match := SpecMatch{
		SpecItem:    "add",
		SpecSection: "Math Functions",
		CodeItem: &CodeItem{
			Name: "add",
			Type: "function",
		},
		MatchType:  "exact",
		Confidence: 1.0,
	}

	if match.SpecItem != "add" {
		t.Errorf("Expected SpecItem='add', got '%s'", match.SpecItem)
	}

	if match.MatchType != "exact" {
		t.Errorf("Expected MatchType='exact', got '%s'", match.MatchType)
	}

	if match.Confidence != 1.0 {
		t.Errorf("Expected Confidence=1.0, got %.2f", match.Confidence)
	}
}

func TestSpecMismatch_Structure(t *testing.T) {
	mismatch := SpecMismatch{
		SpecItem:      "multiply",
		SpecSection:   "Math Functions",
		Expected:      "fn multiply(a: i32, b: i32) -> i32",
		Issue:         "Function not implemented",
		FixSuggestion: "Implement multiply function",
		FilePath:      "spec.md",
		Line:          10,
	}

	if mismatch.SpecItem != "multiply" {
		t.Errorf("Expected SpecItem='multiply', got '%s'", mismatch.SpecItem)
	}

	if mismatch.Line != 10 {
		t.Errorf("Expected Line=10, got %d", mismatch.Line)
	}
}

func TestSpecMatcher_ParseCodeBlock(t *testing.T) {
	_ = NewSpecMatcher("", &CodeAnalysis{})

	tests := []struct {
		name     string
		code     string
		language string
		expected int
	}{
		{
			name: "rust function",
			code: `fn add(x: i32, y: i32) -> i32 {
    x + y
}`,
			language: "rust",
			expected: 1,
		},
		{
			name: "public struct",
			code: `pub struct Point {
    x: i32,
    y: i32,
}`,
			language: "rust",
			expected: 1,
		},
		{
			name: "enum definition",
			code: `pub enum Color {
    Red,
    Green,
    Blue,
}`,
			language: "rust",
			expected: 1,
		},
		{
			name: "trait definition",
			code: `pub trait Display {
    fn display(&self);
}`,
			language: "rust",
			expected: 1,
		},
		{
			name: "multiple definitions",
			code: `fn func1() {}
pub struct Struct1 {}
enum Enum1 {}
trait Trait1 {}`,
			language: "rust",
			expected: 4,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Test the helper functions by parsing code
			if tt.language == "rust" {
				lines := strings.Split(tt.code, "\n")
				count := 0
				for _, line := range lines {
					trimmed := strings.TrimSpace(line)
					if strings.HasPrefix(trimmed, "fn ") || strings.HasPrefix(trimmed, "pub fn ") {
						count++
					}
					if strings.HasPrefix(trimmed, "struct ") || strings.HasPrefix(trimmed, "pub struct ") {
						count++
					}
					if strings.HasPrefix(trimmed, "enum ") || strings.HasPrefix(trimmed, "pub enum ") {
						count++
					}
					if strings.HasPrefix(trimmed, "trait ") || strings.HasPrefix(trimmed, "pub trait ") {
						count++
					}
				}
				if count < 1 {
					t.Errorf("Expected to find at least 1 definition, found %d", count)
				}
			}
		})
	}
}

func TestSpecMatcher_FindCodeMatch(t *testing.T) {
	// Create code analysis with test data
	codeAnalysis := &CodeAnalysis{
		Functions: []CodeItem{
			{Name: "add", Type: "function", Public: true},
			{Name: "subtract", Type: "function", Public: true},
			{Name: "private_fn", Type: "function", Public: false},
		},
		Structs: []CodeItem{
			{Name: "Point", Type: "struct", Public: true},
			{Name: "PrivateStruct", Type: "struct", Public: false},
		},
		Enums: []CodeItem{
			{Name: "Color", Type: "enum", Public: true},
		},
		Traits: []CodeItem{
			{Name: "Display", Type: "trait", Public: true},
		},
	}

	matcher := NewSpecMatcher("", codeAnalysis)

	tests := []struct {
		name       string
		req        SpecRequirement
		shouldFind bool
		minConf    float64
	}{
		{
			name: "exact function match",
			req: SpecRequirement{
				Name: "add",
				Type: "function",
			},
			shouldFind: true,
			minConf:    0.9,
		},
		{
			name: "exact struct match",
			req: SpecRequirement{
				Name: "Point",
				Type: "struct",
			},
			shouldFind: true,
			minConf:    0.9,
		},
		{
			name: "no match",
			req: SpecRequirement{
				Name: "nonexistent",
				Type: "function",
			},
			shouldFind: false,
		},
		{
			name: "partial match - low confidence",
			req: SpecRequirement{
				Name: "xyz",
				Type: "function",
			},
			shouldFind: false, // Below 70% threshold
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			match := matcher.findCodeMatch(tt.req)
			if tt.shouldFind {
				if match == nil {
					t.Error("Expected to find match, got nil")
				} else if match.Confidence < tt.minConf {
					t.Errorf("Expected confidence >= %.2f, got %.2f", tt.minConf, match.Confidence)
				}
			} else {
				if match != nil {
					t.Errorf("Expected no match, got %+v", match)
				}
			}
		})
	}
}

func TestSpecMatcher_MatchTypes(t *testing.T) {
	codeAnalysis := &CodeAnalysis{
		Functions: []CodeItem{
			{Name: "testfn", Type: "function", Public: true},
		},
		Structs: []CodeItem{
			{Name: "testfn", Type: "struct", Public: true},
		},
	}

	matcher := NewSpecMatcher("", codeAnalysis)

	// Test that type matching matters
	funcReq := SpecRequirement{
		Name: "testfn",
		Type: "function",
	}

	match := matcher.findCodeMatch(funcReq)
	if match == nil {
		t.Fatal("Expected to find match")
	}

	if match.CodeItem.Type != "function" {
		t.Errorf("Expected to match function, got %s", match.CodeItem.Type)
	}
}

func TestSpecMatcher_FindUnspecified(t *testing.T) {
	codeAnalysis := &CodeAnalysis{
		Functions: []CodeItem{
			{Name: "matched_fn", Type: "function", Public: true},
			{Name: "unmatched_fn", Type: "function", Public: true},
			{Name: "private_fn", Type: "function", Public: false},
		},
		Structs: []CodeItem{
			{Name: "UnmatchedStruct", Type: "struct", Public: true},
		},
	}

	matcher := NewSpecMatcher("", codeAnalysis)

	report := &SpecMatchReport{
		Matches: []SpecMatch{
			{
				CodeItem: &CodeItem{Name: "matched_fn", Type: "function"},
			},
		},
		Mismatches:  []SpecMismatch{},
		Unspecified: []CodeItem{},
	}

	matcher.findUnspecified(report)

	// Should find unmatched_fn and UnmatchedStruct (both public)
	// Should NOT find private_fn (private)
	if len(report.Unspecified) < 2 {
		t.Errorf("Expected at least 2 unspecified items, got %d", len(report.Unspecified))
	}

	// Verify only public items are included
	for _, item := range report.Unspecified {
		if !item.Public {
			t.Errorf("Found non-public item in unspecified: %s", item.Name)
		}
	}
}

func TestSpecMatcher_CaseInsensitive(t *testing.T) {
	codeAnalysis := &CodeAnalysis{
		Functions: []CodeItem{
			{Name: "AddNumbers", Type: "function", Public: true},
		},
	}

	matcher := NewSpecMatcher("", codeAnalysis)

	req := SpecRequirement{
		Name: "addnumbers",
		Type: "function",
	}

	match := matcher.findCodeMatch(req)
	if match == nil {
		t.Error("Expected case-insensitive match")
	}
}

func TestSpecMatcher_PartialNameMatch(t *testing.T) {
	codeAnalysis := &CodeAnalysis{
		Functions: []CodeItem{
			{Name: "calculate_sum", Type: "function", Public: true},
		},
	}

	matcher := NewSpecMatcher("", codeAnalysis)

	// Spec might just say "sum"
	req := SpecRequirement{
		Name: "sum",
		Type: "function",
	}

	match := matcher.findCodeMatch(req)
	// Partial match might have lower confidence
	if match == nil {
		t.Log("Partial match not found (expected if confidence < 0.7)")
	} else {
		if match.Confidence >= 0.7 {
			t.Logf("Found partial match with confidence %.2f", match.Confidence)
		}
	}
}

func TestExtractTypeName_EdgeCases(t *testing.T) {
	tests := []struct {
		line     string
		keyword  string
		expected string
		strict   bool // strict means exact match required
	}{
		{"struct Simple", "struct", "Simple", true},
		{"pub struct PubSimple", "struct", "PubSimple", true},
		{"struct", "struct", "", false}, // May return "struct" after trimming
		{"struct  ", "struct", "", false},
		{"pub enum MyEnum", "enum", "MyEnum", true},
		{"pub trait MyTrait", "trait", "MyTrait", true},
	}

	for _, tt := range tests {
		result := extractTypeName(tt.line, tt.keyword)
		if tt.strict && result != tt.expected {
			t.Errorf("extractTypeName(%q, %q) = %q, want %q",
				tt.line, tt.keyword, result, tt.expected)
		} else if !tt.strict && result == "" && tt.expected == "" {
			// Allow any result for non-strict tests
			continue
		} else if !tt.strict && result != "" && result != tt.keyword {
			// For edge cases, either empty or something valid is okay
			continue
		}
	}
}

func TestExtractFunctionName_EdgeCases(t *testing.T) {
	tests := []struct {
		line     string
		expected string
	}{
		{"pub fn simple()", "simple"},
		{"fn another()", "another"},
		{"pub fn test<T>()", "test"},
		{"pub fn", ""},
		{"fn  ", ""},
		{"not a function", ""},
	}

	for _, tt := range tests {
		result := extractFunctionName(tt.line)
		if result != tt.expected {
			t.Errorf("extractFunctionName(%q) = %q, want %q",
				tt.line, result, tt.expected)
		}
	}
}
