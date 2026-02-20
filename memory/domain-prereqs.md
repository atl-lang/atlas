# Domain Prerequisites Registry

**Purpose:** Maps implementation domains to required pattern verification.
**Usage:** Before writing code in a domain, verify patterns using these queries.

---

## How to Use

1. **Identify domains** from phase file (AST, stdlib, type system, etc.)
2. **For each domain**, run the verification queries below
3. **Note 3-5 patterns** you'll use before writing any code
4. **If uncertain**, read more — never guess structure

---

## Domain: AST / Parser

**When:** Phase mentions AST nodes, parser, expressions, statements

**Verify:**
```bash
# Expression variants
Grep pattern="^\s+\w+.*//.*[Ee]xpr" path="crates/atlas-runtime/src/ast.rs" output_mode="content"

# Statement variants
Grep pattern="^\s+\w+.*//.*[Ss]tmt" path="crates/atlas-runtime/src/ast.rs" output_mode="content"

# Specific node (replace NodeName)
Grep pattern="NodeName" path="crates/atlas-runtime/src/ast.rs" output_mode="content" -C=3
```

**Key file:** `crates/atlas-runtime/src/ast.rs`
**Memory:** `patterns.md` → "Atlas Grammar Quick Reference"

---

## Domain: Value Types

**When:** Phase mentions Value enum, type representation, runtime values

**Verify:**
```bash
# All Value variants
Grep pattern="^pub enum Value" path="crates/atlas-runtime/src/value.rs" output_mode="content" -A=30

# Specific type (replace TypeName)
Grep pattern="TypeName" path="crates/atlas-runtime/src/value.rs" output_mode="content" -C=2
```

**Key file:** `crates/atlas-runtime/src/value.rs`
**Memory:** `patterns.md` → "Collection Types"

---

## Domain: Stdlib Functions

**When:** Phase mentions stdlib, builtin functions, standard library

**Verify:**
```bash
# Registry functions
Grep pattern="pub fn (is_builtin|call_builtin)" path="crates/atlas-runtime/src/stdlib/mod.rs" output_mode="content" -A=5

# Specific module exports
Grep pattern="pub fn" path="crates/atlas-runtime/src/stdlib/{module}.rs" output_mode="content"
```

**Key file:** `crates/atlas-runtime/src/stdlib/mod.rs`
**Memory:** `patterns.md` → "Intrinsic vs Stdlib Function"

---

## Domain: Interpreter

**When:** Phase mentions interpreter, eval, tree-walking execution

**Verify:**
```bash
# Main eval dispatch
Grep pattern="fn eval_expr|fn eval_stmt" path="crates/atlas-runtime/src/interpreter" output_mode="content" -A=3

# Intrinsic methods
Grep pattern="fn.*intrinsic" path="crates/atlas-runtime/src/interpreter" output_mode="content"
```

**Key file:** `crates/atlas-runtime/src/interpreter/mod.rs`
**Memory:** `patterns.md` → "Intrinsic vs Stdlib Function"

---

## Domain: VM / Bytecode

**When:** Phase mentions VM, bytecode, compiler, opcodes

**Verify:**
```bash
# Opcode enum
Grep pattern="^pub enum Op" path="crates/atlas-runtime/src/vm/opcodes.rs" output_mode="content" -A=20

# VM execution
Grep pattern="fn execute" path="crates/atlas-runtime/src/vm/mod.rs" output_mode="content" -A=3
```

**Key files:** `crates/atlas-runtime/src/vm/mod.rs`, `opcodes.rs`, `compiler.rs`
**Memory:** `patterns.md` → "Intrinsic vs Stdlib Function" (for VM intrinsics)

---

## Domain: Type System

**When:** Phase mentions type checker, type inference, type annotations

**Verify:**
```bash
# Type enum
Grep pattern="^pub enum Type" path="crates/atlas-runtime/src/type_checker" output_mode="content" -A=15

# Type check methods
Grep pattern="fn check|fn infer" path="crates/atlas-runtime/src/type_checker" output_mode="content"
```

**Key file:** `crates/atlas-runtime/src/type_checker/mod.rs`
**Spec:** `docs/specification/types.md`

---

## Domain: LSP

**When:** Phase mentions language server, LSP handlers, editor integration

**Verify:**
```bash
# Handler registration
Grep pattern="(on_request|on_notification)" path="crates/atlas-lsp/src/server.rs" output_mode="content"

# Existing handlers
Grep pattern="pub fn handle" path="crates/atlas-lsp/src/handlers" output_mode="content"
```

**Key files:** `crates/atlas-lsp/src/server.rs`, `handlers/mod.rs`

---

## Domain: Errors

**When:** Phase mentions error handling, RuntimeError, diagnostics

**Verify:**
```bash
# Error variants
Grep pattern="^pub enum RuntimeError" path="crates/atlas-runtime/src/errors.rs" output_mode="content" -A=20

# Error construction pattern
Grep pattern="RuntimeError::" path="crates/atlas-runtime/src" output_mode="content" head_limit=10
```

**Key file:** `crates/atlas-runtime/src/errors.rs`
**Memory:** `patterns.md` → "Error Pattern"

---

## Adding New Domains

When a new domain emerges:
1. Add section with **When** trigger
2. Add **Verify** queries (grep patterns, not line numbers)
3. Add **Key file(s)** reference
4. Add **Memory/Spec** reference if applicable

Keep queries surgical — find definitions, not read entire files.
