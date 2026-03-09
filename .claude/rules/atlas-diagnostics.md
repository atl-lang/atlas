---
paths:
  - "crates/atlas-runtime/src/diagnostic.rs"
  - "crates/atlas-runtime/src/diagnostic/**"
  - "crates/atlas-runtime/src/parser/mod.rs"
  - "crates/atlas-runtime/src/parser/expr.rs"
  - "crates/atlas-runtime/src/parser/stmt.rs"
  - "crates/atlas-runtime/src/typechecker/**"
---

# Atlas Error Quality Contract

Auto-loaded when touching diagnostic, parser error, or typechecker files.

**B14 implements these standards system-wide.**
**Any NEW error code or error site added NOW must meet these standards immediately — no exceptions.**
**Existing pre-B14 error sites will be upgraded by B14. Do not regress them further.**

---

## The 6 Non-Negotiable Standards (D-043)

### 1. Cascade Suppression — MANDATORY
Parser MUST have `in_panic_mode: bool`. After the first error, suppress further errors until
`synchronize()` clears the flag. A program with 1 bug shows 1 primary error.

**Verify:** `atlas run bad_program.atlas` — count primary errors. Must equal number of root bugs.

```rust
// CORRECT — set panic mode on error, clear on sync
fn error_at_with_code(&mut self, ...) {
    if self.in_panic_mode { return; }  // suppress cascade
    self.in_panic_mode = true;
    self.diagnostics.push(...);
}
fn synchronize(&mut self) {
    self.in_panic_mode = false;  // clear on recovery
    // ... walk to next statement boundary
}
```

### 2. "Expected X, found `Y` (TokenKind)" — MANDATORY on ALL parser errors
Every parser error that says "Expected X" MUST also say what was found.
Use `self.peek()` to get the actual token at the error site.

```
// BANNED:
"Expected ')' after parameters"

// REQUIRED:
"Expected ')' after parameters, found `[` (LeftBracket)"
```

Use the `expected()` helper (added in B14-P02). If it doesn't exist yet, write the found clause manually.

### 3. Context-Aware Help Text — NO STATIC LOOKUPS
Help text MUST be passed explicitly at the error site — NOT auto-fetched from the registry.
`error_at_with_code()` must NOT call `help_for(code)` unconditionally.

```rust
// BANNED — attaches wrong help in wrong context:
let help = error_codes::help_for(code).unwrap_or("check syntax");
self.diagnostics.push(Diagnostic::error_with_code(code, msg, span).with_help(help));

// REQUIRED — explicit, context-specific help:
self.diagnostics.push(
    Diagnostic::error_with_code(code, msg, span)
        .with_help("Add the missing closing brace for the function body")
);
```

AT1002 (unterminated string) help ONLY appears on string literal errors.
AT1003 (invalid escape) help ONLY appears on escape sequence errors.

### 4. `is_secondary` field on Diagnostic — MANDATORY
`Diagnostic` struct must have `is_secondary: bool` (default false).
Errors emitted while `in_panic_mode` that are not suppressed must set `is_secondary = true`.
Display must label or visually distinguish secondary errors.

### 5. New AT Error Codes — Must Have Description + Help + Example
Every new AT code added to `error_codes.rs` MUST have:
- `description` — one sentence, what went wrong
- `help` — one sentence, what to do about it
- `example` — minimal Atlas reproduction (for `atlas explain ATxxxx`, B14-P05)

```rust
// BANNED — incomplete:
ErrorCodeInfo { code: "AT1099", description: "Some error", help: None }

// REQUIRED:
ErrorCodeInfo {
    code: "AT1099",
    description: "impl block defined for unknown type",
    help: Some("Check that the type name is spelled correctly and declared with `struct`."),
    example: Some("impl UnknownType { }  // AT1099: type not found"),
}
```

### 6. Span Precision — Point at the Bad Token, Not After It
Error spans must start at the FIRST bad token, not the character after it.
For multi-token errors, the span covers the full range (start of first bad token to end of last).

```rust
// BANNED — points at wrong location:
let span = self.peek().span;  // after consuming the bad token
self.error_at_with_code(code, msg, span);

// REQUIRED — capture span BEFORE consuming:
let bad_span = self.peek().span;
self.advance();  // consume the bad token
self.error_at_with_code(code, msg, bad_span);
```

---

## Pre-B14 / Post-B14 Scope

| Situation | What to do |
|-----------|-----------|
| Adding a NEW error code (e.g. B13-P06) | Follow all 6 standards above — NOW |
| Modifying an EXISTING error site pre-B14 | Don't regress it further. Note the site in B14 scope. |
| Working on B14 phases | Enforce all 6 standards across the whole system |
| Post-B14 ANY error site | All 6 standards are MANDATORY. Violation = blocking. |

---

## Snapshot Tests — Quality Lock

After B14 ships, error output is snapshot-tested. These programs have locked output:

```bash
# Run snapshot tests for error quality
cargo nextest run -p atlas-runtime -E 'test(error_quality)'

# If output changed — review the diff. Regression = BLOCKING.
# Intentional improvement — run: cargo insta review
```

Any change to error formatting, help text, or cascade behavior that breaks a snapshot
is a BLOCKING regression. Fix the snapshot only if the new output is strictly better
(more specific, more accurate, fewer secondary errors).

---

## Quick Checklist (Before Committing Any Diagnostic Change)

- [ ] New AT code has description + help + example
- [ ] Parser error says "Expected X, found `Y` (Kind)"
- [ ] Help text is context-specific, not registry auto-fetched
- [ ] Cascade suppression not bypassed
- [ ] Span points at the bad token, not after it
- [ ] `atlas run bad.atlas` shows correct number of primary errors
- [ ] Snapshot tests pass (post-B14)

---

## References

- **D-043** — Error Quality Contract (this rule, as a standing decision)
- **B14** — Full system-wide implementation of these standards
- `crates/atlas-runtime/src/diagnostic.rs` — Diagnostic struct
- `crates/atlas-runtime/src/diagnostic/error_codes.rs` — AT code registry
- `crates/atlas-runtime/src/parser/mod.rs:1062` — error() / error_with_code()
- `crates/atlas-runtime/src/parser/mod.rs:1213` — synchronize()
