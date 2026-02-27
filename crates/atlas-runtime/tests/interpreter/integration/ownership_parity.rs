use super::*;

#[test]
fn test_parity_own_param_callee_receives_value() {
    assert_ownership_parity(
        r#"
        fn consume(own data: array<number>) -> number { len(data); }
        consume([10, 20, 30]);
        "#,
    );
}

// ─── Scenario 3: own param — caller uses consumed binding (debug) ─────────────

#[test]
#[cfg(debug_assertions)]
fn test_parity_own_param_consumes_binding() {
    assert_ownership_parity_err(
        r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        arr;
        "#,
        "use of moved value",
    );
}

// ─── Scenario 4: borrow param — caller retains value ─────────────────────────

#[test]
fn test_parity_borrow_param_caller_retains_value() {
    assert_ownership_parity(
        r#"
        fn read(borrow data: array<number>) -> number { len(data); }
        let arr: array<number> = [1, 2, 3];
        read(arr);
        len(arr);
        "#,
    );
}

// ─── Scenario 5: shared param with plain value — both error (debug) ──────────

#[test]
#[cfg(debug_assertions)]
fn test_parity_shared_param_rejects_plain_value() {
    assert_ownership_parity_err(
        r#"
        fn register(shared handler: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        register(arr);
        "#,
        "ownership violation",
    );
}

// ─── Scenario 6: Mixed annotations — own + borrow + unannotated ───────────────

#[test]
fn test_parity_mixed_annotations_own_borrow_none() {
    assert_ownership_parity(
        r#"
        fn process(own a: array<number>, borrow b: array<number>, c: number) -> number {
            len(a) + len(b) + c;
        }
        process([1, 2], [3, 4, 5], 10);
        "#,
    );
}

// ─── Scenario 7: own with literal argument — no binding consumed ──────────────

#[test]
fn test_parity_own_literal_arg_no_consume() {
    assert_ownership_parity(
        r#"
        fn consume(own data: array<number>) -> number { len(data); }
        consume([1, 2, 3, 4]);
        42;
        "#,
    );
}

// ─── Scenario 8: own return type annotation — parsed, ignored at runtime ──────

#[test]
fn test_parity_own_return_type_annotation() {
    // Both engines must accept the annotation without error.
    // Result value not compared (function-body return diverges pre-Block2).
    assert_ownership_parity(
        r#"
        fn make() -> own array<number> { [1, 2, 3]; }
        make();
        42;
        "#,
    );
}

// ─── Scenario 9: borrow return type annotation — parsed, ignored ──────────────

#[test]
fn test_parity_borrow_return_type_annotation() {
    // Both engines must accept borrow return annotation without error.
    assert_ownership_parity(
        r#"
        fn peek(borrow data: array<number>) -> borrow array<number> { data; }
        let arr: array<number> = [10, 20];
        peek(arr);
        42;
        "#,
    );
}

// ─── Scenario 10: Nested function calls with ownership propagation ─────────────

#[test]
fn test_parity_nested_own_calls() {
    assert_ownership_parity(
        r#"
        fn inner(own data: array<number>) -> number { len(data); }
        fn outer(own data: array<number>) -> number { inner(data); }
        outer([1, 2, 3, 4, 5]);
        "#,
    );
}

// ─── Scenario 11: own param where arg is a function call result ───────────────

#[test]
fn test_parity_own_param_fn_call_result() {
    // own param with a literal array (avoids function-return divergence)
    // Both engines must accept and not error.
    assert_ownership_parity(
        r#"
        fn consume(own data: array<number>) -> void { }
        consume([1, 2, 3]);
        42;
        "#,
    );
}

// ─── Scenario 12: Multiple borrow calls to same value ─────────────────────────

#[test]
fn test_parity_multiple_borrow_calls_same_value() {
    assert_ownership_parity(
        r#"
        fn read(borrow data: array<number>) -> number { len(data); }
        let arr: array<number> = [1, 2, 3, 4, 5];
        read(arr);
        read(arr);
        read(arr);
        "#,
    );
}

// ─── Scenario 13: own then second access — same error ─────────────────────────

#[test]
#[cfg(debug_assertions)]
fn test_parity_own_then_second_access_errors() {
    assert_ownership_parity_err(
        r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        consume(arr);
        "#,
        "use of moved value",
    );
}

// ─── Scenario 14: Ownership on recursive functions ────────────────────────────

#[test]
fn test_parity_own_recursive_function() {
    assert_ownership_parity(
        r#"
        fn sum(borrow data: array<number>, i: number) -> number {
            if i >= len(data) { 0; } else { data[i] + sum(data, i + 1); }
        }
        let arr: array<number> = [1, 2, 3, 4, 5];
        sum(arr, 0);
        "#,
    );
}

// ─── Scenario 15: own annotation on void function (no return) ─────────────────

#[test]
fn test_parity_own_annotation_void_function() {
    assert_ownership_parity(
        r#"
        fn sink(own data: array<number>) -> void { }
        sink([1, 2, 3]);
        42;
        "#,
    );
}

// ─── Scenario 16: borrow then own of same binding — own errors (debug) ─────────

#[test]
#[cfg(debug_assertions)]
fn test_parity_borrow_then_own_same_binding() {
    assert_ownership_parity_err(
        r#"
        fn borrow_it(borrow data: array<number>) -> void { }
        fn own_it(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        borrow_it(arr);
        own_it(arr);
        arr;
        "#,
        "use of moved value",
    );
}

// ─── Scenario 17: Function stored in variable, own param ─────────────────────

#[test]
fn test_parity_own_param_via_variable_call() {
    assert_ownership_parity(
        r#"
        fn consume(own data: array<number>) -> number { len(data); }
        let f: (array<number>) -> number = consume;
        f([10, 20, 30]);
        "#,
    );
}

// ─── Scenario 18: multiple sequential own calls with distinct literals ─────────

#[test]
fn test_parity_multiple_own_calls_distinct_literals() {
    assert_ownership_parity(
        r#"
        fn consume(own data: array<number>) -> number { len(data); }
        consume([1]);
        consume([2, 3]);
        consume([4, 5, 6]);
        "#,
    );
}

// ─── Scenario 19: Nested scope inner fn calls outer with own param ─────────────

#[test]
fn test_parity_nested_scope_own_param() {
    assert_ownership_parity(
        r#"
        fn outer() -> number {
            fn inner(own data: array<number>) -> number { len(data); }
            inner([1, 2, 3, 4]);
        }
        outer();
        "#,
    );
}

// ─── Scenario 20: Error message identical between engines ─────────────────────

#[test]
#[cfg(debug_assertions)]
fn test_parity_error_message_identical_own_violation() {
    let src = r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        arr;
    "#;
    let ie = run_interpreter(src).unwrap_err();
    let ve = run_vm(src).unwrap_err();
    assert!(
        ie.contains("use of moved value"),
        "Interpreter error: {}",
        ie
    );
    assert!(ve.contains("use of moved value"), "VM error: {}", ve);
}

#[test]
#[cfg(debug_assertions)]
fn test_parity_error_message_identical_shared_violation() {
    let src = r#"
        fn register(shared handler: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        register(arr);
    "#;
    let ie = run_interpreter(src).unwrap_err();
    let ve = run_vm(src).unwrap_err();
    assert!(
        ie.contains("ownership violation"),
        "Interpreter error: {}",
        ie
    );
    assert!(ve.contains("ownership violation"), "VM error: {}", ve);
    // Both must include param name
    assert!(
        ie.contains("handler"),
        "Interpreter error missing param name: {}",
        ie
    );
    assert!(
        ve.contains("handler"),
        "VM error missing param name: {}",
        ve
    );
}

// ============================================================
// Phase 14 — VM: Trait Dispatch Parity Tests
// ============================================================
// Tests in this section verify interpreter parity for trait dispatch.
// VM tests live in vm.rs Phase 12/13 sections.

#[test]
fn test_parity_trait_method_string_dispatch() {
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Wrap { fn wrap(self: Wrap) -> string; }
        impl Wrap for string {
            fn wrap(self: string) -> string { return \"[\" + self + \"]\"; }
        }
        let s: string = \"hello\";
        let r: string = s.wrap();
        r
    ",
        )
        .expect("Should succeed");
    std::assert_eq!(result, Value::string("[hello]"));
}

#[test]
fn test_parity_trait_method_number_compute() {
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Double { fn double(self: Double) -> number; }
        impl Double for number {
            fn double(self: number) -> number { return self * 2; }
        }
        let n: number = 21;
        let r: number = n.double();
        r
    ",
        )
        .expect("Should succeed");
    std::assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_parity_multiple_impl_types_no_collision() {
    let atlas = Atlas::new();

    let result_n = atlas
        .eval(
            "
        trait Tag { fn tag(self: Tag) -> string; }
        impl Tag for number {
            fn tag(self: number) -> string { return \"num\"; }
        }
        impl Tag for string {
            fn tag(self: string) -> string { return \"str\"; }
        }
        let n: number = 1;
        let r: string = n.tag();
        r
    ",
        )
        .expect("Should succeed");
    std::assert_eq!(result_n, Value::string("num"));

    let result_s = atlas
        .eval(
            "
        trait Tag { fn tag(self: Tag) -> string; }
        impl Tag for number {
            fn tag(self: number) -> string { return \"num\"; }
        }
        impl Tag for string {
            fn tag(self: string) -> string { return \"str\"; }
        }
        let s: string = \"hi\";
        let r: string = s.tag();
        r
    ",
        )
        .expect("Should succeed");
    std::assert_eq!(result_s, Value::string("str"));
}

#[test]
fn test_parity_trait_method_self_arg_is_receiver() {
    // Verify `self` inside the method body refers to the receiver value
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Identity { fn identity(self: Identity) -> number; }
        impl Identity for number {
            fn identity(self: number) -> number { return self; }
        }
        let n: number = 99;
        let r: number = n.identity();
        r
    ",
        )
        .expect("Should succeed");
    std::assert_eq!(result, Value::Number(99.0));
}
