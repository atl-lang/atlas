# Phase 16: Method Call Syntax - Frontend (Parser + Type Checking)

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Lexer Dot token must exist.

**Verification:**
```bash
grep "Dot" crates/atlas-runtime/src/token.rs
grep "'\\.'" crates/atlas-runtime/src/lexer/mod.rs
```

**What's needed:**
- TokenKind::Dot variant exists ✓ (DONE)
- Lexer recognizes `.` as separate token ✓ (DONE)
- MemberExpr AST node exists ✓ (DONE)
- Expr::Member variant exists ✓ (DONE)

**Status:** Lexer and AST partially complete. Parser, binder, and typechecker need implementation.

---

## Objective

Complete frontend implementation of method call syntax. Enable `value.method(args)` syntax with full type checking. Methods desugar to function calls following Rust's approach: `value.method(args)` → `Type::method(value, args)`.

## Design Decision

**Approach:** Rust-style method syntax (documented in decision-log.md)
- Methods are syntactic sugar for function calls
- No runtime dispatch, compile-time resolution only
- Built-in methods for stdlib types (json, string, array, etc.)
- Trait-based methods deferred to v0.3+

**Syntax supported:**
```atlas
// Method calls
json["user"].as_string()
value.to_string()

// Chaining
json["data"]["items"][0].as_number()

// Not in v0.2: Property access (no `obj.property` without parens)
```

## Files

**Update:** `crates/atlas-runtime/src/parser/expr.rs` (~150 lines added)
**Update:** `crates/atlas-runtime/src/binder.rs` (~50 lines added)
**Update:** `crates/atlas-runtime/src/typechecker/expr.rs` (~200 lines added)
**Create:** `crates/atlas-runtime/src/typechecker/methods.rs` (~300 lines)
**Update:** `crates/atlas-runtime/src/typechecker/mod.rs` (add methods module)
**Tests:** Parser tests for member syntax (~100 lines)
**Tests:** Type checker tests for method calls (~150 lines)

## Implementation

### 1. Parser Updates (expr.rs)

Add member expression parsing to call/index chain. Member expressions have same precedence as indexing and calls.

**Parsing logic:**
```
postfix_expr:
  primary ( call | index | member )*

member:
  '.' IDENTIFIER [ '(' args ')' ]
```

**Examples:**
- `obj.method()` → MemberExpr with args
- `obj.method(a, b)` → MemberExpr with args
- Chaining: `a.b().c()` → MemberExpr(MemberExpr(...))

**Integration:**
- Add to `parse_postfix()` or similar
- Handle chaining naturally through recursion
- Ensure precedence: `arr[0].method()` parses correctly

### 2. Binder Updates

Add Member case to expression binding. No new symbols to bind, just recurse into target and args.

```rust
Expr::Member(m) => {
    self.bind_expr(&m.target);
    if let Some(args) = &m.args {
        for arg in args {
            self.bind_expr(arg);
        }
    }
}
```

### 3. Type Checker - Method Resolution

Create method table infrastructure for built-in types.

**Method table structure:**
```rust
// Map: (Type, method_name) → (arg_types, return_type)
HashMap<(Type, String), MethodSignature>
```

**Built-in methods (Phase 16 scope):**
```atlas
// JsonValue methods
json.as_string() -> string
json.as_number() -> number
json.as_bool() -> bool
json.is_null() -> bool

// Future: String methods (Phase 17)
// Future: Array methods (Phase 17)
```

**Type checking algorithm:**
1. Type-check target expression
2. Look up (target_type, method_name) in method table
3. Verify argument types match signature
4. Return method return type
5. Error if method doesn't exist for type

**Error messages:**
- "Type 'string' has no method 'as_number'"
- "Method 'as_string' expects 0 arguments, got 1"
- "Cannot call method on type 'null'"

### 4. Method Table Population

Register JSON extraction methods:
```rust
fn populate_json_methods(table: &mut MethodTable) {
    table.register("json", "as_string", [], Type::String);
    table.register("json", "as_number", [], Type::Number);
    table.register("json", "as_bool", [], Type::Bool);
    table.register("json", "is_null", [], Type::Bool);
}
```

## Tests (TDD - Use rstest)

**Parser tests:**
1. Simple method call: `obj.method()`
2. Method with args: `obj.method(a, b)`
3. Chaining: `a.b().c()`
4. Mixed with indexing: `arr[0].method()`
5. Complex: `json["user"]["name"].as_string()`

**Type checker tests:**
1. Valid JSON methods type correctly
2. Invalid method name errors
3. Wrong argument count errors
4. Method on wrong type errors
5. Chained methods type correctly
6. Return types propagate correctly

**Minimum test count:** 30+ tests

## Integration Points

- Uses: Token (Dot)
- Uses: AST (MemberExpr)
- Updates: Parser (expr parsing)
- Updates: Binder (expression binding)
- Updates: Type checker (method resolution)
- Creates: Method table infrastructure
- Output: Parsed and type-checked member expressions

## Acceptance

- Parser successfully parses all member expressions
- AST correctly represents member calls
- Binder handles member expressions
- Type checker resolves methods correctly
- JSON extraction methods type-check
- Error messages clear and helpful
- All 30+ tests pass
- No clippy warnings
- cargo test passes
- Documentation updated

## Notes

**This phase does NOT implement:**
- Interpreter evaluation (Phase 17)
- VM bytecode compilation (Phase 17)
- Runtime execution (Phase 17)

**This phase ONLY implements:**
- Parsing member syntax
- Type checking method calls
- Method table infrastructure

Backend execution happens in Phase 17.

**Why split frontend/backend?**
- Frontend is prerequisite for all method usage
- Backend can be implemented/tested once frontend stable
- Allows type checking method calls even before runtime ready
- Clear separation of concerns
