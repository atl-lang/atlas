# Domain Verification — Pre-verified Quick-Refs + Fallback Queries

**Purpose:** Get correct API facts BEFORE writing code. Never guess at field names, variant names, or method signatures.

**Rule:** Check quick-ref first. Only run grep queries for things not covered.

---

## Quick-Ref Files (auto-loaded by paths: frontmatter)

| Domain | File | Auto-loads when touching |
|--------|------|--------------------------|
| Atlas language syntax | `.claude/rules/atlas-syntax.md` | tests/, corpus/ |
| AST nodes, structs, fields | `.claude/rules/atlas-ast.md` | ast.rs, parser/, typechecker/, binder.rs |
| TypeChecker state + Type enum | `.claude/rules/atlas-typechecker.md` | typechecker/, types.rs |
| Interpreter API + Value types | `.claude/rules/atlas-interpreter.md` | interpreter/ |
| VM, Compiler, Opcodes | `.claude/rules/atlas-vm.md` | vm/, compiler/, bytecode/ |
| Testing rules + file map | `.claude/rules/atlas-testing.md` | tests/, corpus/ |
| Parity (interpreter↔VM) | `.claude/rules/atlas-parity.md` | interpreter/, vm/, compiler/ |

**If the quick-ref is in context** (auto-loaded) — read it. It has the answers.

**If not in context** — run the targeted grep below.

---

## Fallback Grep Queries (when quick-ref not loaded)

### AST Domain
```bash
# All Stmt variants
Grep pattern="^pub enum Stmt" path="crates/atlas-runtime/src/ast.rs" -A=20

# All Expr variants
Grep pattern="^pub enum Expr" path="crates/atlas-runtime/src/ast.rs" -A=30

# Specific struct fields (e.g., "what's in Block?")
Grep pattern="pub struct Block" path="crates/atlas-runtime/src/ast.rs" -A=5
```

### Type System Domain
```bash
# All Type variants
Grep pattern="^pub enum Type" path="crates/atlas-runtime/src/types.rs" -A=50

# Type methods
Grep pattern="pub fn " path="crates/atlas-runtime/src/types.rs" output_mode="content" head_limit=20
```

### TypeChecker Domain
```bash
# TypeChecker fields
Grep pattern="pub struct TypeChecker" path="crates/atlas-runtime/src/typechecker/mod.rs" -A=30

# Key methods
Grep pattern="fn check_\|fn resolve_\|fn enter_scope\|fn exit_scope" path="crates/atlas-runtime/src/typechecker/mod.rs" output_mode="content"
```

### Parser Type Syntax
```bash
# How parse_type_primary works (what type syntax is supported)
Read file="crates/atlas-runtime/src/parser/expr.rs" offset=510 limit=40

# Parser type tests (examples of correct syntax)
Grep pattern="fn test_parse.*type" path="crates/atlas-runtime/src/parser/mod.rs" -A=20
```

### Opcode Domain
```bash
# All opcodes
Read file="crates/atlas-runtime/src/bytecode/opcode.rs"
```

### Error Codes Domain
```bash
# All error codes
Read file="crates/atlas-runtime/src/diagnostic/error_codes.rs"
```

### Stdlib Domain
```bash
# Is a function a builtin?
Grep pattern="fn is_builtin" path="crates/atlas-runtime/src/stdlib/mod.rs" -A=30

# Builtin call dispatch
Grep pattern="fn call_builtin" path="crates/atlas-runtime/src/stdlib/mod.rs" -A=5
```

---

## Verification Protocol

1. **Identify domains** — List all files your phase will touch
2. **Check quick-ref** — If the quick-ref is loaded, it has pre-verified facts
3. **Grep for gaps** — Only run queries for things the quick-ref doesn't cover
4. **Write code using ONLY verified facts** — Never assume a field/variant name

**Cost:** ~100 tokens to check quick-ref, ~300 tokens per grep
**Prevention:** Every incorrect assumption = 5,000–20,000 tokens to diagnose and fix

---

## Anti-Patterns (BANNED — verified wrong from past failures)

```rust
// WRONG — These don't exist
Stmt::Let(...)           // → Stmt::VarDecl(VarDecl)
Stmt::Var(...)           // → Stmt::VarDecl(VarDecl)
Block.tail_expr          // → Block has NO tail_expr — just .statements
types_compatible(a, b)   // → a.is_assignable_to(&b)
Type::TypeParameter(name) // → Type::TypeParameter { name }  struct variant
fn(number) -> number     // as TYPE ANNOTATION → (number) -> number

// RIGHT
Stmt::VarDecl(VarDecl)
block.statements.last()
a.is_assignable_to(&b)
Type::TypeParameter { name: "T".into() }
(number, string) -> bool  // function type annotation
```

---

## Block 4 Notes (2026-02-23)
- `Param.type_ref` is `Option<TypeRef>` — `None` for untyped arrow-fn params
- `Expr::AnonFn` is the unified node for both `fn(x) {}` and `(x) => x` syntax
- Function type annotation: `(T) -> R` NOT `fn(T) -> R` (parser rejects `fn` as type name)
