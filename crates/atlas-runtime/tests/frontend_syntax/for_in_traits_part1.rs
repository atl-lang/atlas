//! For-in and traits tests part 1 (lines 2426-2803 from original frontend_syntax.rs)

use super::*;

// For-In Parsing Tests (from test_for_in_parsing.rs)
// ============================================================================

#[test]
fn test_parse_for_in_basic() {
    let source = r#"
        for item in array {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty(), "Lexer should not produce errors");

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(parse_diags.is_empty(), "Should parse for-in loop");
}

#[test]
fn test_parse_for_in_with_array_literal() {
    let source = r#"
        for x in [1, 2, 3] {
            print(x);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with array literal"
    );
}

#[test]
fn test_parse_for_in_empty_body() {
    let source = r#"
        for x in arr {
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with empty body"
    );
}

#[test]
fn test_parse_for_in_nested() {
    let source = r#"
        for outer in outerArray {
            for inner in innerArray {
                print(inner);
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(parse_diags.is_empty(), "Should parse nested for-in loops");
}

#[test]
fn test_parse_for_in_with_function_call() {
    let source = r#"
        for item in getArray() {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with function call"
    );
}

#[test]
fn test_parse_for_in_error_missing_in() {
    let source = r#"
        for item array {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(!parse_diags.is_empty(), "Should error without 'in' keyword");
}

#[test]
fn test_parse_for_in_error_missing_variable() {
    let source = r#"
        for in array {
            print(x);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        !parse_diags.is_empty(),
        "Should error without variable name"
    );
}

#[test]
fn test_traditional_for_still_works() {
    let source = r#"
        for (let i = 0; i < 10; i = i + 1) {
            print(i);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Traditional for loops should still work"
    );
}

#[test]
fn test_parse_for_in_with_method_call() {
    let source = r#"
        for item in obj.getItems() {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with method call"
    );
}

#[test]
fn test_parse_for_in_with_complex_body() {
    let source = r#"
        for item in items {
            if (item > 5) {
                print("Large: " + toString(item));
            } else {
                print("Small: " + toString(item));
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with complex body"
    );
}

// ============================================================================
// Block 3: Trait system — parser tests
// ============================================================================

#[test]
fn test_parse_empty_trait() {
    let (prog, diags) = parse_source("trait Marker { }");
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 1);
    assert!(matches!(prog.items[0], Item::Trait(_)));
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.name.name, "Marker");
        assert!(t.methods.is_empty());
        assert!(t.type_params.is_empty());
    }
}

#[test]
fn test_parse_trait_single_method() {
    let src = "trait Display { fn display(self: Display) -> string; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 1);
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.name.name, "Display");
        assert_eq!(t.methods.len(), 1);
        assert_eq!(t.methods[0].name.name, "display");
        assert_eq!(t.methods[0].params.len(), 1);
        assert_eq!(t.methods[0].params[0].name.name, "self");
    } else {
        panic!("expected Item::Trait");
    }
}

#[test]
fn test_parse_trait_multiple_methods() {
    let src = "trait Comparable {
        fn compare(self: Comparable, other: Comparable) -> number;
        fn equals(self: Comparable, other: Comparable) -> bool;
    }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.methods.len(), 2);
        assert_eq!(t.methods[0].name.name, "compare");
        assert_eq!(t.methods[0].params.len(), 2);
        assert_eq!(t.methods[1].name.name, "equals");
        assert_eq!(t.methods[1].params.len(), 2);
    } else {
        panic!("expected Item::Trait");
    }
}

#[test]
fn test_parse_generic_trait() {
    let src = "trait Container<T> { fn get(self: Container<T>, index: number) -> T; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.name.name, "Container");
        assert_eq!(t.type_params.len(), 1);
        assert_eq!(t.type_params[0].name, "T");
        assert_eq!(t.methods.len(), 1);
    } else {
        panic!("expected Item::Trait");
    }
}

#[test]
fn test_parse_trait_method_with_ownership_params() {
    let src = "trait Processor { fn process(own data: number) -> number; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(
            t.methods[0].params[0].ownership,
            Some(OwnershipAnnotation::Own)
        );
    } else {
        panic!("expected Item::Trait");
    }
}

#[test]
fn test_trait_method_requires_semicolon() {
    // Missing semicolon after method sig — parse error
    let src = "trait Foo { fn bar() -> number }";
    let (_, diags) = parse_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "Missing semicolon should produce a diagnostic"
    );
}

#[test]
fn test_trait_method_with_body_is_error() {
    // Trait method sigs have no body — `{` after return type is unexpected
    let src = "trait Foo { fn bar() -> number { return 1; } }";
    let (_, diags) = parse_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "Method body in trait declaration should fail"
    );
}

#[test]
fn test_trait_coexists_with_functions() {
    let src = "trait Display { fn display(self: Display) -> string; }
               fn greet() -> string { return \"hello\"; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 2);
    assert!(matches!(prog.items[0], Item::Trait(_)));
    assert!(matches!(prog.items[1], Item::Function(_)));
}

#[test]
fn test_parse_trait_multiple_type_params() {
    let src = "trait BiMap<K, V> {
        fn get(self: BiMap<K, V>, key: K) -> V;
        fn set(self: BiMap<K, V>, key: K, value: V) -> void;
    }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.type_params.len(), 2);
        assert_eq!(t.type_params[0].name, "K");
        assert_eq!(t.type_params[1].name, "V");
        assert_eq!(t.methods.len(), 2);
    } else {
        panic!("expected Item::Trait");
    }
}

#[test]
fn test_parse_trait_method_no_params() {
    let src = "trait Default { fn default() -> number; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.methods[0].params.len(), 0);
    } else {
        panic!("expected Item::Trait");
    }
}

// ============================================================================
// Block 3: Trait system — impl block parser tests
// ============================================================================

#[test]
fn test_parse_simple_impl_block() {
    let src = "
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 2);
    assert!(matches!(prog.items[1], Item::Impl(_)));
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.trait_name.name, "Display");
        assert_eq!(ib.type_name.name, "number");
        assert_eq!(ib.methods.len(), 1);
        assert_eq!(ib.methods[0].name.name, "display");
    } else {
        panic!("expected Item::Impl");
    }
}
