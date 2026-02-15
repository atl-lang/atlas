# Standard Library Usage Guide

**Status:** Phases 06a–06c complete  
**Last Updated:** 2026-02-15

## Purpose
Show practical, real-world patterns using the Atlas stdlib across strings, arrays, math, JSON, file I/O, and Option/Result utilities.

## Real-World Scenarios

### CSV Processing
- Read file: `let csv = readFile("sales.csv")`
- Split lines: `let rows = split(csv, "\n")`
- Parse: `map(rows, fn(line) { let f = split(line, ","); jsonObject() /* ... */ })`
- Filter: `filter(data, fn(r) { jsonGet(r, "amount") > 1000 })`
- Aggregate: `reduce(filtered, fn(sum, r) { sum + jsonGet(r, "amount") }, 0)`
- Write: `writeFile("high.csv", join(output, "\n"))`

### JSON API Handling
- Parse: `let resp = parseJson(readFile("api.json"))`
- Navigate: `let users = jsonGet(resp, "users")`
- Transform: `map(users, fn(u) { jsonGet(u, "email") })`
- Validate: `filter(users, fn(u) { contains(jsonGet(u, "email"), "@") })`

### Log Analysis
- Read log: `let lines = split(readFile("app.log"), "\n")`
- Filter errors: `filter(lines, fn(l) { contains(l, "ERROR") })`
- Grouping: build frequency map with `reduce` + `jsonObject`

### Text Processing
- Word freq: `reduce(split(text, " "), fn(acc, w) { /* count */ }, jsonObject())`
- Find links: `filter(split(text, " "), fn(w) { contains(w, "](") })`

## Patterns and Tips
- Prefer `map`/`filter`/`reduce` over manual loops for clarity.
- Use `jsonObject`, `jsonSet`, `jsonGet` for structured data.
- Combine `Option`/`Result` helpers: `isSome`, `unwrapOr`, `isOk`, `expect`.
- File paths: use `pathJoin` to keep portability.
- Guard types with `typeOf` / `isString` / `isNumber` before heavy ops.

## Performance Notes
- Phase 06c benchmarks cover strings, arrays, JSON, and file I/O; regressions are caught by `stdlib_benchmarks`.
- Large text: prefer single pass transforms; avoid repeated `concat` in tight loops (use `join`).

## Testing Patterns
- Use real-world fixtures under `crates/atlas-runtime/tests/` (06b) as references.
- Parity verification: `stdlib_parity_verification.rs` ensures interpreter/VM consistency.

