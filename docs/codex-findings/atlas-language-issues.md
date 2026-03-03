# Atlas Language Issues Audit (AI-First Syntax/Grammar)

Date: 2026-03-03
Scope: Docs + lexer/parser/AST review. No code changes.

## Atlas Goal (as documented)
Atlas is “AI-First” and aims to be AI-generation-friendly while reaching systems-level capability, with explicit ownership and low ambiguity. See:
- /Users/proxikal/dev/projects/atlas/README.md
- /Users/proxikal/dev/projects/atlas/ROADMAP.md

## What’s Wrong (Syntax/Grammar Drift and Complexity)

1. Docs and implementation contradict each other, which breaks AI reliability.
- Nested functions: syntax spec says nested functions are allowed and hoisted, but grammar-conformance says nested functions are rejected. The parser currently allows `fn` inside blocks.
  Files:
  - /Users/proxikal/dev/projects/atlas/docs/specification/syntax.md
  - /Users/proxikal/dev/projects/atlas/docs/specification/grammar-conformance.md
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/stmt.rs
- Implicit returns: roadmap says implicit returns are a future quick win, but the parser already supports Rust-style tail expressions in blocks.
  Files:
  - /Users/proxikal/dev/projects/atlas/ROADMAP.md
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/stmt.rs
- Modules: module spec claims imports/exports are supported; top-level parser does parse them; statement parser still errors “not supported in v0.1.”
  Files:
  - /Users/proxikal/dev/projects/atlas/docs/specification/modules.md
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/mod.rs
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/stmt.rs

2. Keyword set in docs is incomplete vs lexer.
- Syntax doc lists a small keyword set, but lexer recognizes many more (`type`, `extern`, `trait`, `impl`, `extends`, `is`, `own`, `borrow`, `shared`, `in`, `export`, `import`).
  Files:
  - /Users/proxikal/dev/projects/atlas/docs/specification/syntax.md
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/token.rs

3. Too many parallel ways to express the same concept.
- Mutability: `let`, `let mut`, deprecated `var`.
  Files:
  - /Users/proxikal/dev/projects/atlas/docs/specification/syntax.md
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/stmt.rs
- Functions: named `fn`, `fn` expressions, and arrow `(x) => expr`.
  Files:
  - /Users/proxikal/dev/projects/atlas/docs/specification/types.md
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/expr.rs
- Loops: C-style `for (init; cond; step)` and `for x in y`, plus `while`.
  Files:
  - /Users/proxikal/dev/projects/atlas/docs/specification/syntax.md
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/stmt.rs
- Types: `T[]` and `Array<T>`, plus union/intersection and structural types (not consistently documented).
  Files:
  - /Users/proxikal/dev/projects/atlas/docs/specification/types.md
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/expr.rs

4. Ambiguous brace-based syntax increases parser backtracking and AI ambiguity.
- `{ ... }` is used for block expressions, object literals, and structural types. Parser uses backtracking to decide object vs block.
  Files:
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/expr.rs
  - /Users/proxikal/dev/projects/atlas/docs/specification/syntax.md

5. Statement termination has multiple regimes.
- Files require semicolons, blocks allow implicit tail expressions, REPL treats newlines as separators.
  Files:
  - /Users/proxikal/dev/projects/atlas/docs/specification/syntax.md
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/stmt.rs

6. Structs are partially present but not wired yet.
- AST has `StructDecl` and `StructExpr`, but lexer lacks `struct` keyword and parser doesn’t recognize struct syntax. This is currently inconsistent until the in-progress struct work lands.
  Files:
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/ast.rs
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/token.rs
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/parser/mod.rs

## What Language to Mimic Instead (and Why)
Atlas is currently a hybrid of Rust, TypeScript, JavaScript, and C. For AI optimization, it needs a single syntax spine so there’s one canonical way to express things.

Best-fit spine given the goals:
- Rust-style core grammar (expression-oriented, explicit control flow, match, block expressions)
- TypeScript-style type annotations (`name: Type`, `Type<T>`), which are already present and AI-friendly

In short: adopt Rust’s core grammar, keep TS-style type annotations, and remove JS/C surface syntax variants.

## High-Value Simplifications (Grammar Cuts)

- Remove `var`; keep only `let` / `let mut`.
- Remove `++` / `--`.
- Choose one loop form: prefer `for x in y` + `while`, drop C-style `for (init; cond; step)`.
- Choose one anonymous-function syntax: keep `fn(...) {}` and drop arrow, or keep arrow as sugar with a canonical rewrite.
- Resolve `{}` ambiguity: object literals vs block expressions vs structural types must be unambiguous without backtracking (consider a literal marker or stricter rules).
- Align docs to actual implementation and remove stale constraints immediately; AI agents depend on docs.

## Notes
- This is an audit-only report. No changes made.
- Struct work in progress: keep changes isolated until the grammar spine is confirmed.
