# Language Core Decisions

## DR-001: Strict Type System with No Implicit Coercion
**Status:** Active

No implicit type coercion. `"5" + 3` → type error. Use `toNumber()`, `toString()`.

## DR-002: Scientific Notation for Number Literals
**Status:** Active

Support `1.5e10`, `3e-5`. Token-efficient, industry standard.

## DR-003: Method Call Syntax - Rust-Style Desugaring
**Status:** Active

`value.method(args)` desugars to `Type::method(value, args)`. Both syntaxes valid.

## DR-004: Prelude with Shadowing Protection
**Status:** Active

Built-ins always available. Shadowing prelude names → compile error (AT1012).
