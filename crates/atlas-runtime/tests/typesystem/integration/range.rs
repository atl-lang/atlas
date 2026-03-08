use super::super::*;

#[test]
fn test_range_expression_type_ok() {
    let diagnostics = typecheck("let r = 1..3;");
    assert_no_errors(&diagnostics);
}

#[test]
fn test_range_expression_type_error() {
    let diagnostics = errors("let r = \"a\"..3;");
    assert!(has_error_code(&diagnostics, "AT3001"));
}

#[test]
fn test_range_index_array_ok() {
    let diagnostics = typecheck(
        r#"
        let arr: []number = [1, 2, 3, 4];
        let r = 1..=2;
        let part = arr[r];
        part[0] + part[1];
        "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_range_index_string_error() {
    let diagnostics = errors(
        r#"
        let s = "abc";
        s[1..2];
        "#,
    );
    assert!(has_error_code(&diagnostics, "AT3001"));
}
