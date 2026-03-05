# Code Quality Baseline (Non-Test Code)

Target: Atlas workspace (all crates)
Severity: Medium
Status: Open

## Finding 1: High `unwrap`/`panic` density in production paths

Evidence (counts exclude tests):
- `unwrap(` occurrences: 576
- `expect(` occurrences: 61
- `panic!(` occurrences: 39

Hotspots (non-test, per crate):
- `atlas-cli`: 234 unwrap, 12 panic
- `atlas-build`: 77 unwrap, 2 panic
- `atlas-runtime`: 137 unwrap, 58 expect, 23 panic

What/Why:
- These are acceptable for prototypes but undermine a systems-language runtime, CLI, and build pipeline.

Impact:
- Increases crash risk, obscures error propagation, and complicates CLI tooling and embedding reliability.

Recommendation:
- Replace panic/unwrap in runtime/cli/build with structured error returns.
- Add a release gate for `panic!` and `unwrap` counts in non-test code.

---

## Finding 2: Unsafe is concentrated in JIT/FFI/VM without visible invariants

Evidence (counts exclude tests):
- `unsafe` occurrences: 54
- Hot files: 
  - `crates/atlas-runtime/src/ffi/callbacks.rs` (13)
  - `crates/atlas-jit/src/backend.rs` (8)
  - `crates/atlas-runtime/src/vm/mod.rs` (6)

What/Why:
- Unsafe is expected in FFI/JIT but invariants are not summarized in-code.

Impact:
- Harder to audit and increases risk of UB when the system scales.

Recommendation:
- Add a short safety section comment above each unsafe cluster documenting invariants.

---

## Finding 3: Repo hygiene issues (.DS_Store)

Evidence:
- `.DS_Store` exists in repo root and multiple crates.

Impact:
- Minor professionalism issue; pollutes diffs and tooling.

Recommendation:
- Remove `.DS_Store` files and add to `.gitignore` if not already excluded.

