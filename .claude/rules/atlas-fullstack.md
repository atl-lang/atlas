---
paths:
  - "crates/atlas-runtime/src/stdlib/**"
  - "crates/atlas-runtime/src/parser/**"
  - "crates/atlas-runtime/src/typechecker/**"
  - "crates/atlas-runtime/src/method_dispatch.rs"
  - "crates/atlas-runtime/src/symbol.rs"
---

# Atlas Full-Stack Feature Completeness Protocol

**Auto-loaded when touching stdlib, parser, typechecker, or method_dispatch.**

This rule exists because the single most common AI agent failure in Atlas is
implementing a feature in SOME layers and skipping others — especially the
typechecker. Every incomplete feature becomes a landmine for the next agent.

---

## The 5 Mandatory Layers

Every Atlas feature must be complete across ALL applicable layers before a phase
is considered done. "Applicable" is defined per feature type below.

```
Layer 1: Parser          crates/atlas-runtime/src/parser/mod.rs
Layer 2: Typechecker     crates/atlas-runtime/src/typechecker/expr.rs
                         crates/atlas-runtime/src/typechecker/mod.rs
Layer 3: Interpreter     crates/atlas-runtime/src/interpreter/mod.rs
                         crates/atlas-runtime/src/interpreter/expr.rs
Layer 4: Compiler        crates/atlas-runtime/src/compiler/expr.rs
Layer 5: VM              crates/atlas-runtime/src/vm/mod.rs
```

Supporting layers (also mandatory when applicable):
```
Method dispatch:  crates/atlas-runtime/src/method_dispatch.rs
Symbol table:     crates/atlas-runtime/src/symbol.rs
Stdlib impl:      crates/atlas-runtime/src/stdlib/
Stdlib docs:      docs/stdlib/
Language docs:    docs/language/
```

---

## Completeness Checklists by Feature Type

### Adding a Stdlib Function (e.g. Io.readLine, File.read)

Before closing the issue, ALL of these must be true:

- [ ] **Stdlib impl** — function implemented in `stdlib/<module>.rs`
- [ ] **Stdlib mod** — registered in `stdlib/mod.rs` interpreter map
- [ ] **Symbol table** — registered in `stdlib/mod.rs` VM symbol map (the second map)
- [ ] **Method dispatch** — if namespace method: added to `method_dispatch.rs`
  - TypeTag variant added
  - `is_static_namespace()` updated
  - `namespace_type_tag()` updated
  - `resolve_<ns>_ns_method()` updated
  - `resolve_method()` match arm added
- [ ] **Interpreter sentinel** — if new namespace: added to interpreter globals in `interpreter/mod.rs`
- [ ] **Compiler is_ns** — if new namespace: added to `is_ns` match in `compiler/expr.rs`
- [ ] **TYPECHECKER** — return type registered in `typechecker/expr.rs` → `resolve_namespace_return_type()`
  - MUST be a concrete type — `Type::Unknown` is a BLOCKER, not acceptable
  - If function returns Result<T,E>: use `Type::Generic { name: "Result", type_args: [T, E] }`
  - If function returns Option<T>: use `Type::Generic { name: "Option", type_args: [T] }`
- [ ] **Stdlib docs** — entry added in `docs/stdlib/<module>.md`
- [ ] **Method conventions** — if method syntax, listed in `docs/stdlib/METHOD-CONVENTIONS.md`
- [ ] **Tests** — parity test in `tests/stdlib/`

**GATE: If the typechecker entry is missing, the stdlib function is INCOMPLETE. Do not close the issue.**

### Adding a New Syntax Feature (e.g. export struct, tuple syntax)

- [ ] **Grammar docs** — `docs/language/grammar.md` updated
- [ ] **Parser** — AST node parsed in `parser/mod.rs`
- [ ] **AST node** — new node added to `ast.rs` if needed
- [ ] **Typechecker** — AST node handled in `typechecker/mod.rs` and/or `typechecker/expr.rs`
  - Every new Expr/Stmt variant = new match arm in typechecker
  - Missing match arm = compile error in Rust (use this as your gate)
- [ ] **Interpreter** — AST node evaluated in `interpreter/expr.rs` or `interpreter/stmt.rs`
- [ ] **Compiler** — AST node compiled in `compiler/expr.rs` or `compiler/stmt.rs`
- [ ] **VM** — if new opcode required, handled in `vm/mod.rs`
- [ ] **Error codes** — AT codes for parse/type errors in `diagnostic/error_codes.rs`
- [ ] **Language docs** — documented in `docs/language/`
- [ ] **Tests** — parser test + typechecker test + parity test

