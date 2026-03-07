// H-115: if/else as expression returns '?' type from typechecker
use super::super::*;

#[test]
fn if_else_expr_infers_number_type() {
    let src = r#"
let x: number = 1;
let y: number = if x > 0 { 1 } else { 2 };
"#;
    let diagnostics = typecheck_source(src);
    assert_no_errors(&diagnostics);
}

#[test]
fn if_else_expr_infers_string_type() {
    let src = r#"
let x: bool = true;
let s: string = if x { "yes" } else { "no" };
"#;
    let diagnostics = typecheck_source(src);
    assert_no_errors(&diagnostics);
}
