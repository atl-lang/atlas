# Atlas Intel Audit Prompt

You are conducting an intel audit of Atlas, a production-grade Rust compiler (95K lines, 8 crates).

**Mode: DISCOVERY ONLY. Do not suggest fixes. Report ALL findings.**

## What to Check

### 1. Code Quality Violations
- `todo!()`, `unimplemented!()`, `// TODO`, `// FIXME`, `// HACK`
- `#[allow(dead_code)]`, `#[allow(unused)]`
- `.unwrap()` on user input or external data
- `panic!()` in production paths without justification
- Catch-all `_ =>` in match hiding unhandled cases

### 2. Unsafe Code
- `unsafe` blocks without `// SAFETY:` comments
- Potential undefined behavior
- Memory safety issues
- FFI boundary problems

### 3. Incomplete Implementations
- Stub functions that do nothing
- Features claimed complete but actually partial
- Functions that exist but are never called
- One engine updated without the other (parity violations)

### 4. Architectural Issues
- Duplicate/overlapping systems
- Dead code and orphaned files
- New modules not integrated with existing systems
- Inconsistent patterns across crates

### 5. Compiler-Specific
- Parser edge cases
- Type system gaps
- VM/interpreter divergence
- Error handling gaps

## Output Format

```
VERDICT: PASS or FAIL
SUMMARY: one line

BLOCKERS: (problems that must be fixed)
- file:line - description

ISSUES: (problems to track)
- P0-3|severity|component|title|description

FINDINGS: (all other discoveries)
- file:line - description
```

Components: parser, binder, typechecker, interpreter, vm, codegen, jit, runtime, stdlib, lsp, cli, infra, docs
Severity: critical, high, medium, low

## Rules

- Report ALL findings, not just top N
- Include file paths and line numbers
- Do NOT suggest fixes
- Do NOT skip anything
- This is intel gathering
