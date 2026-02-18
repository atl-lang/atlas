# Phase Infra-03k: Clean async_runtime.rs Banners

## Blocker
**REQUIRED:** Infra-03d complete.

---

## Objective
`async_runtime.rs` (2,336 lines) has no mod wrappers — already flat. This phase:
1. Removes `// ===== filename.rs =====` section banners
2. Replaces them with meaningful behavior-based section comments
3. Verifies `eval_ok` / `eval` helpers are not duplicated (handled in 03a merge)

## Current structure
```rust
use ...

fn eval_ok(...) { ... }
fn eval(...) { ... }

// ===== async_future_tests.rs =====

#[test] fn test_future_...

// ===== async_io_tests.rs =====

#[test] fn test_write_...
#[ignore = "requires network"] fn test_fetch_...

// ===== async_primitives_tests.rs =====

#[ignore = "requires tokio..."] fn test_spawn_...
```

## Target structure
```rust
use ...

fn eval_ok(...) { ... }  // keep — async-specific return type
fn eval(...) { ... }

// --- Future state ---
#[test] fn test_future_...

// --- Async I/O ---
#[test] fn test_write_...
#[ignore = "requires network"] fn test_fetch_...

// --- Async primitives (tokio context) ---
#[ignore = "requires tokio LocalSet context..."] fn test_spawn_...
```

## Implementation

### Step 1: Find the banner lines
```bash
grep -n "^// =====" crates/atlas-runtime/tests/async_runtime.rs
```

### Step 2: Identify what follows each banner (to name sections meaningfully)
```bash
grep -n "^// =====\|^fn test_\|^#\[test\]\|^#\[ignore" \
  crates/atlas-runtime/tests/async_runtime.rs | head -30
```

### Step 3: Replace banners with Edit tool (surgical — 3 replacements only)
Three `Edit` calls replacing each banner with a meaningful `// --- ... ---` comment.

### Step 4: Verify no duplicate helpers
```bash
grep -c "^fn eval_ok\|^fn eval" crates/atlas-runtime/tests/async_runtime.rs
```
Should be 1 each.

### Step 5: Verify
```bash
cargo nextest run -p atlas-runtime --test async_runtime
```
Count must match: 65 passed, 59 skipped.

## Acceptance
- No `// ===== *.rs =====` banners remain
- 3 meaningful section comments in place
- 65 pass, 59 skip
- Commit: `refactor(tests): Infra-03k — clean async_runtime.rs banners`
- Update STATUS.md: mark 03k complete, Next Phase → 03l
