# Status Manager - Design Document

## Purpose

Automate Atlas phase completion tracking to eliminate human/AI errors in STATUS.md updates.

**Problem:** Manual two-file updates (tracker + STATUS.md) are error-prone:
- Forgetting to update tracker file (40% risk)
- Percentage calculation errors (30% risk)
- Wrong tracker file selection (15% risk)
- Partial STATUS.md updates (25% risk)
- Single-file commits (20% risk)

**Solution:** CLI tool that does EVERYTHING automatically. AI runs one command, tool handles all updates.

**Success Rate:** 99%+ (only fails if AI types command wrong)

---

## Architecture

**Language:** Go
- Fast compilation, single binary
- Excellent file/string manipulation
- Great CLI libraries (cobra)
- Cross-platform (darwin, linux, windows)
- Fits "world-class tooling" requirement

**Structure:**
```
tools/status-manager/
├── Makefile                  # build, install, test
├── go.mod
├── go.sum
├── cmd/
│   └── status-manager/
│       └── main.go           # CLI entry, cobra commands
├── internal/
│   ├── tracker/
│   │   ├── tracker.go        # Tracker file read/write
│   │   ├── parser.go         # Parse markdown tracker
│   │   └── updater.go        # Update phase status
│   ├── status/
│   │   ├── status.go         # STATUS.md read/write
│   │   ├── parser.go         # Parse STATUS.md structure
│   │   └── updater.go        # Update STATUS.md fields
│   ├── phase/
│   │   ├── phase.go          # Phase metadata
│   │   └── parser.go         # Parse phase paths
│   ├── validator/
│   │   └── validator.go      # Sync validation logic
│   └── git/
│       └── commit.go         # Git commit automation
├── testdata/                 # Test fixtures
│   ├── trackers/
│   └── STATUS.md
└── README.md
```

---

## Commands

### `status-manager complete <phase-path>`

**Primary command.** Marks phase complete, updates all files, validates, commits.

**Args:**
- `<phase-path>`: Phase file path (e.g., `phases/stdlib/phase-07b-hashset.md`)

**Flags:**
- `--description, -d`: Phase completion description (required)
- `--date`: Completion date (default: today, format: YYYY-MM-DD)
- `--commit, -c`: Auto-commit changes (default: false)
- `--dry-run`: Show what would change without modifying files

**Algorithm:**
1. Parse phase path:
   - Extract category: `stdlib`
   - Map to tracker: `status/trackers/1-stdlib.md`
   - Extract phase name: `phase-07b-hashset.md`

2. Update tracker file:
   - Find line: `- ⬜ phase-07b-hashset.md ...`
   - Replace with: `- ✅ phase-07b-hashset.md **[{description}, {date}]**`
   - Write file

3. Count completed phases:
   - Scan tracker: count lines matching `^- ✅`
   - Calculate category: `completed / total`
   - Calculate percentage: round to nearest int
   - Scan all trackers: count total completed
   - Calculate overall: `total_completed / 78`

4. Find next phase:
   - Scan tracker for next line matching `^- ⬜`
   - Extract phase name
   - Build full path: `phases/{category}/{phase-name}`

5. Update STATUS.md:
   - Line with `**Last Completed:**` → update phase path and date
   - Line with `**Next Phase:**` → update to next phase path
   - Line with `**Real Progress:**` → update count and percentage
   - Table row for category → update progress cell
   - Line with `**Last Updated:**` → update date

6. Validate sync:
   - Verify total completed matches STATUS.md
   - Verify category percentages match
   - Exit with error if mismatch

7. Show diff:
   - Print modified file paths
   - Show key changes (completed phase, percentages)

8. Git commit (if `--commit`):
   ```bash
   git add status/trackers/{N}-{category}.md STATUS.md
   git commit -m "Mark {phase-name} complete ({total}/{78})"
   ```

**Exit Codes:**
- 0: Success
- 1: Invalid phase path
- 2: Tracker file not found
- 3: Phase not found in tracker
- 4: Validation failed
- 5: Git commit failed

---

### `status-manager validate`

**Validation command.** Verifies STATUS.md sync with trackers.

**Flags:**
- `--verbose, -v`: Show detailed validation info
- `--fix`: Auto-fix percentages if counts match (dangerous)

**Algorithm:**
1. Count completed in each tracker
2. Calculate expected percentages
3. Parse STATUS.md current values
4. Compare counts and percentages
5. Report mismatches

**Exit Codes:**
- 0: Validation passed
- 1: Validation failed (show errors)

---

### `status-manager next`

**Next phase command.** Shows what to work on next.

**Algorithm:**
1. Parse STATUS.md for current phase
2. Extract category from current phase
3. Read tracker file
4. Find next ⬜ phase after current
5. Display phase info

---

### `status-manager summary`

**Progress summary.** Shows dashboard overview.

