# Type System Decisions

## DR-001: Monomorphization for Generic Types
**Status:** Active

Rust-style monomorphization. `Option<Number>` and `Option<String>` are separate types.

## DR-002: TypeChecker-Owned Usage Tracking
**Status:** Active

No `used` field on Symbol struct. TypeChecker owns usage tracking separately.
