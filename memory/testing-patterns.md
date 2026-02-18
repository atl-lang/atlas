# Atlas Testing Patterns

**Purpose:** How to write and organize tests in Atlas. Follow this exactly — deviations recreate the 125-binary bloat problem that cost significant engineering time.

---

## THE CARDINAL RULE: No New Test Files Without Authorization

**DO NOT create new `*.rs` files in `crates/atlas-runtime/tests/`.**

The test suite was consolidated from 125 → ~17 domain files (infra phases 01-03). Every new test file is a new binary: more link time, more process overhead, slower CI. New tests go into the **existing domain files** listed below.

**The only exception:** If a new domain is added that genuinely has no existing home, get explicit approval and document the reasoning in `memory/decisions.md`.

---

## The 17 Canonical Test Files

Add new tests to the appropriate existing file:

| File | What goes here |
|------|---------------|
| `tests/frontend_syntax.rs` | Lexer, parser, syntax, operators, keywords, warnings, for-in parsing |
| `tests/diagnostics.rs` | Diagnostic ordering, error spans, source maps, enhanced errors |
| `tests/frontend_integration.rs` | Full frontend pipeline, AST, bytecode validation |
| `tests/typesystem.rs` | Type inference, aliases, guards, union/intersection, generics, constraints |
| `tests/interpreter.rs` | Interpreter execution, member access, nested functions, scope, patterns |
| `tests/vm.rs` | VM execution, regression, performance, first-class functions, generics |
| `tests/stdlib.rs` | All stdlib functions (strings, arrays, json, IO, math, options, results, parity) |
| `tests/collections.rs` | HashMap, HashSet, Queue, Stack, generics runtime |
| `tests/bytecode.rs` | Bytecode compiler, optimizer, profiler, parity |
| `tests/async_runtime.rs` | Futures, channels, timers, async I/O |
| `tests/ffi.rs` | FFI types, parsing, interpreter, VM, callbacks |
| `tests/debugger.rs` | Debugger execution, inspection, protocol |
| `tests/security.rs` | Permissions, sandboxing, audit logging |
| `tests/modules.rs` | Module binding, execution, resolution |
| `tests/http.rs` | HTTP core and advanced (most tests `#[ignore = "requires network"]`) |
| `tests/datetime_regex.rs` | DateTime, regex core and operations |
| `tests/system.rs` | Path, filesystem, process, compression (gzip, tar, zip) |
| `tests/api.rs` | Public embedding API, conversion, native functions, reflection |
| `tests/repl.rs` | REPL state, types |
| `tests/regression.rs` | Regression suite (specific bug reproductions) |

**Unit tests** (testing internal module logic with no external deps) go in `#[cfg(test)]` blocks inside the source file they test — NOT in `tests/`.

---

## The File-Based Corpus (Preferred for New Feature Tests)

The preferred way to add tests for language behavior is via the corpus, not Rust test functions.

**Location:** `crates/atlas-runtime/tests/corpus/`

```
tests/corpus/
├── pass/         # .atlas files that should run and produce expected output
│   └── foo.atlas + foo.stdout
├── fail/         # .atlas files that should produce specific errors
│   └── bar.atlas + bar.stderr
└── warn/         # .atlas files that should produce specific warnings
    └── baz.atlas + baz.stderr
```

**How to add a corpus test:**
1. Write a `.atlas` file demonstrating the feature
2. Run with `UPDATE_CORPUS=1 cargo nextest run -p atlas-runtime --test corpus` to generate the expected output file
3. Commit both files

**Why corpus over Rust tests:** Corpus tests are written in Atlas (readable by anyone), automatically test parity (harness runs both interpreter and VM), and serve as living documentation of language behavior. This is how rustc, clang, and Go test their compilers.

---

## Writing Rust Tests (When Corpus Isn't Enough)

### Standard helper pattern (use the existing helper in each domain file)

