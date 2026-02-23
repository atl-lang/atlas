---
paths:
  - "tests/**"
  - "crates/atlas-runtime/fuzz/**"
---

# Atlas Language Syntax Quick-Ref

**Verified against:** token.rs, parser/mod.rs, ast.rs
**Update trigger:** Any phase that adds new syntax — update this file at GATE 7.

---

## Keywords (token.rs is_keyword)

```
let  var  fn  type  if  else  while  for  in
return  break  continue  import  export  from
extern  match  as  extends  is
own  borrow  shared  trait  impl
true  false  null
```

## Type Annotation Syntax

```
name: TypeName                    // simple
name: TypeName[]                  // array
name: (T1, T2) -> ReturnType      // function type  ← NOT fn(T) -> R
name: Type<T1, T2>                // generic
name: A | B                       // union
name: A & B                       // intersection
name: { field: Type }             // structural
```

**CRITICAL:** Function type is `(T) -> R` not `fn(T) -> R` — parser rejects `fn` as a type name.

## Function Declaration

```atlas
fn name<T, E>(x: T, y: own T) -> E { ... }
fn name<T extends number>(x: T) -> T { ... }
fn predicate(x: number) -> bool is x: PositiveNumber { ... }
```

- Return type is required — omit for `null` return
- `shared` is NOT valid as return ownership annotation
- Type params: `type_params: Vec<TypeParam>`, each has `bound: Option<TypeRef>` and `trait_bounds: Vec<TraitBound>`

## Anonymous Functions (Block 4+)

```atlas
// fn-syntax (body is a Block)
let f = fn(x: number, y: number) -> number { x + y };
// Arrow syntax (body is any Expr, desugared to Expr::AnonFn)
let f = (x) => x + 1;
let f = (x, y) => x + y;
```

- Arrow params: `type_ref: Option<TypeRef>` — may be untyped
- Both desugar to `Expr::AnonFn { params, return_type, body, span }`

## Variable Declaration

```atlas
let x = 5;          // VarDecl { mutable: false }
var x = 5;          // VarDecl { mutable: true }
let x: number = 5;  // with type annotation
```

## Ownership Annotations

```atlas
fn f(own x: T)     // move
fn f(borrow x: T)  // immutable borrow
fn f(shared x: T)  // shared mutable
-> own T           // return ownership (own/borrow only — NOT shared)
```

## Expression Statements

```atlas
f(x);     // ← semicolon REQUIRED
x + 1;    // ← semicolon REQUIRED
```

**Expression statements require semicolons.** `f(x)` without `;` is a parse error.

## Import Syntax

```atlas
import { x, y } from "./path"
import * as ns from "./path"
```
