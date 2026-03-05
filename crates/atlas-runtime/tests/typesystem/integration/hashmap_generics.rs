use super::super::*;

#[test]
fn hashmap_generic_params_must_match() {
    let src = r#"
fn takes_numbers(m: HashMap<string, number>) -> void {
    let _ = m;
}

let m: HashMap<string, string> = hashMapNew();
takes_numbers(m);
"#;
    let diagnostics = typecheck_source(src);
    assert!(has_error_code(&diagnostics, "AT3001"));
}

#[test]
fn hashmap_put_enforces_value_type() {
    let src = r#"
let m: HashMap<string, number> = hashMapNew();
hashMapPut(m, "age", "thirty");
"#;
    let diagnostics = typecheck_source(src);
    assert!(has_error_code(&diagnostics, "AT3001"));
}

#[test]
fn hashmap_get_returns_value_type() {
    let src = r#"
let m: HashMap<string, number> = hashMapNew();
let v: string = hashMapGet(m, "age");
"#;
    let diagnostics = typecheck_source(src);
    assert!(has_error_code(&diagnostics, "AT3001"));
}
