# GATE 5: Update Database (Selective - CLI Only)

**Condition:** Quality gates passed

**CRITICAL:** Database is ONLY source. Use CLI commands to update specs/APIs/decisions.

---

## Tier 1: IMMEDIATE Updates (Do NOW via CLI)

**Update database immediately if ANY of these:**

1. **Spec changed:**
   - Run: `atlas-dev spec sync <file>` → Updates database
   - Grammar, types, semantics, runtime behavior changed
   - Example: `atlas-dev spec sync syntax.md`

2. **Architectural decision made:**
   - Run: `atlas-dev decision create -c COMP -t "title" --decision "..." --rationale "..."`
   - Only if: Chose between approaches AND affects future work
   - Stored in database immediately

3. **Breaking API change:**
   - Run: `atlas-dev api sync <file>` → Updates database
   - Changed function signature, removed function, changed behavior
   - Example: `atlas-dev api sync stdlib.md`

**Cost:** ~30 seconds per command

---

## Tier 2: BATCHED Updates (Queue for Later)

**Use CLI for batch operations:**

1. **New features added:**
   - Run: `atlas-dev feature create --name X --status InProgress`
   - Or: `atlas-dev feature sync <name>` to update from codebase

2. **When to process:**
   - Category complete (all stdlib phases done)
   - OR every 10-20 phases
   - OR at mini-polish checkpoints

---

## Tier 3: NEVER Update (Skip)

**DON'T update database for:**

1. **Implementation details** (refactors, optimizations, internal changes)
2. **Bug fixes** (unless spec ambiguity revealed)
3. **Code cleanup** (unless public API changed)

---

## Decision Tree (CLI Commands)

```
Spec changed? → YES → atlas-dev spec sync <file>
              → NO ↓

Architectural decision? → YES → atlas-dev decision create ...
                       → NO ↓

Breaking API change? → YES → atlas-dev api sync <file>
                    → NO ↓

New features? → YES → atlas-dev feature sync <name>
              → NO ↓

Implementation detail only? → YES → SKIP
                            → NO → Done
```

---

## Commands Reference

**Sync specs:**
```bash
atlas-dev spec sync syntax.md          # Update spec in database
atlas-dev spec sync --all               # Sync all specs
```

**Create decision:**
```bash
atlas-dev decision create \
  -c COMPONENT \
  -t "Decision title" \
  --decision "What we decided" \
  --rationale "Why we decided it"
```

**Sync APIs:**
```bash
atlas-dev api sync stdlib.md            # Update API doc in database
atlas-dev api sync --all                # Sync all APIs
```

**Sync features:**
```bash
atlas-dev feature sync <name>           # Update from codebase
```

---

**Next:** Done (or GATE 6 if structured development)
