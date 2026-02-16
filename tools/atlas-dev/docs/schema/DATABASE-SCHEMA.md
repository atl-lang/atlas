# atlas-dev Database Schema

**Single Source of Truth: `atlas-dev.db` (SQLite)**

---

## Philosophy

- ✅ **DB is canonical** - All tracking data lives in SQLite
- ✅ **Phase files are instructions** - `phases/**/*.md` tell AI what to build
- ✅ **No markdown tracking files** - No STATUS.md, no trackers/*.md, no decision-logs/*.md
- ✅ **Export on demand** - Generate markdown for humans later (DocManager skill)
- ✅ **Web control panel ready** - Direct DB queries for API

---

## Schema Version: 1.0

```sql
-- ============================================================================
-- PHASES (replaces STATUS.md + status/trackers/*.md)
-- ============================================================================

CREATE TABLE phases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,              -- "phases/stdlib/phase-07b-hashset.md"
    name TEXT NOT NULL,                     -- "phase-07b-hashset"
    category TEXT NOT NULL,                 -- "stdlib"
    status TEXT NOT NULL DEFAULT 'pending', -- "pending" | "in_progress" | "completed" | "blocked"
    completed_date TEXT,                    -- "2026-02-15T12:00:00Z" (ISO 8601)
    description TEXT,                       -- "HashSet with 25 tests, 100% parity"
    test_count INTEGER DEFAULT 0,           -- 25
    test_target INTEGER,                    -- 36 (from phase file)
    acceptance_criteria TEXT,               -- JSON array: ["36+ tests", "FIFO semantics"]
    blockers TEXT,                          -- JSON array of phase IDs: [12, 15]
    dependencies TEXT,                      -- JSON array: ["phase-07a", "phase-07b"]
    files_modified TEXT,                    -- JSON array: ["src/queue.rs", "tests/queue_tests.rs"]
    created_at TEXT NOT NULL,               -- Timestamp when added to DB
    updated_at TEXT NOT NULL                -- Timestamp of last update
);

CREATE INDEX idx_phases_category ON phases(category);
CREATE INDEX idx_phases_status ON phases(status);
CREATE INDEX idx_phases_completed_date ON phases(completed_date);

-- ============================================================================
-- CATEGORIES (replaces status/trackers/*.md)
-- ============================================================================

CREATE TABLE categories (
    id INTEGER PRIMARY KEY,                 -- 0, 1, 2, 3, ... (tracker number)
    name TEXT UNIQUE NOT NULL,              -- "stdlib"
    display_name TEXT NOT NULL,             -- "Standard Library"
    completed INTEGER NOT NULL DEFAULT 0,   -- 10
    total INTEGER NOT NULL,                 -- 21
    percentage INTEGER NOT NULL DEFAULT 0,  -- 48
    status TEXT NOT NULL DEFAULT 'pending', -- "pending" | "active" | "complete" | "blocked"
    status_notes TEXT,                      -- "⚠️ blockers at phase-10+"
    updated_at TEXT NOT NULL
);

-- Seed categories (run once at DB init)
INSERT INTO categories (id, name, display_name, total) VALUES
    (0, 'foundation', 'Foundation', 21),
    (1, 'stdlib', 'Standard Library', 21),
    (2, 'bytecode-vm', 'Bytecode & VM', 8),
    (3, 'frontend', 'Frontend', 5),
    (4, 'typing', 'Type System', 7),
    (5, 'interpreter', 'Interpreter', 2),
    (6, 'cli', 'CLI', 6),
    (7, 'lsp', 'LSP', 5),
    (8, 'polish', 'Polish', 5);

-- ============================================================================
-- DECISIONS (replaces docs/decision-logs/**/*.md)
-- ============================================================================

