# Component Audit: Config

**Grammar Friction Points**
- Structural type annotations are verbose and undocumented in `syntax.md`.
- No record field assignment, so merges require full-record reconstruction.
- `if` requires parentheses; missing parens produce misleading parser errors.
- `hashMapPut` is not in-place; every set must reassign, which is easy to miss.
- Empty array literal `[]` needs explicit context; `let x: string[] = []` still fails.

**Missing Features (Atlas should have)**
- JSON decode into typed records (serde-style mapping).
- Map merge helpers and deep-merge utilities.
- Regex-based string replace for env substitution.
- Safe env access in restricted runtimes (e.g., `getEnvOrDefault`) or sandbox policy introspection.

**Syntax Quality Rating:** 6/10

**AI Generation Experience**
- Boilerplate is high: merging nested records is painful without field assignment.
- The core syntax is clear, but the stdlib lacks config-specific primitives.
