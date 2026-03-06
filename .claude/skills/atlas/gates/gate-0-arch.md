# GATE 0 — Architecture Health Check + Baseline Sizing (Implementation Only)

**Load when:** About to write code (not brainstorming/research)

---

## Before writing any code, verify files are not in violation AND record current sizes.

```bash
# 1. Source file violations (line-based)
find crates/ -name "*.rs" -not -path "*/target/*" -not -path "*/tests/*" | xargs wc -l 2>/dev/null | sort -rn | awk '$1 > 1500 {print}' | head -20

# 2. Test file violations (KB-based)
find crates/ -path "*/tests/*.rs" -not -path "*/target/*" -size +12k | xargs du -sh 2>/dev/null | sort -rh | head -20

# 3. Record current size for EVERY file you will write to
wc -l <each target source file>
du -sh <each target test file>
```

**Record and carry into GATE 1 (MANDATORY):**

```
Target file: src/foo.rs              — current: 1,240 lines
Target file: tests/stdlib/strings.rs — current: 8KB
```

| Result | Action |
|--------|--------|
| Source file > 2,000 lines (no ARCH-EXCEPTION comment) | **BLOCKING** — split before adding any code |
| Source file 1,500–2,000 lines | Flag in phase summary — do not grow further |
| Test file > 12KB | **BLOCKING** — split before adding any tests |
| Test file 10–12KB | Warning — flag in phase summary, plan split |

**ARCH-EXCEPTION protocol:** If a file legitimately cannot be split (e.g. VM execute loop),
it must have `// ARCH-EXCEPTION: <reason>` at the top. Missing comment = violation.

**Test file routing:** See `.claude/rules/atlas-testing.md`
**Architecture limits:** See `.claude/lazy/architecture.md`
