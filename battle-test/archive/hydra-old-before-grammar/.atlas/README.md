# Atlas Language Documentation

AI-friendly documentation for Atlas language features, limitations, and workarounds discovered during Hydra Atlas development.

## Structure

### 📖 [Syntax Rules](./syntax/)
Core language syntax patterns and gotchas:
- [basics.md](./syntax/basics.md) - Variables, functions, imports
- [match.md](./syntax/match.md) - Match expressions (critical!)
- [control-flow.md](./syntax/control-flow.md) - If, for, loops
- [types.md](./syntax/types.md) - Type system and Result

### 📚 [Standard Library](./stdlib/)
Available functions and their behavior:
- [strings.md](./stdlib/strings.md) - String operations
- [json.md](./stdlib/json.md) - JSON parsing/handling
- [files.md](./stdlib/files.md) - File I/O
- [arrays.md](./stdlib/arrays.md) - Array operations
- [missing.md](./stdlib/missing.md) - Functions that don't exist

### 🐛 [Issues & Workarounds](./issues/)
Known problems and solutions:
- [blockers.md](./issues/blockers.md) - Blocking issues
- [workarounds.md](./issues/workarounds.md) - How to work around limitations
- [type-system.md](./issues/type-system.md) - Type system quirks

### 💡 [Working Examples](./examples/)
Tested, working code snippets:
- [hello.atl](./examples/hello.atl) - Basic example
- [match-patterns.atl](./examples/match-patterns.atl) - Match expressions
- [strings.atl](./examples/strings.atl) - String operations
- [imports.atl](./examples/imports.atl) - Module system

---

**Last Updated**: 2026-03-03
**Atlas Version**: Unknown (CLI available)
**Project**: Hydra Atlas (MCP server supervisor port from Go)
