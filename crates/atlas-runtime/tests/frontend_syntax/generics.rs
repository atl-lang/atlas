//! Generic syntax tests (lines 1264-1267, 1285-1670 from original frontend_syntax.rs)

use super::*;

// Generic Syntax Tests (from generic_syntax_tests.rs)
// ============================================================================

/// Helper to parse a source string and return the program

// ============================================================================
// Basic Generic Type Syntax
// ============================================================================

#[test]
fn test_single_type_param() {
    let source = "let x: Option<number> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_two_type_params() {
    let source = "let x: Result<number, string> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_three_type_params() {
    let source = "let x: Map<string, number, bool> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Nested Generic Types
// ============================================================================

#[test]
fn test_nested_single() {
    let source = "let x: Option<Result<T, E>> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_nested_double() {
    let source = "let x: HashMap<string, Option<number>> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_deeply_nested() {
    let source = "let x: Option<Result<Option<T>, E>> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_complex_nesting() {
    let source = "let x: HashMap<string, Result<Option<T>, E>> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Generic Types with Arrays
// ============================================================================

#[test]
fn test_generic_with_array_arg() {
    let source = "let x: Option<number[]> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_generic_with_array_result() {
    let source = "let x: Result<string[], Error> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_array_of_generic() {
    let source = "let x: Option<number>[] = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_generic_array_complex() {
    let source = "let x: HashMap<string, number[]>[] = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Generic Types in Function Signatures
// ============================================================================

#[test]
fn test_function_param_generic() {
    let source = "fn foo(x: Option<number>) -> void {}";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_function_return_generic() {
    let source = "fn bar() -> Result<number, string> { return null; }";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_function_both_generic() {
    let source = "fn baz(x: Option<T>) -> Result<T, E> { return null; }";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_function_multiple_generic_params() {
    let source = "fn test(a: Option<T>, b: Result<T, E>) -> HashMap<K, V> { return null; }";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_empty_type_args() {
    let source = "let x: Result<> = null;";
    let result = try_parse(source);
    assert!(result.is_err(), "Should fail with empty type args");
}

#[test]
fn test_missing_closing_bracket() {
    let source = "let x: Option<T = null;";
    let result = try_parse(source);
    assert!(result.is_err(), "Should fail with missing >");
}

#[test]
fn test_unterminated_multi_param() {
    let source = "let x: HashMap<K, V = null;";
    let result = try_parse(source);
    assert!(result.is_err(), "Should fail with unterminated type args");
}

#[test]
fn test_trailing_comma() {
    let source = "let x: Result<T, E,> = null;";
    let result = try_parse(source);
    // Trailing commas are currently not supported
    assert!(result.is_err(), "Trailing comma should cause error");
}

// ============================================================================
// AST Structure Verification
// ============================================================================

#[test]
fn test_ast_structure_simple() {
    let source = "let x: Result<number, string> = null;";
    let program = try_parse(source).unwrap();

    // Verify we have a variable declaration
    assert_eq!(program.items.len(), 1);

    // TODO: Add more detailed AST verification once we have helpers
}

#[test]
fn test_ast_structure_nested() {
    let source = "let x: Option<Result<T, E>> = null;";
    let program = try_parse(source).unwrap();

    // Verify parsing succeeded
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_ultra_nested() {
    let source = "let x: A<B<C<D<E<F<G<H<number>>>>>>>> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Module Syntax Tests (from module_syntax_tests.rs)
// ============================================================================

/// Helper to parse code and check for errors
fn parse(source: &str) -> (bool, Vec<String>) {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return (false, lex_diags.iter().map(|d| d.message.clone()).collect());
    }

    let mut parser = Parser::new(tokens);
    let (_, parse_diags) = parser.parse();

    let success = parse_diags.is_empty();
    let messages = parse_diags.iter().map(|d| d.message.clone()).collect();
    (success, messages)
}

// ============================================================================
// Import Syntax Tests
// ============================================================================

#[test]
fn test_parse_named_import_single() {
    let source = r#"import { add } from "./math";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse single named import: {:?}", msgs);
}

#[test]
fn test_parse_named_import_multiple() {
    let source = r#"import { add, sub, mul } from "./math";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse multiple named imports: {:?}", msgs);
}

#[test]
fn test_parse_namespace_import() {
    let source = r#"import * as math from "./math";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse namespace import: {:?}", msgs);
}

#[test]
fn test_parse_import_relative_path() {
    let source = r#"import { x } from "./sibling";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse relative path: {:?}", msgs);
}

#[test]
fn test_parse_import_parent_path() {
    let source = r#"import { x } from "../parent";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse parent path: {:?}", msgs);
}

#[test]
fn test_parse_import_absolute_path() {
    let source = r#"import { x } from "/src/utils";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse absolute path: {:?}", msgs);
}

#[test]
fn test_parse_import_with_extension() {
    let source = r#"import { x } from "./mod.atl";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse path with .atl extension: {:?}", msgs);
}

#[test]
fn test_parse_multiple_imports() {
    let source = r#"
        import { add } from "./math";
        import { log } from "./logger";
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse multiple imports: {:?}", msgs);
}

// ============================================================================
// Export Syntax Tests
// ============================================================================

#[test]
fn test_parse_export_function() {
    let source = r#"
        export fn add(a: number, b: number) -> number {
            return a + b;
        }
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export function: {:?}", msgs);
}

#[test]
fn test_parse_export_let() {
    let source = r#"export let PI = 3.14159;"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export let: {:?}", msgs);
}

#[test]
fn test_parse_export_var() {
    let source = r#"export var counter = 0;"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export var: {:?}", msgs);
}

#[test]
fn test_parse_export_generic_function() {
    let source = r#"
        export fn identity<T>(x: T) -> T {
            return x;
        }
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export generic function: {:?}", msgs);
}

#[test]
fn test_parse_multiple_exports() {
    let source = r#"
        export fn add(a: number, b: number) -> number {
            return a + b;
        }
        export let PI = 3.14;
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse multiple exports: {:?}", msgs);
}

// ============================================================================
// Combined Import/Export Tests
// ============================================================================

#[test]
fn test_parse_module_with_import_and_export() {
    let source = r#"
        import { log } from "./logger";

        export fn greet(name: string) -> string {
            log("greeting " + name);
            return "Hello, " + name;
        }
    "#;
    let (success, msgs) = parse(source);
    assert!(
        success,
        "Should parse module with imports and exports: {:?}",
        msgs
    );
}

#[test]
fn test_parse_module_with_multiple_imports_exports() {
    let source = r#"
        import { add, sub } from "./math";
        import * as logger from "./logger";

        export fn calculate(a: number, b: number) -> number {
            return add(a, b);
        }

        export let VERSION = "1.0";
    "#;
    let (success, msgs) = parse(source);
    assert!(
        success,
        "Should parse module with multiple imports/exports: {:?}",
        msgs
    );
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_import_missing_from() {
    let source = r#"import { x }"#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: missing 'from' keyword");
}

#[test]
fn test_import_missing_braces() {
    let source = r#"import x from "./mod""#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: missing braces for named import");
}

#[test]
fn test_namespace_import_missing_as() {
    let source = r#"import * from "./mod""#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: namespace import missing 'as'");
}
