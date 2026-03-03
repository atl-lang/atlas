# Atlas Intel Rules

## Violations (FAIL immediately)

- `todo!()`, `unimplemented!()`
- `// TODO`, `// FIXME`, `// HACK`
- `#[allow(dead_code)]`, `#[allow(unused)]`
- `.unwrap()` on user input or external data
- `panic!()` without justification
- `_ =>` catch-all hiding unhandled cases
- `unsafe` without `// SAFETY:` comment
- One engine changed without the other (parity)

## What to Find

1. Incomplete implementations (stubs, partial work)
2. Dead code and orphaned files
3. Unsafe code without safety docs
4. Architectural issues (duplicate systems, inconsistent patterns)
5. Quality issues (missing error handling, untested code)

## Output Format

```
VERDICT: PASS or FAIL
SUMMARY: one line

BLOCKERS:
- file:line - description

ISSUES:
- PRIORITY|SEVERITY|COMPONENT|title|description

FINDINGS:
- file:line - description
```

Priority: P0 (blocker), P1 (critical), P2 (important), P3 (nice)
Severity: critical, high, medium, low
Components: parser, binder, typechecker, interpreter, vm, codegen, jit, runtime, stdlib, lsp, cli, infra, docs

## Rules

- Report ALL findings
- Include file:line
- Do NOT suggest fixes
- Do NOT skip anything
