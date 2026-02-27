use super::super::*;
use super::type_guards_part1::eval;
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) || isNumber(x)) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isType(x, "string")) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isString(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && !isNull(x)) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "string") || isType(x, "number")) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isNumber(x)) { return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; }
        if (isNumber(x)) { let _y: number = x; }
        return 1;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; return 1; }
        if (isNumber(x)) { let _y: number = x; return 2; }
        return 3;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        var result: number = 0;
        if (isString(x)) { result = 1; }
        if (isNumber(x)) { result = 2; }
        return result;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        while (isString(x)) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
fn test_guard_composition_and_flow(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Structural + discriminated guards
// =============================================================================

#[rstest]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name")) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn test(x: WithLen | WithId) -> number {
        if (hasMethod(x, "len")) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithTag = { tag: string, value: number };
    type WithNumTag = { tag: number, message: string };
    fn test(x: WithTag | WithNumTag) -> number {
        if (hasTag(x, "ok")) { let _y: WithTag = x; return 1; }
        else { let _y: WithNumTag = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { name: string, id: number };
    type Two = { id: number };
    fn test(x: One | Two) -> number {
        if (hasField(x, "name")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { len: () -> number, id: number };
    type Two = { id: number };
    fn test(x: One | Two) -> number {
        if (hasMethod(x, "len")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { tag: string, id: number };
    type Two = { tag: number, id: number };
    fn test(x: One | Two) -> number {
        if (hasTag(x, "one")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name") && hasField(x, "name")) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name") || hasField(x, "id")) { let _y: WithName | WithId = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (!hasField(x, "name")) { let _y: WithId = x; return 1; }
        else { let _y: WithName = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name")) { let _y: { name: string } = x; return 1; }
        else { let _y: { id: number } = x; return 2; }
    }
    "#
)]
fn test_structural_guards(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// isType guard tests
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "string")) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: bool | null) -> number {
        if (isType(x, "bool")) { let _y: bool = x; return 1; }
        else { let _y: null = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: null | string) -> number {
        if (isType(x, "null")) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number[] | string) -> number {
        if (isType(x, "array")) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn f(x: number) -> number { return x; }
    fn test(x: ((number) -> number) | string) -> number {
        if (isType(x, "function")) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isType(x, "json")) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isType(x, "object")) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number") || isType(x, "string")) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isType(x, "string")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_is_type_guard(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Runtime guard behavior
// =============================================================================

#[rstest]
#[case("isString(\"ok\")", Value::Bool(true))]
#[case("isString(1)", Value::Bool(false))]
#[case("isNumber(1)", Value::Bool(true))]
#[case("isBool(true)", Value::Bool(true))]
#[case("isNull(null)", Value::Bool(true))]
#[case("isArray([1, 2])", Value::Bool(true))]
#[case("isType(\"ok\", \"string\")", Value::Bool(true))]
#[case("isType(1, \"number\")", Value::Bool(true))]
#[case("isType([1, 2], \"array\")", Value::Bool(true))]
#[case("isType(null, \"null\")", Value::Bool(true))]
fn test_runtime_basic_guards(#[case] expr: &str, #[case] expected: Value) {
    let code = expr.to_string();
    let result = eval(&code);
    assert_eq!(result, expected);
}

#[rstest]
#[case(
    r#"
    let obj = parseJSON("{\"tag\":\"ok\", \"value\": 1}");
    hasTag(obj, "ok")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"tag\":\"bad\", \"value\": 1}");
    hasTag(obj, "ok")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasField(obj, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasField(obj, "missing")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasMethod(obj, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasMethod(obj, "missing")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    isObject(obj)
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    isType(obj, "object")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let hmap = hashMapNew();
    hashMapPut(hmap, "name", 1);
    hasField(hmap, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let hmap = hashMapNew();
    hashMapPut(hmap, "tag", "ok");
    hasTag(hmap, "ok")
    "#,
    Value::Bool(true)
)]
fn test_runtime_structural_guards(#[case] code: &str, #[case] expected: Value) {
    let result = eval(code);
    assert_eq!(result, expected);
}

// ============================================================================

// NOTE: test block removed — required access to private function `merge_flow_states`
