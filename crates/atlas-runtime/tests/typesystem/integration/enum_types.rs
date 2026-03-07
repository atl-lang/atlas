// H-110: match on user-defined enum in function body returns '?' type
// H-111: matching same user enum variable twice at top level — second result is '?'
use super::super::*;

/// H-110: enum param in fn body — calling describe(s) should not trigger AT2013
/// (AT2013 fires when the argument type is '?' / Unknown, meaning Status resolved to Unknown)
#[test]
fn test_h110_enum_match_in_fn_body_no_at2013() {
    let diagnostics = typecheck_source(
        r#"
        enum Status { Active, Inactive }
        fn describe(s: Status) -> string {
            match s {
                Status::Active => "active",
                Status::Inactive => "inactive"
            }
        }
        let s: Status = Status::Active;
        let result: string = describe(s);
        "#,
    );
    let at2013: Vec<_> = diagnostics.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        at2013.is_empty(),
        "H-110: AT2013 (Type '?' is not Copy) should not fire when passing a user enum arg — enum type resolves to Unknown. Got: {:?}",
        at2013.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

/// H-110: enum param — AT2013 should not fire for enum-typed arguments
#[test]
fn test_h110_enum_param_not_flagged_as_move() {
    let diagnostics = typecheck_source(
        r#"
        enum Color { Red, Green, Blue }
        fn name(c: Color) -> string {
            match c {
                Color::Red => "red",
                Color::Green => "green",
                Color::Blue => "blue"
            }
        }
        let c: Color = Color::Red;
        let n: string = name(c);
        "#,
    );
    let at2013: Vec<_> = diagnostics.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        at2013.is_empty(),
        "H-110: AT2013 should not fire for user-defined enum arguments, got: {:?}",
        at2013.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

/// H-111: second match on same enum variable at top-level should not return '?'
#[test]
fn test_h111_double_match_same_enum_var() {
    let diagnostics = typecheck_source(
        r#"
        enum Status { Active, Inactive }
        let s: Status = Status::Active;
        let label1: string = match s { Status::Active => "active", Status::Inactive => "inactive" };
        let label2: string = match s { Status::Active => "yes", Status::Inactive => "no" };
        "#,
    );
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == atlas_runtime::diagnostic::DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "H-111: second match on same enum var should not produce errors, got: {:?}",
        errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// NOTE: Enum tuple-variant binding types (Shape::Circle(r) => r has type number)
// require a full variant-field registry in the typechecker.
// That is tracked as H-119 and is out of scope for H-110/H-111.
