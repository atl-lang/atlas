use super::common::*;

// ============================================================================
// new T() constructor syntax (H-374)
// ============================================================================

#[test]
fn test_h374_new_map_creates_empty_hashmap() {
    // new Map<string, number>() should create an empty map
    let code = r#"
        let m = new Map<string, number>();
        m.size()
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_h374_new_map_set_and_get() {
    let code = r#"
        let m = new Map<string, number>();
        m.set("x", 42);
        m.get("x").unwrap()
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_h374_new_map_no_type_args() {
    // new Map() without explicit type args should also work
    let code = r#"
        let m = new Map();
        m.size()
    "#;
    assert_eval_number(code, 0.0);
}
