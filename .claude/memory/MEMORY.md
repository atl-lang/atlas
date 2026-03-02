# Atlas Memory Index

**Auto-loads every message. LEAN only.**

## Read On-Demand
| File | When |
|------|------|
| `.claude/lazy/git.md` | Git operations (commit, push, PR, branch management) |
| `.claude/lazy/architecture.md` | GATE 0 (file size limits), adding code to large files, subagent decisions |
| `.claude/lazy/comms.md` | Writing PRs, commits, docs, changelogs |
| `domain-prereqs.md` | Before writing ANY code touching AST/Type/Value |
| `patterns.md` | Implementing features (collections, stdlib, trait dispatch, frontend API) |
| `testing-patterns.md` | Full detail beyond what rule file covers |
| `decisions/{domain}.md` | Architectural context before any domain decision |

## Decisions Index
`runtime` · `language` · `stdlib` · `cli` · `typechecker` · `vm` · `workflow`
- runtime: Memory model LOCKED (CoW, own/borrow/shared, DR-B01/B02)
- cli: Permissions CRITICAL — NO permission flags ever
- workflow: File size limits, subagent policy, branch rules (DR-W01–W06)

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

## CI Facts
- `strict_required_status_checks_policy=true` — no bypass
- Codecov via GitHub App, not token
- Bench regression hard-fails at 115%
- **PR BEHIND main = invalid CI run** — auto-merge won't fire. Rebase with `git merge origin/main && git push` (not `gh api update-branch`).

## Doc Auditor
Run `atlas-doc-auditor` after every block completion (GATE 7).
Audits: CLAUDE.md files + `.claude/rules/*.md` + `.claude/memory/*.md` + `.claude/memory/decisions/*.md`
224 lines — approved exception (global 150-line limit does not apply; atlas-specific 6-domain auditor).

## CodeRabbit Pre-Push (2026-03-01)
MANDATORY before any batch push to remote. Task Haiku agent (background):
`coderabbit review --base main --plain` in ~/dev/projects/atlas
Review findings before pushing. Documented in git-workflow.md Step 0.
