# Phase 06: Inference Error Messages

**Block:** 5 (Type Inference)
**Depends on:** Phase 03 + Phase 05 complete

## Goal

When inference fails or produces a mismatch, the error message must show what was inferred
vs. what was expected — not just "type mismatch".

## New error codes

| Code | Message | When |
|------|---------|------|
| AT3050 | `cannot infer return type for '{}' — add explicit '-> T'` | infer_return_type returns Unknown |
| AT3051 | `cannot infer type argument '{}' — not used in parameter position` | generic type param only in return |
| AT3052 | `inferred type '{}' is incompatible with usage as '{}'` | inferred type conflicts with a use site |

## AT3001 improvement

Current: `"Type mismatch: expected X, found Y"`
Improved: when Y was inferred (not annotated), add context: `"note: Y was inferred from the initializer at line N"`

This uses the existing `suggestions.rs` mechanism — add a new suggestion entry for the
"inferred from initializer" context.

## Examples

```atlas
// AT3050
fn f(x: number) {
    return x * 2;   // infers number
    return "oops";  // infers string — conflict → AT3050 with "inferred number from line 2"
}

// AT3052
let x = 42;       // inferred: number
x + "string";     // AT3052: inferred type 'number' incompatible with 'string'
```

## Acceptance Criteria

- [ ] AT3050 emitted with function name and suggestion to add annotation
- [ ] AT3051 emitted with type param name
- [ ] AT3052 emitted with "inferred from X" context when applicable
- [ ] AT3001 message mentions "inferred" when the inferred type caused the mismatch
- [ ] All new error codes registered in `diagnostic.rs`
- [ ] Minimum 6 new error-path tests
