# Lightweight Concurrency Research: Document Index

**Research conducted:** March 13, 2026
**Scope:** Analysis of concurrent execution models across 7 major systems languages
**Atlas recommendation:** Hybrid work-stealing model (Tokio-based)

---

## Documents in This Research

### 1. CONCURRENCY_RESEARCH.md (Primary Report)
**32 KB, 12 sections, 1,200 lines**

Comprehensive deep-dive covering:
- **Section 1-7:** Detailed analysis of Go, Rust, Erlang, C#, Python, Lua, Java
- **Section 8:** Comparative analysis matrix
- **Section 9:** Five architecture options for non-thread-safe runtimes
- **Section 10:** Specific recommendations for Atlas
- **Section 11:** Key takeaways and universal principles
- **Section 12:** References with hyperlinks

**Read if you need:** Complete understanding of concurrency models and trade-offs.

### 2. CONCURRENCY_QUICK_REFERENCE.md (Decision Guide)
**14 KB, 450 lines, visual decision trees**

One-page comparisons and quick lookups:
- Comparison table (all 5 models side-by-side)
- Memory footprint visualization
- Decision tree (which model to choose?)
- Problem/solution matrix
- Implementation effort estimates
- FAQ section

**Read if you need:** Quick decision-making, visual overviews, implementation estimates.

### 3. CONCURRENCY_INDEX.md (This File)
Navigation and summary guide.

---

## Quick Navigation by Role

### For the Architect (you)
1. Read **CONCURRENCY_RESEARCH.md Section 10** (Atlas Recommendations) first
2. Scan **CONCURRENCY_QUICK_REFERENCE.md** for the decision matrix
3. Reference **CONCURRENCY_RESEARCH.md Sections 1-7** as needed for deep dives

### For the Developer
1. Start with **CONCURRENCY_QUICK_REFERENCE.md** (overview)
2. Read **CONCURRENCY_RESEARCH.md Section 9** (five options explained)
3. Reference **CONCURRENCY_RESEARCH.md Section 12** for implementation details

### For Code Review
1. Check **CONCURRENCY_RESEARCH.md Section 10** (recommendations) for what to look for
2. Use **CONCURRENCY_QUICK_REFERENCE.md Testing Checklist** for validation
3. Reference **CONCURRENCY_RESEARCH.md Sections 1-7** if specific language patterns needed

---

## Key Findings at a Glance

### The Recommendation: Hybrid Model with Work-Stealing

**Why?** It's the model used by:
- Go (goroutines)
- Rust (Tokio)
- Java (virtual threads)
- C# (ThreadPool)

**For Atlas specifically:**
- Adopts Tokio's existing multi-threaded executor
- Each thread gets a LocalSet (preserves VM isolation)
- Work-stealing provides load balancing across cores
- Scales from 1 to 1M+ concurrent tasks

### Implementation Roadmap

| Phase | Duration | Scope |
|-------|----------|-------|
| 1: Core | 2-3 weeks | Stop spawning OS threads; use Tokio LocalSet directly |
| 2: Isolation | 2-3 weeks | Per-thread VM context; proper task isolation |
| 3: I/O | 3-4 weeks | spawn_blocking for CPU-bound; integrate with stdlib I/O |
| 4: Testing | 2 weeks | Parity tests, stress tests, memory profiling |

**Total: 9-12 weeks** for full production-ready implementation.

### Memory Efficiency

**Current (suboptimal):**
- ~1 MB per spawned task (OS thread)
- Scales to ~100 concurrent tasks

**Recommended (hybrid):**
- ~100 bytes per task overhead
- Scales to ~1,000,000 concurrent tasks
- **10,000x improvement**

---

## Language-Specific Insights

### Go (Sections 1)
- **Key insight:** M:N scheduling isolates scheduler state per-thread
- **Memory:** 2 KB per goroutine
- **Relevant for Atlas:** Stack management, scheduling model

### Rust/Tokio (Section 2)
- **Key insight:** Send + 'static bounds enforce thread-safety without locks
- **Memory:** ~64 bytes per future (stackless)
- **Relevant for Atlas:** Exact model to adopt; proven at scale

