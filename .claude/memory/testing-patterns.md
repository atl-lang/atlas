# Atlas Testing Patterns

**Auto-load:** Key rules auto-load via `.claude/rules/atlas-testing.md` when touching test files. This file is the full reference — read it when you need LSP lifetime details, full corpus workflow, or edge cases beyond what the rule file covers.

**Purpose:** How to write and organize tests in Atlas. Follow this exactly — deviations recreate the binary bloat problem that cost significant engineering time.

---

## THE CARDINAL RULE: No New Test Files Without Authorization

**DO NOT create new `*.rs` files in `crates/atlas-runtime/tests/`.**

The test suite was consolidated from many small files into domain files (infra phases 01-03). Every new test file is a new binary: more link time, more process overhead, slower CI. New tests go into the **existing domain files** listed below.

**The only exception:** If a new domain is added that genuinely has no existing home, get explicit approval and document the reasoning in `memory/patterns.md`.

**IMPORTANT:** This rule applies to `atlas-runtime` only. Other crates (`atlas-lsp`, `atlas-cli`) may have different test organization patterns.

---

## Canonical Test Files

**Five domains now use subdirectory splits. The `.rs` root files are THIN ROUTERS — do not add tests to them. Open the subdir file directly.**

### Subdirectory-split domains

| Domain | Add tests to... |
|--------|----------------|
| Stdlib | `tests/stdlib/strings.rs`, `json.rs`, `io.rs`, `types.rs`, `functions.rs`, `collections.rs`, `parity.rs`, `vm_stdlib.rs`, `integration.rs`, `docs_verification.rs`, `array_intrinsics.rs`, `array_pure.rs`, `math_basic.rs`, `math_trig.rs`, `math_utils_constants.rs` |
| Type system | `tests/typesystem/inference.rs`, `constraints.rs`, `flow.rs`, `generics.rs`, `bindings.rs`, `integration.rs` |
| VM | `tests/vm/integration.rs`, `member.rs`, `complex_programs.rs`, `regression.rs`, `performance.rs`, `functions.rs`, `nested.rs`, `for_in.rs`, `array_intrinsics.rs`, `array_pure.rs`, `math_basic.rs`, `math_trig.rs`, `math_utils_constants.rs` |
| Interpreter | `tests/interpreter/member.rs`, `nested_functions.rs`, `scope.rs`, `pattern_matching.rs`, `assignment.rs`, `for_in.rs`, `integration.rs` |
| System | `tests/system/path.rs`, `filesystem.rs`, `process.rs`, `compression.rs` |

**Pick by domain match** (e.g., new string builtin → `tests/stdlib/strings.rs`, new VM opcode → `tests/vm/regression.rs`).

**Pattern for new submodule files** (if a subdirectory hits threshold and needs further splitting): these are already in subdirectory form — just add a new file there and declare it with `#[path]` in the router root.

### Single-file domains

| File | What goes here |
|------|---------------|
| `tests/frontend_syntax.rs` | Lexer, parser, syntax, operators, keywords, warnings |
| `tests/diagnostics.rs` | Diagnostic ordering, error spans, source maps |
| `tests/frontend_integration.rs` | Full frontend pipeline, AST, bytecode validation |
| `tests/collections.rs` | HashMap, HashSet, Queue, Stack, generics runtime |
| `tests/bytecode/` | Bytecode: `compiler.rs`, `optimizer.rs`, `profiler.rs`, `parity.rs`, `patterns.rs`, `mod_tests.rs`, `validator.rs` (router: `tests/bytecode.rs`) |
| `tests/async_runtime.rs` | Futures, channels, timers, async I/O |
| `tests/ffi.rs` | FFI types, parsing, interpreter, VM, callbacks |
| `tests/debugger.rs` | Debugger execution, inspection, protocol |
| `tests/security.rs` | Permissions, sandboxing, audit logging |
| `tests/modules.rs` | Module binding, execution, resolution |
| `tests/http.rs` | HTTP core and advanced (most `#[ignore = "requires network"]`) |
| `tests/datetime_regex.rs` | DateTime, regex core and operations |
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

**Minimum counts:** Defined per-phase in the phase file's acceptance criteria. Always meet or exceed.

**Corpus requirement:**
- New language syntax feature: add corpus files (pass and fail cases)
- New stdlib function: add at least one corpus file (pass case)

---

## LSP Testing Pattern (CRITICAL - Different from Runtime)

**⚠️ LSP tests CANNOT use the same pattern as runtime tests.**

### The Lifetime Problem

LSP tests use `tower-lsp::LspService` which owns the server. You **CANNOT** extract server creation to a helper function:

```rust
// ❌ WRONG - This will not compile
async fn init_server() -> AtlasLspServer {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner(); // Returns &AtlasLspServer
    server // ERROR: cannot return reference, service is dropped
}
```

**Why it fails:**
- `service.inner()` returns `&AtlasLspServer` (reference)
- `service` owns the server
- When function returns, `service` is dropped
- Reference becomes invalid (lifetime error)
- `AtlasLspServer` doesn't implement `Clone`, so can't clone it

### The Correct Pattern

**Every LSP test must inline server creation:**

```rust
#[tokio::test]
async fn test_feature() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    // Initialize
    server.initialize(InitializeParams::default()).await.unwrap();
    server.initialized(InitializedParams {}).await;

    // Open document
    let uri = Url::parse("file:///test.atl").unwrap();
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "fn test() {}".to_string(),
        },
    }).await;

    // Test your feature
    let result = server.hover(HoverParams { /* ... */ }).await;
    assert!(result.is_ok());
}
```

### Quick Reference

**Before writing LSP tests:**
1. Read existing LSP test files first (`crates/atlas-lsp/tests/*.rs`)
2. Copy the inline server creation pattern exactly
3. Don't try to create helper functions for server setup

**LSP test structure:**
- Each test file is independent (OK to create new files in `atlas-lsp/tests/`)
- Each test creates its own service inline
- Use existing tests as templates (they already use the correct pattern)

---

## What Changed (Context for Agents)

Pre-infra: one test file per feature, massive binary bloat, slow test runs.
Post-infra: consolidated domain files, fast builds, fast test runs.

The old pattern (`{feature}_tests.rs`) is what caused the problem. Don't recreate it **in atlas-runtime**.
