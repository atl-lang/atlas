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

**B14 implemented these standards system-wide. B17 unified the descriptor and render layers.**
**Any NEW error code or error site added NOW must meet these standards immediately — no exceptions.**

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

### 3. Descriptor Emit Pattern — MANDATORY (B17, D-044)
All error emit call sites MUST use the descriptor API. Bare `Diagnostic::error_with_code()` chains are BANNED.

```rust
// BANNED — bare chain, no descriptor:
Diagnostic::error_with_code("AT1002", "Unterminated string", span)
    .with_help("add a closing quote")

// REQUIRED — descriptor emit:
use crate::diagnostic::error_codes::UNTERMINATED_STRING;
UNTERMINATED_STRING.emit(span)
    .arg("key", value)          // fills {key} holes in message_template
    .with_help("extra context") // additive: appended after static_help
    .build()                    // -> Diagnostic
```

`static_help` on the descriptor provides the canonical help. Call sites may ADD extra context
via `.with_help()` / `.with_note()` — never duplicate what the descriptor already says.

The render path is unified: `DiagnosticFormatter::write_diagnostic()` is the single implementation.
`Diagnostic::to_human_string()` delegates to it. Do not add render logic anywhere else.

### 4. `is_secondary` field on Diagnostic — MANDATORY
`Diagnostic` struct must have `is_secondary: bool` (default false).
Errors emitted while `in_panic_mode` that are not suppressed must set `is_secondary = true`.
Display must label or visually distinguish secondary errors.

### 5. New AT/AW Error Codes — Must Be a Full DiagnosticDescriptor (B17, D-044)
Every new AT/AW code added to `error_codes.rs` MUST be a `DiagnosticDescriptor` constant with ALL fields:

```rust
// BANNED — incomplete, missing static_help:
pub const MY_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1099",
    title: "My error",
    message_template: "something went wrong",
    static_help: None,  // ← BANNED — must have help
    ..
};

// REQUIRED — full descriptor:
pub const MY_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1099",
    level: DiagnosticLevel::Error,
    title: "Impl block for unknown type",
    message_template: "no type named `{name}` found in scope",
    static_help: Some("check that the type is declared with `struct` before the `impl` block"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};
```

Rules:
- `static_help` is **mandatory** — every code must have actionable guidance
- No embedded `\n` in `static_help` or `static_note` — use separate help lines at call sites
- `message_template` uses named `{key}` holes — filled via `.arg("key", val)` at call sites
- Add to `DESCRIPTOR_REGISTRY` at the bottom of `error_codes.rs`

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

## Scope (Post-B17)

B14 + B17 are complete. All 6 standards are now enforced system-wide.

| Situation | What to do |
|-----------|-----------|
| Adding a NEW error code | Full `DiagnosticDescriptor` constant — all fields mandatory |
| Emitting a NEW error | Use `AT_CODE.emit(span).arg().build()` — bare chains are BANNED |
| Modifying an EXISTING error site | Use descriptor API — migrate if still using bare chain |
| Adding render logic | Add to `DiagnosticFormatter::write_diagnostic()` ONLY |

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

- [ ] New AT/AW code is a full `DiagnosticDescriptor` with `static_help` (no `None`)
- [ ] No embedded `\n` in `static_help` or `static_note`
- [ ] New code added to `DESCRIPTOR_REGISTRY` in `error_codes.rs`
- [ ] Emit call uses `AT_CODE.emit(span).arg().build()` — no bare `Diagnostic::error_with_code()`
- [ ] Parser error says "Expected X, found `Y` (Kind)"
- [ ] Cascade suppression not bypassed
- [ ] Span points at the bad token, not after it
- [ ] `atlas run bad.atlas` shows correct number of primary errors
- [ ] Descriptor tests pass: `cargo nextest run -p atlas-runtime -E 'test(descriptor)'`

---

## References

- **D-043** — Error Quality Contract (this rule, as a standing decision)
- **D-044** — DiagnosticDescriptor is the mandatory emit pattern (B17)
- **B14** — System-wide quality enforcement
- **B17** — Unified descriptor + render layer
- `crates/atlas-runtime/src/diagnostic.rs` — `Diagnostic` struct; `to_human_string()` delegates to formatter
- `crates/atlas-runtime/src/diagnostic/descriptor.rs` — `DiagnosticDescriptor`, `DiagnosticBuilder`, `.emit()` API
- `crates/atlas-runtime/src/diagnostic/error_codes.rs` — all AT/AW descriptor constants + `DESCRIPTOR_REGISTRY`
- `crates/atlas-runtime/src/diagnostic/formatter.rs` — single authoritative renderer (`write_diagnostic`)
- `crates/atlas-runtime/src/parser/mod.rs:1062` — error() / error_with_code()
- `crates/atlas-runtime/src/parser/mod.rs:1213` — synchronize()