### Erlang (Section 3)
- **Key insight:** Complete process isolation eliminates synchronization
- **Memory:** 2 KB per process
- **Relevant for Atlas:** Process model too heavy; useful for understanding isolation

### C# (Section 4)
- **Key insight:** ThreadPool + Task abstraction decouples logical work from OS threads
- **Memory:** Negligible for async tasks
- **Relevant for Atlas:** Task/thread distinction; work-stealing deques

### Python (Section 5)
- **Key insight:** Greenlets work without changing the runtime (monkey-patching)
- **Memory:** 100-1000 bytes per greenlet
- **Relevant for Atlas:** Alternative for single-threaded; GIL shows why isolation matters

### Lua (Section 6)
- **Key insight:** Coroutines need no scheduler at VM level (user-defined)
- **Memory:** Stack-based, variable
- **Relevant for Atlas:** Minimal approach; useful for understanding VM requirements

### Java (Section 7)
- **Key insight:** Virtual Threads provide M:N with transparent blocking I/O
- **Memory:** ~100 bytes per virtual thread
- **Relevant for Atlas:** Newest model; shows the future direction

---

## Architectural Decision Points

### Decision 1: Single-threaded vs Multi-threaded?
**Answer:** Multi-threaded (hybrid model)
- Single-threaded limits to 1 core
- Atlas will run on modern multi-core hardware

### Decision 2: Thread-safe VM or VM per task?
**Answer:** VM per thread (hybrid approach)
- VM per task: Too expensive (2 KB+ each)
- Shared + locks: Defeats purpose of threading
- Per-thread: Sweet spot (1 LocalSet per Tokio worker thread)

