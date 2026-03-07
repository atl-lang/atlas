---
name: atlas-brainstorm
description: Atlas architecture brainstorm and design exploration. Use when discussing language design, evaluating tradeoffs, planning new features, or exploring approaches BEFORE committing to implementation. Always loads context first — never brainstorms blind.
---

# Atlas — Brainstorm & Design Exploration

**This skill exists because brainstorming without context = inventing things that were already decided.**
Every brainstorm session loads state first, then thinks. Never the other way around.

---

## Step 0: Load Context Before Thinking (MANDATORY — takes 30 seconds, saves hours)

```bash
atlas-track context                    # Current state: blocks, P0s, CI
atlas-track decisions all              # ALL standing decisions — D-001 through D-030+
atlas-track issues [component]         # Open issues relevant to what we're discussing
```

Read the decisions output completely. Every sentence. If you're about to brainstorm something
that a decision already covers — present the existing decision to the architect first.
Don't re-explore closed questions.

---

## The AI-First Filter (Apply to Every Idea)

**Single question:** "Does this make AI code generation easier or harder?"

- Harder → wrong direction, needs redesign or strong justification
- Easier → right direction, explore further
- Neutral → acceptable if it solves a real problem

Load for detailed grammar/syntax decisions: `.claude/skills/atlas/gates/ai-grammar-principles.md`

---

## Brainstorm Protocol

### Opening: Orient Before Exploring
1. State what question/problem we're exploring
2. Run Step 0 — load decisions + issues relevant to the topic
3. Surface any existing decisions that touch this area: "D-014 says X, so we can't do Y"
4. Only then: explore the open space

### During: Think in Tradeoffs, Not Conclusions
- Present 2-3 concrete options with explicit tradeoffs
- For each option: token cost, AI generation friction, implementation complexity
- Reference prior decisions when they constrain options
- Flag when an option would require overriding an existing decision (architect call)

### Closing: Capture Outcomes (Non-Negotiable)

Every brainstorm ends with one of these actions — never just ends:

**If a decision was reached:**
```bash
atlas-track add-decision "Title" <component> "Rule: what was decided" "Rationale: why this over alternatives"
```

**If work was identified:**
```bash
atlas-track add "Feature/bug title" P0|P1|P2 "what it is, why it matters, approach discussed"
```

**If the question is still open:**
```bash
atlas-track add "Open question: X" P2 "what we explored, what's still unclear, what info is needed to decide"
```

**If an approach/plan was decided (not just a decision — a concrete implementation direction):**
```bash
atlas-track plan add "Title of plan" "Approach: what will be built, how, key tradeoffs. Enough for a cold-start agent to understand the direction without re-exploring." "H-XXX" "D-XXX"
```
Plans (PL-XXX) capture implementation intent — timestamped with a git commit snapshot. They survive context drain and connect brainstorm outcomes to future work.

No brainstorm session ends with "let's think about this more" as the only output. That's lost.

**After brainstorm — also update `.atlas-handoff.md`:**
If the brainstorm produced a concrete next action or changed what the next agent should do, update the handoff file. Include: what was decided, what PL-XXX was created, what the next concrete step is.

---

## Component → Decision Map (Quick Reference)

| Topic | Component filter | Key decisions to check |
|-------|-----------------|------------------------|
| Syntax, grammar, new operators | `parser` | D-006 through D-013 |
| Type system, inference | `typechecker` | D-003, D-010, D-015, D-017, D-018 |
| Collections, CoW semantics | `runtime` | D-028, D-029 |
| Async model | `runtime` | D-030 |
| Stdlib API design, method vs fn | `stdlib` | D-021 |
| Ownership, borrow semantics | `runtime` | D-020, D-022 |
| Any language-level design | `all` | `atlas-track decisions all` |

---

## AI Continuity — Non-Negotiable

**Never narrate outcomes — capture them:**
- ❌ "We should think about X" → `atlas-track add "Open question: X" P2 "context"`
- ❌ "That's interesting to consider" → file it or drop it
- ✅ Every idea worth keeping gets filed. Everything else is noise.

**Block tracking (if brainstorm leads to a new block being scoped):**
```bash
# After architect approves a new feature block:
atlas-track add-decision "Block N: <name>" infra "Scope: <what it covers>" "Approved in S-XXX brainstorm"
# Then scaffold: trigger atlas-blocks skill
```

---

## What This Skill Is NOT

- Not for implementation — when you have a direction, switch to `atlas-blocks` or `atlas-bugfix`
- Not for battle testing — that's `atlas-battle`
- Not a substitute for reading the spec — always check `docs/language/` for what's already defined
