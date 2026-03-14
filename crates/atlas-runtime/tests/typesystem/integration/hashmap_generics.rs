use super::super::*;

#[test]
fn hashmap_generic_params_must_match() {
    let src = r#"
fn takes_numbers(borrow m: Map<string, number>): void {
    let _ = m;
}

let m: Map<string, string> = new Map<string, string>();
takes_numbers(m);
"#;
    let diagnostics = typecheck_source(src);
    assert!(has_error_code(&diagnostics, "AT3001"));
}

#[test]
fn hashmap_put_enforces_value_type() {
    let src = r#"
let m: Map<string, number> = new Map<string, number>();
m.set("age", "thirty");
"#;
    let diagnostics = typecheck_source(src);
    assert!(has_error_code(&diagnostics, "AT3001"));
}

#[test]
fn hashmap_get_returns_value_type() {
    let src = r#"
let m: Map<string, number> = new Map<string, number>();
let v: string = m.get("age");
"#;
    let diagnostics = typecheck_source(src);
    assert!(has_error_code(&diagnostics, "AT3001"));
}

// H-112: hashMapHas / hashSetHas must typecheck to bool so they can be used in if-conditions
#[test]
fn hashmap_has_returns_bool() {
    let src = r#"
let m: Map<string, number> = new Map<string, number>();
let has: bool = m.has("key");
"#;
    let diagnostics = typecheck_source(src);
    assert_no_errors(&diagnostics);
}

#[test]
fn hashmap_has_usable_in_if_condition() {
    let src = r#"
let m: Map<string, number> = new Map<string, number>();
if m.has("key") {
    let x: number = 1;
}
"#;
    let diagnostics = typecheck_source(src);
    assert_no_errors(&diagnostics);
}
