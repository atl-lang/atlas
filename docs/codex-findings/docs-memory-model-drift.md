# Documentation Drift: Memory Model and Value Representation

Target: docs/specification + guides + JIT docs
Severity: High
Status: Open

## Finding 1: Language semantics doc still describes Arc<Mutex> aliasing

Evidence:
- /Users/proxikal/dev/projects/atlas/docs/specification/language-semantics.md:61-86

What/Why:
- The semantics doc states arrays are `Arc<Mutex<Vec<Value>>>` with reference aliasing. This contradicts the CoW value model implemented in v0.3 (ValueArray = Arc<Vec<Value>> with write-back), and conflicts with current tests and `CLAUDE.md`.

Impact:
- AI agents will generate and reason about code using the wrong aliasing semantics, leading to incorrect assumptions about mutation visibility.

Recommendation:
- Update semantics to match CoW write-back rules and explicitly document when mutations are visible across aliases.

---

## Finding 2: Interpreter status doc still describes Arc<Mutex> values

Evidence:
- /Users/proxikal/dev/projects/atlas/docs/interpreter-status.md:69-75

What/Why:
- Value representation section is outdated and conflicts with current runtime implementation.

Impact:
- Contributors may introduce regression back toward old model or misunderstand performance characteristics.

Recommendation:
- Align with `docs/specification/memory-model.md` and runtime `value.rs`.

---

## Finding 3: Embedding guide shows outdated `Value` enum and examples

Evidence:
- /Users/proxikal/dev/projects/atlas/docs/embedding-guide.md:111-149

What/Why:
- The guide uses `Array(Arc<Mutex<Vec<Value>>>)` and `Object(Arc<Mutex<HashMap<...>>>)`, which are no longer valid.

Impact:
- External embedding users will fail to compile examples or use incorrect memory assumptions.

Recommendation:
- Update `Value` examples to match current API and add migration notes for v0.2 → v0.3.

---

## Finding 4: JIT status doc references old Arc<Mutex> model

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/JIT_STATUS.md:67-69

What/Why:
- JIT status mentions arrays as `Arc<Mutex<Vec<Value>>>`, which is outdated.

Impact:
- Confuses JIT contributors and AI agents when evaluating feasibility of opcode support.

Recommendation:
- Update JIT status to reference CoW array types and the current Value model.
