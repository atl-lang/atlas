# Component Audit: Sanitizer

**Grammar Friction Points**
- `json` is isolated; extracting fields requires `jsonAsString` which throws if type mismatched.
- No safe JSON field access (`getStringOrNull`) leads to brittle checks.
- `if` requires parentheses; missing parens trip parser errors even in simple predicates.

**Missing Features (Atlas should have)**
- Safe JSON accessors that return `Option<T>` without runtime errors.
- UTF-8 validation/repair utilities in stdlib (currently no-op in Atlas code).

**Syntax Quality Rating:** 7/10

**AI Generation Experience**
- JSON handling is the main friction. The type boundary is clear but too strict for defensive code.
- Required workaround: string-based checks or risk runtime errors.
