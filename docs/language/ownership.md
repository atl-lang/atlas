# Ownership

Atlas uses an ownership system for memory safety without garbage collection.

## Ownership Annotations

Parameters must specify how they interact with values:

```atlas
fn example(borrow x: number): void { }  // Borrow (default)
fn example(own x: number): void { }     // Take ownership
fn example(share x: number): void { }   // Shared reference
```

### `borrow` (Default)

Borrows the value for read-only access. The caller retains ownership.

```atlas
fn print_value(borrow x: number): void {
    console.log(x.toString());
}

let n = 42;
print_value(n);     // n is borrowed
console.log(n);     // n still valid
```

### `own`

Takes ownership of the value. The caller cannot use it after the call.

```atlas
fn consume(own x: string): void {
    console.log(x);
}

let s = "hello";
consume(s);         // s is moved
// console.log(s);  // ERROR: s was moved
```

### `share`

Creates a shared reference. Multiple readers can access simultaneously.

```atlas
fn read_shared(share x: number): number {
    return x * 2;
}
```

## Self Receivers

Methods require explicit ownership on `self`:

```atlas
impl Point {
    fn distance(borrow self): number {     // reads self
        return Math.sqrt(self.x * self.x + self.y * self.y);
    }

    fn reset(own self): Point {            // consumes self
        return Point { x: 0, y: 0 };
    }
}
```

## Error Codes

| Code   | Description                              |
|--------|------------------------------------------|
| AT3053 | Use after move                           |
| AT3054 | Moved value used in subsequent argument  |
| AT3055 | Cannot take ownership of borrowed value  |
| AT3058 | Missing ownership on self parameter      |

## Examples

### Basic Ownership Transfer

```atlas
struct Buffer {
    data: string,
}

fn process(own buf: Buffer): string {
    return buf.data;
}

let buf = Buffer { data: "hello" };
let result = process(buf);
// buf is no longer valid here
```

### Borrowing for Read Access

```atlas
fn sum(borrow arr: number[]): number {
    let mut total = 0;
    for n in arr {
        total = total + n;
    }
    return total;
}

let nums = [1, 2, 3, 4, 5];
console.log(sum(nums).toString());     // 15
console.log(nums.length().toString()); // 5 - still valid
```
