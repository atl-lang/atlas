# Atlas

## Philosophy
- **AI-first.** "What's best for AI?" is the decision tiebreaker.
- **No MVP.** Complete implementations only. Do it right once.
- **100% AI developed.** This project is built entirely by AI.

## Source of Truth
- **Code is law.** The codebase is the only source of truth.
- **Docs may be wrong.** If docs contradict code, docs are wrong.
- **Test against reality.** Run `atlas check` and `atlas run` to verify claims.
- **See `docs/`** for accurate documentation generated from codebase analysis.
- **Old docs archived.** `docs-archive/` contains stale historical docs - do not use.

## Guardian Protocol
- **Verify before agreeing.** User expresses doubt? Check the facts first, then state confidently.
- **Protect atlas from everyone.** User confusion, AI shortcuts, bad ideas—all threats.
- **User is architect, not infallible.** Explain why something is wrong. User makes final call.
- **Pushback on scope creep.** If user asks for tooling/infra/enhancements while P0 issues exist, say: "We have X P0 blockers. Should we fix those first, or is this more urgent?" Don't build nice-to-haves when the language is broken.

## Git Process (Local-First v2)
- **Local CI first.** All validation via `cargo fmt/clippy/nextest` + `coderabbit` CLI locally.
- **Batch pushes.** Commits accumulate on local main. Push every 168 hours.
- **No PRs for fixes.** Direct push to main after local CI passes. PRs only for major blocks.
- **Single workspace:** `~/dev/projects/atlas/` — no other worktrees.
- **See `.claude/lazy/git.md`** for full local-first workflow.

## Quick Check (every fix)
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo nextest run --workspace
```
Full CI commands and batch tracking: `.claude/lazy/git.md`

## Session Start (MANDATORY)
```bash
atlas-track go opus   # or sonnet/haiku — returns sitrep, handoff, P0s, stale issues
```
Act on what you see: stale issues need `fix` or `abandon`, P0 blockers before block work.

## Atlas Quick Reference (VERIFIED)

```atlas
// Variables
let x = 42;
let mut y = 0;

// Types
let n: number = 42;
let s: string = "hello";
let b: bool = true;
let arr: number[] = [1, 2, 3];

// Structs
struct Point { x: number, y: number }
let p = Point { x: 1, y: 2 };
print(p.x);  // Works as of H-066 fix

// Functions
fn add(a: number, b: number) -> number { a + b }

// Stdlib (camelCase, global)
let arr2 = arrayPush(arr, 4);     // NOT push()
let length = len(arr);             // NOT arr.length()
let m: HashMap<string, number> = hashMapNew();
hashMapPut(m, "key", 42);          // NOT m.put()

// Template strings
let msg = `Hello {name}!`;         // {x} not ${x}

// File extension
// Use .atlas for execution and tests; .atl may parse but runtime support is unreliable
```

## Known Issues (check docs/known-issues.md)
- H-063: Multi-file imports broken
- H-069: Closure global mutations (in progress)

## Cross-Platform Testing
- Use `std::path::Path` APIs, not string manipulation for paths.
- Use `Path::is_absolute()`, not `starts_with('/')`.
- Normalize separators in test assertions: `path.replace('\\', "/")`.
- Platform-specific test paths: use `#[cfg(unix)]` / `#[cfg(windows)]` helpers.
