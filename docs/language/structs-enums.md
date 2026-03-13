# Structs and Enums

Atlas structs and enums are nominal, statically-typed data types. Structs hold named fields. Enums hold tagged variants with optional associated data. Both support generics, visibility modifiers, methods via `impl`, and pattern matching.

---

## Structs

### Declaration

```atlas
struct Point {
    x: number,
    y: number,
}
```

Fields are `name: Type` pairs separated by commas. A trailing comma before `}` is allowed. All fields are required — there are no optional fields at the struct level (use `Option<T>` for nullable fields).

### Generic Structs

```atlas
struct Pair<T, U> {
    first: T,
    second: U,
}

struct Stack<T> {
    items: T[],
    capacity: number,
}
```

### Visibility

```atlas
pub struct User {
    id: number,
    name: string,
}
```

Visibility modifiers (`pub`, `private`, `internal`) precede the `struct` keyword. Default is `private` (file-local).

### Instantiation

```atlas
let p = Point { x: 10, y: 20 };
let pair = Pair<number, string> { first: 1, second: "one" };
```

All fields must be provided. Field order does not matter.

### Field Access

```atlas
let p = Point { x: 3, y: 4 };
console.log(p.x.toString());  // "3"
console.log(p.y.toString());  // "4"
```

Field access uses `.` notation. Fields always return their declared type — never `unknown` or nullable unless the declared type is `Option<T>` or a union.

### Field Mutation

Fields can be mutated on `let mut` bindings:

```atlas
let mut p = Point { x: 0, y: 0 };
p.x = 10;
p.y = 20;
```

### Methods via impl

Methods are attached to structs through `impl` blocks:

```atlas
impl Point {
    fn distance(self): number {
        let sq = self.x * self.x + self.y * self.y;
        return math.sqrt(sq).unwrap();
    }

    fn scale(self, factor: number): Point {
        return Point { x: self.x * factor, y: self.y * factor };
    }

    static fn origin(): Point {
        return Point { x: 0, y: 0 };
    }
}

let p = Point { x: 3, y: 4 };
let d = p.distance();         // instance method call
let p2 = Point.origin();      // static method call
```

See the [Traits](traits.md) documentation for full impl details.

### Struct Patterns in match

```atlas
struct Point { x: number, y: number }

let p = Point { x: 3, y: 4 };

match p {
    Point { x: 0, y: 0 } => console.log("origin"),
    Point { x, y } => console.log(`(${x}, ${y})`),
}
```

Shorthand `{ x, y }` binds the field value to a variable of the same name. Explicit form `{ x: px, y: py }` binds to renamed variables.

---

## Enums

### Declaration

```atlas
enum Direction {
    North,
    South,
    East,
    West,
}
```

### Unit Variants

Variants with no associated data are unit variants:

```atlas
enum Status {
    Active,
    Inactive,
    Pending,
}

let s = Status::Active;
```

Unit variants are referenced with `::` path syntax: `EnumName::VariantName`.

### Tuple Variants

Variants can carry positional data:

```atlas
enum Color {
    Red,
    Green,
    Blue,
    Rgb(number, number, number),
}

let c = Color::Red;
let custom = Color::Rgb(255, 128, 0);
```

### Struct Variants

Variants can carry named fields:

```atlas
enum Shape {
    Circle { radius: number },
    Rectangle { width: number, height: number },
    Point,
}

let s = Shape::Circle { radius: 5.0 };
```

### Generic Enums

```atlas
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`Option` and `Result` are built into the stdlib and follow this shape.

### Visibility

```atlas
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}
```

### Enum Patterns in match

Pattern matching on enums is the primary way to access variant data. Atlas supports both `EnumName::Variant` (qualified) and bare variant names for patterns.

**Qualified patterns:**

```atlas
let color = Color::Rgb(255, 128, 0);

match color {
    Color::Red => console.log("red"),
    Color::Green => console.log("green"),
    Color::Blue => console.log("blue"),
    Color::Rgb(r, g, b) => console.log(`rgb(${r}, ${g}, ${b})`),
}
```

**Bare variant patterns** (uppercase identifiers not in the built-in set are treated as bare variants):

```atlas
match status {
    Active => console.log("active"),
    Inactive => console.log("inactive"),
    Pending => console.log("pending"),
}
```

**Struct variant patterns:**

```atlas
match shape {
    Shape::Circle { radius } => console.log(`circle r=${radius}`),
    Shape::Rectangle { width, height } => console.log(`rect ${width}x${height}`),
    Shape::Point => console.log("point"),
}
```

**OR patterns:**

```atlas
match color {
    Color::Red | Color::Green | Color::Blue => console.log("primary"),
    _ => console.log("other"),
}
```

**Guard clauses:**

```atlas
match color {
    Color::Rgb(r, g, b) if r > 200 => console.log("bright red dominant"),
    Color::Rgb(r, g, b) => console.log("other rgb"),
    _ => console.log("solid color"),
}
```

**Wildcard:**

```atlas
match value {
    42 => console.log("the answer"),
    _ => console.log("something else"),
}
```

---

## Pattern Reference

All patterns available in `match` arms:

| Pattern | Example | Matches |
|---------|---------|---------|
| Literal | `42`, `"hello"`, `true` | Exact value |
| Wildcard | `_` | Anything (no binding) |
| Variable | `x` | Anything, binds to `x` |
| Constructor | `Ok(v)`, `Some(x)` | Built-in constructors |
| Enum variant | `Color::Red`, `Color::Rgb(r, g, b)` | Qualified enum variant |
| Bare variant | `Active`, `Pending(msg)` | Unqualified enum variant |
| Struct | `Point { x, y }` | Named struct fields |
| Array | `[a, b, c]`, `[]` | Array elements |
| Tuple | `(a, b)` | Tuple elements |
| OR | `A \| B \| C` | Any of the listed patterns |

---

## Combining Structs and Enums

```atlas
struct Point {
    x: number,
    y: number,
}

enum Shape {
    Circle { center: Point, radius: number },
    Line { start: Point, end: Point },
}

let s = Shape::Circle {
    center: Point { x: 0, y: 0 },
    radius: 10.0,
};

match s {
    Shape::Circle { center, radius } => {
        console.log(`circle at (${center.x}, ${center.y}) r=${radius}`);
    }
    Shape::Line { start, end } => {
        console.log("line");
    }
}
```

---

## Gotchas

**All struct fields required at instantiation.** There is no struct update syntax (`..other`) or partial initialization. Every field must be explicitly provided.

**Enum variants use `::` for construction.** `Color.Red` is not valid — use `Color::Red`. The dot `.` is reserved for method calls and field access.

**`match` must be exhaustive.** The compiler requires all variants to be covered. Use `_` as a catch-all if not all cases need explicit handling.

**Bare variant patterns require an uppercase first letter.** The parser treats uppercase identifiers as potential bare variant names. Lowercase identifiers in patterns are always treated as variable bindings.

**No implicit copy.** Structs are moved (transferred ownership) when assigned or passed. Use `borrow` parameters to avoid consuming the value. Value types (`number`, `bool`, `string`) copy implicitly.
