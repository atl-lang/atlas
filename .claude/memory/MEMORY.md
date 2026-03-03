# Atlas Memory Index

**Auto-loads every message. LEAN only.**

## Read On-Demand
| File | When |
|------|------|
| `.claude/lazy/git.md` | Git operations (commit, push, PR, branch management) |
| `.claude/lazy/architecture.md` | GATE 0 (file size limits), adding code to large files, subagent decisions |
| `.claude/lazy/comms.md` | Writing PRs, commits, docs, changelogs |
| `.claude/lazy/tracking-db.md` | SQL queries, issue management, session tracking schema |
| `domain-prereqs.md` | Before writing ANY code touching AST/Type/Value |
| `patterns.md` | Implementing features (collections, stdlib, trait dispatch, frontend API) |
| `testing-patterns.md` | Full detail beyond what rule file covers |
| `patterns/{domain}.md` | Implementation patterns (P-XXX format) |

## Decisions
**Source of truth:** `atlas-track decisions` (D-XXX format)
- Run `atlas-track decisions` to see all decisions
- Extended rationale: `docs/language-design/rationale/`
- Implementation patterns: `.claude/memory/patterns/`

**Guardian enforcement:** If adding a decision with syntax rules, you MUST also add pattern to `~/.claude/hooks/atlas/decision-patterns.json`. Pre-write hook blocks violations automatically.

## P0 BLOCKERS (DO THESE FIRST)
**NOTHING else ships until these are done. Added 2026-03-03.**

1. **Struct/Enum Types** — User-defined types. Currently can only use primitives + HashMap.
   - Struct declaration: `struct User { name: string, age: number }`
   - Struct instantiation: `User {name: "Alice", age: 30}`
   - Enum with variants: `enum Status { Active, Pending(string) }`
   - Pattern matching on user types
   - Trait impl for user types

2. **Object Literal Syntax** — `{key: value}` syntax for inline structured data.
   - Every AI model expects this syntax
   - Current workaround (`hashMapNew` + `hashMapPut`) is unacceptable
   - Critical for "AI-first" claim in PRD

**Why P0:** These were deferred too long. Atlas has traits, ownership, CoW, closures, type inference — but no way to define domain types. That's backwards.

## Status
`STATUS.md` in repo. `ROADMAP.md` for long-term direction (systems language, no GC, AI-gen-friendly).

## Recent Blocks
Block 5 (2026-02-26) — Type inference: `FunctionDecl.return_type: Option<TypeRef>`, `infer_return_type()`, AT3050/AT3051/AT3052, LSP inlay hints
Block 4 (2026-02-23) — AnonFn + HOFs: `fn(p) { body }` and `(p) => expr`, free function syntax `map(arr, fn...)`, capture semantics (AT3040)

## Pattern-Triggered Rules (Auto-Load on File Match)
All rules use `paths:` frontmatter — load only when touching matching files:
- **AST:** `.claude/rules/atlas-ast.md` — ast.rs, parser/, typechecker/, binder.rs
- **CI:** `.claude/rules/atlas-ci.md` — .github/workflows/**, codecov.yml, deny.toml
- **Interpreter:** `.claude/rules/atlas-interpreter.md` — interpreter/**
- **Parity:** `.claude/rules/atlas-parity.md` — interpreter/**, vm/**, compiler/**
- **Syntax:** `.claude/rules/atlas-syntax.md` — tests/**, fuzz/**
- **Testing:** `.claude/rules/atlas-testing.md` — **/tests/**
- **Typechecker:** `.claude/rules/atlas-typechecker.md` — typechecker/**, types.rs
- **VM:** `.claude/rules/atlas-vm.md` — vm/**, compiler/**, bytecode/**

## Local-First CI
**Canonical:** `.claude/lazy/git.md` | **Tracking:** `.claude/memory/local-ci.md`

## Doc Auditor
Run `atlas-doc-auditor` after every block (GATE 7). Audits all CLAUDE.md, rules, memory, decisions. 224 lines — approved exception.
