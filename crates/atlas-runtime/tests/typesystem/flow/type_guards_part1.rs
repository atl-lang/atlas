use super::super::*;
// From type_guard_tests.rs
// ============================================================================

// Tests for type guard predicates and narrowing.

pub(super) fn eval(code: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(code).expect("Interpretation failed")
}

// =============================================================================
// Predicate syntax + validation
// =============================================================================

#[rstest]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is x: string { return isString(x); }
    fn test(x: number | string) -> number {
        if (isStr(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNum(x: number | string) -> bool is x: number { return isNumber(x); }
    fn test(x: number | string) -> number {
        if (isNum(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isBoolish(x: bool | null) -> bool is x: bool { return isBool(x); }
    fn test(x: bool | null) -> bool {
        if (isBoolish(x)) { let _y: bool = x; return _y; }
        else { return false; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn hasName(x: WithName | WithId) -> bool is x: WithName { return hasField(x, "name"); }
    fn test(x: WithName | WithId) -> number {
        if (hasName(x)) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn hasLen(x: WithLen | WithId) -> bool is x: WithLen { return hasMethod(x, "len"); }
    fn test(x: WithLen | WithId) -> number {
        if (hasLen(x)) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type Ok = { tag: string, value: number };
    type Err = { tag: number, message: string };
    fn isOk(x: Ok | Err) -> bool is x: Ok { return hasTag(x, "ok"); }
    fn test(x: Ok | Err) -> number {
        if (isOk(x)) { let _y: Ok = x; return 1; }
        else { let _y: Err = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNullish(x: null | string) -> bool is x: null { return isNull(x); }
    fn test(x: null | string) -> number {
        if (isNullish(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isObj(x: json | string) -> bool is x: json { return isObject(x); }
    fn test(x: json | string) -> number {
        if (isObj(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isArr(x: number[] | string) -> bool is x: number[] { return isArray(x); }
    fn test(x: number[] | string) -> number {
        if (isArr(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isFunc(x: ((number) -> number) | string) -> bool is x: (number) -> number { return isFunction(x); }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunc(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_predicate_syntax_valid(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

#[rstest]
#[case(
    r#"
    fn isStr(x: number) -> number is x: number { return 1; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is y: number { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: string { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number {
        return 1; // return type mismatch
    }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) is x: number { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { let _y: string = x; return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is x: bool { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is missing: string { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { return false; }
    fn test(x: number | string) -> number { if (isStr(x)) { return 1; } return 2; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { return true; }
    fn test(x: number) -> number { if (isStr(x)) { let _y: string = x; } return 1; }
    "#
)]
fn test_predicate_syntax_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// =============================================================================
// Built-in guard narrowing
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isNumber(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: bool | null) -> number {
        if (isBool(x)) { let _y: bool = x; return 1; }
        else { let _y: null = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: null | string) -> number {
        if (isNull(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number[] | string) -> number {
        if (isArray(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn f(x: number) -> number { return x; }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunction(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isObject(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
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
        if (isType(x, "number")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_builtin_guard_narrowing(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// User-defined guards
// =============================================================================

#[rstest]
#[case(
    r#"
    fn isText(x: number | string) -> bool is x: string { return isString(x); }
    fn test(x: number | string) -> number {
        if (isText(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn isNamed(x: WithName | WithId) -> bool is x: WithName { return hasField(x, "name"); }
    fn test(x: WithName | WithId) -> number {
        if (isNamed(x)) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn isLen(x: WithLen | WithId) -> bool is x: WithLen { return hasMethod(x, "len"); }
    fn test(x: WithLen | WithId) -> number {
        if (isLen(x)) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNum(x: number | string) -> bool is x: number { return isNumber(x); }
    fn test(x: number | string) -> number {
        if (isNum(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isArr(x: number[] | string) -> bool is x: number[] { return isArray(x); }
    fn test(x: number[] | string) -> number {
        if (isArr(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNullish(x: null | string) -> bool is x: null { return isNull(x); }
    fn test(x: null | string) -> number {
        if (isNullish(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type Ok = { tag: string, value: number };
    type Err = { tag: number, message: string };
    fn isOk(x: Ok | Err) -> bool is x: Ok { return hasTag(x, "ok"); }
    fn test(x: Ok | Err) -> number {
        if (isOk(x)) { let _y: Ok = x; return 1; }
        else { let _y: Err = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isObj(x: json | string) -> bool is x: json { return isObject(x); }
    fn test(x: json | string) -> number {
        if (isObj(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isFunc(x: ((number) -> number) | string) -> bool is x: (number) -> number { return isFunction(x); }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunc(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isTypeString(x: number | string) -> bool is x: string { return isType(x, "string"); }
    fn test(x: number | string) -> number {
        if (isTypeString(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
fn test_user_defined_guards(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Guard composition + control flow
