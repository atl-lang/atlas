---
paths:
  - "crates/atlas-runtime/tests/**"
  - "crates/atlas-lsp/tests/**"
  - "crates/**/tests/**"
---

# Atlas Testing Rules

Auto-loaded when touching test files. Full patterns in `/Users/proxikal/.claude/projects/-Users-proxikal-dev-projects-atlas/memory/testing-patterns.md`.

## Argus — Official Atlas Test Framework (atl-pkg)

**Two-tier rule — no exceptions:**

| Context | Use |
|---------|-----|
| Compiler corpus tests (`pass/*.atlas`, `fail/*.atlas`) | Built-in `test.*` only. No package deps — compiler must work before packages resolve. |
| Integration / scenario `.atl` tests in user projects | **Use Argus** (`import { describe, it } from 'argus'`). |
| Writing Argus-style tests for Atlas stdlib examples | Use Argus. |

**Argus lives at:** `~/dev/projects/atl-pkg/argus` | `github.com/atl-pkg/argus`

**Argus API quick ref (AI generation guide):**
```atlas
import { describe, it, beforeEach, each, run } from 'argus';
import { expect } from 'argus';
import { spy, spy1, stub, verify } from 'argus';
import { fixture } from 'argus';
import { pretty } from 'argus';

describe("Suite name", fn(): void {
    beforeEach(fn(): void { /* setup */ });

    it("test name", fn(): void {
        expect(actual).toEqual(expected);
        expect(value).not().toBeEmpty();
    });

    // Parametric
    each([[1, 2], [3, 4]], fn(row: any[]): void {
        it("row " + row[0].string(), fn(): void {
            expect(row[0] + row[1]).toBeGreaterThan(0);
        });
    });
});

pretty(run());
```

## Cardinal Rule: No New Test Files in atlas-runtime

Every new test file = a new binary = more link time + slower CI. **Add to existing domain files.**

## Size Limit: 12KB Maximum Per Test File

Test files are token-dense (Atlas source snippets, string literals, long assertions). An agent
reading a large test file burns significant tokens before writing a line.

**Before touching any test file:**
```bash
du -sh <target-file>
```
- **> 12KB:** BLOCKING — split into domain subfiles first
- **10–12KB:** Acceptable — monitor for future split if it grows
- **Target: ~10KB per file**

**Subdirectory structure:** `stdlib`, `typesystem`, `vm`, `system`, `bytecode`, `frontend_syntax`, `frontend_integration` are split into domain submodules. Each monolith `.rs` is a thin router. Add tests to the appropriate submodule file, NOT to the router root.

| Domain | File |
|--------|------|
| Lexer, parser, syntax | `tests/frontend_syntax/` (lexer, parser_basics, parser_errors, operator_precedence_keywords, generics, modules_warnings_part1, warnings_part2, for_in_traits_part1, traits_part2) — router: `tests/frontend_syntax.rs` |
| Diagnostics, error spans | `tests/diagnostics.rs` |
| Full frontend pipeline | `tests/frontend_integration/` (integration_part_{1-5}, ast_part_{1-2}, bytecode_validator, ownership, traits, anonfn_part_{1-2}) — router: `tests/frontend_integration.rs` |
| Type inference, generics | `tests/typesystem/` (inference, constraints, flow, generics, bindings, integration) |
| VM execution | `tests/vm/` (integration, member, complex_programs, regression, performance, functions, nested, for_in, array_intrinsics, array_pure, math_basic, math_trig, math_utils_constants, opcodes) |
| Stdlib functions | `tests/stdlib/` (integration, real_world, strings, json, io, types, functions, collections, parity, vm_stdlib, docs_verification, array_intrinsics, array_pure, math_basic, math_trig, math_utils_constants) |
| Collections (HashMap, Set, Queue) | `tests/collections.rs` |
| Bytecode compiler, optimizer, profiler | `tests/bytecode/` (compiler, optimizer, profiler, parity, patterns, mod_tests, validator) — router: `tests/bytecode.rs` |
| Async, futures, channels | `tests/async_runtime.rs` |
| Closures | `tests/closures.rs` |
| Pattern matching | `tests/pattern_matching.rs` |
| FFI | `tests/ffi.rs` |
| Security, permissions | `tests/security.rs` |
| Regression (bug reproductions) | `tests/regression.rs` |

Exception: explicit approval required for genuinely new domains.

## Preferred: Corpus Tests

