# Phase 11: Parity Test Suite

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Phase 07 + Phase 09 + Phase 10 complete

## Goal

Comprehensive parity test suite for Block 4. Every meaningful closure behavior verified in both engines. Minimum 20 new parity tests added to `tests/closures.rs`.

## Test categories

### Anonymous function basics (4 tests)
```atlas
// basic fn expression
let f = fn(x: number) -> number { x + 1; };
f(5);  // → 6

// arrow basic
let g = (x) => x * 2;
g(3);  // → 6

// immediately invoked
fn(x: number) -> number { x + 1; }(10);  // → 11

// multi-param arrow
let add = (x, y) => x + y;
add(3, 4);  // → 7
```

### Capture semantics (5 tests)
```atlas
// capture let (Copy) — identical in both engines
let n = 10;
let f = fn() -> number { n; };
f();  // → 10

// capture var at creation time — both engines snapshot
var x = 5;
let f = fn() -> number { x; };
x = 99;
f();  // → 5 (snapshot, NOT 99)

// mutation inside closure
var count = 0;
let inc = fn() -> number { count = count + 1; count; };
inc();  // → 1
inc();  // → 2

// nested closures
let make_adder = fn(n: number) -> (number) -> number {
    fn(x: number) -> number { x + n; }
};
let add5 = make_adder(5);
add5(3);  // → 8

// closure returned from fn and called later
fn make_counter() -> () -> number {
    var c = 0;
    fn() -> number { c = c + 1; c; }
}
let counter = make_counter();
counter();  // → 1
counter();  // → 2
```

### Higher-order patterns (4 tests)
```atlas
// map with arrow
[1, 2, 3].map((x) => x * 10);  // → [10, 20, 30]

// filter with fn expression
[1, 2, 3, 4].filter(fn(x: number) -> bool { x % 2 == 0; });  // → [2, 4]

// reduce with fn expression
[1, 2, 3].reduce(fn(acc: number, x: number) -> number { acc + x; }, 0);  // → 6

// function composition
let compose = fn(f: (number) -> number, g: (number) -> number) -> (number) -> number {
    fn(x: number) -> number { f(g(x)); }
};
let double_then_add1 = compose((x) => x + 1, (x) => x * 2);
double_then_add1(3);  // → 7
```

### Trait interaction (3 tests)
```atlas
// Copy type captured by copy — both engines agree
// Non-Copy type moved into closure — both engines agree
// Anon fn passed to trait-bound higher-order fn
```

### Edge cases (4+ tests)
```atlas
// zero-param arrow (not valid without parens — verify parse error)
// recursive closure via var binding
var fib: (number) -> number = fn(n: number) -> number {
    if (n <= 1) { n; } else { fib(n - 1) + fib(n - 2); }
};
fib(10);  // → 55

// closure in array
let ops = [(x) => x + 1, (x) => x * 2, (x) => x - 1];
ops[1](5);  // → 10
```

## Acceptance Criteria

- [ ] Minimum 20 new parity tests in `tests/closures.rs`
- [ ] All tests use `assert_parity_*` — both engines verified
- [ ] Zero test failures
- [ ] `cargo clippy -- -D warnings` clean
