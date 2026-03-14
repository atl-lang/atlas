# Find Anything in Atlas

Quick reference for locating source, docs, decisions, and tooling.

## Source Code

| What | Where |
|------|-------|
| Lexer / tokenizer | `crates/atlas-runtime/src/lexer/` |
| Parser | `crates/atlas-runtime/src/parser/` |
| AST node definitions | `crates/atlas-runtime/src/ast.rs` |
| All token kinds | `crates/atlas-runtime/src/token.rs` |
| Type checker | `crates/atlas-runtime/src/typechecker/` |
| Bytecode compiler | `crates/atlas-runtime/src/compiler/` |
| VM execution engine | `crates/atlas-runtime/src/vm/mod.rs` |
| Bytecode definitions | `crates/atlas-runtime/src/bytecode/` |
| Value enum (all runtime types) | `crates/atlas-runtime/src/value.rs` |
| Type system structures | `crates/atlas-runtime/src/types.rs` |
| Stdlib (all modules) | `crates/atlas-runtime/src/stdlib/` |
| Stdlib registration | `crates/atlas-runtime/src/stdlib/mod.rs` |
| Method dispatch | `crates/atlas-runtime/src/method_dispatch.rs` |
| Module loader | `crates/atlas-runtime/src/module_loader.rs` |
| Async runtime | `crates/atlas-runtime/src/async_vm.rs` |
| FFI | `crates/atlas-runtime/src/ffi/` |
| Security / sandbox | `crates/atlas-runtime/src/security/` |
| LSP server | `crates/atlas-lsp/src/` |
| Formatter | `crates/atlas-formatter/src/` |
| Package manager | `crates/atlas-package/src/` |
| CLI | `crates/atlas-cli/src/` |
| Runtime manager | `crates/atlas-runtime/src/runtime.rs` |
| Diagnostics | `crates/atlas-runtime/src/diagnostic/` |
| Error codes | `crates/atlas-runtime/src/diagnostic/error_codes.rs` |
| Optimizer | `crates/atlas-runtime/src/optimizer/` |

## Documentation

| Topic | Where |
|-------|-------|
| Language overview | `docs/README.md` |
| Syntax quick reference | `docs/language/syntax-quickref.md` |
| Type system | `docs/language/types.md` |
| Grammar (EBNF) | `docs/language/grammar.md` |
| Functions | `docs/language/functions.md` |
| Control flow | `docs/language/control-flow.md` |
| Pattern matching | `docs/language/pattern-matching.md` |
| Modules | `docs/language/modules.md` |
| Structs and enums | `docs/language/structs-enums.md` |
| Traits | `docs/language/traits.md` |
| Async/concurrency | `docs/language/async.md` |
| Closures | `docs/language/closures.md` |
| Error handling | `docs/language/errors.md` |
| Ownership / CoW | `docs/language/ownership.md` |
| Generics | `docs/language/generics.md` |
| Visibility / pub | `docs/language/visibility.md` |
| FFI | `docs/language/ffi.md` |
| Security model | `docs/language/security.md` |
| Build system | `docs/language/build.md` |
| Stdlib index | `docs/stdlib/index.md` |
| Array | `docs/stdlib/array.md` |
| String | `docs/stdlib/string.md` |
| Math | `docs/stdlib/math.md` |
| Map / Set / Queue / Stack | `docs/stdlib/collections/` |
| File system | `docs/stdlib/file.md` |
| I/O (read/write/append) | `docs/stdlib/io.md` |
| Path | `docs/stdlib/path.md` |
| HTTP | `docs/stdlib/http.md` |
| Net (TCP/UDP/TLS) | `docs/stdlib/net.md` |
| WebSocket | `docs/stdlib/websocket.md` |
| Async primitives | `docs/stdlib/async.md` |
| Sync primitives | `docs/stdlib/sync.md` |
| Process | `docs/stdlib/process.md` |
| JSON | `docs/stdlib/json.md` |
| Encoding | `docs/stdlib/encoding.md` |
| Compression | `docs/stdlib/compression.md` |
| DateTime | `docs/stdlib/datetime.md` |
| Regex | `docs/stdlib/regex.md` |
| Reflect | `docs/stdlib/reflect.md` |
| SQLite | `docs/stdlib/sqlite.md` |
| Crypto | `docs/stdlib/crypto.md` |
| Console | `docs/stdlib/console.md` |
| Compiler pipeline | `docs/architecture/compiler-pipeline.md` |
| VM architecture | `docs/architecture/vm.md` |
| Concurrency architecture | `docs/architecture/concurrency/` |
| LSP capabilities | `docs/tooling/lsp.md` |
| Package manager | `docs/tooling/package-manager.md` |
| Formatter | `docs/tooling/formatter.md` |
| CLI reference | `docs/cli.md` |
| Testing | `docs/testing.md` |
| AI design principles | `docs/AI-DESIGN-PRINCIPLES.md` |
| AI generation notes (stdlib) | `docs/stdlib/AI-GENERATION-NOTES.md` |
| Method naming conventions | `docs/stdlib/METHOD-CONVENTIONS.md` |

## Project Tracking

```bash
pt go                    # session start — state, P0s, block progress
pt decisions all         # ALL language decisions (D-001 through D-060+)
pt decisions <component> # decisions for: parser|typechecker|vm|stdlib|runtime|lsp
pt decision D-XXX        # full detail on one decision
pt issues                # open issues
pt issues P0             # P0 blockers only
pt in-progress           # what's currently claimed
pt blocks                # block progress summary
pt ci-status             # last CI run results
```

## Finding a Specific Thing

**"What tokens/keywords exist?"**
→ `crates/atlas-runtime/src/token.rs` — `TokenKind` enum

**"What AST nodes exist?"**
→ `crates/atlas-runtime/src/ast.rs` — `Expr`, `Stmt`, `TypeRef`, `Pattern` enums

**"What opcodes does the VM support?"**
→ `docs/architecture/vm.md` — full opcode table
→ `crates/atlas-runtime/src/bytecode/` — source

**"What stdlib functions are registered?"**
→ `crates/atlas-runtime/src/stdlib/mod.rs` — `builtin_registry()` + main registration block

**"What Value variants exist at runtime?"**
→ `crates/atlas-runtime/src/value.rs` — `Value` enum

**"Why was X designed this way?"**
→ `pt decisions all` and look for the relevant D-XXX decision

**"What's the error code for X?"**
→ `crates/atlas-runtime/src/diagnostic/error_codes.rs`

**"Where do I add tests for X?"**
→ `crates/atlas-runtime/src/CLAUDE.md` — test file routing table

**"Where is basic file read/write?"**
→ `docs/stdlib/io.md` — `io.readText`, `io.writeText`, `io.appendText`, `io.exists`
→ `docs/stdlib/file.md` — directory ops, metadata, symlinks, temp files

**"What is the correct file namespace?"**
→ `file.readText`, `file.writeText`, `file.exists` — NOT `fs.readFile`
→ See `docs/stdlib/file.md` and `docs/stdlib/io.md`