CREATE TABLE decisions (
    id TEXT PRIMARY KEY,                    -- "DR-007"
    component TEXT NOT NULL,                -- "stdlib"
    title TEXT NOT NULL,                    -- "Hash function design"
    decision TEXT NOT NULL,                 -- Full decision text (markdown)
    rationale TEXT NOT NULL,                -- Why this decision
    alternatives TEXT,                      -- Alternatives considered (markdown)
    consequences TEXT,                      -- Consequences/tradeoffs
    date TEXT NOT NULL,                     -- "2026-02-15"
    status TEXT NOT NULL DEFAULT 'accepted',-- "accepted" | "rejected" | "superseded" | "proposed"
    superseded_by TEXT,                     -- "DR-015" (if superseded)
    related_phases TEXT,                    -- JSON array: ["phase-07a", "phase-07b"]
    tags TEXT,                              -- JSON array: ["performance", "api-design"]
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_decisions_component ON decisions(component);
CREATE INDEX idx_decisions_date ON decisions(date);
CREATE INDEX idx_decisions_status ON decisions(status);

-- ============================================================================
-- FEATURES (replaces docs/features/**/*.md)
-- ============================================================================

CREATE TABLE features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,              -- "pattern-matching"
    display_name TEXT NOT NULL,             -- "Pattern Matching"
    version TEXT NOT NULL,                  -- "v0.1"
    status TEXT NOT NULL,                   -- "complete" | "in-progress" | "planned"
    description TEXT,                       -- Brief description
    implementation_notes TEXT,              -- Technical details (markdown)
    related_phases TEXT,                    -- JSON array: ["phase-03-pattern-matching"]
    spec_path TEXT,                         -- "docs/specification/pattern-matching.md"
    api_path TEXT,                          -- "docs/api/pattern-matching.md"
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_features_version ON features(version);
CREATE INDEX idx_features_status ON features(status);

-- ============================================================================
-- SPECS (tracks specification docs)
-- ============================================================================

CREATE TABLE specs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,              -- "docs/specification/types.md"
    name TEXT NOT NULL,                     -- "types"
    section TEXT NOT NULL,                  -- "types" | "syntax" | "semantics" | "runtime"
    title TEXT NOT NULL,                    -- "Type System Specification"
    summary TEXT,                           -- Brief summary
    last_validated TEXT,                    -- ISO 8601 timestamp
    validation_status TEXT,                 -- "valid" | "needs_update" | "invalid"
    related_features TEXT,                  -- JSON array
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_specs_section ON specs(section);

-- ============================================================================
-- API_DOCS (tracks API documentation)
-- ============================================================================

CREATE TABLE api_docs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,              -- "docs/api/stdlib.md"
    module TEXT NOT NULL,                   -- "stdlib" | "runtime" | "compiler"
    name TEXT NOT NULL,                     -- "stdlib"
    title TEXT NOT NULL,                    -- "Standard Library API"
    functions_count INTEGER DEFAULT 0,      -- 125 (tracked functions)
    last_validated TEXT,
    validation_status TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ============================================================================
-- METADATA (replaces STATUS.md header + global config)
-- ============================================================================

CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Seed metadata
INSERT INTO metadata (key, value, updated_at) VALUES
    ('schema_version', '1', datetime('now')),
    ('atlas_version', 'v0.2', datetime('now')),
    ('current_phase_id', NULL, datetime('now')),      -- Phase ID currently being worked on
    ('last_completed_phase_id', NULL, datetime('now')),
    ('next_phase_id', NULL, datetime('now')),
    ('total_phases', '78', datetime('now')),
    ('completed_phases', '0', datetime('now')),
    ('last_updated', datetime('now'), datetime('now'));

-- ============================================================================
-- AUDIT_LOG (change history - replaces git log for tracking data)
-- ============================================================================

CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    action TEXT NOT NULL,                   -- "phase_complete" | "decision_create" | "feature_add"
    entity_type TEXT NOT NULL,              -- "phase" | "decision" | "feature"
    entity_id TEXT NOT NULL,                -- Phase ID, decision ID, etc.
    changes TEXT NOT NULL,                  -- JSON with before/after values
    commit_sha TEXT,                        -- Git commit SHA (if committed)
    agent TEXT                              -- AI agent identifier (optional)
);

CREATE INDEX idx_audit_timestamp ON audit_log(timestamp);
CREATE INDEX idx_audit_entity ON audit_log(entity_type, entity_id);

-- ============================================================================
-- PARITY_CHECKS (tracks code/spec/docs/tests parity validation)
-- ============================================================================

CREATE TABLE parity_checks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    check_type TEXT NOT NULL,               -- "code_spec" | "code_tests" | "spec_docs" | "full"
    status TEXT NOT NULL,                   -- "pass" | "fail" | "warning"
    issues_count INTEGER NOT NULL DEFAULT 0,
    issues TEXT,                            -- JSON array of issue objects
    summary TEXT,                           -- Human-readable summary
    duration_ms INTEGER                     -- How long check took
);

CREATE INDEX idx_parity_timestamp ON parity_checks(timestamp);
CREATE INDEX idx_parity_status ON parity_checks(status);

-- ============================================================================
-- TEST_COVERAGE (tracks test counts and coverage)
-- ============================================================================

CREATE TABLE test_coverage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    category TEXT NOT NULL,                 -- "stdlib" | "runtime" | "compiler"
    test_count INTEGER NOT NULL,            -- 125
    passing INTEGER NOT NULL,               -- 123
    failing INTEGER NOT NULL,               -- 2
    coverage_percent REAL,                  -- 87.5 (if available)
    details TEXT                            -- JSON with per-module breakdown
);

CREATE INDEX idx_test_coverage_timestamp ON test_coverage(timestamp);
CREATE INDEX idx_test_coverage_category ON test_coverage(category);

-- ============================================================================
-- VIEWS (convenience queries)
-- ============================================================================

-- Current progress summary
CREATE VIEW v_progress AS
SELECT
    c.name AS category,
    c.display_name,
    c.completed,
    c.total,
    c.percentage,
    c.status,
    c.status_notes
FROM categories c
ORDER BY c.id;

-- Active phases
CREATE VIEW v_active_phases AS
SELECT
    p.id,
    p.path,
    p.name,
    p.category,
    p.description,
    p.test_count,
    p.test_target
FROM phases p
WHERE p.status IN ('in_progress', 'blocked')
ORDER BY p.category, p.name;

