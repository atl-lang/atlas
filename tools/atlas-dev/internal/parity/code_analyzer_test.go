package parity

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestCodeAnalyzer_AnalyzeCodebase(t *testing.T) {
	// Create temp directory with test Rust files
	tempDir := t.TempDir()

	// Create a test Rust file
	testFile := filepath.Join(tempDir, "test.rs")
	content := `
pub fn public_function(x: i32) -> i32 {
    x + 1
}

fn private_function() {
    println!("private");
}

pub struct PublicStruct {
    field: i32,
}

struct PrivateStruct {
    data: String,
}

pub enum Color {
    Red,
    Green,
    Blue,
}

pub trait Display {
    fn display(&self);
}

impl Display for PublicStruct {
    fn display(&self) {
        println!("{}", self.field);
    }
}

#[test]
fn test_addition() {
    assert_eq!(public_function(1), 2);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_private() {
        assert!(true);
    }
}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	// Analyze codebase
	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatalf("AnalyzeCodebase failed: %v", err)
	}

	// Verify results
	if analysis.TotalFiles != 1 {
		t.Errorf("Expected 1 file, got %d", analysis.TotalFiles)
	}

	// Check functions (should find public_function and private_function)
	if len(analysis.Functions) < 2 {
		t.Errorf("Expected at least 2 functions, got %d", len(analysis.Functions))
	}

	// Check structs (should find PublicStruct and PrivateStruct)
	if len(analysis.Structs) < 2 {
		t.Errorf("Expected at least 2 structs, got %d", len(analysis.Structs))
	}

	// Check enums (should find Color)
	if len(analysis.Enums) < 1 {
		t.Errorf("Expected at least 1 enum, got %d", len(analysis.Enums))
	}

	// Check traits (should find Display)
	if len(analysis.Traits) < 1 {
		t.Errorf("Expected at least 1 trait, got %d", len(analysis.Traits))
	}

	// Check impl blocks (should find impl Display for PublicStruct)
	if len(analysis.Impls) < 1 {
		t.Errorf("Expected at least 1 impl block, got %d", len(analysis.Impls))
	}

	// Check tests (should find both test functions)
	if len(analysis.Tests) < 2 {
		t.Errorf("Expected at least 2 tests, got %d", len(analysis.Tests))
	}
}

func TestCodeAnalyzer_PublicVisibility(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "visibility.rs")
	content := `
pub fn public_fn() {}
fn private_fn() {}
pub struct PublicStruct {}
struct PrivateStruct {}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Check public function
	publicFnCount := 0
	privateFnCount := 0
	for _, fn := range analysis.Functions {
		if fn.Public {
			publicFnCount++
		} else {
			privateFnCount++
		}
	}

	if publicFnCount < 1 {
		t.Error("Expected at least 1 public function")
	}
	if privateFnCount < 1 {
		t.Error("Expected at least 1 private function")
	}

	// Check public struct
	publicStructCount := 0
	privateStructCount := 0
	for _, s := range analysis.Structs {
		if s.Public {
			publicStructCount++
		} else {
			privateStructCount++
		}
	}

	if publicStructCount < 1 {
		t.Error("Expected at least 1 public struct")
	}
	if privateStructCount < 1 {
		t.Error("Expected at least 1 private struct")
	}
}

