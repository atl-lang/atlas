# Generics

Atlas supports generic type parameters on functions and structs.

## Basic Generic Functions

```atlas
fn identity<T>(x: T): T {
    return x;
}

fn pair<A, B>(first: A, second: B): A {
    return first;
}
```

## Trait Bounds

Constrain type parameters to types implementing specific traits using `extends`:

```atlas
trait Printable {
    fn to_str(borrow self): string;
}

fn print_it<T extends Printable>(item: T): string {
    return item.to_str();
}
```

When calling methods on a bounded type parameter, the typechecker resolves
the method from the trait definition and uses dynamic dispatch at runtime.

### Multiple Bounds

Use `&` to require multiple traits:

```atlas
trait Reader {
    fn read(borrow self): string;
}

trait Writer {
    fn write(borrow self, data: string): void;
}

fn copy<T extends Reader & Writer>(x: T): void {
    let data = x.read();
    x.write(data);
}
```

## Using Generic Functions

Type arguments are inferred from call site:

```atlas
struct Point { x: number, y: number }

impl Printable for Point {
    fn to_str(borrow self): string {
        return "Point";
    }
}

let p = Point { x: 1, y: 2 };
let s = print_it(p);  // T inferred as Point
```

## Error Cases

### Method Not in Bounds

```atlas
trait Counter {
    fn count(borrow self): number;
}

fn broken<T extends Counter>(x: T): string {
    return x.to_str();  // Error: Counter has no method 'to_str'
}
```

Error: `AT3010: type parameter 'T' with bounds [Counter] has no method 'to_str'`

### Missing Required Argument

```atlas
trait Adder {
    fn add(borrow self, n: number): number;
}

fn broken<T extends Adder>(x: T): number {
    return x.add();  // Error: missing argument
}
```

Error: `AT3005: function 'add' expects 1 argument(s), found 0`

## See Also

- [Traits](traits.md) - Defining and implementing traits
- [Functions](functions.md) - Function declarations
