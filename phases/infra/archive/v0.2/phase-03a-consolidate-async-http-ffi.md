# Phase Infra-03a: Test Consolidation — Async, HTTP, FFI

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase Infra-01 and Infra-02 complete.

```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -5
ls crates/atlas-runtime/tests/*.rs | wc -l  # should be ~50
```

---

## Objective
Merge 11 source files into 3 domain files. These domains are handled first because they contain the most sensitive annotation requirements: tokio context ignores, network ignores, and platform-specific `cfg_attr` ignores that must be preserved exactly.

## Source → Target Mapping

| Target | Sources | Notes |
|--------|---------|-------|
| `async_runtime.rs` | `async_future_tests.rs`, `async_io_tests.rs`, `async_primitives_tests.rs` | Most tests `#[ignore]`'d — preserve all |
| `http.rs` | `http_core_tests.rs`, `http_advanced_tests.rs` | All carry `#[ignore = "requires network"]` |
| `ffi.rs` | `ffi_callback_tests.rs`, `ffi_integration_complete_tests.rs`, `ffi_interpreter_tests.rs`, `ffi_parsing_tests.rs`, `ffi_types_tests.rs`, `ffi_vm_tests.rs` | `#[cfg_attr(target_os = ..., ignore = ...)]` critical |

---

## CRITICAL EXECUTION APPROACH

**DO NOT use the `Read` tool to read source files in full.** These files are 400–1100 lines each.

Use this pattern instead:

```bash
# Step 1: Extract all unique use/extern crate lines from sources
grep -h "^use \|^extern crate \|^mod " \
  crates/atlas-runtime/tests/async_future_tests.rs \
  crates/atlas-runtime/tests/async_io_tests.rs \
  crates/atlas-runtime/tests/async_primitives_tests.rs \
  | sort -u

# Step 2: Pipe source content directly into merged file via python
python3 - <<'EOF'
import re, sys

sources = [
    "crates/atlas-runtime/tests/async_future_tests.rs",
    "crates/atlas-runtime/tests/async_io_tests.rs",
    "crates/atlas-runtime/tests/async_primitives_tests.rs",
]

# Collect unique use lines
uses = set()
bodies = []

for path in sources:
    with open(path) as f:
        content = f.read()
    for line in content.splitlines():
        if line.startswith("use ") or line.startswith("extern crate "):
            uses.add(line)
    # Strip top-level use lines, keep the rest
    body = re.sub(r'^(?:use |extern crate )[^\n]+\n', '', content, flags=re.MULTILINE)
    body = body.strip()
    bodies.append(f"// ===== {path.split('/')[-1]} =====\n\n{body}")

header = "// Merged: async_future_tests + async_io_tests + async_primitives_tests\n\n"
header += "\n".join(sorted(uses)) + "\n\n"
print(header + "\n\n".join(bodies))
EOF
```

Apply this same pattern for each merge. Adjust for deduplication of helper functions using Grep first:
```bash
grep -n "^fn \|^async fn \|^pub fn " \
  crates/atlas-runtime/tests/async_future_tests.rs \
  crates/atlas-runtime/tests/async_io_tests.rs \
  crates/atlas-runtime/tests/async_primitives_tests.rs \
  | sort -t: -k3
```

---

## Implementation Steps

### Step 1: Create `tests/async_runtime.rs`

```bash
# Check for duplicate helpers
grep -h "^fn \|^async fn \|^pub fn " \
  crates/atlas-runtime/tests/async_future_tests.rs \
  crates/atlas-runtime/tests/async_io_tests.rs \
  crates/atlas-runtime/tests/async_primitives_tests.rs | sort | uniq -d
```

Write merged file. Verify:
```bash
cargo nextest run -p atlas-runtime --test async_runtime
```

### Step 2: Create `tests/http.rs`

```bash
grep -h "^fn \|^async fn \|^pub fn " \
  crates/atlas-runtime/tests/http_core_tests.rs \
  crates/atlas-runtime/tests/http_advanced_tests.rs | sort | uniq -d
```

Write merged file. Verify:
```bash
cargo nextest run -p atlas-runtime --test http
```

### Step 3: Create `tests/ffi.rs`

Check platform annotations to ensure none are lost:
```bash
grep -n "cfg_attr\|target_os\|#\[ignore" \
  crates/atlas-runtime/tests/ffi_callback_tests.rs \
  crates/atlas-runtime/tests/ffi_integration_complete_tests.rs \
  crates/atlas-runtime/tests/ffi_interpreter_tests.rs \
  crates/atlas-runtime/tests/ffi_parsing_tests.rs \
  crates/atlas-runtime/tests/ffi_types_tests.rs \
  crates/atlas-runtime/tests/ffi_vm_tests.rs
```

Write merged file preserving all platform guards. Verify:
```bash
cargo nextest run -p atlas-runtime --test ffi
```

### Step 4: Delete old files
```bash
git rm crates/atlas-runtime/tests/async_future_tests.rs \
       crates/atlas-runtime/tests/async_io_tests.rs \
       crates/atlas-runtime/tests/async_primitives_tests.rs \
       crates/atlas-runtime/tests/http_core_tests.rs \
       crates/atlas-runtime/tests/http_advanced_tests.rs \
       crates/atlas-runtime/tests/ffi_callback_tests.rs \
       crates/atlas-runtime/tests/ffi_integration_complete_tests.rs \
       crates/atlas-runtime/tests/ffi_interpreter_tests.rs \
       crates/atlas-runtime/tests/ffi_parsing_tests.rs \
       crates/atlas-runtime/tests/ffi_types_tests.rs \
       crates/atlas-runtime/tests/ffi_vm_tests.rs
```

### Step 5: Verify count integrity
```bash
cargo nextest list -p atlas-runtime | grep -c "async_runtime\|http\|ffi"
cargo clippy -p atlas-runtime -- -D warnings
```

## Tests
No new tests. All existing tests must pass with identical count.

## Acceptance
- `async_runtime.rs`, `http.rs`, `ffi.rs` compile and tests pass
- All 11 old files deleted via `git rm`
- All `#[ignore]` and `#[cfg_attr]` annotations preserved exactly
- Zero clippy warnings
- Commit: `refactor(tests): Infra-03a — merge async/http/ffi test files`
- Update STATUS.md: mark Infra-03a complete, Next Phase → Infra-03b
