# pt Workflow — Complete Reference

**Live DB:** `~/.project-tracker/atlas/tracking.db` (via `pt` commands only)
**Dead file:** `tracking/atlas.db` in repo — ignore, never query directly

---

## Session

```bash
pt go                                        # START: sitrep, session ID, handoff, P0s, CI, block
pt context                                   # mid-session orientation (no session side-effects)
pt in-progress                               # what's claimed right now
pt done S-XXX success "what was done" "what next agent does first (specific enough to act cold)"
```

`pt go` shows `── Next Action ──` from last session — read it first.
`pt done` arg 3 = backward (what was done, issue IDs + root causes). Arg 4 = forward (next agent's first action, specific file/function if known).

---

## Issues

```bash
pt issues [P0-P3] [component|limit]         # list open + in_progress (default 5, max 25)
                                             # pt issues P3         → all P3s (up to 15)
                                             # pt issues vm         → component filter
                                             # pt issues P2 vm      → priority + component
                                             # pt issues P1 10      → priority + custom limit
pt issue H-XXX                              # full detail
pt add "Title" P0|P1|P2|P3 "problem"        # create → returns H-XXX
pt claim H-XXX                              # lock before starting (prevents duplicates)
pt fix H-XXX "root cause" "fix" "scope-audit"  # close — ALL 4 ARGS REQUIRED
pt fix-batch H-001,H-002 "cause" "fix" "scope" # close multiple same cause
pt next                                     # smart triage: groups chains, surfaces delete-first
pt search "keyword"                         # search title + problem
pt update H-XXX priority P0                 # update: priority|component|title|problem
pt abandon H-XXX "reason"                   # release in_progress → open
pt reopen H-XXX                             # reopen resolved
pt link H-001 blocks H-002                  # link: blocks|blocked-by|related
pt note S-XXX "note text"                   # mid-session progress note (insurance against interruption)
```

Always `pt claim` before starting. Always `pt fix` immediately after verifying — never batch at session end.

---

## Decisions

```bash
pt decisions CORE                           # 17 CORE-tagged decisions — run at session start
pt decisions all                            # all decisions, CORE listed first
pt decisions [component]                    # filter: parser|typechecker|runtime|stdlib|infra
pt decision D-XXX                           # full detail  ← NOT 'pt issue D-XXX'
pt check-decision "keyword"                 # search before implementing anything
pt add-decision "Title" comp "Rule: ..." "Rationale: ..."
pt update-decision D-XXX rule "new text"    # amend: rule|rationale|title|component|consequences
pt supersede D-XXX D-YYY                    # D-XXX superseded by D-YYY
pt deprecate D-XXX "reason"
```

Decision component map:
| Changing | Component |
|----------|-----------|
| Parser, grammar, syntax | `parser` |
| Type inference, checking | `typechecker` |
| VM bytecode, opcodes | `vm` |
| Stdlib functions, builtins | `stdlib` |
| Runtime, values, memory, CoW | `runtime` |
| LSP server | `lsp` |
| CI, tooling, infra | `infra` |

---

## Blocks + Phases

```bash
pt blocks                                   # all blocks + status
pt block B<N>                               # detail + AC + phase list
pt block-add B<N> "Title" "AC"             # CREATE BLOCK FIRST before any phase-add
pt block-update B<N> name "New Title"       # update: name|ac|blockers|notes|status
pt block-delete B<N>                        # delete block + all phases
pt complete-block B<N> "summary"            # mark block complete

pt phases B<N>                              # list all phases
pt phase B<N>-P<XX>                         # full phase detail (all fields)
pt phase-start B<N>-P<XX>                   # mark in_progress (= pt claim for phases)
pt phase-done B<N>-P<XX> "outcome"          # MANDATORY after every phase commit
pt phase-skip B<N>-P<XX> "reason"
pt phase-delete B<N>-P<XX>
pt phase-update B<N>-P<XX> title "New"      # update single field: title|description|status|do|dont|verify|ac
```

### ⚠️ Scaffolding — ALWAYS use scaffold-phases, never sequential phase-add

`pt phase-add` only sets title + description. `scaffold-phases` sets ALL 9 fields in one transaction. Always use `scaffold-phases` when creating a block.

**Scaffolding order:**
1. `pt block-add B<N> "Title" "Acceptance Criteria"`
2. Pipe full JSON to `pt scaffold-phases B<N>` (one call, all phases, all fields)
3. `pt phases B<N>` to verify

```bash
# WRONG — loses 7 fields per phase, requires N sequential calls
pt phase-add B46 "Parser changes" "Update grammar"
pt phase-add B46 "Type checker" "Add inference"

# RIGHT — all phases, all fields, single transaction
echo '[
  {
    "title": "Parser changes",
    "description": "What this phase delivers and why",
    "deps": "B<N>-P<XX>",
    "files": "crates/atlas-runtime/src/parser/mod.rs, crates/atlas-runtime/src/token.rs",
    "do": "What agent MUST do — specific actions",
    "dont": "What agent must NOT do — guardrails",
    "verify": "cargo check -p atlas-runtime",
    "ac": "Acceptance criteria — what done looks like",
    "refs": "D-XXX, H-XXX"
  },
  {
    "title": "Type checker integration",
    "description": "...",
    "deps": "B<N>-P01",
    "files": "crates/atlas-runtime/src/typechecker/expr.rs",
    "do": "...",
    "dont": "...",
    "verify": "cargo check -p atlas-runtime",
    "ac": "...",
    "refs": "D-XXX"
  }
]' | pt scaffold-phases B46
```

### Phase field reference

| Field | Purpose | AI Continuity Value |
|-------|---------|---------------------|
| `title` | One-line phase name | Agent knows what the phase is |
| `description` | What this phase delivers and why | Agent understands scope |
| `deps` | Phase IDs this depends on (e.g. `B46-P01`) | Agent knows ordering |
| `files` | Exact file paths to touch | Agent knows where to work |
| `do` | Explicit required actions | Agent has a checklist |
| `dont` | Guardrails / forbidden approaches | Agent avoids known pitfalls |
| `verify` | Command to run to verify success | Agent knows definition of done |
| `ac` | Acceptance criteria | Gate 6 / phase-done evidence |
| `refs` | Related decisions + issues (D-XXX, H-XXX) | Agent has full context |

Skipping `pt phase-done` = next agent re-derives state from scratch.
Empty fields = next agent guesses = errors and re-work.

---

## CI

```bash
pt ci-status                                # last run: status, failed tests (first 20)
pt run-ci                                   # trigger full suite on-demand
pt mark-ci-pass "reason"                    # after resolving — never leave stale FAIL in sitrep
```

CI failures = P0 blocker. Fix before any new work.

---

## Brainstorm Outcomes

Every brainstorm ends with one of these — never just ends:
```bash
pt add-decision "Title" comp "Rule: ..." "Rationale: ..."    # decision reached
pt add "Title" P0|P1|P2 "context"                            # work identified
pt add "Open question: X" P2 "what's unclear"
pt plan add "Title" "Approach: ..." "H-XXX" "D-XXX"          # concrete implementation plan
```

---

## Doc Drift

Stop hook shows DOC DRIFT ALERT → fire `atlas-doc-patch` agent (Haiku, ~1-2 min).
Stop hook shows DOC EDITS UNCOMMITTED → already ran → `git add docs/ && git commit`.
Never leave unresolved across sessions.

---

## Maintenance

```bash
pt health
pt gc
pt gc --aggressive
```