New language behavior → write `.atlas` files in `crates/atlas-runtime/tests/corpus/`:
- `pass/foo.atlas` + `pass/foo.stdout` — must run and produce expected output
- `fail/bar.atlas` + `fail/bar.stderr` — must produce specific error
- Generate expected: `UPDATE_CORPUS=1 cargo nextest run -p atlas-runtime --test corpus`

Corpus tests verify VM execution behavior. Prefer corpus over Rust tests.

## Test Pattern

```rust
#[test]
fn test_feature() {
    let result = run_vm(r#"len("hello")"#);
    assert_eq!(result, Ok("5".to_string()));
}
```

## `#[ignore]` Rules

Bare `#[ignore]` is banned. Always give a reason:
```rust
#[ignore = "requires network"]
#[ignore = "not yet implemented: closure-capture"]
```

## LSP Tests — Different Pattern

LSP tests **cannot** use helper functions for server creation (lifetime error). Every test inlines:
```rust
#[tokio::test]
async fn test_feature() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    // inline all setup here
}
```

Add LSP tests to existing files in `crates/atlas-lsp/tests/`. Creating new LSP test files is allowed (different from runtime — no binary bloat issue).

## Snapshot Tests (insta) — Strict Protocol

Snapshots live exclusively in `tests/**/snapshots/`. They are test assertions — not noise to clear.

### When you MAY accept a snapshot change

A changed snapshot is only valid if it was caused by an **intentional, correct improvement**:

| Valid reason to accept | Example |
|------------------------|---------|
| New feature added output | Added tuple Display — snapshot gains `(1, 2)` |
| Bug fix corrected wrong output | Fixed wrong type name — snapshot changes `"int"` → `"number"` |
| Error message made more specific | B14 improvement — snapshot gains `found \`[\` (LeftBracket)` |
| Syntax changed by decision | D-041 array prefix — snapshot changes `T[]` → `[]T` |

### When you MUST REJECT a snapshot change (regression)

**BLOCKING — do not accept, fix the code instead:**

| Banned reason | Example |
|---------------|---------|
| Cascade error regression | Snapshot gains 3 extra secondary errors |
| Help text became less specific | `"add closing quote"` → `"check your syntax"` |
| Output became empty or shorter | Snapshot loses expected output lines |
| Format changed without a decision | Span format changed with no D-XXX backing it |
| Wrong feature accepted to make CI green | Snapshot accepted to unblock a commit, not because it's correct |

### The Review Process — MANDATORY before accepting

```bash
# 1. See what changed — READ the diff before doing anything
cargo nextest run -p atlas-runtime -E 'test(failing_test)' 2>&1
# insta prints the diff to terminal — read every line

# 2. Decide: is this change correct?
#    YES (intentional improvement) → accept it
#    NO (regression) → fix the code, do NOT accept

# 3. If correct — accept only the specific snapshot, not all:
cargo insta review                    # interactive — review each one
# OR for a specific test:
cargo insta accept --test-name test_exact_name

# BANNED — bulk accepts without review:
cargo insta accept                    # ❌ accepts everything blindly
INSTA_UPDATE=always cargo nextest ... # ❌ auto-accepts all changes
cargo insta accept --all              # ❌ never
```

### Snapshot files are committed as source

`.snap` files are committed to git — they are the ground truth.
A changed `.snap` file in a commit means the behavior changed.
The commit message must explain WHY (which decision, which fix).

```bash
# Good commit message for snapshot changes:
git commit -m "fix(parser): H-178 — correct error cascade, update snapshots"
# NOT:
git commit -m "chore: accept snapshots"  # ❌ tells nobody what changed or why
```

### Post-B14: Error quality snapshots are BLOCKING regressions

After B14 ships, `tests/diagnostics/error_quality.rs` contains locked snapshots
for error output quality. These are the permanent regression gate for D-043.
A changed error quality snapshot requires explicit architect approval — not just review.

## Run Commands

```bash
# DURING DEVELOPMENT — targeted tests only:
cargo nextest run -E 'test(test_name)'               # single test
cargo nextest run -p atlas-runtime --test closures    # one domain file
cargo nextest run -p atlas-runtime --test corpus      # corpus only
cargo nextest run -E 'test(parity)'                   # parity sweep

# NEVER run `cargo nextest run --workspace` manually.
# The pre-commit Guardian hook runs the full suite on every `git commit`.
# Never kill a running cargo process — it leaves lock files that block all future runs.
```