-- Recent decisions
CREATE VIEW v_recent_decisions AS
SELECT
    d.id,
    d.component,
    d.title,
    d.date,
    d.status
FROM decisions d
ORDER BY d.date DESC
LIMIT 20;

-- Parity validation summary
CREATE VIEW v_parity_summary AS
SELECT
    check_type,
    status,
    COUNT(*) as check_count,
    MAX(timestamp) as last_check
FROM parity_checks
GROUP BY check_type, status
ORDER BY last_check DESC;
```

---

## Triggers (Automatic Updates)

```sql
-- Auto-update category progress when phase completed
CREATE TRIGGER update_category_progress
AFTER UPDATE ON phases
WHEN NEW.status = 'completed' AND OLD.status != 'completed'
BEGIN
    UPDATE categories
    SET
        completed = (
            SELECT COUNT(*)
            FROM phases
            WHERE category = NEW.category AND status = 'completed'
        ),
        percentage = (
            SELECT ROUND(CAST(COUNT(*) AS REAL) / total * 100)
            FROM phases
            WHERE category = NEW.category AND status = 'completed'
        ),
        status = CASE
            WHEN (SELECT COUNT(*) FROM phases WHERE category = NEW.category AND status = 'completed') = total
            THEN 'complete'
            WHEN (SELECT COUNT(*) FROM phases WHERE category = NEW.category AND status = 'completed') > 0
            THEN 'active'
            ELSE 'pending'
        END,
        updated_at = datetime('now')
    WHERE name = NEW.category;

    -- Update global metadata
    UPDATE metadata
    SET value = (SELECT COUNT(*) FROM phases WHERE status = 'completed'),
        updated_at = datetime('now')
    WHERE key = 'completed_phases';

    UPDATE metadata
    SET value = datetime('now'),
        updated_at = datetime('now')
    WHERE key = 'last_updated';
END;

-- Auto-update timestamps
CREATE TRIGGER update_phases_timestamp
AFTER UPDATE ON phases
BEGIN
    UPDATE phases SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER update_decisions_timestamp
AFTER UPDATE ON decisions
BEGIN
    UPDATE decisions SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER update_features_timestamp
AFTER UPDATE ON features
BEGIN
    UPDATE features SET updated_at = datetime('now') WHERE id = NEW.id;
END;
```

---

## Common Queries

### Get Current Phase
```sql
SELECT p.path, p.category, p.name
FROM phases p
JOIN metadata m ON p.id = CAST(m.value AS INTEGER)
WHERE m.key = 'next_phase_id';
```

### Get Next Phase
```sql
SELECT p.path, p.name, p.description
FROM phases p
WHERE p.status = 'pending'
  AND p.category = (
      SELECT category FROM phases WHERE id = (
          SELECT CAST(value AS INTEGER) FROM metadata WHERE key = 'current_phase_id'
      )
  )
ORDER BY p.id
LIMIT 1;
```

### Get Progress Summary
```sql
SELECT name, completed, total, percentage, status
FROM categories
ORDER BY id;
```

### Find Blocked Phases
```sql
SELECT p.path, p.name, p.category, p.blockers
FROM phases p
WHERE p.status = 'blocked'
ORDER BY p.category;
```

### Search Decisions
```sql
SELECT id, title, component, date
FROM decisions
WHERE component = 'stdlib'
  AND status = 'accepted'
  AND (title LIKE '%hash%' OR decision LIKE '%hash%')
ORDER BY date DESC;
```

### Get Phase with Full Context
```sql
SELECT
    p.*,
    c.completed AS category_completed,
    c.total AS category_total,
    c.percentage AS category_percentage,
    (SELECT GROUP_CONCAT(id) FROM decisions WHERE component = p.category) AS related_decisions
FROM phases p
JOIN categories c ON p.category = c.name
WHERE p.id = ?;
```

---

## Migration from Markdown

See `MIGRATION.md` for one-time migration script to populate DB from existing:
- STATUS.md → phases + categories + metadata
- status/trackers/*.md → phases + categories
- docs/decision-logs/**/*.md → decisions

---

## Backup & Restore

### Backup
```bash
# SQLite dump (SQL format)
sqlite3 atlas-dev.db .dump > backup.sql

# Binary backup
cp atlas-dev.db atlas-dev.backup.db

# JSON export (human-readable)
atlas-dev export json > backup.json
```

### Restore
```bash
# From SQL dump
sqlite3 atlas-dev.db < backup.sql

# From binary backup
cp atlas-dev.backup.db atlas-dev.db

# From JSON export
atlas-dev import json backup.json
```

---

## Performance

- All queries indexed (< 1ms for 1000s of rows)
- Triggers auto-update aggregates (no manual recalculation)
- Views provide convenient access patterns
- Transactions ensure consistency
- WAL mode for concurrent reads

---

## Schema Evolution

Version bumps tracked in `metadata.schema_version`:

```sql
-- Check current version
SELECT value FROM metadata WHERE key = 'schema_version';

-- Migrations applied as needed
-- See migrations/ directory for version-specific SQL scripts
```
