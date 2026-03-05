# Structs and Enums

This document reflects the actual parser and AST in `crates/atlas-runtime/src/parser/` and `crates/atlas-runtime/src/ast.rs`.

**Struct Declarations**
```
struct Name<T> {
    field: Type,
}
```
- Type parameters are optional.
- Trailing commas are allowed.

Example (tested):
```atlas
struct User { id: number, name: string }
```

**Struct Instantiation**
```
Name { field: value, ... }
```
- The parser only treats an identifier as a struct constructor when the name starts with an uppercase letter.

Example (tested):
```atlas
let user: User = User { id: 1, name: "Ada" };
```

**Anonymous Struct Literals**
```
{ field: value, field }
```
- Parsed as object literals with shorthand support.
- Must contain at least one field.

Example (tested):
```atlas
let point = { x: 1, y: 2 };
```

**Enum Declarations**
```
enum Name<T> {
    Unit,
    Tuple(Type, Type),
    Struct { field: Type },
}
```

Example (tested):
```atlas
enum Status {
    Ok,
    Err(string),
    Tagged { code: number },
}
```

**Enum Variant Expressions**
```
EnumName::Variant
EnumName::Variant(expr, ...)
```
- The parser supports unit and tuple variants in expressions.
- Struct variants are parsed in declarations but are **not** currently constructible with `Enum::Variant { ... }` syntax.

Example (tested):
```atlas
let ok: Status = Status::Ok;
let err: Status = Status::Err("bad");
```

**Field Access**
```
expr.field
expr.method(arg, ...)
```
- Member access parses to a unified member expression.

Example (tested):
```atlas
let id: number = user.id;
```

**Pattern Matching**
- Enum variants are matchable: `Status::Ok`, `Status::Err(x)`.
- Constructor patterns also work for `Option` and `Result`: `Some(x)`, `None`, `Ok(x)`, `Err(e)`.

Example (tested):
```atlas
let result: Result<number, string> = Ok(1);
let out: number = match result {
    Ok(x) => x,
    Err(_) => 0,
};
```

**Current limitations:** See `docs/known-issues.md`

