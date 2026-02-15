# Phase-to-Implementation Mapping

| Phase Category | Key Implementation Files | Primary Guide Docs |
|----------------|-------------------------|-------------------|
| **Foundation** | `runtime/api.rs` | `02-core-types.md` |
| **Stdlib** | `stdlib/{string,array,math,json,io,types}.rs` | `13-stdlib.md` |
| **Bytecode-VM** | `{optimizer,profiler,debugger}/mod.rs` | `11-bytecode.md`, `12-vm.md` |
| **Frontend** | `diagnostics/formatter.rs`, `formatter/` | `03-lexer.md`, `04-parser.md` |
| **Typing** | `typechecker/inference.rs` | `07-typechecker.md` |
| **Interpreter** | `interpreter/debugger.rs` | `10-interpreter.md`, `14-repl.md` |
| **CLI** | `atlas-cli/src/commands/` | CLI framework |
| **LSP** | `atlas-lsp/src/handlers.rs` | `16-lsp.md` |
