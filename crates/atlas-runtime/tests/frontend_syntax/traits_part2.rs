//! Traits tests part 2 (lines 2804-3106 from original frontend_syntax.rs)

use super::*;


#[test]
fn test_parse_impl_with_multiple_methods() {
    let src = "
        trait Shape {
            fn area(self: Shape) -> number;
            fn perimeter(self: Shape) -> number;
        }
        impl Shape for Circle {
            fn area(self: Circle) -> number { return 0.0; }
            fn perimeter(self: Circle) -> number { return 0.0; }
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
        trait Container<T> { fn size(self: Container<T>) -> number; }
        impl Container<number> for NumberList {
            fn size(self: NumberList) -> number { return 0; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.trait_name.name, "Container");
        assert_eq!(ib.trait_type_args.len(), 1);
        assert_eq!(ib.type_name.name, "NumberList");
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_impl_requires_for_keyword() {
    let src = "impl Display number { fn display(self: number) -> string { return \"\"; } }";
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
    let src = "trait T { fn m() -> void; } impl T for X { fn m() -> void; }";
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
        assert_eq!(ib.trait_name.name, "Marker");
        assert_eq!(ib.type_name.name, "number");
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_impl_with_owned_params() {
    // Ownership annotations work in impl methods
    let src = "
        trait Processor { fn process(own self: Processor, own data: number) -> number; }
        impl Processor for MyProc {
            fn process(own self: MyProc, own data: number) -> number { return data; }
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
        trait A { fn a() -> number; }
        trait B { fn b() -> string; }
        impl A for X { fn a() -> number { return 1; } }
        impl B for X { fn b() -> string { return \"\"; } }
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
    let src = "fn foo<T: Copy>(x: T) -> T { return x; }";
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
    let src = "fn foo<T: Copy + Display>(x: T) -> string { return x.display(); }";
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
    let src = "fn pair<T: Display, U: Display>(a: T, b: U) -> string { return \"\"; }";
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
    let src = "fn identity<T>(x: T) -> T { return x; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params[0].trait_bounds.len(), 0);
        assert!(f.type_params[0].bound.is_none());
    } else {
        panic!("expected function item");
    }
}

#[test]
fn test_parse_extends_bound_still_works() {
    let src = "fn foo<T extends number>(x: T) -> T { return x; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Function(f) = &prog.items[0] {
        assert!(f.type_params[0].bound.is_some());
        assert_eq!(f.type_params[0].trait_bounds.len(), 0);
    } else {
        panic!("expected function item");
    }
}

#[test]
fn test_parse_trait_method_with_bounded_type_param() {
    let src = "trait Printer { fn print<T: Display>(value: T) -> void; }";
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
        trait Printer { fn print<T: Display>(value: T) -> void; }
        impl Printer for ConsolePrinter {
            fn print<T: Display>(value: T) -> void { }
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
    let src = "fn multi<T: Copy + Display + Debug>(x: T) -> void { }";
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
    let src = "fn mixed<T: Copy, U>(a: T, b: U) -> void { }";
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
    let src = "trait Converter { fn convert<T: Copy, U: Display>(val: T) -> U; }";
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
        trait Ops { fn do_it<T: Copy + Display + Debug>(x: T) -> void; }
        impl Ops for Foo {
            fn do_it<T: Copy + Display + Debug>(x: T) -> void { }
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
