# Atlas — AI Agent Orientation

Start here. Read this file first. It takes 2 minutes and tells you everything you need to know before writing a single line of Atlas code or touching the compiler.

## What is Atlas?

**"TypeScript's module system and type annotations wrapped around Rust's runtime model."**

- **Surface syntax** → TypeScript. If TypeScript has an answer, Atlas uses it.
- **Return types** → `:` colon, not `->` arrow. `fn foo(): number` not `fn foo() -> number`.
- **Runtime model** → Rust. CoW collections, Result/Option, ownership annotations (invisible in everyday code).
- **Execution** → Compiler + VM only. There is NO interpreter. Every Atlas program compiles to bytecode, runs on the VM.
- **AI-first** → If it's hard for an AI to generate correct Atlas code, the language is wrong, not the AI.

## The 6 Things You Must Know

1. **No interpreter.** `atlas run foo.atlas` compiles then executes. Never say "the interpreter does X."
2. **Semicolons required.** Every statement ends with `;`. `f(x)` without `;` fails to parse.
3. **Collections are CoW.** `myMap.set(k, v)` returns a new map. You must capture the return value.
4. **`new Map<K,V>()`** is how you create a Map. Not `{}`, not `hashMapNew()` — `new Map<K,V>()`.
5. **Return types use `:` not `->`.** `fn add(x: number): number` — TypeScript style.
6. **Output is `console.log()`.** Never `print()` or `println()`.

## Files in This Directory

| File | What it covers |
|------|---------------|
| [`atlas-quickstart.md`](atlas-quickstart.md) | Variables, functions, arrays, structs, control flow — write Atlas in 5 min |
| [`common-patterns.md`](common-patterns.md) | File I/O, JSON, HashMap, math, Option/Result — idiomatic Atlas patterns |
| [`gotchas.md`](gotchas.md) | Most common AI mistakes and the correct alternatives |
| [`mental-model.md`](mental-model.md) | How Atlas thinks: TypeScript surface + Rust runtime, CoW, execution model |
| [`find-anything.md`](find-anything.md) | Where things live in docs/language/, docs/stdlib/, and crates/ |

## Quick Reference

**Output:**
```atlas
console.log("hello");
console.log(`value: ${x}`);
```

**Function with return type:**
```atlas
fn add(x: number, y: number): number { x + y }
```

**File I/O:**
```atlas
match file.readText("path.txt") {
    Ok(text) => console.log(text),
    Err(e) => console.log(`error: ${e}`),
}
file.writeText("out.txt", content);
```

**JSON:**
```atlas
let text = json.stringify(value);
let data = json.parse(text);
```

**Top-level code runs directly:**
```atlas
// This runs. fn main() does NOT auto-execute.
console.log("hello from top level");
```

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

## First Principle

**Code is truth. Docs may lag.** If a doc and the source disagree, the source wins. Use `pt decisions all` to see locked decisions before making architectural choices.
