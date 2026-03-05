# Known Issues

Current limitations in Atlas. This is the honest truth.

**Source of truth:** `atlas-track issues` — this file is a quick reference.
**Full details:** `atlas-track issue H-XXX`
**Search:** `atlas-track search "keyword"`

## P0 - Critical Blockers

### H-069: Closure Global Mutations
**Status:** In Progress
**Problem:** Closures passed as function parameters don't persist mutations to global mutable arrays/state.
**Workaround:** Avoid callback-based patterns. Use imperative style instead of `describe(fn() { ... })`.

## Recently Fixed

### H-063: Module Resolution (FIXED 2026-03-05)
**Was:** `import` statements compiled but didn't resolve at runtime.
**Now:** Module resolution works for multi-file projects.

### H-066: Struct Field Access (FIXED 2026-03-05)
**Was:** Struct field access returned `?` instead of declared type.
**Now:** `item.id` correctly returns `number` if declared as such.

### H-064: HashMap Generic Enforcement (FIXED 2026-03-04)
**Was:** `HashMap<K,V>` generics were cosmetic.
**Now:** Type annotations are enforced on all HashMap operations.

### H-062: Array<T> vs T[] (FIXED 2026-03-04)
**Was:** `Array<T>` and `T[]` were separate types.
**Now:** Unified - both are interchangeable.

## P1 - Painful But Workable

### H-067: File Extension
`.atl` files don't execute properly. Use `.atlas` extension.

### H-068: No main() Entry Point
Code wrapped in `fn main() {}` doesn't execute. Use top-level statements.

## Reporting Issues

Found a bug? Document it in battle test `audit/FRICTION.md` or report to the Atlas repo.
