# Atlas Type System

Atlas uses a static, structural type system. The surface syntax follows TypeScript conventions; the runtime model follows Rust. Type annotations are colon-separated (`x: number`), generics use angle brackets (`Array<T>`), and inference handles the common case so annotations are optional on local variables.

---

## Primitive Types

| Type keyword | What it represents |
|---|---|
| `number` | Single unified numeric type — no int/float split. Covers integers and floats. |
| `string` | UTF-8 string. |
| `bool` | Boolean: `true` or `false`. |
| `null` | The null value. Also serves as unit in no-value contexts. |
| `void` | Explicit return-nothing type for functions. Assignable to/from `null`. |
| `never` | Bottom type — a value that can never be produced. Inhabits every other type. |

```atlas
let n: number = 42;
let f: number = 3.14;
let s: string = "hello";
let b: bool = true;
let nothing: null = null;
```

**Design note:** `number` is intentionally unified (D-060). There is no `int`, `float`, `i32`, `u64`, etc. at the user level. AI code generators never pick the wrong numeric type.

---

## Compound Types

### Arrays

TypeScript-style postfix syntax. `T[]` is an array of `T`. Nesting is `T[][]`.

```atlas
let arr: number[] = [1, 2, 3];
let matrix: string[][] = [["a", "b"], ["c", "d"]];
```

`[]Type` (prefix syntax) is rejected with a migration error — always write `T[]`.

`Array<T>` and `T[]` are identical — the typechecker normalizes `Array<T>` to `T[]` internally.

### Tuples

Fixed-arity, heterogeneous sequences. Use parenthesized comma-separated types.

```atlas
let pair: (number, string) = (1, "one");
let triple: (bool, number, string) = (true, 0, "x");

// Destructuring
let (x, y) = pair;

// One-tuple: trailing comma required at both type and value level
let one: (number,) = (42,);
```

Without the trailing comma, `(T)` is a grouped type (equivalent to `T`), not a tuple.

### Option and Result

Rust-style error handling at the type level. Both are known to AI from Rust training.

```atlas
let opt: Option<number> = Some(42);
let none: Option<string> = None;

let ok: Result<number, string> = Ok(42);
let err: Result<number, string> = Err("division by zero");
```

Common methods: `opt.isSome()`, `opt.unwrap()`, `opt.unwrapOr(default)`, `result.isOk()`, `result.unwrapErr()`.

### HashMap and HashSet

Generic keyed collections.

```atlas
// Constructor syntax (H-374)
let map: HashMap<string, number> = new Map<string, number>();
let set: HashSet<string> = new Set<string>();
```

**Note:** The type name is currently `HashMap<K,V>` / `HashSet<V>`. Rename to `Map`/`Set` is in progress (D-060, H-373/H-374).

### JSON values

`json` is an isolated dynamic type for JSON interop. It cannot be implicitly assigned to or from any other type — extraction requires explicit methods.

```atlas
let data: json = Json.parse(raw_string);
```

### Ranges

`range` is the type produced by range expressions (`0..10`, `0..=10`). Primarily used with `for` loops and slice operations.

---

## Type Aliases

```atlas
type UserId = string;
type Pair<T> = (T, T);

// Function type alias
type Callback = (number) => void;
type BinaryOp = (left: number, right: number) => number;

// Parameterized alias
type Wrapped<T> = { value: T, label: string };
```

Aliases are fully expanded during type checking — `UserId` and `string` are interchangeable everywhere.

---

## Union Types

`A | B` — a value that may be of type `A` or type `B`. Unions are normalized automatically (nested unions are flattened, `never` members are dropped, duplicates removed).

```atlas
type StringOrNumber = string | number;
let x: string | null = null;
```

Assignability: `A | B` is assignable to `C` only if both `A` and `B` are assignable to `C`. A value of type `A` is assignable to `B | C` if `A` is assignable to either `B` or `C`.

---

## Intersection Types

`A & B` — a value that simultaneously satisfies both types. Primarily useful with structural types and trait bounds.

```atlas
type Named = { name: string };
type Aged  = { age: number };
type Person = Named & Aged;

let p: Person = { name: "Alice", age: 30 };
```

Intersecting incompatible primitives (e.g., `string & number`) normalizes to `never`.

---

## Structural Types

Inline anonymous object shape syntax. Requires at least one named member.

```atlas
fn greet(person: { name: string, age: number }): string {
    return person.name;
}
```

Structural typing is shape-compatible: any value whose field set is a superset of the required members is assignable to a structural type.

---

## Generic Types

### Using generics

Pass type arguments in angle brackets:

```atlas
let items: number[] = [1, 2, 3];           // same as Array<number>
let lookup: HashMap<string, number> = new Map<string, number>();
let result: Result<User, string> = Ok(user);
```

### Generic type parameters in aliases and structs

```atlas
type Either<L, R> = L | R;
type Box<T> = { value: T };
```

For generic functions, see `functions.md`.

---

## Function Types

Function types use `=>` arrow syntax in type position only:

```atlas
// Zero-parameter function returning string
type Thunk = () => string;

// Single parameter (name optional, only type matters)
type Transform = (number) => number;

// Named parameters — TypeScript style, names are discarded
type BinaryOp = (left: number, right: number) => number;
```

**`=>` is type-position only.** Function declarations use a colon: `fn foo(): string { }`. The arrow `->` is rejected with an error — it is the old syntax.

---

## Type Inference

The typechecker infers types for variable initializers and generic call sites:

```atlas
let message = "hello world";   // inferred: string
let count = 0;                 // inferred: number
let flags = [true, false];    // inferred: bool[]
```

Named function return types are required (see `functions.md`). Anonymous function return types are optional.

---

## Type Assignability Summary

| Assignment | Rule |
|---|---|
| `never → T` | Always valid (bottom type) |
| `void ↔ null` | Mutually assignable |
| `A\|B → C` | Valid only if `A→C` AND `B→C` |
| `A → B\|C` | Valid if `A→B` OR `A→C` |
| `A&B → C` | Valid if `A→C` OR `B→C` |
| `A → B&C` | Valid if `A→B` AND `A→C` |
| `json → T` | Not valid (isolated type) |
| `unknown → T` | Only valid if `T` is also `unknown` |

---

## Gotchas

**`number` is unified.** There is no integer type. `42` and `3.14` are both `number`. Division always produces `number`.

**`void` vs `null`.** Prefer `void` as the return type of procedures. Both work and are mutually assignable.

**Prefix array syntax is rejected.** Write `number[]`, not `[]number`. The parser emits a specific error with a migration hint.

**Structural type requires at least one member.** `{}` is not a valid structural type.

**Function type arrow (`=>`) vs function declaration colon (`:`).**
In type position: `(number) => string`.
In declaration position: `fn foo(x: number): string { }`.
These are not interchangeable. Using `->` in a declaration is an error.
