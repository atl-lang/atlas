use super::*;

#[test]
fn test_parity_block03_scenario_a_interpreter() {
    // Multiple traits on same type
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Addable { fn add(borrow self: Addable, borrow n: number) -> number; }
        trait Subtractable { fn sub(borrow self: Subtractable, borrow n: number) -> number; }
        impl Addable for number { fn add(borrow self: number, borrow n: number) -> number { return self + n; } }
        impl Subtractable for number { fn sub(borrow self: number, borrow n: number) -> number { return self - n; } }
        let x: number = 10;
        let a: number = x.add(5);
        let b: number = a.sub(3);
        b
        ",
        )
        .expect("scenario A should succeed");
    std::assert_eq!(result, Value::Number(12.0));
}

#[test]
fn test_parity_block03_scenario_b_interpreter() {
    // Trait method returning bool, used in condition
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            r#"
        trait Comparable { fn greater_than(borrow self: Comparable, borrow other: number) -> bool; }
        impl Comparable for number {
            fn greater_than(borrow self: number, borrow other: number) -> bool { return self > other; }
        }
        let x: number = 10;
        let mut r: string = "no";
        if (x.greater_than(5)) { r = "yes"; }
        r
        "#,
        )
        .expect("scenario B should succeed");
    std::assert_eq!(result, Value::string("yes"));
}

#[test]
fn test_parity_block03_scenario_c_interpreter() {
    // Trait method calling stdlib function
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            r#"
        trait Formatted { fn fmt(borrow self: Formatted) -> string; }
        impl Formatted for number {
            fn fmt(borrow self: number) -> string { return "Value: " + str(self); }
        }
        let x: number = 42;
        let r: string = x.fmt();
        r
        "#,
        )
        .expect("scenario C should succeed");
    std::assert_eq!(result, Value::string("Value: 42"));
}

#[test]
fn test_parity_block03_scenario_d_interpreter() {
    // Chained trait method calls (via intermediate variables)
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Inc { fn inc(borrow self: Inc) -> number; }
        impl Inc for number { fn inc(borrow self: number) -> number { return self + 1; } }
        let x: number = 40;
        let y: number = x.inc();
        let z: number = y.inc();
        z
        ",
        )
        .expect("scenario D should succeed");
    std::assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_parity_block03_scenario_e_interpreter() {
    // Trait method with multiple parameters
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Interpolator { fn interpolate(borrow self: Interpolator, borrow t: number, borrow other: number) -> number; }
        impl Interpolator for number {
            fn interpolate(borrow self: number, borrow t: number, borrow other: number) -> number {
                return self + (other - self) * t;
            }
        }
        let a: number = 0;
        let r: number = a.interpolate(0.5, 100);
        r
        ",
        )
        .expect("scenario E should succeed");
    std::assert_eq!(result, Value::Number(50.0));
}

#[test]
fn test_parity_block03_scenario_f_interpreter() {
    // Trait method with conditional return paths (clamp)
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Clamp { fn clamp(borrow self: Clamp, borrow min: number, borrow max: number) -> number; }
        impl Clamp for number {
            fn clamp(borrow self: number, borrow min: number, borrow max: number) -> number {
                if (self < min) { return min; }
                if (self > max) { return max; }
                return self;
            }
        }
        let x: number = 150;
        let r: number = x.clamp(0, 100);
        r
        ",
        )
        .expect("scenario F should succeed");
    std::assert_eq!(result, Value::Number(100.0));
}

#[test]
fn test_parity_block03_scenario_g_interpreter() {
    // Impl method with local state (no leakage)
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Counter { fn count_to(borrow self: Counter, borrow n: number) -> number; }
        impl Counter for number {
            fn count_to(borrow self: number, borrow n: number) -> number {
                let mut total: number = 0;
                let mut i: number = self;
                while (i <= n) { total = total + i; i = i + 1; }
                return total;
            }
        }
        let x: number = 1;
        let r: number = x.count_to(10);
        r
        ",
        )
        .expect("scenario G should succeed");
    std::assert_eq!(result, Value::Number(55.0));
}

#[test]
fn test_parity_block03_scenario_h_interpreter() {
    // String type impl
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            r#"
        trait Shouter { fn shout(borrow self: Shouter) -> string; }
        impl Shouter for string {
            fn shout(borrow self: string) -> string { return self + "!!!"; }
        }
        let s: string = "hello";
        let r: string = s.shout();
        r
        "#,
        )
        .expect("scenario H should succeed");
    std::assert_eq!(result, Value::string("hello!!!"));
}

#[test]
fn test_parity_block03_scenario_i_interpreter() {
    // Bool type impl
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Toggle { fn toggle(borrow self: Toggle) -> bool; }
        impl Toggle for bool { fn toggle(borrow self: bool) -> bool { return !self; } }
        let b: bool = true;
        let r: bool = b.toggle();
        r
        ",
        )
        .expect("scenario I should succeed");
    std::assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_parity_block03_scenario_j_interpreter() {
    // Trait method returning array, index into result
    let atlas = Atlas::new();
    let result = atlas
        .eval(
            "
        trait Pair { fn pair(borrow self: Pair) -> []number; }
        impl Pair for number { fn pair(borrow self: number) -> []number { return [self, self * 2]; } }
        let x: number = 7;
        let p: []number = x.pair();
        let r: number = p[1];
        r
        ",
        )
        .expect("scenario J should succeed");
    std::assert_eq!(result, Value::Number(14.0));
}

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `get`

// NOTE: test block removed — required access to private function `byte_offset_to_line_column`