**GATE: If any of parser/typechecker/interpreter/compiler is missing, the feature is INCOMPLETE.**

### Adding a New Namespace (e.g. Io, Db, Crypto)

All items from "Adding a Stdlib Function" PLUS:

- [ ] `is_static_namespace()` — new name added
- [ ] `namespace_type_tag()` — new TypeTag returned
- [ ] `TypeTag` enum — new variant added
- [ ] `resolve_method()` — new arm for the TypeTag
- [ ] `interpreter/mod.rs` — name added to namespace sentinels list
- [ ] `compiler/expr.rs` — TypeTag added to `is_ns` match
- [ ] **All methods in the namespace** — each one has a typechecker return type

---

## The Typechecker/Runtime Parity Rule (D-NEW)

**This is the most violated rule in the codebase.**

> Whenever the runtime (stdlib) can return an error, the typechecker MUST reflect that
> with `Result<T, E>`. If the runtime panics on bad input, the typechecker MUST reflect
> that with the concrete success type. They must agree.

Current violations (to fix in H-182):
- `File.read()` was `Type::String` — runtime returns `Result<string, string>` ✅ fixed this session
- `File.write()` is `Type::Null` — runtime returns `Result<void, string>` ❌ needs fix
- `Json.stringify()` is `Type::String` — runtime may throw on circular refs ❌ debatable

**How to verify:** For every stdlib function call, ask:
1. Can this fail at runtime? → typechecker return type must be `Result<T, E>`
2. Can this return nothing? → typechecker return type must be `Option<T>`
3. Does it always succeed? → typechecker return type must be the concrete success type

**Never use `Type::Unknown` as a stdlib return type.** Unknown means "I don't know" —
which is a bug in the typechecker, not a feature. If you don't know the return type,
look at the Rust implementation and infer it. File a P1 issue if it's complex.

---

## The Parser/Typechecker Sync Rule

> If `export struct` is in the grammar doc, it must be in the parser.
> If it's in the parser, it must be in the typechecker.
> If it's in the typechecker, it must be in the interpreter and compiler.

**Grammar docs are not aspirational.** If a grammar rule exists in `docs/language/grammar.md`
but is not implemented in the parser, that is a P1 bug — file it immediately.

When you add a grammar rule to docs: implement it in the parser in the same commit.
When you add a parser rule: add the typechecker handling in the same commit.
These are never separate phases.

---

## The Decisions Gate (MANDATORY before any design choice)

Before ANY design decision in this file's domain, run:

```bash
pt decisions all
```

Key decisions that are frequently violated:

| Decision | Rule | Most Commonly Violated By |
|----------|------|--------------------------|
| D-003 | No implicit coercion | Letting Type::Unknown propagate silently |
| D-010 | Type::Unknown is error state, not wildcard | Every agent that adds stdlib without typechecker entry |
| D-021 | Stdlib convention: TypeScript method model | Agents adding bare globals instead of namespace methods |
| D-032 | Same as D-021 (reinforced) | Same pattern |
| D-043 | Error quality contract | Agents adding AT codes without description/help/example |

If your change contradicts a decision — STOP. Surface to architect. Do not proceed.

---

## What "Done" Means for a Stdlib Feature

A stdlib function or namespace is DONE when an Atlas user can:

1. Write `Namespace.method(args)` in an Atlas file
2. `atlas check` passes with correct types (no Unknown, no spurious errors)
3. `atlas run` executes correctly in both interpreter and VM
4. A parity test verifies identical output in both engines
5. The function is documented in `docs/stdlib/`

If ANY of these 5 fail, the feature is not done. Do not close the issue.

---

## Red Flags — Stop and Audit If You See These

- You're adding a function to `stdlib/mod.rs` but not touching `typechecker/expr.rs` → STOP
- You see `Type::Unknown` as a return type in `resolve_namespace_return_type()` → STOP, fix it
- A grammar rule exists in docs but `atlas check` gives a parse error → STOP, file H-XXX
- You're adding a new namespace but only updating method_dispatch, not the interpreter sentinel → STOP
- The interpreter handles a new feature but the compiler/VM doesn't → STOP (parity break)
- You implement something in 4 places. Count to 9. What are you missing? → AUDIT before commit