Each domain file already has a canonical `eval_ok` / `run` helper at the top. Use it — don't define a new one.

```rust
// Use the existing helper in the file, e.g.:
#[test]
fn test_new_behavior() {
    assert_eq!(eval_ok("1 + 2"), Value::Number(3.0));
}
```

### Parity pattern (interpreter + VM identical output)

For anything in `stdlib.rs` or `bytecode.rs`, use the existing `assert_parity` helper:

```rust
#[test]
fn test_feature_parity() {
    assert_parity(r#"len("hello")"#, "5");
}
```

This runs the code in both engines and asserts identical output. Never write two separate functions (`test_feature_interpreter` and `test_feature_vm`) — that's the old pattern.

### Parameterized tests (rstest)

```rust
use rstest::rstest;

#[rstest]
#[case("hello", 5)]
#[case("", 0)]
#[case("hello世界", 7)]
fn test_len(#[case] input: &str, #[case] expected: f64) {
    assert_parity(&format!(r#"len("{}")"#, input), &expected.to_string());
}
```

### Snapshot tests (insta) — for error messages and complex output

```rust
use insta::assert_snapshot;

#[test]
fn test_error_message_quality() {
    let err = eval_err(r#"let x: string = 42;"#);
    assert_snapshot!(err);
}
```

---

## The #[ignore] Rules (Non-Negotiable)

**Bare `#[ignore]` is banned.** Every ignored test must have an explicit reason:

```rust
// ✅ CORRECT
#[ignore = "requires network"]
#[ignore = "requires tokio LocalSet context — re-enable when async runtime phase completes"]
#[ignore = "requires platform: linux"]
#[ignore = "not yet implemented: feature-name"]

// ❌ BANNED
#[ignore]
```

If you find a bare `#[ignore]`, fix it before writing new tests. Don't add to the debt.

---

## Test Execution Commands

### During development (use these)
```bash
# Single test — always use --exact
cargo nextest run -p atlas-runtime -E 'test(exact_test_name)'

# One domain file
cargo nextest run -p atlas-runtime --test stdlib

# Corpus only
cargo nextest run -p atlas-runtime --test corpus
```

### Before handoff (GATE 6)
```bash
cargo nextest run -p atlas-runtime --test <domain_file>  # Domain file for phase
cargo clippy -p atlas-runtime -- -D warnings
```

### Full suite
```bash
cargo nextest run -p atlas-runtime          # ~15-20 seconds, excludes network/slow tests
cargo nextest run -p atlas-runtime --run-ignored all  # Includes network tests (slow)
```

### Benchmarks (performance-sensitive changes only)
```bash
cargo bench -p atlas-runtime --bench vm     # VM benchmark
cargo bench -p atlas-runtime               # Full benchmark suite
```

### Fuzz (when modifying lexer/parser/typechecker)
```bash
cargo +nightly fuzz run fuzz_lexer -- -max_total_time=60
cargo +nightly fuzz run fuzz_parser -- -max_total_time=60
```

---

## Quality Standards

Every new feature must add tests to the appropriate domain file covering:

1. **Happy path** — basic correct usage
2. **Edge cases** — empty input, boundary values, large input
3. **Error cases** — wrong types, invalid arguments, out of bounds
4. **Parity** — interpreter and VM produce identical output (use `assert_parity`)

**Minimum counts (from phase acceptance criteria):**
- New stdlib function: 10+ tests
- New collection type: 15+ tests
- New language feature: 20+ tests

**Corpus requirement (new from infra-05):**
- Every new language syntax feature: add at least 2 corpus files (one pass, one fail)
- Every new stdlib function: add at least 1 corpus file (pass case)

---

## What Changed (Context for Agents)

Pre-infra: 125 test files, one per feature, ~2.3GB of binaries, 60-90s test runs.
Post-infra: ~17-20 domain files, <400MB of binaries, <20s test runs.

The old pattern (`{feature}_tests.rs`) is what caused the problem. Don't recreate it.
