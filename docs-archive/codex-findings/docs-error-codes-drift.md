# Documentation Drift: Trait Error Codes

Target: docs/specification vs runtime diagnostics
Severity: High
Status: Open

## Finding: Trait error codes in spec do not match runtime codes

Evidence:
- /Users/proxikal/dev/projects/atlas/docs/specification/types.md:312-325
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/diagnostic/error_codes.rs:62-115

What/Why:
- The spec documents trait errors as AT3001–AT3009, but the runtime uses AT3030–AT3037 for trait system errors. AT3001–AT3006 are currently used for general type errors (type mismatch, arity, not callable, etc.).

Impact:
- AI agents and users following the spec will misinterpret diagnostics and may implement incorrect tooling or tests.

Recommendation:
- Update `docs/specification/types.md` to reflect the actual runtime error codes (AT3030+). Add a cross-check step in release gates to prevent spec/runtime divergence.
