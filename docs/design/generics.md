# Generic Type System Design

**Status:** Approved Design (v0.2)
**Last Updated:** 2026-02-13

---

## Objective

Add generic type parameters to Atlas, enabling parameterized types like `Result<T, E>`, `Option<T>`, and `HashMap<K, V>`. Provides type safety and code reuse without sacrificing performance.

---

## Syntax

### Type Parameters

**Generic type application:**
```atlas
Type<T1, T2, ...>
```

**Examples:**
```atlas
let opt: Option<number>;
let res: Result<string, Error>;
let map: HashMap<string, number>;
let nested: Option<Result<T, E>>;
```

### Generic Functions

**Declaration:**
```atlas
fn functionName<T, E, ...>(param: T) -> Result<T, E> {
    // body
}
```

**Examples:**
```atlas
// Single type parameter
fn identity<T>(x: T) -> T {
    return x;
}

// Multiple type parameters
fn pair<A, B>(first: A, second: B) -> Result<A, B> {
    return Ok([first, second]);
}

// Type parameter in return only
fn getDefault<T>() -> Option<T> {
    return None;
}
```

### Type Parameter Constraints (v0.2 - None)

**v0.2 Limitation:** No constraints. All type parameters are unbounded.

**Future (v0.3+):** Trait bounds like `fn sort<T: Comparable>(arr: T[])`

---

## Semantics

### Monomorphization

**Strategy:** Generate specialized code for each type instantiation (like Rust, C++).

**Why monomorphization:**
- Better performance (specialized, no runtime type checks)
- Compile-time type safety
- No runtime type information needed
- Fits Atlas's "compiled for speed" goal
- Works naturally in both interpreter and VM

**Example:**
```atlas
fn identity<T>(x: T) -> T { return x; }

let a = identity(42);        // Generates identity$number
let b = identity("hello");   // Generates identity$string
```

**Compiler generates:**
```
identity$number(x: number) -> number { return x; }
identity$string(x: string) -> string { return x; }
```

### Type Inference

**Inference from arguments:**
```atlas
fn identity<T>(x: T) -> T { return x; }

identity(42);        // T inferred as number
identity("hello");   // T inferred as string
```

**Explicit type arguments (when inference fails):**
```atlas
fn getDefault<T>() -> Option<T> { return None; }

let x = getDefault<number>();  // Explicit: T = number
```

**Inference algorithm:** Hindley-Milner style unification
- Collect constraints from usage
- Unify types via substitution
- Occurs check prevents infinite types (`T = Option<T>`)

### Type Parameter Scope

**Lexical scoping:**
```atlas
fn outer<T>() {
    fn inner<T>() {  // Different T, shadows outer T
        // ...
    }
}
```

**Type parameters are independent across functions.**

### Instantiation

**Explicit instantiation:**
```atlas
identity<number>(42)
```

**Implicit instantiation:**
```atlas
identity(42)  // T = number inferred
```

**Partial instantiation:** Not supported in v0.2

---

## Built-in Generic Types

### Option<T>

**Purpose:** Explicitly represents presence or absence of a value.

**Variants:**
```atlas
Some<T>(value: T)  // Has a value
None<T>            // No value
```

**Construction:**
```atlas
let some: Option<number> = Some(42);
let none: Option<number> = None;
```

**Usage:** Paired with pattern matching (BLOCKER 03)

### Result<T, E>

**Purpose:** Explicitly represents success or failure with error information.

**Variants:**
```atlas
Ok<T>(value: T)      // Success with value
Err<E>(error: E)     // Failure with error
```

**Construction:**
```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}
```

**Usage:** Paired with pattern matching (BLOCKER 03)

### Array<T>

**Migration:** `T[]` syntax becomes sugar for `Array<T>`

**Both syntaxes valid:**
```atlas
let arr1: number[] = [1, 2, 3];        // Sugar
let arr2: Array<number> = [1, 2, 3];  // Explicit
```

**Equivalence:** `T[]` desugars to `Array<T>` during parsing

---

## Grammar (EBNF Extension)

**Type references extended:**
```ebnf
type_ref = primitive_type
         | array_type
         | function_type
         | generic_type ;

generic_type = IDENTIFIER "<" type_arg_list ">" ;
type_arg_list = type_ref { "," type_ref } ;

array_type = type_ref "[" "]"           (* sugar for Array<T> *)
           | "Array" "<" type_ref ">" ; (* explicit *)
```

**Function declarations extended:**
```ebnf
fn_decl = "fn" IDENTIFIER type_params? "(" param_list ")" "->" type_ref block ;
type_params = "<" type_param_list ">" ;
type_param_list = IDENTIFIER { "," IDENTIFIER } ;
```

**Function calls extended:**
```ebnf
call_expr = primary_expr type_args? "(" arg_list ")" ;
type_args = "<" type_arg_list ">" ;
```

---

## Type System Extensions

### Type Representation

**Internal representation:**
```rust
enum Type {
    // Existing...
    Number, String, Bool, Null, Void,
    Array(Box<Type>),
    Function { params: Vec<Type>, return_type: Box<Type> },

    // New for generics:
    Generic {
        name: String,              // "Result", "Option", "HashMap"
        type_args: Vec<Type>,      // Instantiated arguments
    },
    TypeParameter {
        name: String,              // "T", "E", "K", "V"
    },
}
```

### Type Checking Rules

**Generic instantiation:**
- Arity must match: `Option` expects 1 arg, `Result` expects 2
- Type arguments must be valid types
- Recursive instantiation allowed: `Option<Result<T, E>>`

**Type parameter usage:**
- Must be in scope (declared in function signature)
- Can appear in parameter types, return type, body
- Cannot be shadowed in same scope

