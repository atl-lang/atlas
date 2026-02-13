# Atlas Coverage Matrix

This matrix maps specification areas to implementation phases to ensure full coverage.

## Spec → Phases
- Lexer & tokens: `frontend/phase-01-lexer.md`, `frontend/phase-07-lexer-edge-cases.md`
- Parser & AST: `frontend/phase-02-parser.md`, `frontend/phase-03-ast-build.md`, `frontend/phase-05-grammar-conformance.md`
- Diagnostics core: `typing/phase-03-diagnostics-pipeline.md`, `typing/phase-04-diagnostic-normalization.md`, `typing/phase-08-diagnostics-versioning.md`
- Binder & scopes: `typing/phase-01-binder.md`, `typing/phase-06-scopes-shadowing.md`
- Type rules: `typing/phase-02-typechecker.md`
- Warnings: `typing/phase-14-warnings.md`
- Runtime values: `interpreter/phase-03-runtime-values.md`, `interpreter/phase-07-value-model-tests.md`
- Interpreter: `interpreter/phase-01-interpreter-core.md`, `interpreter/phase-04-arrays-mutation.md`, `interpreter/phase-06-control-flow.md`
- Runtime errors: `interpreter/phase-08-runtime-errors.md`
- REPL: `interpreter/phase-02-repl.md`, `cli/phase-03-repl-modes.md`
- Bytecode format: `bytecode-vm/phase-03-bytecode-format.md`, `bytecode-vm/phase-16-bytecode-format-tests.md`
- VM: `bytecode-vm/phase-02-vm.md`, `bytecode-vm/phase-07-stack-frames.md`, `bytecode-vm/phase-08-branching.md`
- Debug info: `bytecode-vm/phase-14-debug-info.md`, `bytecode-vm/phase-15-debug-info-defaults.md`
- Stdlib: `stdlib/phase-01-stdlib.md`, `stdlib/phase-02-stdlib-tests.md`
- Prelude: `stdlib/phase-07-prelude-binding.md`, `stdlib/phase-08-prelude-tests.md`
- CLI: `cli/phase-01-cli.md`, `cli/phase-02-cli-diagnostics.md`, `cli/phase-04-build-output.md`
- AST/typecheck dumps: `cli/phase-07-ast-typecheck-dumps.md`, `cli/phase-08-ast-typecheck-tests.md`, `cli/phase-09-json-dump-stability-tests.md`
