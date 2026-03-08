# GATE 6: Session Handoff (Structured Development Only)

**Condition:** Structured development workflow, all gates passed

---

## Action (MANDATORY — do not skip)

Run `pt done` to record your session:

```bash
pt done <session-id> success "Phase XX complete: <what was done>" "Next: Phase XX+1 <what comes next>"
```

### Rules

1. **Session ID** → from your `go` command at session start (e.g., S-004)
2. **Outcome** → `success`, `partial`, `blocked`, or `failed`
3. **Summary** → what you actually accomplished (phase file name + key work)
4. **Next steps** → what the next agent should do

### Example (mid-block)

```bash
pt done S-004 success "Phase 03 complete: migrated collection variants to CoW" "Next: Phase 04 - implement shared type"
```

### Example (block complete)

```bash
pt done S-004 success "Phase 25 complete: Block 1 finished, all ACs met" "Next: Scaffold Block 2 after verifying V03_PLAN.md acceptance criteria"
```

---

**BLOCKING:** Required for structured development. The next agent runs `pt go`
and sees your handoff in the "Handoff" section. If you skip this, they have no context.

**Next:** GATE 7
