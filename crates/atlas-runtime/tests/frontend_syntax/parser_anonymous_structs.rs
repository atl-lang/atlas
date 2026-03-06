//! Parser tests for anonymous struct literals

use super::*;

fn find_var_init<'a>(program: &'a Program, name: &str) -> &'a Expr {
    for item in &program.items {
        if let Item::Statement(Stmt::VarDecl(var)) = item {
            if var.name.name == name {
                return &var.init;
            }
        }
    }
    panic!("Variable '{}' not found", name);
}

#[test]
fn test_parse_anonymous_struct_disambiguation() {
    let (program, diagnostics) = parse_source(
        r#"
        let block = { };
        let explicit = { x: 1, y: 2 };
        let shorthand = { x };
        let mixed = { x, y: 3 };
        "#,
    );

    let block_init = find_var_init(&program, "block");
    assert!(
        matches!(block_init, Expr::Block(_)),
        "Expected empty block, got {:?}",
        block_init
    );

    let explicit_init = find_var_init(&program, "explicit");
    assert!(
        matches!(explicit_init, Expr::ObjectLiteral(obj) if obj.entries.len() == 2),
        "Expected anonymous struct literal with 2 entries, got {:?}",
        explicit_init
    );

    let shorthand_init = find_var_init(&program, "shorthand");
    assert!(
        matches!(shorthand_init, Expr::ObjectLiteral(obj) if obj.entries.len() == 1),
        "Expected shorthand anonymous struct literal, got {:?}",
        shorthand_init
    );

    let mixed_init = find_var_init(&program, "mixed");
    assert!(
        matches!(mixed_init, Expr::ObjectLiteral(obj) if obj.entries.len() == 2),
        "Expected mixed anonymous struct literal, got {:?}",
        mixed_init
    );

    // H-086: All 3 anonymous structs should emit deprecation warnings
    let deprecation_count = diagnostics
        .iter()
        .filter(|d| {
            d.level == DiagnosticLevel::Warning
                && d.message.contains("deprecated")
                && d.message.contains("record")
        })
        .count();
    assert_eq!(
        deprecation_count,
        3,
        "Expected 3 deprecation warnings for anonymous struct syntax, got {}: {:?}",
        deprecation_count,
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_anonymous_struct_shorthand_warning() {
    let (_program, diagnostics) = parse_source(
        r#"
        let x = 1;
        let p = { x: x };
        "#,
    );

    // Should have both the deprecation warning (H-086) and the shorthand warning
    let has_shorthand_warning = diagnostics.iter().any(|d| {
        d.level == DiagnosticLevel::Warning && d.message.to_lowercase().contains("shorthand")
    });
    assert!(
        has_shorthand_warning,
        "Expected shorthand warning, got: {:?}",
        diagnostics
            .iter()
            .map(|d| (&d.code, &d.message))
            .collect::<Vec<_>>()
    );

    let has_deprecation_warning = diagnostics.iter().any(|d| {
        d.level == DiagnosticLevel::Warning
            && d.message.contains("deprecated")
            && d.message.contains("record")
    });
    assert!(
        has_deprecation_warning,
        "Expected deprecation warning for anonymous struct syntax"
    );
}

#[test]
fn test_record_syntax_no_deprecation_warning() {
    let (program, diagnostics) = parse_source(
        r#"
        let r = record { x: 1, y: 2 };
        "#,
    );

    let r_init = find_var_init(&program, "r");
    assert!(
        matches!(r_init, Expr::ObjectLiteral(obj) if obj.entries.len() == 2),
        "Expected record literal with 2 entries, got {:?}",
        r_init
    );

    let has_deprecation = diagnostics
        .iter()
        .any(|d| d.level == DiagnosticLevel::Warning && d.message.contains("deprecated"));
    assert!(
        !has_deprecation,
        "record {{ }} syntax should NOT emit deprecation warning, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}
