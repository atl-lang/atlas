# Atlas Grammar Reality (Observed in Parser)

**Scope:** Findings based on actual parser implementation under `crates/atlas-runtime/src/parser/*` (v0.3 behavior). This document intentionally calls out conflicts with `docs/specification/syntax.md` and `docs/specification/grammar-conformance.md`.

## High-Confidence Facts (Verified in Parser)

- `let` / `let mut` are the only variable declaration forms. `var` is removed.
- `for-in` is the only loop form. C-style `for (init; cond; step)` is removed.
- `++` / `--` do not parse. Increment/decrement statements are gone.
- Anonymous functions exist, but **only** `fn(...) { ... }` syntax. Arrow functions are removed.
- Record literals require the `record` keyword: `record { key: value }`.
- Struct expressions still exist: `TypeName { field: value }` (only if the type name starts with uppercase).
- Import/export/extern/type/trait/impl/struct/enum are top-level items.
- `import` is **not** allowed as a statement inside blocks; it is top-level only.
- Assignment targets are **only** names or index expressions. Member assignment (`obj.field = ...`) is not a valid assignment target.
- Range tokens `..` exist but are only used for **slice syntax** in `arr[start..end]` (no standalone range expression).
- `record` keyword is tokenized and parsed; this is not mentioned in `docs/specification/syntax.md`.

## Mismatches vs `docs/specification/syntax.md`

- `syntax.md` describes v0.2 grammar; it still documents `var`, C-style `for`, `++/--`, and arrow functions — all removed in parser.
- `syntax.md` shows object literals `{ key: val }`; parser requires `record { ... }`.
- `syntax.md` says `import`/`export` are valid; parser supports them at top-level only, but `parse_statement` emits an error if `import` appears inside a block.

## Mismatches vs `docs/specification/grammar-conformance.md`

- `grammar-conformance.md` claims **no anonymous functions** and **no closure capture**; parser implements anonymous functions and the AST supports them.
- The conformance doc still lists C-style `for` as implemented. Parser only implements `for-in`.
- Conformance doc omits `record` literals and `struct`/`enum`/`type` declarations now present in parser.

## Practical Implications for Code Generation

- Use `let mut` exclusively. Any `var` will fail parsing.
- Use `for item in array { ... }` only. No C-style loops.
- `if` requires parentheses: `if (cond) { ... }`. `if cond { ... }` fails to parse.
- `match` arms require commas, and a `match` used as a statement needs a trailing `;`.
- Use `record { ... }` for anonymous records; `{ ... }` parses as a block, not an object literal.
- Avoid property assignment; update records by rebuilding the record or use maps/arrays with index assignment.
- Avoid range expressions like `0..n` in `for-in` loops. Ranges only appear in slice syntax.
- `hashMapPut` returns a new map (no in-place mutation). You must reassign.
- Empty array literal `[]` requires type context and still fails in `let x: string[] = []` (typechecker). Workaround: `slice([""], 0, 0)`.
- Array concatenation with `+` is rejected; use `arrayPush` or `concat`.

## What This Means for the Battle Test

- The Hydra port must target **actual parser behavior**, not the v0.2 syntax in `syntax.md`.
- Any friction points below are considered **real** if they stem from this mismatch.
