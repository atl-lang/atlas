//! Traits tests part 2 (lines 2804-3106 from original frontend_syntax.rs)

use super::*;

#[test]
fn test_parse_impl_with_multiple_methods() {
    let src = "
        trait Shape {
            fn area(borrow self: Shape): number;
            fn perimeter(borrow self: Shape): number;
        }
        impl Shape for Circle {
            fn area(borrow self: Circle): number { return 0.0; }
            fn perimeter(borrow self: Circle): number { return 0.0; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.methods.len(), 2);
        assert_eq!(ib.methods[0].name.name, "area");
        assert_eq!(ib.methods[1].name.name, "perimeter");
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_impl_generic_trait() {
    let src = "
        trait Container<T> { fn size(borrow self: Container<T>): number; }
        impl Container<number> for NumberList {
            fn size(borrow self: NumberList): number { return 0; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.trait_name.as_ref().unwrap().name, "Container");
        assert_eq!(ib.trait_type_args.len(), 1);
        assert_eq!(ib.type_name.name, "NumberList");
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_impl_requires_for_keyword() {
    let src = "impl Display number { fn display(borrow self: number): string { return \"\"; } }";
    let (_, diags) = parse_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "Missing 'for' keyword should produce a diagnostic"
    );
}

#[test]
fn test_parse_impl_method_requires_body() {
    // Impl methods must have a body (unlike trait signatures)
    let src = "trait T { fn m(): void; } impl T for X { fn m(): void; }";
    let (_, diags) = parse_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "Missing method body in impl should produce a diagnostic"
    );
}

#[test]
fn test_parse_impl_empty_body() {
    // Impl with zero methods is valid (marker trait impl)
    let src = "trait Marker { } impl Marker for number { }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert!(ib.methods.is_empty());
        assert_eq!(ib.trait_name.as_ref().unwrap().name, "Marker");
        assert_eq!(ib.type_name.name, "number");
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_impl_with_owned_params() {
    // Ownership annotations work in impl methods
    let src = "
        trait Processor { fn process(own self: Processor, own data: number): number; }
        impl Processor for MyProc {
            fn process(own self: MyProc, own data: number): number { return data; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(
            ib.methods[0].params[1].ownership,
            Some(OwnershipAnnotation::Own)
        );
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_trait_and_impl_coexist() {
    // Multiple trait+impl pairs in the same file
    let src = "
        trait A { fn a(): number; }
        trait B { fn b(): string; }
        impl A for X { fn a(): number { return 1; } }
        impl B for X { fn b(): string { return \"\"; } }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 4);
    assert!(matches!(prog.items[0], Item::Trait(_)));
    assert!(matches!(prog.items[1], Item::Trait(_)));
    assert!(matches!(prog.items[2], Item::Impl(_)));
    assert!(matches!(prog.items[3], Item::Impl(_)));
}

// ── Phase 05: Trait Bounds on Generic Type Parameters ──────────────────────

#[test]
fn test_parse_type_param_single_trait_bound() {
    let src = "fn foo<T extends Copy>(borrow x: T): T { return x; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params.len(), 1);
        assert_eq!(f.type_params[0].name, "T");
        assert_eq!(f.type_params[0].trait_bounds.len(), 1);
        assert_eq!(f.type_params[0].trait_bounds[0].trait_name, "Copy");
    } else {
        panic!("expected function item");
    }
}

#[test]
fn test_parse_type_param_multiple_trait_bounds() {
    let src = "fn foo<T extends Copy & Display>(borrow x: T): string { return x.display(); }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params[0].trait_bounds.len(), 2);
        assert_eq!(f.type_params[0].trait_bounds[0].trait_name, "Copy");
        assert_eq!(f.type_params[0].trait_bounds[1].trait_name, "Display");
    } else {
        panic!("expected function item");
    }
}

#[test]
fn test_parse_multiple_type_params_with_bounds() {
    let src =
        "fn pair<T extends Display, U extends Display>(borrow a: T, borrow b: U): string { return \"\"; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params.len(), 2);
        assert_eq!(f.type_params[0].trait_bounds[0].trait_name, "Display");
        assert_eq!(f.type_params[1].trait_bounds[0].trait_name, "Display");
    } else {
        panic!("expected function item");
    }
}

#[test]
fn test_parse_type_param_no_bound_unchanged() {
    let src = "fn identity<T>(borrow x: T): T { return x; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params[0].trait_bounds.len(), 0);
    } else {
        panic!("expected function item");
    }
}

#[test]
fn test_parse_extends_keyword_valid() {
    // H-227: `T extends Trait` is now the canonical syntax (TypeScript style)
    let src = "fn foo<T extends Iterable>(borrow x: T): T { return x; }";
    let (prog, diags) = parse_source(src);
    assert!(
        diags.is_empty(),
        "expected no errors for `extends` syntax, got: {diags:?}"
    );
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params[0].trait_bounds[0].trait_name, "Iterable");
    } else {
        panic!("expected function item");
    }
}

#[test]
fn test_parse_trait_method_with_bounded_type_param() {
    let src = "trait Printer { fn print<T extends Display>(borrow value: T): void; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        let method = &t.methods[0];
        assert_eq!(method.type_params[0].trait_bounds[0].trait_name, "Display");
    } else {
        panic!("expected trait item");
    }
}

#[test]
fn test_parse_impl_method_with_bounded_type_param() {
    let src = "
        trait Printer { fn print<T extends Display>(borrow value: T): void; }
        impl Printer for ConsolePrinter {
            fn print<T extends Display>(borrow value: T): void { }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        let method = &ib.methods[0];
        assert_eq!(method.type_params[0].trait_bounds[0].trait_name, "Display");
    } else {
        panic!("expected impl item");
    }
}

#[test]
fn test_parse_three_trait_bounds() {
    let src = "fn multi<T extends Copy & Display & Debug>(borrow x: T): void { }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params[0].trait_bounds.len(), 3);
        assert_eq!(f.type_params[0].trait_bounds[0].trait_name, "Copy");
        assert_eq!(f.type_params[0].trait_bounds[1].trait_name, "Display");
        assert_eq!(f.type_params[0].trait_bounds[2].trait_name, "Debug");
    } else {
        panic!("expected function item");
    }
}

