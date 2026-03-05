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
    let program = parse_valid(
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
}

#[test]
fn test_anonymous_struct_shorthand_warning() {
    let (_program, diagnostics) = parse_source(
        r#"
        let x = 1;
        let p = { x: x };
        "#,
    );

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
}
