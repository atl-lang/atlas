use super::parse_and_bind;
use pretty_assertions::assert_eq;

#[test]
fn test_bind_nested_function_in_while_block() {
    let source = r#"
        fn outer() -> number {
            let mut i: number = 0;
            while (i < 1) {
                fn helper() -> number {
                    return 42;
                }
                i += 1;
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_in_for_block() {
    let source = r#"
        fn outer() -> number {
            for i in [0, 1, 2, 3, 4] {
                fn helper(x: number) -> number {
                    return x;
                }
                let _unused = i;
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}
