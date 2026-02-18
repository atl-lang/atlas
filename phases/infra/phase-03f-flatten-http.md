# Phase Infra-03f: Clean http.rs Banners

## Blocker
**REQUIRED:** Infra-03e complete.

---

## Objective
`http.rs` (862 lines) has no mod wrappers — it's already flat. This phase removes the
`// ===== filename.rs =====` section banners and replaces them with meaningful section
headers that describe behavior, not file origin.

## Current structure
```rust
use ...

// ===== http_core_tests.rs =====

fn eval_ok(...) { ... }
fn eval_expect_error(...) { ... }

#[test] fn test_http_...

// ===== http_advanced_tests.rs =====

#[test] fn test_http_builder_...
```

## Target structure
```rust
use ...

fn eval_ok(...) { ... }  // keep — not in common/ (http-specific return type)
fn eval_expect_error(...) { ... }

// --- Core HTTP requests ---

#[test] fn test_http_get_...

// --- Builder pattern ---

#[test] fn test_http_builder_...
```

## Implementation

### Step 1: Scout
```bash
grep -n "^// =====\|^#\[test\]\|^fn " crates/atlas-runtime/tests/http.rs | head -30
```

### Step 2: Replace banners
```bash
# Identify what tests follow each banner to name sections meaningfully
grep -n "^// =====\|^fn test_" crates/atlas-runtime/tests/http.rs
```

Replace `// ===== http_core_tests.rs =====` with `// --- Core HTTP requests ---`
Replace `// ===== http_advanced_tests.rs =====` with `// --- Builder pattern ---`

Use the Edit tool (surgical replacement, not a full rewrite).

### Step 3: Check for common/ helpers
```bash
grep -n "^fn eval_ok\|^fn eval_expect_error" crates/atlas-runtime/tests/http.rs
```
These helpers have HTTP-specific return types — they stay in the file (not extracted to common/).

### Step 4: Verify
```bash
cargo nextest run -p atlas-runtime --test http
```

## Acceptance
- No `// ===== *.rs =====` banners remain
- Meaningful section comments in place
- All tests pass (same count)
- Commit: `refactor(tests): Infra-03f — clean http.rs section banners`
- Update STATUS.md: mark 03f complete, Next Phase → 03g