func TestCodeAnalyzer_Generics(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "generics.rs")
	content := `
pub fn generic_fn<T>(value: T) -> T {
    value
}

pub struct GenericStruct<T, U> {
    field1: T,
    field2: U,
}

pub enum Option<T> {
    Some(T),
    None,
}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Verify generic function was parsed
	found := false
	for _, fn := range analysis.Functions {
		if fn.Name == "generic_fn" {
			found = true
			if generics, ok := fn.Details["generics"].(string); ok && generics != "" {
				t.Logf("Found generics: %s", generics)
			}
		}
	}
	if !found {
		t.Error("Expected to find generic_fn")
	}

	// Verify generic struct was parsed
	found = false
	for _, s := range analysis.Structs {
		if s.Name == "GenericStruct" {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected to find GenericStruct")
	}
}

func TestCodeAnalyzer_SkipTargetDir(t *testing.T) {
	tempDir := t.TempDir()

	// Create a target directory (should be skipped)
	targetDir := filepath.Join(tempDir, "target")
	if err := os.MkdirAll(targetDir, 0755); err != nil {
		t.Fatal(err)
	}

	targetFile := filepath.Join(targetDir, "ignored.rs")
	if err := os.WriteFile(targetFile, []byte("pub fn ignored() {}"), 0644); err != nil {
		t.Fatal(err)
	}

	// Create a normal file
	normalFile := filepath.Join(tempDir, "normal.rs")
	if err := os.WriteFile(normalFile, []byte("pub fn normal() {}"), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Should only find 1 file (target dir should be skipped)
	if analysis.TotalFiles != 1 {
		t.Errorf("Expected 1 file (target dir should be skipped), got %d", analysis.TotalFiles)
	}
}

func TestCodeAnalyzer_ToCompactJSON(t *testing.T) {
	analysis := &CodeAnalysis{
		Functions:  make([]CodeItem, 5),
		Structs:    make([]CodeItem, 3),
		Enums:      make([]CodeItem, 2),
		Traits:     make([]CodeItem, 1),
		Tests:      make([]CodeItem, 10),
		TotalFiles: 7,
	}

	result := analysis.ToCompactJSON()

	tests := []struct {
		key      string
		expected int
	}{
		{"fn_cnt", 5},
		{"struct_cnt", 3},
		{"enum_cnt", 2},
		{"trait_cnt", 1},
		{"test_cnt", 10},
		{"file_cnt", 7},
	}

	for _, tt := range tests {
		if val, ok := result[tt.key].(int); !ok || val != tt.expected {
			t.Errorf("Expected %s=%d, got %v", tt.key, tt.expected, result[tt.key])
		}
	}
}

func TestCodeAnalyzer_Comments(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "comments.rs")
	content := `
// Single line comment
pub fn function1() {}

/* Multi-line comment
   should be ignored */
pub fn function2() {}

/*
 * Block comment
 * with multiple lines
 */
pub struct Commented {
    field: i32,
}

pub fn function3() {} // Inline comment
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Should find all 3 functions despite comments
	if len(analysis.Functions) < 3 {
		t.Errorf("Expected at least 3 functions, got %d", len(analysis.Functions))
	}

	// Should find the struct
	if len(analysis.Structs) < 1 {
		t.Errorf("Expected at least 1 struct, got %d", len(analysis.Structs))
	}
}

func TestCodeAnalyzer_Lifetimes(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "lifetimes.rs")
	content := `
pub fn with_lifetime<'a>(s: &'a str) -> &'a str {
    s
}

pub struct WithLifetime<'a> {
    data: &'a str,
}

pub trait WithLifetimeTrait<'a> {
    fn get_data(&self) -> &'a str;
}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Should parse functions with lifetimes
	if len(analysis.Functions) < 1 {
		t.Errorf("Expected at least 1 function with lifetime, got %d", len(analysis.Functions))
	}

	// Should parse structs with lifetimes
	if len(analysis.Structs) < 1 {
		t.Errorf("Expected at least 1 struct with lifetime, got %d", len(analysis.Structs))
	}

	// Should parse traits with lifetimes
	if len(analysis.Traits) < 1 {
		t.Errorf("Expected at least 1 trait with lifetime, got %d", len(analysis.Traits))
	}
}

func TestCodeAnalyzer_ComplexGenerics(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "complex_generics.rs")
	content := `
pub fn multi_generic<T, U, V>(a: T, b: U) -> V
where
    T: Clone,
    U: Display,
{
    unimplemented!()
}

pub struct Complex<'a, T: Clone, U> {
    field1: &'a T,
    field2: U,
}

pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T> Display for Complex<'_, T, String>
where
    T: Clone
{
    fn display(&self) {}
}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Should find complex generic function
	foundMultiGeneric := false
	for _, fn := range analysis.Functions {
		if fn.Name == "multi_generic" {
			foundMultiGeneric = true
			break
		}
	}
	if !foundMultiGeneric {
		t.Error("Expected to find multi_generic function")
	}

	// Should find generic struct
	if len(analysis.Structs) < 1 {
		t.Errorf("Expected at least 1 struct, got %d", len(analysis.Structs))
	}

	// Should find generic enum
	if len(analysis.Enums) < 1 {
		t.Errorf("Expected at least 1 enum, got %d", len(analysis.Enums))
	}

	// Should find impl block
	if len(analysis.Impls) < 1 {
		t.Errorf("Expected at least 1 impl block, got %d", len(analysis.Impls))
	}
}