**Assignability:**
- `Generic { name: "Result", args: [Number, String] }` assignable to `Generic { name: "Result", args: [Number, String] }` (exact match)
- No variance in v0.2 (all type parameters invariant)

### Constraint Checking (v0.2 - None)

**v0.2:** No constraints, all type parameters unbounded.

**v0.3+:** Trait bounds will restrict operations on `T`.

---

## Implementation Strategy

### Phase 1: Type System Foundation (1 week)
- Add `TypeRef::Generic` to AST
- Add `Type::Generic` and `Type::TypeParameter` to type system
- Parser for `Type<T>` syntax
- Tests: parsing generic types

### Phase 2: Type Checker & Inference (2 weeks)
- Binder: type parameter scoping
- Type checker: generic validation, arity checking
- Inference: Hindley-Milner unification
- Tests: type checking, inference

### Phase 3: Runtime Implementation (2 weeks)
- Monomorphization engine
- Interpreter: generic function execution
- VM: name mangling, bytecode generation
- Tests: runtime execution, parity

### Phase 4: Built-in Types (1 week)
- Register `Option<T>` and `Result<T, E>`
- Constructors: `Some`, `None`, `Ok`, `Err`
- Array syntax migration
- Tests: built-in types usage

**Total:** 6 weeks (matches BLOCKER 02 estimate)

---

## Examples

### Basic Generics

```atlas
fn identity<T>(x: T) -> T {
    return x;
}

let num = identity(42);         // T = number
let str = identity("hello");    // T = string
```

### Multiple Type Parameters

```atlas
fn pair<A, B>(first: A, second: B) -> Result<A, B> {
    // Implementation depends on Result definition
    return Ok([first, second]);
}

let p = pair(1, "two");  // A = number, B = string
```

### Nested Generics

```atlas
fn tryParse(s: string) -> Result<Option<number>, string> {
    if (s == "") {
        return Ok(None);
    }
    // ... parse logic
    return Ok(Some(42));
}
```

### Generic with Arrays

```atlas
fn first<T>(arr: T[]) -> Option<T> {
    if (len(arr) == 0) {
        return None;
    }
    return Some(arr[0]);
}

let f = first([1, 2, 3]);  // T = number, returns Option<number>
```

---

## Limitations (v0.2)

### No User-Defined Generic Types

**Cannot define:**
```atlas
// NOT in v0.2:
struct Point<T> {
    x: T,
    y: T
}
```

**Only built-in generics:** `Option`, `Result`, `Array`

**v0.3+:** User-defined generic structs/classes

### No Constraints

**Cannot constrain:**
```atlas
// NOT in v0.2:
fn sort<T: Comparable>(arr: T[]) -> T[] {
    // ...
}
```

**v0.3+:** Trait bounds for constraints

### No Variance

**All type parameters invariant:**
```atlas
// Option<number> NOT assignable to Option<any>
// Result<string, Error> NOT assignable to Result<string, any>
```

**v0.3+:** Covariance/contravariance if needed

### No Higher-Kinded Types

**Cannot abstract over type constructors:**
```atlas
// NOT in v0.2:
fn map<F<_>, T, U>(f: F<T>, fn: (T) -> U) -> F<U>
```

**v0.3+:** If design justified

---

## Rationale

### Why Monomorphization?

**Alternatives considered:**
1. **Type Erasure** (Java, TypeScript)
   - Pros: Smaller binary, faster compile
   - Cons: Runtime overhead, no specialization, need type info at runtime

2. **Monomorphization** (Rust, C++) ✅ **CHOSEN**
   - Pros: Zero runtime cost, specialized code, no runtime type info
   - Cons: Code bloat, slower compile

3. **Reified Generics** (C#)
   - Pros: Type info at runtime, reflection
   - Cons: Runtime overhead, complex implementation

**Decision:** Monomorphization aligns with Atlas goals:
- "Compiled path for speed"
- Type safety without runtime cost
- AI agents can handle compilation complexity
- Binary size not a concern for AI tooling

### Why Angle Brackets?

**Alternatives:**
1. **Angle brackets** `<T>` ✅ **CHOSEN**
   - Universal (Rust, TypeScript, Java, C++, Swift)
   - Familiar to developers

2. **Square brackets** `[T]`
   - Used in Python type hints
   - Conflicts with array syntax

3. **Parentheses** `(T)`
   - Rare, confusing

**Decision:** Angle brackets are standard, familiar, unambiguous.

### Why Option<T> and Result<T,E>?

**Replaces:**
- Null checks (use `Option<T>`)
- Exception throwing (use `Result<T, E>`)

**Benefits:**
- Explicit error handling
- Compiler enforces checking
- Pairs with pattern matching
- AI-friendly (explicit is better than implicit)

---

## Verification

**After implementation, verify:**
1. ✅ Parse `Type<T>` syntax
2. ✅ Parse generic function declarations
3. ✅ Type check generic instantiations
4. ✅ Infer type arguments from usage
5. ✅ Monomorphize generates correct code
6. ✅ Interpreter executes generics correctly
7. ✅ VM executes generics correctly
8. ✅ 100% parity between engines
9. ✅ Option<T> and Result<T,E> work
10. ✅ Error messages clear and helpful

---

## References

**Inspiration:**
- Rust: Monomorphization, syntax, Option/Result
- TypeScript: Type inference, familiar syntax
- Swift: Generic syntax, pattern matching integration
- Haskell: Hindley-Milner inference

**Atlas Philosophy:**
- Explicit over implicit (no auto type coercion)
- AI-optimized (clear, predictable)
- Type safety without runtime cost

---

**Design Status:** ✅ Complete and approved
**Ready for Implementation:** Yes (BLOCKER 02)
