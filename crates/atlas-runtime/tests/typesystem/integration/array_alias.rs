use super::super::*;

#[test]
fn array_generic_is_alias_for_array_syntax() {
    let src = r#"
fn first(nums: Array<number>) -> number {
    return nums[0];
}

fn sum(nums: Array<number>) -> number {
    let mut total = 0;
    for n in nums {
        total = total + n;
    }
    return total;
}

let a: number[] = [1, 2, 3];
let b: Array<number> = [1, 2, 3];

let _c: Array<number> = a;
let _d: number[] = b;

let _first_a = first(a);
let _first_b = first(b);
let _sum_a = sum(a);
let _sum_b = sum(b);
let _len_b = len(b);
"#;
    let diagnostics = typecheck_source(src);
    assert_no_errors(&diagnostics);
}
