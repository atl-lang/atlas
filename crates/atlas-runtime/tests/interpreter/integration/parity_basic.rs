use super::*;

// Arithmetic parity tests
#[rstest]
#[case("1 + 2;")]
#[case("10 - 3;")]
#[case("5 * 4;")]
#[case("20 / 4;")]
#[case("17 % 5;")]
#[case("2 + 3 * 4;")]
#[case("(2 + 3) * 4;")]
#[case("-5;")]
#[case("--5;")]
fn test_parity_arithmetic(#[case] code: &str) {
    assert_parity(code);
}

// Boolean parity tests
#[rstest]
#[case("true;")]
#[case("false;")]
#[case("!true;")]
#[case("!false;")]
#[case("true && true;")]
#[case("true && false;")]
#[case("false || true;")]
#[case("false || false;")]
#[case("1 < 2;")]
#[case("2 <= 2;")]
#[case("3 > 2;")]
#[case("3 >= 3;")]
#[case("1 == 1;")]
#[case("1 != 2;")]
fn test_parity_boolean(#[case] code: &str) {
    assert_parity(code);
}

// Variable parity tests
#[rstest]
#[case("let x = 10; x;")]
#[case("var y = 5; y = y + 1; y;")]
#[case("let a = 1; let b = 2; a + b;")]
#[case("var c = 0; c = c + 1; c = c + 1; c;")]
fn test_parity_variables(#[case] code: &str) {
    assert_parity(code);
}

// Function parity tests
#[rstest]
#[case("fn add(a: number, b: number) -> number { return a + b; } add(2, 3);")]
#[case("fn identity(x: number) -> number { return x; } identity(42);")]
#[case("fn constant() -> number { return 99; } constant();")]
#[case("fn inc(x: number) -> number { return x + 1; } inc(inc(inc(0)));")]
fn test_parity_functions(#[case] code: &str) {
    assert_parity(code);
}

// Control flow parity tests
#[rstest]
#[case("var r = 0; if (true) { r = 1; } else { r = 2; } r;")]
#[case("var r = 0; if (false) { r = 1; } else { r = 2; } r;")]
#[case("var r = 0; if (1 < 2) { r = 10; } else { r = 20; } r;")]
#[case("var x = 0; if (x == 0) { x = 1; } x;")]
fn test_parity_if_else(#[case] code: &str) {
    assert_parity(code);
}

// Loop parity tests
#[rstest]
#[case("var i = 0; while (i < 5) { i = i + 1; } i;")]
#[case("var sum = 0; var i = 0; while (i < 10) { sum = sum + i; i = i + 1; } sum;")]
#[case("var count = 0; while (count < 3) { count = count + 1; } count;")]
fn test_parity_while_loop(#[case] code: &str) {
    assert_parity(code);
}

// Array parity tests
#[rstest]
#[case("[1, 2, 3];")]
#[case("let arr = [10, 20, 30]; arr[0];")]
#[case("let arr = [1, 2, 3]; arr[2];")]
#[case("let arr: number[] = [5]; len(arr);")]
fn test_parity_arrays(#[case] code: &str) {
    assert_parity(code);
}

// String parity tests
#[rstest]
#[case(r#""hello";"#)]
#[case(r#""foo" + "bar";"#)]
#[case(r#"let s = "test"; len(s);"#)]
#[case(r#"toUpperCase("hello");"#)]
#[case(r#"toLowerCase("WORLD");"#)]
fn test_parity_strings(#[case] code: &str) {
    assert_parity(code);
}

// ============================================================================
// Phase 19: Interpreter/VM Parity — Array & Collection Operations
// ============================================================================

// Array: index read
#[rstest]
#[case("let arr: number[] = [10, 20, 30]; arr[1];")]
#[case("let arr: number[] = [10, 20, 30]; arr[0];")]
#[case("let arr: number[] = [10, 20, 30]; arr[2];")]
fn test_parity_array_index_read(#[case] code: &str) {
    assert_parity(code);
}

// Array: length
#[rstest]
#[case("let arr: number[] = [1, 2, 3]; len(arr);")]
#[case("let arr: number[] = []; len(arr);")]
#[case("let arr: number[] = [1, 2, 3]; arr.len();")]
fn test_parity_array_length(#[case] code: &str) {
    assert_parity(code);
}

// Array: push (CoW — original unaffected)
#[rstest]
#[case("var a: array = [1, 2]; var b: array = a; b.push(3); len(a);")]
#[case("var a: array = [1]; a.push(2); a.push(3); len(a);")]
fn test_parity_array_push_cow(#[case] code: &str) {
    assert_parity(code);
}

