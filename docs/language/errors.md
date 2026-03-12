# Error Handling

Atlas uses `Result<T, E>` and `Option<T>` types for error handling, with the `?` operator for ergonomic error propagation.

## Result Type

`Result<T, E>` represents either success (`Ok(T)`) or failure (`Err(E)`).

```atlas
fn divide(borrow a: number, borrow b: number): Result<number, string> {
    if b == 0 {
        return Err("division by zero");
    }
    return Ok(a / b);
}

let result = divide(10, 2);
match result {
    Ok(v) => console.log(v.toString()),
    Err(e) => console.log("Error: " + e),
}
```

## Option Type

`Option<T>` represents either a value (`Some(T)`) or absence (`None`).

```atlas
fn find(borrow arr: []number, borrow target: number): Option<number> {
    for i in 0..arr.length() {
        if arr[i] == target {
            return Some(i);
        }
    }
    return None;
}

let idx = find([1, 2, 3], 2);
if idx.isSome() {
    console.log("Found at: " + idx.unwrap().toString());
}
```

## The `?` Operator

The `?` operator unwraps `Ok`/`Some` or propagates `Err`/`None`.

### With Result

```atlas
fn parse_and_double(borrow s: string): Result<number, string> {
    let n = parseInt(s)?;  // propagates Err if parsing fails
    return Ok(n * 2);
}

fn main(): Result<number, string> {
    let x = parse_and_double("21")?;
    let y = parse_and_double("10")?;
    return Ok(x + y);  // Ok(52)
}
```

### With Option

```atlas
fn get_first(borrow arr: []number): Option<number> {
    return arr.get(0);
}

fn double_first(borrow arr: []number): Option<number> {
    let first = get_first(arr)?;  // propagates None if empty
    return Some(first * 2);
}
```

### Chaining

```atlas
fn complex_operation(): Result<number, string> {
    let a = step_one()?;
    let b = step_two(a)?;
    let c = step_three(b)?;
    return Ok(c);
}
```

## Methods

### Result Methods

| Method          | Description                                    |
|-----------------|------------------------------------------------|
| `isOk()`        | Returns true if Ok                             |
| `isErr()`       | Returns true if Err                            |
| `unwrap()`      | Returns Ok value, panics on Err                |
| `unwrapOr(v)`   | Returns Ok value or default                    |
| `unwrapErr()`   | Returns Err value, panics on Ok                |
| `map(fn)`       | Transforms Ok value                            |
| `mapErr(fn)`    | Transforms Err value                           |

### Option Methods

| Method          | Description                                    |
|-----------------|------------------------------------------------|
| `isSome()`      | Returns true if Some                           |
| `isNone()`      | Returns true if None                           |
| `unwrap()`      | Returns Some value, panics on None             |
| `unwrapOr(v)`   | Returns Some value or default                  |
| `map(fn)`       | Transforms Some value                          |
