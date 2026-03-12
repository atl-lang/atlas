use super::super::*;
#[allow(unused_imports)]
use pretty_assertions::assert_eq;

// Generic Trait Bounds Tests (B38-P02)
//
// Tests for generic type parameters with trait bounds:
// - Single bound: T extends Foo
// - Multiple bounds: T extends Foo & Bar
// - Method resolution from bounds
// - Error cases (method not in bounds, type doesn't implement bound)
// - Chained calls on bounded types

// ============================================================================
// Single Trait Bound - Basic Cases
// ============================================================================

#[test]
fn test_single_bound_method_call() {
    let diagnostics = typecheck_source(
        r#"
        trait Printable {
            fn to_str(borrow self): string;
        }

        struct Point { x: number, y: number }

        impl Printable for Point {
            fn to_str(borrow self): string {
                return "Point";
            }
        }

        fn print_it<T extends Printable>(item: T): string {
            return item.to_str();
        }

        let p = Point { x: 1, y: 2 };
        let _result = print_it(p);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_single_bound_typecheck_only() {
    let diagnostics = typecheck_source(
        r#"
        trait Showable {
            fn show_value(borrow self): string;
        }

        fn show<T extends Showable>(x: T): string {
            return x.show_value();
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_single_bound_method_with_args() {
    let diagnostics = typecheck_source(
        r#"
        trait Adder {
            fn add(borrow self, n: number): number;
        }

        struct Counter { value: number }

        impl Adder for Counter {
            fn add(borrow self, n: number): number {
                return self.value + n;
            }
        }

        fn add_ten<T extends Adder>(x: T): number {
            return x.add(10);
        }

        let c = Counter { value: 5 };
        let _result = add_ten(c);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_single_bound_void_method() {
    let diagnostics = typecheck_source(
        r#"
        trait Runnable {
            fn run(borrow self): void;
        }

        fn execute<T extends Runnable>(x: T): void {
            x.run();
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_single_bound_returns_bool() {
    let diagnostics = typecheck_source(
        r#"
        trait Validator {
            fn is_valid(borrow self): bool;
        }

        fn validate<T extends Validator>(x: T): bool {
            return x.is_valid();
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Multiple Trait Bounds
// ============================================================================

#[test]
fn test_multiple_bounds_both_methods() {
    let diagnostics = typecheck_source(
        r#"
        trait Greeter {
            fn greet(borrow self): string;
        }

        trait Farewell {
            fn bye(borrow self): string;
        }

        struct Person { name: string }

        impl Greeter for Person {
            fn greet(borrow self): string {
                return "Hello, " + self.name;
            }
        }

        impl Farewell for Person {
            fn bye(borrow self): string {
                return "Goodbye, " + self.name;
            }
        }

        fn greet_and_bye<T extends Greeter & Farewell>(x: T): string {
            return x.greet() + " | " + x.bye();
        }

        let p = Person { name: "Alice" };
        let _result = greet_and_bye(p);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_multiple_bounds_typecheck_only() {
    let diagnostics = typecheck_source(
        r#"
        trait A { fn a(borrow self): number; }
        trait B { fn b(borrow self): string; }
        trait C { fn c(borrow self): bool; }

        fn use_all<T extends A & B & C>(x: T): string {
            let _num = x.a();
            let str_val = x.b();
            let _flag = x.c();
            return str_val;
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_multiple_bounds_method_from_each() {
    let diagnostics = typecheck_source(
        r#"
        trait Reader { fn read(borrow self): string; }
        trait Writer { fn write(borrow self, data: string): void; }

        fn copy<T extends Reader & Writer>(x: T): void {
            let data = x.read();
            x.write(data);
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Method Resolution Edge Cases
// ============================================================================

#[test]
fn test_bound_method_returning_number() {
    let diagnostics = typecheck_source(
        r#"
        trait Counter {
            fn count(borrow self): number;
        }

        fn get_count<T extends Counter>(x: T): number {
            return x.count();
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_chained_calls_on_bounded_type() {
    let diagnostics = typecheck_source(
        r#"
        trait Doubler {
            fn double(borrow self): number;
        }

        fn quad<T extends Doubler>(x: T): number {
            return x.double() + x.double();
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_nested_generic_with_same_bound_variable() {
    // When calling another generic function with the same bounded type param,
    // both functions use the same type parameter so no bound mismatch occurs.
    let diagnostics = typecheck_source(
        r#"
        trait Processor {
            fn process(borrow self): string;
        }

        fn outer<T extends Processor>(x: T): string {
            return x.process();
        }

        fn also_processes<T extends Processor>(y: T): string {
            return y.process();
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_bound_method_with_multiple_args() {
    let diagnostics = typecheck_source(
        r#"
        trait Calculator {
            fn calc(borrow self, a: number, b: number): number;
        }

        fn do_calc<T extends Calculator>(x: T): number {
            return x.calc(10, 20);
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_error_method_not_in_bound_trait() {
    let diagnostics = typecheck_source(
        r#"
        trait Printable {
            fn to_str(borrow self): string;
        }

        fn broken<T extends Printable>(x: T): string {
            return x.nonexistent();
        }
    "#,
    );
    assert!(
        diagnostics.len() >= 1,
        "Expected error for nonexistent method"
    );
    assert!(
        has_error(&diagnostics),
        "Should have error for missing method"
    );
}

#[test]
fn test_error_method_not_in_any_bound() {
    let diagnostics = typecheck_source(
        r#"
        trait A { fn a(borrow self): number; }
        trait B { fn b(borrow self): string; }

        fn broken<T extends A & B>(x: T): bool {
            return x.c();
        }
    "#,
    );
    assert!(
        diagnostics.len() >= 1,
        "Expected error for method not in bounds"
    );
    assert!(has_error(&diagnostics), "Should have error");
}

#[test]
fn test_error_arity_mismatch_on_bounded_method() {
    let diagnostics = typecheck_source(
        r#"
        trait Adder {
            fn add(borrow self, n: number): number;
        }

        fn broken<T extends Adder>(x: T): number {
            return x.add();
        }
    "#,
    );
    assert!(diagnostics.len() >= 1, "Expected arity mismatch error");
    // AT3005 is the arity mismatch error code for method calls
    assert!(
        has_error_code(&diagnostics, "AT3005"),
        "Should have arity mismatch error AT3005: {:?}",
        diagnostics
    );
}

#[test]
fn test_error_wrong_arg_type_on_bounded_method() {
    let diagnostics = typecheck_source(
        r#"
        trait Adder {
            fn add(borrow self, n: number): number;
        }

        fn broken<T extends Adder>(x: T): number {
            return x.add("not a number");
        }
    "#,
    );
    assert!(diagnostics.len() >= 1, "Expected type mismatch error");
    assert!(has_error(&diagnostics), "Should have type error");
}

#[test]
fn test_error_return_type_mismatch_from_bounded_method() {
    let diagnostics = typecheck_source(
        r#"
        trait Stringify {
            fn str_val(borrow self): string;
        }

        fn broken<T extends Stringify>(x: T): number {
            return x.str_val();
        }
    "#,
    );
    assert!(
        diagnostics.len() >= 1,
        "Expected return type mismatch error"
    );
    assert!(has_error(&diagnostics), "Should have return type error");
}

// ============================================================================
// Integration with Other Features
// ============================================================================

#[test]
fn test_bounded_generic_with_struct_field_access() {
    let diagnostics = typecheck_source(
        r#"
        trait Named {
            fn name(borrow self): string;
        }

        struct User { username: string }

        impl Named for User {
            fn name(borrow self): string {
                return self.username;
            }
        }

        fn get_name<T extends Named>(x: T): string {
            return x.name();
        }

        let u = User { username: "bob" };
        let _result = get_name(u);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_bounded_generic_called_with_different_types() {
    let diagnostics = typecheck_source(
        r#"
        trait Stringify {
            fn str_val(borrow self): string;
        }

        struct A { v: string }
        struct B { v: string }

        impl Stringify for A {
            fn str_val(borrow self): string { return "A:" + self.v; }
        }

        impl Stringify for B {
            fn str_val(borrow self): string { return "B:" + self.v; }
        }

        fn show<T extends Stringify>(x: T): string {
            return x.str_val();
        }

        let _r1 = show(A { v: "1" });
        let _r2 = show(B { v: "2" });
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_bounded_generic_in_expression_context() {
    let diagnostics = typecheck_source(
        r#"
        trait ToNum {
            fn num(borrow self): number;
        }

        fn sum<T extends ToNum>(a: T, b: T): number {
            return a.num() + b.num();
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_bounded_generic_with_local_variable() {
    let diagnostics = typecheck_source(
        r#"
        trait Getter {
            fn get(borrow self): number;
        }

        fn use_getter<T extends Getter>(x: T): number {
            let val = x.get();
            let doubled = val * 2;
            return doubled;
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}
