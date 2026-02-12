# Atlas Versioning Policy

## Scheme
- Semantic Versioning: `MAJOR.MINOR.PATCH`.

## Rules
- MAJOR: breaking changes to syntax, semantics, or diagnostics.
- MINOR: new features that are backward compatible.
- PATCH: bug fixes only.

## Diagnostics Stability
- Diagnostic format changes are treated as breaking changes.
- Error code changes require a MAJOR bump.
