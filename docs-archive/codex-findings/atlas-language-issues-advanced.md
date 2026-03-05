# Atlas Language Issues Audit (Advanced, Codebase-Only)

Date: 2026-03-03
Scope: Codebase only. Lexer/parser/AST/typechecker/compiler/runtime/stdlib. No docs used.
Goal: For every syntax/grammar surface area, identify what the codebase is doing that it shouldn’t, and what Atlas should do instead to be AI-optimized.

---

## Method
Reviewed:
- Lexer/token definitions
- Parser (expressions, statements, items)
- AST structures
- Typechecker behavior
- Compiler/runtime behavior for syntax-backed constructs

Files used (code only):
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/token.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/mod.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/expr.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/stmt.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/ast.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/typechecker/expr.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/types.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/compiler/expr.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/stdlib/json.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/symbol.rs

---

## A. Core Syntax and Grammar Issues (Codebase Behavior vs AI-First Goal)

### A1. Object literal vs block expression ambiguity (brace overloading)
- **What the code does:** `{ ... }` in expression context is parsed by speculative backtracking: try object literal first, else treat as block expression.
- **Why it’s wrong:** Ambiguous grammar increases AI error rate and makes syntax generation brittle.
- **What Atlas should do instead:** Make object literals unambiguous. Options:
  - Introduce a distinct literal marker (e.g., `object { ... }` or `map { ... }`).
  - Or require explicit `key = value` for blocks and `key: value` for objects AND forbid empty `{}` object literals.
- **Code:** parser backtracking in `parse_prefix` and `try_parse_object_literal`.

### A2. Object literals produce HashMap but typecheck to `Unknown`
- **What the code does:** Object literals compile to HashMap at runtime. Typechecker returns `Type::Unknown` for object literals and does not enforce a concrete type.
- **Why it’s wrong:** `Unknown` is assignable to anything, so object literals “fit” any expected type. AI learns invalid type relationships.
- **What Atlas should do instead:** Introduce a concrete object/record type (or make object literals illegal until object type exists). If kept, object literals must typecheck to a specific type and be non-assignable to `json` without explicit conversion.
- **Code:** `compiler/expr.rs::compile_object_literal`, `typechecker/expr.rs::Expr::ObjectLiteral`.

### A3. `Unknown` is too permissive and erodes grammar guarantees
- **What the code does:** `Type::Unknown` is assignable to any type.
- **Why it’s wrong:** It bypasses type safety, hides grammar/semantic errors, and teaches AI that weakly-typed code is acceptable.
- **What Atlas should do instead:** Restrict `Unknown` to error recovery only; emit a diagnostic when `Unknown` flows into concrete types (especially `json`, structural, function types).
- **Code:** `types.rs::is_assignable_to`.

### A4. Structural types and object literals share brace syntax with no guardrails
- **What the code does:** Structural types use `{ field: type, ... }` in type position; object literals use `{ key: value, ... }` in expression position; blocks also use `{ ... }`.
- **Why it’s wrong:** Same token sequence in different contexts leads to AI confusion and grammar collisions in generation.
- **What Atlas should do instead:** Introduce a distinct keyword or delimiter for structural types or object literals (e.g., `record { ... }` for structural types or `object { ... }` for values).
- **Code:** `parser/expr.rs::parse_structural_type`, `parser/expr.rs::try_parse_object_literal`, `parser/stmt.rs::parse_block`.

### A5. Arrow functions + `fn` expressions create multiple syntactic paths
- **What the code does:** Supports both `fn(...) { ... }` and `(x) => expr`.
- **Why it’s wrong:** Multiple function syntaxes increase AI variance and lead to inconsistent code generation.
- **What Atlas should do instead:** Choose one canonical anonymous function syntax and make the other a strict desugaring (or remove it).
- **Code:** `parser/expr.rs::parse_anon_fn`, `parser/expr.rs::try_parse_arrow_fn`.

### A6. `var` is deprecated but still part of grammar
- **What the code does:** Accepts `var` and `let mut` and `let`.
- **Why it’s wrong:** Multiple mutable declarations increase syntax surface area and AI confusion.
- **What Atlas should do instead:** Remove `var` from the grammar and lexer, or hard-error it (not just warn).
- **Code:** `token.rs::TokenKind::Var`, `parser/stmt.rs::parse_var_decl`.

### A7. C-style `for` loop + `for-in` loop = redundant grammar
- **What the code does:** If `for` is followed by `(`, it parses C-style `for (init; cond; step)`; otherwise `for-in`.
- **Why it’s wrong:** Two loop syntaxes create ambiguity and more failure modes for AI.
- **What Atlas should do instead:** Keep one loop form (prefer `for-in` + `while`) and remove C-style `for` from grammar.
- **Code:** `parser/stmt.rs::parse_statement` dispatch.

### A8. Increment/decrement are statement-only but supported
- **What the code does:** Supports `++x`, `x++`, `--x`, `x--` as statements only.
- **Why it’s wrong:** Legacy C syntax adds complexity and special parsing/typing rules with little benefit.
- **What Atlas should do instead:** Remove increment/decrement entirely; use `x = x + 1`.
- **Code:** `token.rs` operators, `parser/stmt.rs::parse_assign_or_expr_stmt`.