### Decision 3: Stack management?
**Answer:** Stackless futures (state machines)
- Stackful: More memory, easier to understand
- Stackless: Less memory, proven at scale (Rust, C#)
- Atlas uses bytecode VM (already context-switching), stackless is natural

### Decision 4: Preemptive vs cooperative scheduling?
**Answer:** Cooperative (async/await)
- Preemptive: More complex, race conditions
- Cooperative: Explicit yield points, safer, easier to reason about
- Atlas: async annotations make semantics clear

### Decision 5: How to handle blocking I/O?
**Answer:** spawn_blocking for user code
- Delegate to thread pool for CPU-bound / blocking I/O
- Keeps main executor free for async tasks
- Same as Tokio's proven approach

---

## Critical Risk Areas

### Risk 1: Parity Maintenance
**Problem:** Interpreter and VM must produce identical output when tasks run in different orders
**Mitigation:** Deterministic task scheduling in tests; battle tests catch race conditions

### Risk 2: Deadlock Potential
**Problem:** Users can deadlock with nested spawns or circular dependencies
**Mitigation:** Lint warnings; documentation; "async" annotation enforces intent

### Risk 3: Memory Overhead
**Problem:** Per-thread VM duplication
**Mitigation:** Measure and optimize; reference-counted bytecode; memory pooling

### Risk 4: Complexity
**Problem:** Hybrid model is more complex than event loop
**Mitigation:** Clear separation of concerns; LocalSet handles complexity

---

## How to Read This Research

### Path 1: Executive Summary (30 minutes)
1. CONCURRENCY_QUICK_REFERENCE.md (top section)
2. CONCURRENCY_RESEARCH.md (Sections 10-11)
3. Return to this index for decision points

### Path 2: Technical Deep-Dive (2 hours)
1. CONCURRENCY_RESEARCH.md Section 9 (five options)
2. CONCURRENCY_RESEARCH.md Sections 1-7 (language details)
3. CONCURRENCY_RESEARCH.md Section 10 (Atlas specifics)
4. CONCURRENCY_QUICK_REFERENCE.md (validate with comparison tables)

### Path 3: Implementation Prep (3 hours)
1. CONCURRENCY_RESEARCH.md Sections 9-10 (architecture options and recommendation)
2. CONCURRENCY_QUICK_REFERENCE.md (implementation effort, testing checklist)
3. CONCURRENCY_RESEARCH.md Sections 1-2 (Go and Rust patterns)
4. CONCURRENCY_RESEARCH.md Section 10 (detailed implementation phases)

### Path 4: Code Review (1 hour)
1. CONCURRENCY_QUICK_REFERENCE.md (decision matrix)
2. CONCURRENCY_RESEARCH.md Section 10 (what to look for)
3. CONCURRENCY_QUICK_REFERENCE.md (testing checklist)

---

## Key Statistics

### Current State (Atlas)
- **Task spawning:** OS thread per task
- **Memory per task:** ~1 MB
- **Max concurrent:** ~100 tasks
- **Scaling:** None (no work-stealing)

### Recommended State (Hybrid)
- **Task spawning:** Tokio work-stealing
- **Memory per task:** ~100 B
- **Max concurrent:** ~1,000,000 tasks
- **Scaling:** Linear with cores (4-16x on modern hardware)

### Implementation Cost
- **Design complexity:** Medium (Tokio handles most of it)
- **Development time:** 9-12 weeks
- **Risk level:** Medium (parity and deadlock concerns)
- **Expected payoff:** 10,000x scalability improvement

---

## Document Maintenance

These documents were generated on **March 13, 2026** based on:
- **Web research:** Go, Rust, Erlang, C#, Python, Lua, Java docs
- **Atlas source code:** Reading async_runtime/, runtime.rs, value.rs
- **Academic papers:** Work-stealing scheduler, VM concurrency papers
- **Production systems:** Go runtime, Tokio runtime, BEAM VM

**Update triggers:**
- Major changes to Atlas task spawning API → update Section 10
- New language versions with concurrency models → update Sections 1-7
- Benchmark results → update quick reference memory/scalability tables

---

## Questions This Research Answers

### For Architecture Decisions
- ✓ What concurrency models exist?
- ✓ How do they handle non-thread-safe runtimes?
- ✓ Which scales to 1M+ concurrent tasks?
- ✓ What's the memory cost per task?
- ✓ Which model suits Atlas best?

### For Implementation Planning
- ✓ How much work is full implementation?
- ✓ What are the phases?
- ✓ What are the key risks?
- ✓ How do we test it?
- ✓ What does the user-facing API look like?

### For Code Review
- ✓ What patterns should we look for?
- ✓ What tests are critical?
- ✓ What are common pitfalls?
- ✓ How do we verify parity?
- ✓ What's the performance baseline?

---

## Next Steps After Reading

### Immediate (This Week)
- [ ] Share with team for review
- [ ] Identify any gaps in recommendation
- [ ] Confirm resource availability (9-12 weeks)

### Short-term (This Month)
- [ ] Create detailed implementation plan (JIRA/pt tasks)
- [ ] Spike: Validate Tokio LocalSet approach with proof-of-concept
- [ ] Design user-facing API (async syntax)

### Medium-term (Next 3 Months)
- [ ] Implement Phase 1 (stop spawning OS threads)
- [ ] Implement Phase 2 (per-thread VM isolation)
- [ ] Start Phase 3 (I/O integration)

### Long-term (After Implementation)
- [ ] Benchmark against single-threaded event loop
- [ ] Compare memory usage to Go/Rust
- [ ] Publish performance report
- [ ] Document patterns in `.claude/memory/`

---

## Related Reading (Atlas Docs)

- `.claude/memory/MEMORY.md` — Project overview
- `crates/atlas-runtime/src/CLAUDE.md` — Runtime architecture
- `.claude/rules/atlas-vm.md` — VM quick reference
- `docs/language/async.md` — Async language design (current)

---

## Sources

All research documents include full references. See:
- CONCURRENCY_RESEARCH.md Section 12 (complete reference list with hyperlinks)
- CONCURRENCY_QUICK_REFERENCE.md (quick links section)

---

**Document version:** 1.0
**Status:** Complete research, ready for architecture review
**Confidence level:** High (all major languages covered, patterns consistent)
