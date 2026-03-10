use atlas_runtime::{is_input_complete, IncompleteReason, InputCompleteness, MultilineInput};

// ============================================================================
// MULTILINE INPUT DETECTION TESTS (Phase 01)
// ============================================================================

// --- Basic completeness tests ---

#[test]
fn multiline_simple_statement_complete() {
    assert_eq!(is_input_complete("let x = 1;"), InputCompleteness::Complete);
}

#[test]
fn multiline_empty_complete() {
    assert_eq!(is_input_complete(""), InputCompleteness::Complete);
}

#[test]
fn multiline_whitespace_complete() {
    assert_eq!(is_input_complete("   \n  "), InputCompleteness::Complete);
}

#[test]
fn multiline_expression_complete() {
    assert_eq!(is_input_complete("1 + 2 + 3;"), InputCompleteness::Complete);
}

// --- Unclosed brace tests ---

#[test]
fn multiline_unclosed_brace_function() {
    match is_input_complete("fn foo() {") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedBrace);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_unclosed_brace_if() {
    match is_input_complete("if true {") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedBrace);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_unclosed_brace_nested() {
    match is_input_complete("fn foo() { if true {") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedBrace);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_matched_braces_complete() {
    assert_eq!(
        is_input_complete("fn foo() { return 1; }"),
        InputCompleteness::Complete
    );
}

// --- Unclosed bracket tests ---

#[test]
fn multiline_unclosed_bracket_array() {
    match is_input_complete("let arr = [1, 2") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedBracket);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_unclosed_bracket_nested_array() {
    match is_input_complete("let arr = [[1, 2], [3") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedBracket);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_matched_brackets_complete() {
    assert_eq!(
        is_input_complete("let arr = [1, 2, 3];"),
        InputCompleteness::Complete
    );
}

// --- Unclosed paren tests ---

#[test]
fn multiline_unclosed_paren_call() {
    match is_input_complete("print(") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedParen);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_unclosed_paren_expr() {
    match is_input_complete("(1 + 2 * (3 + 4)") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedParen);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_matched_parens_complete() {
    assert_eq!(
        is_input_complete("print(1 + 2);"),
        InputCompleteness::Complete
    );
}

// --- Unclosed string tests ---

#[test]
fn multiline_unclosed_string() {
    match is_input_complete("let s = \"hello") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedString);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_string_with_escape_incomplete() {
    match is_input_complete("let s = \"hello\\n") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedString);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_closed_string_complete() {
    assert_eq!(
        is_input_complete("let s = \"hello world\";"),
        InputCompleteness::Complete
    );
}

#[test]
fn multiline_string_with_escape_complete() {
    assert_eq!(
        is_input_complete("let s = \"hello\\nworld\";"),
        InputCompleteness::Complete
    );
}

// --- Unclosed comment tests ---

#[test]
fn multiline_unclosed_block_comment() {
    match is_input_complete("/* this is a comment") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedComment);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_closed_block_comment_complete() {
    assert_eq!(
        is_input_complete("/* comment */ let x = 1;"),
        InputCompleteness::Complete
    );
}

#[test]
fn multiline_line_comment_complete() {
    assert_eq!(
        is_input_complete("let x = 1; // comment"),
        InputCompleteness::Complete
    );
}

// --- Complex multiline scenarios ---

