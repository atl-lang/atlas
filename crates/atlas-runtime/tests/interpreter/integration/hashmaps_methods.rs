use super::*;

// ============================================================================
// B10-P06: HashMap method surface — dot-syntax for all map.method() calls
// Both interpreter and VM parity tested throughout.
// ============================================================================

// --- map.size() / map.len() ---

#[test]
fn test_hashmap_method_size() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("a", 1);
        m.set("b", 2);
        m.size();
    "#;
    assert_eval_number(src, 2.0);
    assert_parity(src);
}

#[test]
fn test_hashmap_method_len() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("x", 10);
        m.len();
    "#;
    assert_eval_number(src, 1.0);
    assert_parity(src);
}

#[test]
fn test_hashmap_method_size_empty() {
    let src = r#"let m: HashMap = hashMapNew(); m.size();"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

// --- map.isEmpty() ---

#[test]
fn test_hashmap_method_is_empty_true() {
    let src = r#"let m: HashMap = hashMapNew(); m.isEmpty();"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_hashmap_method_is_empty_false() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("k", 1);
        m.isEmpty();
    "#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- map.has() / map.containsKey() ---

#[test]
fn test_hashmap_method_has_true() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("name", "Alice");
        m.has("name");
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_hashmap_method_has_false() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("name", "Alice");
        m.has("missing");
    "#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

#[test]
fn test_hashmap_method_contains_key() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("k", 42);
        m.containsKey("k");
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// --- map.get() ---

#[test]
fn test_hashmap_method_get_found() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("score", 99);
        unwrap(m.get("score"));
    "#;
    assert_eval_number(src, 99.0);
    assert_parity(src);
}

#[test]
fn test_hashmap_method_get_not_found() {
    let src = r#"
        let m: HashMap = hashMapNew();
        let v: Option<number> = m.get("missing");
        is_none(v);
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// --- map.set() / map.put() (mutating, CoW write-back) ---

#[test]
fn test_hashmap_method_set_and_get() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("a", 1);
        m.set("b", 2);
        unwrap(m.get("b"));
    "#;
    assert_eval_number(src, 2.0);
    assert_parity(src);
}

#[test]
fn test_hashmap_method_put_alias() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.put("x", 7);
        unwrap(m.get("x"));
    "#;
    assert_eval_number(src, 7.0);
    assert_parity(src);
}

#[test]
fn test_hashmap_method_set_overwrites() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("k", 1);
        m.set("k", 99);
        unwrap(m.get("k"));
    "#;
    assert_eval_number(src, 99.0);
    assert_parity(src);
}

// --- map.remove() / map.delete() (mutating, CoW write-back) ---

#[test]
fn test_hashmap_method_remove() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("a", 1);
        m.set("b", 2);
        m.remove("a");
        m.size();
    "#;
    assert_eval_number(src, 1.0);
    assert_parity(src);
}

#[test]
fn test_hashmap_method_delete_alias() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("x", 10);
        m.delete("x");
        m.has("x");
    "#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- map.clear() (mutating, CoW write-back) ---

#[test]
fn test_hashmap_method_clear() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("a", 1);
        m.set("b", 2);
        m.clear();
        m.size();
    "#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

// --- map.keys() / map.values() ---

#[test]
fn test_hashmap_method_keys_len() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("a", 1);
        m.set("b", 2);
        let k: string[] = m.keys();
        k.len();
    "#;
    assert_eval_number(src, 2.0);
    assert_parity(src);
}

#[test]
fn test_hashmap_method_values_len() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("a", 1);
        m.set("b", 2);
        let v: number[] = m.values();
        v.len();
    "#;
    assert_eval_number(src, 2.0);
    assert_parity(src);
}

// --- map.entries() ---
// entries() returns array of [key, value] pairs; type is ?[][] due to H-137 (map return type)

#[test]
fn test_hashmap_method_entries_len() {
    let src = r#"
        let m: HashMap = hashMapNew();
        m.set("a", 1);
        m.set("b", 2);
        let e = m.entries();
        len(e);
    "#;
    assert_eval_number(src, 2.0);
    assert_parity(src);
}

// ============================================================================
// End B10-P06 tests
// ============================================================================