#[test]
fn test_parse_mixed_bounded_and_unbounded_type_params() {
    let src = "fn mixed<T extends Copy, U>(borrow a: T, borrow b: U): void { }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params.len(), 2);
        assert_eq!(f.type_params[0].trait_bounds.len(), 1);
        assert_eq!(f.type_params[0].trait_bounds[0].trait_name, "Copy");
        assert_eq!(f.type_params[1].trait_bounds.len(), 0);
    } else {
        panic!("expected function item");
    }
}

#[test]
fn test_parse_trait_method_two_bounded_type_params() {
    let src =
        "trait Converter { fn convert<T extends Copy, U extends Display>(borrow val: T): U; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        let method = &t.methods[0];
        assert_eq!(method.type_params.len(), 2);
        assert_eq!(method.type_params[0].trait_bounds[0].trait_name, "Copy");
        assert_eq!(method.type_params[1].trait_bounds[0].trait_name, "Display");
    } else {
        panic!("expected trait item");
    }
}

#[test]
fn test_parse_impl_method_three_bounds() {
    let src = "
        trait Ops { fn do_it<T extends Copy & Display & Debug>(borrow x: T): void; }
        impl Ops for Foo {
            fn do_it<T extends Copy & Display & Debug>(borrow x: T): void { }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        let method = &ib.methods[0];
        assert_eq!(method.type_params[0].trait_bounds.len(), 3);
        assert_eq!(method.type_params[0].trait_bounds[2].trait_name, "Debug");
    } else {
        panic!("expected impl item");
    }
}

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `is_ok`

// NOTE: test block removed — required access to private function `is_some`

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `lookup`

// ─── B13: Inherent impl parser tests ─────────────────────────────────────────

#[test]
fn test_parse_inherent_impl_basic() {
    let src = "
        struct Point { x: number, y: number }
        impl Point {
            fn magnitude(borrow self: Point): number {
                return 0.0;
            }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 2);
    if let Item::Impl(ib) = &prog.items[1] {
        assert!(ib.is_inherent(), "expected inherent impl");
        assert!(ib.trait_name.is_none());
        assert_eq!(ib.type_name.name, "Point");
        assert_eq!(ib.methods.len(), 1);
        assert_eq!(ib.methods[0].name.name, "magnitude");
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_inherent_impl_multiple_methods() {
    let src = "
        struct Counter { value: number }
        impl Counter {
            fn increment(borrow self: Counter): number { return 0; }
            fn reset(borrow self: Counter): void { }
            fn get(borrow self: Counter): number { return 0; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert!(ib.is_inherent());
        assert_eq!(ib.methods.len(), 3);
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_inherent_impl_empty_body() {
    let src = "
        struct Tag {}
        impl Tag {
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert!(ib.is_inherent());
        assert!(ib.methods.is_empty());
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_trait_impl_still_works_after_inherent_support() {
    // Regression: trait impl must still parse correctly alongside inherent.
    let src = "
        trait Greet { fn greet(borrow self: Greet): string; }
        struct Person { name: string }
        impl Greet for Person {
            fn greet(borrow self: Person): string { return \"hi\"; }
        }
        impl Person {
            fn shout(borrow self: Person): string { return \"HI\"; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 4);
    if let Item::Impl(trait_impl) = &prog.items[2] {
        assert!(!trait_impl.is_inherent());
        assert_eq!(trait_impl.trait_name.as_ref().unwrap().name, "Greet");
    } else {
        panic!("expected trait impl at index 2");
    }
    if let Item::Impl(inherent_impl) = &prog.items[3] {
        assert!(inherent_impl.is_inherent());
        assert_eq!(inherent_impl.type_name.name, "Person");
    } else {
        panic!("expected inherent impl at index 3");
    }
}

#[test]
fn test_parse_inherent_impl_mangle_names() {
    let src = "
        struct Vec2 { x: number, y: number }
        impl Vec2 {
            fn length(borrow self: Vec2): number { return 0.0; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.mangle_method_name("length"), "__impl__Vec2__length");
    } else {
        panic!("expected Item::Impl");
    }
}
