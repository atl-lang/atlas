//! Folding range tests
//!
//! Tests for LSP folding functionality including:
//! - Function body folding
//! - Block statement folding
//! - Comment folding
//! - Array literal folding

use atlas_lsp::folding::generate_folding_ranges;
use atlas_runtime::{Lexer, Parser};
use tower_lsp::lsp_types::FoldingRangeKind;

/// Parse source and get AST for testing
fn parse_source(source: &str) -> atlas_runtime::ast::Program {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    ast
}

// === Function Body Folding ===

#[test]
fn test_function_body_folding() {
    let source = "fn test() {\n  let x = 1;\n  return x;\n}";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    // Should have at least one range for the function body
    assert!(!ranges.is_empty());

    let body_range = ranges
        .iter()
        .find(|r| r.kind == Some(FoldingRangeKind::Region));
    assert!(body_range.is_some());
}

#[test]
fn test_multiple_functions_folding() {
    let source = "fn foo() {\n  return 1;\n}\n\nfn bar() {\n  return 2;\n}";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    let region_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Region))
        .collect();

    assert!(region_ranges.len() >= 2);
}

#[test]
fn test_single_line_function_no_fold() {
    let source = "fn test() { return 1; }";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    // Single-line function should not have a fold
    let body_folds: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Region) && r.end_line > r.start_line)
        .collect();

    assert!(body_folds.is_empty());
}

// === Block Statement Folding ===

#[test]
fn test_if_statement_folding() {
    let source = "fn test() {\n  if true {\n    let x = 1;\n  }\n}";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    // Should have at least the function body fold
    let region_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Region))
        .collect();

    assert!(!region_ranges.is_empty());
}

#[test]
fn test_if_else_folding() {
    let source = "fn test() {\n  if true {\n    let x = 1;\n  } else {\n    let y = 2;\n  }\n}";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    // Should have at least the function body fold
    let region_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Region))
        .collect();

    assert!(!region_ranges.is_empty());
}

#[test]
fn test_while_loop_folding() {
    let source = "fn test() {\n  while true {\n    let x = 1;\n  }\n}";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    let region_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Region))
        .collect();

    // Should have at least the function body fold
    assert!(!region_ranges.is_empty());
}

#[test]
fn test_for_loop_folding() {
    let source = "fn test() {\n  for i in [1, 2, 3] {\n    println(i);\n  }\n}";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    let region_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Region))
        .collect();

    // Should have at least the function body fold
    assert!(!region_ranges.is_empty());
}

#[test]
fn test_nested_blocks_folding() {
    let source = "fn test() {\n  if true {\n    while true {\n      let x = 1;\n    }\n  }\n}";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    let region_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Region))
        .collect();

    // Should have at least one region (function body)
    assert!(!region_ranges.is_empty());
}

// === Comment Folding ===

#[test]
fn test_multiline_comment_folding() {
    let source = "/*\n * This is a\n * multi-line comment\n */\nlet x = 1;";
    let ranges = generate_folding_ranges(source, None);

    let comment_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Comment))
        .collect();

    assert_eq!(comment_ranges.len(), 1);
    assert_eq!(comment_ranges[0].start_line, 0);
    assert_eq!(comment_ranges[0].end_line, 3);
}

#[test]
fn test_consecutive_single_line_comments_folding() {
    let source = "// Line 1\n// Line 2\n// Line 3\nlet x = 1;";
    let ranges = generate_folding_ranges(source, None);

    let comment_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Comment))
        .collect();

    assert_eq!(comment_ranges.len(), 1);
    assert_eq!(comment_ranges[0].start_line, 0);
    assert_eq!(comment_ranges[0].end_line, 2);
}

#[test]
fn test_single_comment_no_fold() {
    let source = "// Just one comment\nlet x = 1;";
    let ranges = generate_folding_ranges(source, None);

    let comment_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Comment))
        .collect();

    assert!(comment_ranges.is_empty());
}

// === Import Folding ===

#[test]
fn test_import_block_folding() {
    let source = "import { foo } from \"mod1\";\nimport { bar } from \"mod2\";\nimport { baz } from \"mod3\";\n\nlet x = 1;";
    let ranges = generate_folding_ranges(source, None);

    let import_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Imports))
        .collect();

    assert_eq!(import_ranges.len(), 1);
    assert_eq!(import_ranges[0].start_line, 0);
    assert_eq!(import_ranges[0].end_line, 2);
}

#[test]
fn test_single_import_no_fold() {
    let source = "import { foo } from \"mod\";\n\nlet x = 1;";
    let ranges = generate_folding_ranges(source, None);

    let import_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Imports))
        .collect();

    assert!(import_ranges.is_empty());
}

// === Array Literal Folding ===

#[test]
fn test_multiline_array_folding() {
    let source = "let arr = [\n  1,\n  2,\n  3\n];";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    let region_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Region))
        .collect();

    assert!(!region_ranges.is_empty());
}

#[test]
fn test_single_line_array_no_fold() {
    let source = "let arr = [1, 2, 3];";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    // Single-line arrays shouldn't fold
    let multi_line_folds: Vec<_> = ranges
        .iter()
        .filter(|r| r.end_line > r.start_line)
        .collect();

    assert!(multi_line_folds.is_empty());
}

// === Edge Cases ===

#[test]
fn test_empty_document() {
    let source = "";
    let ranges = generate_folding_ranges(source, None);
    assert!(ranges.is_empty());
}

#[test]
fn test_no_ast_with_comments() {
    let source = "// Comment 1\n// Comment 2\nlet x = 1;";
    let ranges = generate_folding_ranges(source, None);

    // Should still extract comment folds without AST
    let comment_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind == Some(FoldingRangeKind::Comment))
        .collect();

    assert_eq!(comment_ranges.len(), 1);
}

#[test]
fn test_folding_ranges_sorted() {
    let source = "// Comment 1\n// Comment 2\n\nfn test() {\n  let x = 1;\n}";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    // Ranges should be sorted by start line
    for i in 1..ranges.len() {
        assert!(ranges[i].start_line >= ranges[i - 1].start_line);
    }
}

#[test]
fn test_folding_line_accuracy() {
    let source = "fn test() {\n  return 1;\n}";
    let ast = parse_source(source);
    let ranges = generate_folding_ranges(source, Some(&ast));

    let body_fold = ranges
        .iter()
        .find(|r| r.kind == Some(FoldingRangeKind::Region));
    assert!(body_fold.is_some());

    let fold = body_fold.unwrap();
    // Should start at line 0 (where { is) and end at line 2 (where } is)
    assert_eq!(fold.start_line, 0);
    assert_eq!(fold.end_line, 2);
}
