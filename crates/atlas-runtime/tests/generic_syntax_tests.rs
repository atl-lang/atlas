//! Generic type syntax parsing tests (BLOCKER 02-A)
//!
//! Tests parser support for generic type syntax: Type<T1, T2, ...>
//! This is phase 1 of 4 for generic types - just syntax and AST.
//! Type checking and inference will come in BLOCKER 02-B.

use atlas_runtime::ast::Program;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;

/// Helper to parse a source string and return the program
fn parse_source(source: &str) -> Result<Program, Vec<atlas_runtime::Diagnostic>> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();

    if !lex_diags.is_empty() {
        return Err(lex_diags);
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    if !parse_diags.is_empty() {
        return Err(parse_diags);
    }

    Ok(program)
}

// ============================================================================
// Basic Generic Type Syntax
// ============================================================================

#[test]
fn test_single_type_param() {
    let source = "let x: Option<number> = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_two_type_params() {
    let source = "let x: Result<number, string> = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_three_type_params() {
    let source = "let x: Map<string, number, bool> = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Nested Generic Types
// ============================================================================

#[test]
fn test_nested_single() {
    let source = "let x: Option<Result<T, E>> = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_nested_double() {
    let source = "let x: HashMap<string, Option<number>> = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_deeply_nested() {
    let source = "let x: Option<Result<Option<T>, E>> = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_complex_nesting() {
    let source = "let x: HashMap<string, Result<Option<T>, E>> = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Generic Types with Arrays
// ============================================================================

#[test]
fn test_generic_with_array_arg() {
    let source = "let x: Option<number[]> = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_generic_with_array_result() {
    let source = "let x: Result<string[], Error> = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_array_of_generic() {
    let source = "let x: Option<number>[] = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_generic_array_complex() {
    let source = "let x: HashMap<string, number[]>[] = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Generic Types in Function Signatures
// ============================================================================

#[test]
fn test_function_param_generic() {
    let source = "fn foo(x: Option<number>) -> void {}";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_function_return_generic() {
    let source = "fn bar() -> Result<number, string> { return null; }";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_function_both_generic() {
    let source = "fn baz(x: Option<T>) -> Result<T, E> { return null; }";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_function_multiple_generic_params() {
    let source = "fn test(a: Option<T>, b: Result<T, E>) -> HashMap<K, V> { return null; }";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_empty_type_args() {
    let source = "let x: Result<> = null;";
    let result = parse_source(source);
    assert!(result.is_err(), "Should fail with empty type args");
}

#[test]
fn test_missing_closing_bracket() {
    let source = "let x: Option<T = null;";
    let result = parse_source(source);
    assert!(result.is_err(), "Should fail with missing >");
}

#[test]
fn test_unterminated_multi_param() {
    let source = "let x: HashMap<K, V = null;";
    let result = parse_source(source);
    assert!(result.is_err(), "Should fail with unterminated type args");
}

#[test]
fn test_trailing_comma() {
    let source = "let x: Result<T, E,> = null;";
    let result = parse_source(source);
    // Trailing commas are currently not supported
    assert!(result.is_err(), "Trailing comma should cause error");
}

// ============================================================================
// AST Structure Verification
// ============================================================================

#[test]
fn test_ast_structure_simple() {
    let source = "let x: Result<number, string> = null;";
    let program = parse_source(source).unwrap();

    // Verify we have a variable declaration
    assert_eq!(program.items.len(), 1);

    // TODO: Add more detailed AST verification once we have helpers
}

#[test]
fn test_ast_structure_nested() {
    let source = "let x: Option<Result<T, E>> = null;";
    let program = parse_source(source).unwrap();

    // Verify parsing succeeded
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_ultra_nested() {
    let source = "let x: A<B<C<D<E<F<G<H<number>>>>>>>> = null;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
