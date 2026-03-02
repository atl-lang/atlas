# Archived Patterns — 2026-02

These patterns are stable and documented in quick-refs. Kept here for historical reference.

## Test Structure

**rstest:**
```rust
#[rstest]
#[case(input, expected)]
fn test_name(#[case] input: X, #[case] expected: Y) { ... }
```

**insta:**
```rust
use insta::assert_snapshot;
assert_snapshot!(output);
```

## Frontend API

**Lexer** — NOT iterator:
```rust
let (tokens, diagnostics) = Lexer::new(source).tokenize();
```

**Parser** — Returns tuple:
```rust
let (program, diagnostics) = Parser::new(tokens).parse();
```

**Compiler** — Returns Result:
```rust
let bytecode = Compiler::new().compile(&program)?;
```

## Value Type Checking

```rust
impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            // etc.
        }
    }
}
```

## Trait Dispatch (Block 3 — Static Dispatch)

Impl methods compile to mangled globals: `__impl__TypeName__TraitName__MethodName`

**Typechecker annotation:** `MemberExpr.trait_dispatch: RefCell<Option<(String, String)>>`
Set by `resolve_trait_method_call_with_info()` in typechecker/expr.rs.

**Compiler (expr.rs `compile_member`):**
```rust
if let Some((type_name, trait_name)) = member.trait_dispatch.borrow().clone() {
    let mangled = format!("__impl__{}__{}_{}", type_name, trait_name, method_name);
    emit GetGlobal(mangled), then compile receiver + args, then emit Call(arg_count)
}
```

**Interpreter (expr.rs `eval_member`):**
```rust
if let Some((type_name, trait_name)) = member.trait_dispatch.borrow().clone() {
    let mangled = format!("__impl__{}__{}_{}", ...);
    call_user_function(&func, args, span)
}
```

Both engines require typechecker to run first (for `trait_dispatch` annotation).

**assert_parity helper** — MUST run Binder + TypeChecker before VM compile:
```rust
// Wrong (panics on method calls — "TypeTag not set"):
let mut compiler = Compiler::new();
compiler.compile(&program2)

// Correct:
let mut binder = Binder::new();
let (mut st, _) = binder.bind(&program2);
let mut tc = TypeChecker::new(&mut st);
let _ = tc.check(&program2);
let mut compiler = Compiler::new();
compiler.compile(&program2)
```
