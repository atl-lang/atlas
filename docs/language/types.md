# Atlas Types (Typechecker-Accurate)

This document reflects the type system implemented in `crates/atlas-runtime/src/types.rs` and the typechecker (`crates/atlas-runtime/src/typechecker/`).

**Primitive Types**
- `number` — unified integer/float numeric type.
- `string`
- `bool`
- `null`

**Additional Built-in Types**
- `void` — return type for functions that return nothing.
- `any` — wildcard type used by the typechecker.
- `json` — isolated JSON value type (only assignable to `json`).

**Arrays**
- Syntax: `[]T`
- Generic form (equivalent): `Array<T>`

Example (tested):
```atlas
let numbers: []number = [1, 2, 3];
let aliases: Array<string> = ["a", "b"]; // same as []string
```

**HashMap and HashSet**
- `HashMap<K, V>` — key/value map.
- `HashSet<T>` — unique value set.

Example (tested):
```atlas
let ages: HashMap<string, number> = hashMapNew();
let tags: HashSet<string> = hashSetNew();
```

**Function Types**
- Syntax: `(T1, T2) : R`
- Function types can be generic via type parameters in declarations, not inline.

Example (tested):
```atlas
fn add(a: number, b: number) : number {
    return a + b;
}
let f: (number, number) : number = add;
```

**Structural Types**
- Syntax: `{ field: Type, method: (params) : return }`
- Must contain at least one member.

Example (tested):
```atlas
type PointLike = { x: number, y: number };
```

**Union and Intersection Types**
- Union: `A | B`
- Intersection: `A & B`

Example (tested):
```atlas
type Scalar = number | string | bool | null;
type Serializable = Scalar | json;
```

**Generics**
- Type parameters: `<T, U>`
- Optional bounds: `T extends Trait` and `T extends Trait1 & Trait2` (H-227)

Example (tested):
```atlas
fn identity<T extends Copy>(value: T) : T {
    return value;
}
```

**Structs and Enums**
- Struct types are declared with `struct` and used by name.
- Enum types are declared with `enum` and used by name.

Example (tested):
```atlas
struct User { id: number, name: string }

enum Status {
    Ok,
    Err(string),
    Tagged { code: number },
}
```

**Option<T> and Result<T, E>**
- Built-in generic types with standard constructors and helpers.
- Constructors: `Some(value)`, `None`, `Ok(value)`, `Err(value)`.
- Use bare `None` (not `None()`) — consistent in both expression and pattern position.
- Optional type suffix (`T?`) is not supported. Use `Option<T>`.

Example (tested):
```atlas
let maybe_id: Option<number> = Some(1);
let failure: Result<number, string> = Err("nope");
```

**Type Aliases**
```
type Name = Type;
type<T> Name = GenericType<T>;
```
- Creates a named alias for any type expression.
- Exportable: `export type Name = Type;`

Example (tested):
```atlas
type ID = number | string;
type StateStore = HashMap<string, string>;
type Callback = (string) : void;
```

**Mutation Semantics (CRITICAL for correct code generation)**

Atlas uses two mutation models depending on the collection type:

*Copy-on-Write (Arrays, Queue, Stack):*
Mutation functions return a NEW collection. You MUST rebind:
```atlas
let mut arr: []number = [1, 2, 3];
arr = arrayPush(arr, 4);       // CORRECT: rebind result
// arrayPush(arr, 4);          // WRONG: result is discarded, arr unchanged
```

*HashMap, HashSet — CoW value semantics:*
These use Copy-on-Write (`Arc<AtlasHashMap>` + `Arc::make_mut`) internally — same pattern as arrays. Aliases do NOT share mutable state. Always rebind:
```atlas
let map: HashMap<string, number> = hashMapNew();
map = hashMapPut(map, "key", 42);  // Rebind pattern still works
```


---

## Tuples

Tuples are immutable, fixed-length sequences of heterogeneous values. They are a lightweight grouping primitive.

**Idiomatic guidance:** Named structs are preferred for public APIs, complex records, and anything with named fields. Use tuples for:
- Iterator-adjacent pairs (index + value)
- Small local throwaway groupings
- Multi-return functions where names would be redundant

### Syntax

**Type annotation:**
```atlas
(number, string)          // 2-element tuple
(number, number, number)  // 3-element tuple
()                        // unit tuple (empty)
```

**Literal:**
```atlas
let point = (3, 4);
let labeled = (1, "hello", true);
let unit = ();
let single = (42,);   // trailing comma = 1-tuple (not grouping)
```

**Element access** — zero-indexed via `.N`:
```atlas
let t = (10, "atlas");
print(t.0);   // 10
print(t.1);   // atlas
```

**Let destructuring:**
```atlas
let (x, y) = (1, 2);
let mut (a, b) = (10, 20);
a = 99;   // mutable binding
```

**Match patterns:**
```atlas
match pair {
    (0, y) => print(y),
    (x, 0) => print(x),
    (x, y) => print(x + y)
}
```

**Function return:**
```atlas
fn min_max(arr: []number) : (number, number) {
    // ...
    return (min_val, max_val);
}
let (lo, hi) = min_max(data);
```

### Error cases

```atlas
let t = (1, 2);
t.5;              // runtime error: out-of-bounds index
let (a, b, c) = (1, 2);  // error: count mismatch (3 names, 2 elements)
let (x, y) = 42;          // error: cannot destructure non-tuple
```