### A9. Implicit block tail expressions exist but not tied to explicit function returns
- **What the code does:** Blocks can return the last expression without semicolon. Typechecker for `Expr::Block` returns `Type::Unknown` rather than the tail expression type.
- **Why it’s wrong:** Semantics are partial: grammar allows implicit returns, but types don’t reflect it. AI will generate blocks expecting value semantics and get weak typing.
- **What Atlas should do instead:** If block expressions are supported, typechecker must return the tail expression’s type. Otherwise disable tail expressions.
- **Code:** `parser/stmt.rs::parse_block`, `typechecker/expr.rs::Expr::Block`.

---

## B. JSON/Grammar Friction (Codebase Only)

### B1. JSON type and object literals are conflated by permissive typing
- **What the code does:** `json` is a distinct runtime type (`JsonValue`), but object literals compile to HashMap and typecheck to `Unknown`, which is assignable to `json`.
- **Why it’s wrong:** AI assumes `{ ... }` is `json` because the type system doesn’t reject it.
- **What Atlas should do instead:**
  - Disallow object literal assignment to `json` without explicit conversion.
  - Or add a dedicated JSON literal form.
- **Code:** `typechecker/expr.rs` (object literal → Unknown), `types.rs::is_assignable_to` (Unknown is permissive), `compiler/expr.rs` (object literal → HashMap).

### B2. `parseJSON` returns `Result<json, string>` but JSON usage is friction-heavy
- **What the code does:** `parseJSON` is typed as `Result<json, string>` and implemented as `Value::Result`.
- **Why it’s wrong:** Without strict conventions or syntax support (`?`), AI frequently treats it as `json` directly, leading to errors.
- **What Atlas should do instead:**
  - Either keep Result and ensure language ergonomics (`?` or `match`) is the canonical usage, or
  - Change `parseJSON` to return `json` and throw runtime error on parse failure.
- **Code:** `symbol.rs` builtin signature, `stdlib/json.rs::parse_json`.

### B3. `toJSON` accepts any `Value` and serializes non-JSON types
- **What the code does:** `toJSON` takes `Type::Unknown`, serializes many runtime types into JSON strings.
- **Why it’s wrong:** It blurs the boundary between runtime values and `json` and encourages AI to treat any object as JSON.
- **What Atlas should do instead:** Split into `json_stringify(json)` for `json` only and `serialize(value)` for others, or make JSON serialization a trait.
- **Code:** `symbol.rs` builtin signature, `stdlib/json.rs::value_to_json`.

### B4. JSON indexing is allowed, but indexing behavior depends on runtime type
- **What the code does:** JSON indexing accepts string or number; arrays and strings accept number; HashMap indexing is not formally typed.
- **Why it’s wrong:** Same syntax `x["key"]` can mean JSON indexing or HashMap indexing or string indexing. AI can’t reliably predict which.
- **What Atlas should do instead:** Make indexing only valid for arrays/strings/json. Force HashMap/record access through methods (`get`, `set`) to avoid ambiguity.
- **Code:** `typechecker/expr.rs::check_index`, `compiler/expr.rs::compile_index`.

---

## C. Parser/AST Constructs That Exist Without Full Grammar Support

### C1. Struct expressions are parsed based on capitalization convention
- **What the code does:** `TypeName { ... }` is parsed as a struct expression only if the identifier starts with an uppercase letter.
- **Why it’s wrong:** Grammar depends on naming convention, which is brittle for AI and not enforced elsewhere.
- **What Atlas should do instead:** Use explicit `struct` syntax (or keyword) to disambiguate, not name casing.
- **Code:** `parser/expr.rs::parse_identifier` and `parse_struct_expr`.

### C2. Structs exist in AST but lexer/parser for `struct` keyword is incomplete
- **What the code does:** AST has `StructDecl`/`StructExpr`. Parser includes struct expressions but no `struct` keyword token in lexer. This is inconsistent.
- **Why it’s wrong:** Partial grammar surfaces create dead syntax paths that confuse AI.
- **What Atlas should do instead:** Either fully implement struct syntax or remove/guard it behind a feature gate.
- **Code:** `ast.rs` struct nodes, `token.rs` keyword list, `parser/mod.rs` lacks `struct` parsing.

---

## D. Recommendations: Canonical Grammar Strategy (Code-Driven)

For AI-optimization, the grammar must be single-path and unambiguous. Based on the codebase issues:

1) **Disambiguate `{}` syntax**
- Pick one meaning per context and enforce it (no backtracking). Introduce a distinct syntax for object literals or structural types.

2) **Remove redundant syntax**
- Remove `var`, increment/decrement, arrow functions, and C-style `for` loops.

3) **Make typing reflect syntax**
- Block expressions must return the tail expression type.
- Object literals must have a concrete type (or be banned until they do).
- `Unknown` should not silently satisfy concrete types.

4) **Clarify JSON vs object semantics**
- Either add explicit JSON literal syntax or enforce explicit conversion to/from `json`.
- Make `parseJSON` usage unambiguous (Result + `?`, or direct json + runtime error).

---

## Notes
- This is codebase-only; no documentation referenced.
- If you want a follow-up that proposes concrete patches, specify which grammar direction you want (JSON literals vs record literals, Result-returning parse vs runtime-error parse).
