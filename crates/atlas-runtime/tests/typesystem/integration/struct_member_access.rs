use super::super::*;

#[test]
fn test_struct_field_access_type_inference() {
    let diagnostics = typecheck_source(
        r#"
        struct Item { id: number }
        let item = Item { id: 5 };
        let item_id: number = item.id;
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

#[test]
fn test_struct_field_access_comparison() {
    let diagnostics = typecheck_source(
        r#"
        struct Item { id: number }
        let item = Item { id: 5 };
        if (item.id == 5) { }
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

#[test]
fn test_struct_field_access_assignment() {
    let diagnostics = typecheck_source(
        r#"
        struct Item { id: number }
        let mut item = Item { id: 5 };
        item.id = 6;
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

#[test]
fn test_struct_field_access_nested() {
    let diagnostics = typecheck_source(
        r#"
        struct Address { city: string }
        struct Person { address: Address }
        let person = Person { address: Address { city: "NY" } };
        let city: string = person.address.city;
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

#[test]
fn test_struct_field_access_global() {
    let diagnostics = typecheck_source(
        r#"
        struct Point { x: number }
        let mut point = Point { x: 1 };
        let x: number = point.x;
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

#[test]
fn test_struct_field_access_in_closure() {
    let diagnostics = typecheck_source(
        r#"
        struct Item { id: number }
        let item = Item { id: 5 };
        let get_id = fn (): number {
            return item.id;
        };
        let value: number = get_id();
        "#,
    );
    assert!(!has_error(&diagnostics), "Errors: {:?}", diagnostics);
}

// H-117: struct []T as fn parameter — binder stores ?[], typechecker must update to struct type
#[test]
fn struct_array_fn_param_resolves_correctly() {
    let src = r#"
struct Point { x: number, y: number }

fn sum_x(borrow pts: Point[]): number {
    let mut total: number = 0;
    for p in pts {
        total = total + p.x;
    }
    return total;
}

let arr: Point[] = [Point { x: 1, y: 2 }];
let result: number = sum_x(arr);
"#;
    let diagnostics = typecheck_source(src);
    assert_no_errors(&diagnostics);
}
