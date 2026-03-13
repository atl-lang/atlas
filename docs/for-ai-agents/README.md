# Atlas — AI Agent Orientation

Start here. Read this file first. It takes 2 minutes and tells you everything you need to know before writing a single line of Atlas code or touching the compiler.

## What is Atlas?

**"TypeScript's module system and type annotations wrapped around Rust's runtime model."**

- **Surface syntax** → TypeScript. If TypeScript has an answer, Atlas uses it.
- **Runtime model** → Rust. CoW collections, Result/Option, ownership annotations (invisible in everyday code).
- **Execution** → Compiler + VM only. There is NO interpreter. Every Atlas program compiles to bytecode, runs on the VM.
- **AI-first** → If it's hard for an AI to generate correct Atlas code, the language is wrong, not the AI.

## The 5 Things You Must Know

1. **No interpreter.** `atlas run foo.atlas` compiles then executes. Never say "the interpreter does X."
2. **Semicolons required.** Every statement ends with `;`. `f(x)` without `;` fails to parse.
3. **Collections are CoW.** `myMap.set(k, v)` returns a new map. You must capture the return value.
4. **`new Map<K,V>()`** is how you create a Map. Not `{}`, not `hashMapNew()` — `new Map<K,V>()`.
5. **Types use TypeScript syntax.** `<T extends Foo & Bar>`, `T[]`, `(T) => R`, `string | null`.

## Where Everything Lives

| What you need | Where to look |
|---------------|---------------|
| Language syntax | `docs/language/` |
| Stdlib API | `docs/stdlib/` |
| Architecture | `docs/architecture/` |
| Tooling (LSP, formatter, packages) | `docs/tooling/` |
| Compiler source | `crates/atlas-runtime/src/compiler/` |
| VM source | `crates/atlas-runtime/src/vm/mod.rs` |
| Stdlib source | `crates/atlas-runtime/src/stdlib/` |
| Type system | `crates/atlas-runtime/src/typechecker/` |
| All language decisions | `pt decisions all` (run in project root) |
| Open issues | `pt issues` |
| Session state | `pt go` |

## Quick Navigation

| Task | Read this |
|------|-----------|
| Write Atlas code | [`atlas-quickstart.md`](atlas-quickstart.md) |
| Understand the runtime model | [`mental-model.md`](mental-model.md) |
| Avoid common mistakes | [`gotchas.md`](gotchas.md) |
| Find an API / file / decision | [`find-anything.md`](find-anything.md) |
| Write idiomatic Atlas | [`common-patterns.md`](common-patterns.md) |

## First Principle

**Code is truth. Docs may lag.** If a doc and the source disagree, the source wins. Use `pt decisions all` to see locked decisions before making architectural choices.
