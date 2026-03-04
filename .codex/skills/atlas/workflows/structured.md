# Structured Development Workflow

**When to use:** Following documented development plan (check `atlas-track sitrep` for current phase)

**Approach:** Gate-based, implementation-driven (NOT strict TDD)

---

## Gates Used

| Gate | Action |
|------|--------|
| -1 | Sanity check + spec verification |
| 0 | Read docs + check dependencies |
| 1 | Size estimation + foundation check |
| 2 | Implement + test (implementation-driven) |
| 3 | Verify interpreter/VM parity |
| 4 | Quality gates (clippy, fmt) |
| 5 | Doc update (selective) |
| 6 | Session handoff (`atlas-track done`) |
| 7 | Memory check |

**GATE 0 additions:** Read complete development plan from phase file (objective, files, dependencies, implementation details, tests, acceptance criteria). Source: `atlas-track sitrep` shows current block/phase.

**GATE 6 mandatory:** Run `atlas-track done` to record handoff for next session.

---

## Emergency Procedures

- **Tests fail:** Debug systematically, check parity, max 2 retry attempts
- **Dependencies missing:** STOP at GATE 0, report what's missing
- **Parity fails:** CRITICAL — debug both engines, don't proceed
- **Quality gates fail:** Fix at GATE 4, max 2 retry attempts