func TestCodeAnalyzer_ImplBlocks(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "impls.rs")
	content := `
pub struct MyStruct {}

impl MyStruct {
    pub fn new() -> Self {
        MyStruct {}
    }
}

pub trait MyTrait {
    fn do_something(&self);
}

impl MyTrait for MyStruct {
    fn do_something(&self) {}
}

impl<T> MyTrait for Vec<T> {
    fn do_something(&self) {}
}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Should find at least 3 impl blocks
	if len(analysis.Impls) < 3 {
		t.Errorf("Expected at least 3 impl blocks, got %d", len(analysis.Impls))
	}

	// Verify impl naming
	foundTraitImpl := false
	for _, impl := range analysis.Impls {
		if impl.Name == "MyTrait for MyStruct" {
			foundTraitImpl = true
			break
		}
	}
	if !foundTraitImpl {
		t.Error("Expected to find 'MyTrait for MyStruct' impl")
	}
}

func TestCodeAnalyzer_ReturnTypes(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "returns.rs")
	content := `
pub fn returns_i32() -> i32 { 42 }
pub fn returns_tuple() -> (i32, String) { (1, String::new()) }
pub fn returns_result() -> Result<i32, String> { Ok(42) }
pub fn returns_option() -> Option<Vec<i32>> { None }
pub fn no_return() {}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	if len(analysis.Functions) < 5 {
		t.Errorf("Expected at least 5 functions, got %d", len(analysis.Functions))
	}

	// Check that return types are captured
	for _, fn := range analysis.Functions {
		if fn.Name == "returns_i32" {
			if returns, ok := fn.Details["returns"].(string); ok {
				if !strings.Contains(returns, "i32") {
					t.Errorf("Expected returns to contain 'i32', got '%s'", returns)
				}
			}
		}
	}
}

func TestCodeAnalyzer_ParseErrors(t *testing.T) {
	tempDir := t.TempDir()

	// Create a file that exists but can't be read properly
	testFile := filepath.Join(tempDir, "test.rs")
	if err := os.WriteFile(testFile, []byte("pub fn valid() {}"), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatalf("AnalyzeCodebase failed: %v", err)
	}

	// Should successfully parse valid file
	if len(analysis.ParseErrors) > 0 {
		t.Errorf("Expected no parse errors, got %d: %v", len(analysis.ParseErrors), analysis.ParseErrors)
	}
}

func TestCodeAnalyzer_MultipleFiles(t *testing.T) {
	tempDir := t.TempDir()

	// Create multiple Rust files
	files := map[string]string{
		"lib.rs":  "pub fn lib_fn() {}",
		"main.rs": "fn main() {}",
		"mod1.rs": "pub struct Mod1Struct {}",
		"mod2.rs": "pub enum Mod2Enum { A, B }",
	}

	for name, content := range files {
		path := filepath.Join(tempDir, name)
		if err := os.WriteFile(path, []byte(content), 0644); err != nil {
			t.Fatal(err)
		}
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Should find all 4 files
	if analysis.TotalFiles != 4 {
		t.Errorf("Expected 4 files, got %d", analysis.TotalFiles)
	}

	// Should find items from all files
	if len(analysis.Functions) < 2 {
		t.Errorf("Expected at least 2 functions, got %d", len(analysis.Functions))
	}
	if len(analysis.Structs) < 1 {
		t.Errorf("Expected at least 1 struct, got %d", len(analysis.Structs))
	}
	if len(analysis.Enums) < 1 {
		t.Errorf("Expected at least 1 enum, got %d", len(analysis.Enums))
	}
}

func TestCodeAnalyzer_TestDetection(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "tests.rs")
	content := `
#[test]
fn test_with_attribute() {
    assert!(true);
}

fn test_with_prefix() {
    assert!(true);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_in_mod() {
        assert!(true);
    }

    fn test_prefix_in_mod() {
        assert!(true);
    }
}

pub fn not_a_test() {}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Should detect tests
	if len(analysis.Tests) < 2 {
		t.Errorf("Expected at least 2 tests, got %d", len(analysis.Tests))
	}

	// not_a_test should be in Functions, not Tests
	foundInFunctions := false
	for _, fn := range analysis.Functions {
		if fn.Name == "not_a_test" {
			foundInFunctions = true
			break
		}
	}
	if !foundInFunctions {
		t.Error("Expected not_a_test in Functions")
	}
}
