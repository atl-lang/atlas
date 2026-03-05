# Atlas

## Philosophy
- **AI-first.** "What's best for AI?" is the decision tiebreaker.
- **No MVP.** Complete implementations only. Do it right once.
- **100% AI developed.** This project is built entirely by AI.

## Systems-Level Context (READ THIS)
Atlas is mid-conversion from "AI experiment" (v0.1-v0.2) to proper systems-level architecture (v0.3+).
Currently paused for battle-testing and hardening.

**Non-negotiables:**
- **Fix correctly, not temporarily.** Hacks create conversion debt. Correct fixes align with systems-level.
- **Partial implementations are intentional.** Some AST nodes exist but aren't wired up yet - this is scaffolding, not dead code.
- **Before deleting "incomplete" code:** Check git history + `atlas-track decisions`. Ask user if uncertain.
- **See `docs/SYSTEMS_LEVEL_STATUS.md`** for what's converted vs legacy vs scaffolded.

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

## See Something, File Something (MANDATORY)
**Never defer. Never assume "out of scope." File it.**

If you notice Atlas is missing something that Go/Rust/TypeScript had in v1.0:
1. **Do NOT defer** to a future version
2. **Do NOT assume** previous AI agents scoped it out correctly
3. **File an issue immediately:** `atlas-track add "Missing: X" P1 "reason"`
4. **Flag to user:** "I noticed Atlas lacks X. Most languages have this. Filed as issue."

**Examples of "should already exist":**
- Basic type system features (generics, traits, interfaces)
- Standard control flow
- Module/import system that works
- Error handling patterns

**The rule:** If AI has to work around something that should be built-in, that's a bug, not a feature request. File it.

Previous AI agents deferred too much. That stops now.

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

// Entry point (optional — top-level code also runs)
fn main() {
    print("Hello from main!");
}

// Traits
trait Greetable {
    fn greet(self: Greetable) -> string;
}
impl Greetable for Point {
    fn greet(self: Point) -> string { return "I am a point"; }
}
let greeting = p.greet();  // Trait method dispatch works

// Stdlib (camelCase, global — method syntax coming in H-065)
let arr2 = arrayPush(arr, 4);     // NOT push()
let length = len(arr);             // NOT arr.length()
let m: HashMap<string, number> = hashMapNew();
hashMapPut(m, "key", 42);          // NOT m.put()

// Template strings
let msg = `Hello {name}!`;         // {x} not ${x}

// File extension — both .atlas and .atl work
```

## Known Issues (check docs/known-issues.md)
- H-069: Closure global mutations (in progress)
- H-070: Trait system incomplete (self inference, default methods, trait objects)
- H-065: Stdlib needs method syntax (arr.push() vs arrayPush())

## Cross-Platform Testing
- Use `std::path::Path` APIs, not string manipulation for paths.
- Use `Path::is_absolute()`, not `starts_with('/')`.
- Normalize separators in test assertions: `path.replace('\\', "/")`.
- Platform-specific test paths: use `#[cfg(unix)]` / `#[cfg(windows)]` helpers.
