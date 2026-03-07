# Phase 13: Battle Tests

## Dependencies

**Required:** Phase 12 complete (parity sweep passed — zero known divergences)

**Verification:**
```bash
ls battle-tests/async-*/
atlas run battle-tests/async-basic/01_simple_async.atl
```

**If missing:** Parity must be confirmed before battle tests — a parity bug discovered during battle testing is a Phase 12 failure, not a Phase 13 issue.

---

## Objective

Validate async/await against real-world Atlas programs that unit tests do not cover: concurrent pipelines, producer/consumer patterns, error recovery under async, and stress patterns. Catch what isolated unit tests miss.

---

## Files

**Create:** `battle-tests/async-basic/` (~4 Atlas programs)
**Create:** `battle-tests/async-concurrency/` (~4 Atlas programs)
**Create:** `battle-tests/async-error-recovery/` (~3 Atlas programs)
**Create:** `battle-tests/async-stdlib/` (~3 Atlas programs)
**Update:** `crates/atlas-runtime/tests/` — test harness to run battle test files (~40 lines Rust)

**Total new code:** ~500 lines Atlas programs, ~40 lines Rust harness
**Total tests:** 14 Atlas programs (each counts as a battle test)

---

## Dependencies (Components)

- Both engines — interpreter and VM (Phases 09, 10)
- Stdlib async (Phase 11)
- Parity baseline (Phase 12)
- `atlas run` CLI (existing)

---

## Implementation Notes

**Key patterns to analyze:**
- Review `compiler-quality/battle-testing.md` in auto-memory for the battle test structure and harness used in earlier blocks
- Check existing `battle-tests/` directory structure to match the naming and layout convention

**Battle test program requirements:**
- Each program must be self-contained and runnable via `atlas run`
- Each program has a documented expected output
- Programs must exercise features that unit tests do not: multi-step async pipelines, concurrent task collections, real file I/O, error chains

**async-basic programs** (4 programs):
- A simple fetch simulation: async fn that sleeps briefly then returns a constructed string
- Sequential pipeline: three async fns chained with await, each transforming the previous result
- Async fn inside a loop: awaiting results collected into an array
- Async fn with struct method calls

**async-concurrency programs** (4 programs):
- Parallel tasks: spawn three tasks simultaneously, collect all results with `all`
- Race pattern: two tasks with different simulated delays, race to get the first result
- Producer/consumer: one async fn produces values into a channel, another consumes
- Fan-out/fan-in: one input spawns N tasks, results aggregated

**async-error-recovery programs** (3 programs):
- Retry logic: async fn that fails on first attempt, retries with exponential backoff using timeout
- Fallback: race two futures, use fallback value if both fail
- Partial failure: `all` with one failing future — correct error surfacing

**async-stdlib programs** (3 programs):
- File pipeline: read a file async, transform contents, write to a new file async
- Concurrent file reads: read multiple files in parallel using spawn + all
- HTTP simulation: fetch a URL (local mock), parse the response, print a field

**Error handling:**
- All programs must handle errors gracefully — no panics, no unhandled RuntimeError leaks

**Integration points:**
- Uses: all async features from Phases 01–12
- Run via `atlas run` CLI and via Rust test harness

---

## Tests (TDD Approach)

Each Atlas battle test program is one test. The Rust harness:
1. Runs the program through the interpreter
2. Runs the program through the VM
3. Asserts the output matches the documented expected output
4. Asserts interpreter output equals VM output

**Minimum test count:** 14 battle programs (28 engine runs total)

---

## Acceptance Criteria

- ✅ 14 battle test Atlas programs authored across four categories
- ✅ All programs produce correct expected output in interpreter
- ✅ All programs produce correct expected output in VM
- ✅ Interpreter and VM output identical for all programs
- ✅ No panics or unhandled errors in any program
- ✅ `atlas run` executes all programs without crash
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030 (concurrent battle tests validate multi-thread runtime)
**Specifications:** auto-memory `compiler-quality/battle-testing.md`
**Related phases:** Phase 12 (parity baseline), Phase 14 (LSP), Phase 15 (final AC gate)
