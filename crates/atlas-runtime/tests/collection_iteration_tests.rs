//! HashMap and HashSet Iteration Tests
//!
//! Simplified tests for forEach, map, and filter intrinsics.

use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;

fn run(code: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut interpreter = Interpreter::new();
    let security = SecurityContext::allow_all();
    match interpreter.eval(&ast, &security) {
        Ok(val) => Ok(format!("{:?}", val)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

// HashMap Iteration Tests

#[test]
fn test_hashmap_map_transforms_values() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "a", 1);
        hashMapPut(map, "b", 2);
        let result = hashMapMap(map, fn(v, k) { v * 2; });
        hashMapGet(result, "a")
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("2.0"));
}

#[test]
fn test_hashmap_filter_keeps_matching_entries() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "a", 1);
        hashMapPut(map, "b", 2);
        hashMapPut(map, "c", 3);
        let result = hashMapFilter(map, fn(v, k) { v > 1; });
        hashMapSize(result)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(2.0)");
}

#[test]
fn test_hashmap_foreach_returns_null() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "a", 1);
        hashMapForEach(map, fn(v, k) { v; })
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Null");
}

#[test]
fn test_hashmap_empty_iteration() {
    let code = r#"
        let map = hashMapNew();
        hashMapMap(map, fn(v, k) { v; })
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("HashMap"));
}

#[test]
fn test_hashmap_filter_with_predicate() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "a", 1);
        hashMapPut(map, "b", 2);
        hashMapPut(map, "c", 3);
        hashMapPut(map, "d", 4);
        let result = hashMapFilter(map, fn(v, k) { v % 2 == 0; });
        hashMapSize(result)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(2.0)");
}

#[test]
fn test_hashmap_map_preserves_keys() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "x", 10);
        let result = hashMapMap(map, fn(v, k) { v + 5; });
        hashMapHas(result, "x")
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Bool(true)");
}

#[test]
fn test_hashmap_chaining_operations() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "a", 1);
        hashMapPut(map, "b", 2);
        hashMapPut(map, "c", 3);
        let doubled = hashMapMap(map, fn(v, k) { v * 2; });
        let filtered = hashMapFilter(doubled, fn(v, k) { v > 2; });
        hashMapSize(filtered)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(2.0)");
}

#[test]
fn test_hashmap_callback_error_type() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "a", 1);
        hashMapMap(map, "not a function")
    "#;
    let result = run(code);
    assert!(result.is_err());
}

#[test]
fn test_hashmap_filter_bool_return() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "a", 1);
        hashMapFilter(map, fn(v, k) { v; })
    "#;
    let result = run(code);
    assert!(result.is_err());
}

#[test]
fn test_hashmap_large_map() {
    let code = r#"
        let map = hashMapNew();
        let i = 0;
        while (i < 50) {
            hashMapPut(map, toString(i), i);
            i = i + 1;
        }
        let filtered = hashMapFilter(map, fn(v, k) { v < 25; });
        hashMapSize(filtered)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(25.0)");
}

// HashSet Iteration Tests

#[test]
fn test_hashset_filter_keeps_matching() {
    let code = r#"
        let set = hashSetNew();
        hashSetAdd(set, 1);
        hashSetAdd(set, 2);
        hashSetAdd(set, 3);
        hashSetAdd(set, 4);
        let result = hashSetFilter(set, fn(elem) { elem > 2; });
        hashSetSize(result)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(2.0)");
}

#[test]
fn test_hashset_map_to_array() {
    let code = r#"
        let set = hashSetNew();
        hashSetAdd(set, 1);
        hashSetAdd(set, 2);
        let result = hashSetMap(set, fn(elem) { elem * 2; });
        typeof(result)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "String(\"array\")");
}

#[test]
fn test_hashset_foreach_returns_null() {
    let code = r#"
        let set = hashSetNew();
        hashSetAdd(set, 1);
        hashSetForEach(set, fn(elem) { elem; })
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Null");
}

#[test]
fn test_hashset_empty_filter() {
    let code = r#"
        let set = hashSetNew();
        let result = hashSetFilter(set, fn(elem) { true; });
        hashSetSize(result)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(0.0)");
}

#[test]
fn test_hashset_map_array_length() {
    let code = r#"
        let set = hashSetNew();
        hashSetAdd(set, 1);
        hashSetAdd(set, 2);
        hashSetAdd(set, 3);
        let arr = hashSetMap(set, fn(elem) { elem * 10; });
        len(arr)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(3.0)");
}

#[test]
fn test_hashset_filter_chaining() {
    let code = r#"
        let set = hashSetNew();
        hashSetAdd(set, 1);
        hashSetAdd(set, 2);
        hashSetAdd(set, 3);
        hashSetAdd(set, 4);
        let f1 = hashSetFilter(set, fn(elem) { elem > 1; });
        let f2 = hashSetFilter(f1, fn(elem) { elem < 4; });
        hashSetSize(f2)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(2.0)");
}

#[test]
fn test_hashset_callback_error() {
    let code = r#"
        let set = hashSetNew();
        hashSetAdd(set, 1);
        hashSetFilter(set, 42)
    "#;
    let result = run(code);
    assert!(result.is_err());
}

#[test]
fn test_hashset_large_set() {
    let code = r#"
        let set = hashSetNew();
        let i = 0;
        while (i < 30) {
            hashSetAdd(set, i);
            i = i + 1;
        }
        let filtered = hashSetFilter(set, fn(elem) { elem % 3 == 0; });
        hashSetSize(filtered)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(10.0)");
}

// Integration: simple cross-collection tests

#[test]
fn test_integration_hashmap_to_hashset() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "a", 1);
        hashMapPut(map, "b", 2);
        let values = hashMapValues(map);
        let set = hashSetFromArray(values);
        hashSetSize(set)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(2.0)");
}

#[test]
fn test_integration_hashset_map_filter() {
    let code = r#"
        let set = hashSetNew();
        hashSetAdd(set, 1);
        hashSetAdd(set, 2);
        hashSetAdd(set, 3);
        let arr = hashSetMap(set, fn(x) { x * 2; });
        let filtered = filter(arr, fn(x) { x > 2; });
        len(filtered)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(2.0)");
}

#[test]
fn test_integration_empty_collections() {
    let code = r#"
        let m = hashMapNew();
        let s = hashSetNew();
        let mr = hashMapFilter(m, fn(v, k) { true; });
        let sr = hashSetFilter(s, fn(x) { true; });
        hashMapSize(mr) + hashSetSize(sr)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(0.0)");
}

#[test]
fn test_parity_hashmap_map() {
    // Test same operation in interpreter
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "test", 5);
        let result = hashMapMap(map, fn(v, k) { v * 3; });
        hashMapGet(result, "test")
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("15.0"));
}