#[test]
fn multiline_function_body_incomplete() {
    match is_input_complete("fn add(borrow a: number, borrow b: number): number {\n  return a + b")
    {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedBrace);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_function_body_complete() {
    assert_eq!(
        is_input_complete(
            "fn add(borrow a: number, borrow b: number): number {\n  return a + b;\n}"
        ),
        InputCompleteness::Complete
    );
}

#[test]
fn multiline_nested_structures_incomplete() {
    match is_input_complete("if (true) { let arr = [1, 2,") {
        InputCompleteness::Incomplete { reason } => {
            // Brace checked before bracket in priority order
            assert_eq!(reason, IncompleteReason::UnclosedBrace);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_string_in_array_incomplete() {
    match is_input_complete("[\"hello") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedString);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

// --- MultilineInput state tests ---

#[test]
fn multiline_input_new_empty() {
    let ml = MultilineInput::new();
    assert!(ml.is_empty());
    assert_eq!(ml.line_count(), 0);
}

#[test]
fn multiline_input_add_line() {
    let mut ml = MultilineInput::new();
    ml.add_line("let x = 1;");
    assert!(!ml.is_empty());
    assert_eq!(ml.line_count(), 1);
}

#[test]
fn multiline_input_add_multiple_lines() {
    let mut ml = MultilineInput::new();
    ml.add_line("fn foo() {");
    ml.add_line("  return 1;");
    ml.add_line("}");
    assert_eq!(ml.line_count(), 3);
}

#[test]
fn multiline_input_combined() {
    let mut ml = MultilineInput::new();
    ml.add_line("fn foo() {");
    ml.add_line("  return 1;");
    ml.add_line("}");
    let combined = ml.combined();
    assert!(combined.contains("fn foo()"));
    assert!(combined.contains("return 1"));
}

#[test]
fn multiline_input_clear() {
    let mut ml = MultilineInput::new();
    ml.add_line("line 1");
    ml.add_line("line 2");
    ml.clear();
    assert!(ml.is_empty());
    assert_eq!(ml.line_count(), 0);
}

#[test]
fn multiline_input_check_completeness_complete() {
    let mut ml = MultilineInput::new();
    ml.add_line("let x = 1;");
    assert_eq!(ml.check_completeness(), InputCompleteness::Complete);
}

#[test]
fn multiline_input_check_completeness_incomplete() {
    let mut ml = MultilineInput::new();
    ml.add_line("fn foo() {");
    match ml.check_completeness() {
        InputCompleteness::Incomplete { .. } => {}
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_input_accumulate_then_complete() {
    let mut ml = MultilineInput::new();
    ml.add_line("fn foo() {");
    assert!(matches!(
        ml.check_completeness(),
        InputCompleteness::Incomplete { .. }
    ));

    ml.add_line("  return 1;");
    assert!(matches!(
        ml.check_completeness(),
        InputCompleteness::Incomplete { .. }
    ));

    ml.add_line("}");
    assert_eq!(ml.check_completeness(), InputCompleteness::Complete);
}

// --- IncompleteReason description tests ---

#[test]
fn incomplete_reason_description_brace() {
    let reason = IncompleteReason::UnclosedBrace;
    assert!(reason.description().contains("brace"));
}

#[test]
fn incomplete_reason_description_bracket() {
    let reason = IncompleteReason::UnclosedBracket;
    assert!(reason.description().contains("bracket"));
}

#[test]
fn incomplete_reason_description_paren() {
    let reason = IncompleteReason::UnclosedParen;
    assert!(reason.description().contains("parenthesis"));
}

#[test]
fn incomplete_reason_description_string() {
    let reason = IncompleteReason::UnclosedString;
    assert!(reason.description().contains("string"));
}

#[test]
fn incomplete_reason_description_comment() {
    let reason = IncompleteReason::UnclosedComment;
    assert!(reason.description().contains("comment"));
}

// --- Edge cases ---

#[test]
fn multiline_closing_in_string_not_counted() {
    // The } inside the string should not close the actual brace
    match is_input_complete("fn foo() { let s = \"}\"") {
        InputCompleteness::Incomplete { reason } => {
            assert_eq!(reason, IncompleteReason::UnclosedBrace);
        }
        InputCompleteness::Complete => panic!("expected incomplete"),
    }
}

#[test]
fn multiline_closing_in_comment_not_counted() {
    // The } inside the comment should not close the actual brace
    assert_eq!(
        is_input_complete("fn foo() { /* } */ return 1; }"),
        InputCompleteness::Complete
    );
}

#[test]
fn multiline_delimiter_in_line_comment_not_counted() {
    // The { in the line comment should not count
    assert_eq!(
        is_input_complete("let x = 1; // fn foo() {"),
        InputCompleteness::Complete
    );
}
