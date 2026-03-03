# Language Core Patterns

> **v0.3 Grammar Rewrite:** Major syntax changes tracked in `atlas-track decisions`.
> Run `atlas-track decisions` for D-006 to D-010. Extended rationale in
> `docs/language-design/rationale/`. Migration guides in `docs/language-design/migration/`.
>
> Changes: Remove `var`, `++/--`, C-style for, arrow functions. Add `record` keyword.

---

## P-001: Strict Type System with No Implicit Coercion
**Status:** Active

No implicit type coercion. `"5" + 3` → type error. Use `toNumber()`, `toString()`.

## P-002: Scientific Notation for Number Literals
**Status:** Active

Support `1.5e10`, `3e-5`. Token-efficient, industry standard.

## P-003: Method Call Syntax - Rust-Style Desugaring
**Status:** Active

`value.method(args)` desugars to `Type::method(value, args)`. Both syntaxes valid.

## P-004: Prelude with Shadowing Protection
**Status:** Active

Built-ins always available. Shadowing prelude names → compile error (AT1012).