// Array: pop (CoW — pops from receiver, returns length)
#[rstest]
#[case("var a: array = [1, 2, 3]; a.pop(); len(a);")]
#[case("var a: array = [1, 2, 3]; var b: array = a; a.pop(); len(b);")]
fn test_parity_array_pop(#[case] code: &str) {
    assert_parity(code);
}

// Array: sort (returns new sorted array, receiver unchanged)
#[rstest]
#[case("var a: array = [3, 1, 2]; let s = a.sort(); s[0];")]
#[case("var a: array = [3, 1, 2]; let s = a.sort(); a[0];")]
fn test_parity_array_sort(#[case] code: &str) {
    assert_parity(code);
}

// Array: concat via + operator
#[rstest]
#[case("let a: number[] = [1, 2]; let b: number[] = [3, 4]; let c = a + b; len(c);")]
#[case("let a: number[] = [1, 2]; let b: number[] = [3, 4]; let c = a + b; c[0];")]
fn test_parity_array_concat(#[case] code: &str) {
    assert_parity(code);
}

// Array: for-each (sum over elements)
#[rstest]
#[case("var sum: number = 0; for x in [1, 2, 3] { sum = sum + x; } sum;")]
#[case("var count: number = 0; for _x in [10, 20, 30] { count = count + 1; } count;")]
fn test_parity_array_foreach(#[case] code: &str) {
    assert_parity(code);
}

// Array: map/filter with closures — both engines error (acceptable parity until Block 4)
// These are included so parity is verified even for unsupported operations.
#[rstest]
#[case("let a: number[] = [1, 2, 3]; map(a, fn(x: number) -> number { return x * 2; });")]
#[case("let a: number[] = [1, 2, 3, 4]; filter(a, fn(x: number) -> bool { return x > 2; });")]
fn test_parity_array_map_filter_both_error(#[case] code: &str) {
    assert_parity(code); // Both engines must agree (both succeed or both fail)
}

// Map (HashMap): get
#[rstest]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); unwrap(hashMapGet(m, \"a\"));")]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"x\", 42); unwrap(hashMapGet(m, \"x\"));")]
fn test_parity_hashmap_get(#[case] code: &str) {
    assert_parity(code);
}

// Map (HashMap): set with CoW — original unaffected after copy
#[rstest]
#[case("var m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); var n: HashMap = m; hashMapPut(n, \"b\", 2); hashMapSize(m);")]
fn test_parity_hashmap_set_cow(#[case] code: &str) {
    assert_parity(code);
}

// Map (HashMap): keys count
#[rstest]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); hashMapPut(m, \"b\", 2); hashMapSize(m);")]
fn test_parity_hashmap_keys(#[case] code: &str) {
    assert_parity(code);
}

// Map (HashMap): remove (delete a key)
#[rstest]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); hashMapPut(m, \"b\", 2); hashMapRemove(m, \"a\"); hashMapSize(m);")]
fn test_parity_hashmap_remove(#[case] code: &str) {
    assert_parity(code);
}

// Queue: enqueue/dequeue/size
#[rstest]
#[case("let q: Queue = queueNew(); queueEnqueue(q, 1); queueEnqueue(q, 2); queueEnqueue(q, 3); queueSize(q);")]
#[case("let q: Queue = queueNew(); queueEnqueue(q, 10); queueEnqueue(q, 20); unwrap(queueDequeue(q)); queueSize(q);")]
#[case("let q: Queue = queueNew(); queueEnqueue(q, 42); unwrap(queueDequeue(q));")]
fn test_parity_queue_operations(#[case] code: &str) {
    assert_parity(code);
}

// Stack: push/pop/size
#[rstest]
#[case(
    "let s: Stack = stackNew(); stackPush(s, 1); stackPush(s, 2); stackPush(s, 3); stackSize(s);"
)]
#[case("let s: Stack = stackNew(); stackPush(s, 10); stackPush(s, 20); unwrap(stackPop(s)); stackSize(s);")]
#[case("let s: Stack = stackNew(); stackPush(s, 99); unwrap(stackPop(s));")]
fn test_parity_stack_operations(#[case] code: &str) {
    assert_parity(code);
}

// CoW semantics: identical behavior in both engines
#[rstest]
#[case("let a: number[] = [1, 2, 3]; let b: number[] = a; a[0] = 99; b[0];")]
#[case("var a: array = [1, 2]; var b: array = a; b.push(9); len(a);")]
#[case("let a: number[] = [1, 2, 3]; let b: number[] = a; b[2] = 100; a[2];")]
fn test_parity_cow_semantics(#[case] code: &str) {
    assert_parity(code);
}