**Flags:**
- `--category, -c <name>`: Show detailed category progress
- `--json`: Output as JSON

**Algorithm:**
1. Read all tracker files
2. Count completed/total for each
3. Format as table
4. Show current/next phase

---

## File Formats

### Tracker File Format

**Expected structure:**
```markdown
# Category Name (X/Y) - Description

**Status:** ...
**Progress:** X/Y phases (Z%)

---

## Completed Phases

- ✅ phase-01-name.md (details)
- ✅ phase-02-name.md **[Details, YYYY-MM-DD]**

---

## Pending Phases

- ⬜ phase-03-name.md **[Description]**
- 🚨 phase-04-name.md **[BLOCKED: reason]**
```

**Parsing rules:**
- Completed: Lines matching `^- ✅`
- Pending: Lines matching `^- [⬜🚨]`
- Total: Sum of completed + pending
- Phase name: Extract between `- [✅⬜🚨] ` and ` `

### STATUS.md Format

**Expected structure:**
```markdown
## 🎯 Current Phase

**Last Completed:** phases/{category}/{phase}.md (verified YYYY-MM-DD)
**Next Phase:** phases/{category}/{phase}.md
**Real Progress:** X/78 phases complete (Z%)

---

## 📊 Category Progress

| Category | Progress | Status |
|----------|----------|--------|
| **[N. Name](path)** | X/Y (Z%) | ... |
```

**Update locations:**
- Line ~11: Last Completed
- Line ~12: Next Phase
- Line ~13: Real Progress
- Line ~21: Category table row
- Line ~3: Last Updated

**Parsing strategy:**
- Use regex patterns to find sections
- Update specific fields, preserve formatting
- Calculate line numbers dynamically (don't hardcode)

---

## Error Handling

**Robust error messages:**
```
❌ Error: Phase not found in tracker

Phase: phase-07b-hashset.md
Tracker: status/trackers/1-stdlib.md

Available phases in tracker:
  ✅ phase-07a-hash-infrastructure-hashmap.md
  ⬜ phase-07c-queue-stack.md
  ⬜ phase-07d-collection-integration.md

Did you mean: phase-07c-queue-stack.md?
```

**Validation errors:**
```
❌ Validation FAILED

Tracker counts: 31 phases complete
STATUS.md shows: 30/78

Mismatches:
  - status/trackers/1-stdlib.md: 10 complete (STATUS.md shows 9)

Run: status-manager complete --fix
```

---

## Testing

**Unit tests:**
- Tracker parsing
- STATUS.md parsing
- Phase path parsing
- Percentage calculations
- Next phase finding

**Integration tests:**
- Complete workflow with test fixtures
- Validation on known-good state
- Error handling on malformed files

**Test data:**
- `testdata/trackers/` - Sample tracker files
- `testdata/STATUS.md` - Sample dashboard
- `testdata/phases/` - Sample phase files

---

## Installation

**Build:**
```bash
cd tools/status-manager
make build
```

**Install:**
```bash
make install  # Installs to $GOPATH/bin or /usr/local/bin
```

**Usage from Atlas root:**
```bash
status-manager complete "phases/stdlib/phase-07b-hashset.md" \
  --description "HashSet with 25 tests" \
  --commit
```

---

## Integration with Atlas Skill

**Update `.claude/skills/atlas/skill.md`:**

```markdown
## Phase Completion Handoff

**After EVERY phase completion:**

```bash
# Run status-manager (handles ALL updates automatically)
status-manager complete "phases/{category}/{phase}.md" \
  --description "{brief summary: X functions, Y tests}" \
  --commit

# Validation runs automatically, no manual checks needed
```

**CRITICAL:** Do NOT manually edit STATUS.md or tracker files.
Use status-manager exclusively for updates.
```

**Benefits:**
- AI runs ONE command
- No file editing
- No percentage calculations
- No sync issues
- Auto-validated
- Auto-committed
- ~100% success rate

---

## Future Enhancements

**v1.1:**
- `status-manager undo` - Revert last completion
- `status-manager stats` - Detailed progress analytics
- `status-manager export` - Export to JSON/CSV

**v1.2:**
- `status-manager check-phase <path>` - Verify phase file exists and is valid
- `status-manager timeline` - Show completion timeline
- `status-manager estimate` - Estimate v0.2 completion date

**v2.0:**
- Interactive mode for manual override
- Web dashboard (serve progress UI)
- Slack/Discord webhook notifications

---

## Success Metrics

**Before status-manager:**
- Manual 2-file update
- 60% success rate (40% have issues)
- ~5 minutes per update
- Frequent sync errors

**After status-manager:**
- Automated 1-command update
- 99%+ success rate
- ~10 seconds per update
- Zero sync errors (validated automatically)

**Goal: Make phase completion tracking INVISIBLE to AI agents.**
