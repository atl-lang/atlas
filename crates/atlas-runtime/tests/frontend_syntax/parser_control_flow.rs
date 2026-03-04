//! Parser control flow and match tests (split from parser_basics.rs)

use super::*;

fn parse_source_with_comments(
    source: &str,
) -> (Program, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize_with_comments();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// ============================================================================
// Control Flow Statements
// ============================================================================

#[rstest]
#[case::if_stmt("if (true) { x; }", "if_statement")]
#[case::if_stmt_no_parens("if true { x; }", "if_statement_no_parens")]
#[case::if_else("if (true) { x; } else { y; }", "if_else_statement")]
#[case::if_else_no_parens("if true { x; } else { y; }", "if_else_statement_no_parens")]
#[case::while_loop("while (true) { x; }", "while_loop")]
#[case::while_loop_no_parens("while true { x; }", "while_loop_no_parens")]
#[case::for_loop("for i in [0, 1, 2, 3, 4] { x; }", "for_loop")]
#[case::for_loop_parens("for (i in [0, 1, 2, 3, 4]) { x; }", "for_loop_parens")]
fn test_parse_control_flow(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

#[test]
fn test_if_parentheses_optional() {
    let (program_parens, diagnostics_parens) =
        parse_source("if (x < y) { let z = x; } else { let z = y; }");
    assert_eq!(
        diagnostics_parens.len(),
        0,
        "Expected no errors for if with parentheses"
    );
    let (program_no_parens, diagnostics_no_parens) =
        parse_source("if x < y { let z = x; } else { let z = y; }");
    assert_eq!(
        diagnostics_no_parens.len(),
        0,
        "Expected no errors for if without parentheses"
    );
    let _ = program_parens;
    let _ = program_no_parens;
}

// ============================================================================
// Match Expressions
// ============================================================================

#[rstest]
#[case::match_commas(
    "let r = match x { 1 => \"one\", 2 => \"two\", _ => \"other\" };",
    "match_expr_commas"
)]
#[case::match_semicolons(
    "let r = match x { 1 => \"one\"; 2 => \"two\"; _ => \"other\"; };",
    "match_expr_semicolons"
)]
fn test_parse_match_expressions(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no errors for: {}", source);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Return, Break, Continue
// ============================================================================

#[rstest]
#[case::return_value("return 42;", "return_statement")]
#[case::return_void("return;", "return_no_value")]
#[case::break_stmt("break;", "break_statement")]
#[case::continue_stmt("continue;", "continue_statement")]
fn test_parse_flow_control_statements(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Block Statements
// ============================================================================

#[test]
fn test_parse_block_in_if() {
    let (program, diagnostics) = parse_source("if true { let x = 1; let y = 2; }");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_blocks() {
    let (program, diagnostics) = parse_source("if true { if false { let x = 1; } }");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Comment Handling
// ============================================================================

#[test]
fn test_parse_comments_inside_expressions() {
    let source = "let x = 1 /* add two */ + 2; let y = x // keep\n + 3;";
    let (_program, diagnostics) = parse_source_with_comments(source);
    assert_eq!(diagnostics.len(), 0, "Expected no errors for: {}", source);
}

#[test]
fn test_parse_comments_inside_blocks() {
    let source = r#"
        fn add(x: number, y: number) -> number {
            let sum = x + y; // inline comment
            /// doc comment inside block should be ignored
            return sum;
        }
    "#;
    let (_program, diagnostics) = parse_source_with_comments(source);
    assert_eq!(
        diagnostics.len(),
        0,
        "Expected no errors for block comments"
    );
}
