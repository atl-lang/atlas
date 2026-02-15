# Phase 01: SQLite Infrastructure

**Objective:** Set up SQLite database, schema, transactions, JSON output, and core CLI framework.

**Priority:** CRITICAL (blocks all other phases)
**Architecture:** Pure SQLite (single source of truth)

---

## Deliverables

1. ✅ SQLite database setup (`atlas-dev.db`)
2. ✅ Complete schema creation (see `DATABASE-SCHEMA.md`)
3. ✅ Transaction handling (ACID guarantees)
4. ✅ JSON output system (DB → compact JSON)
5. ✅ File locking (prevent concurrent corruption)
6. ✅ Audit logging infrastructure
7. ✅ CLI framework (Cobra)
8. ✅ `atlas-dev version` works
9. ✅ `atlas-dev migrate bootstrap` works (one-time migration)

---

## File Structure

```
tools/atlas-dev/
├── atlas-dev.db                         # SQLite database (SINGLE SOURCE OF TRUTH)
├── go.mod                               # Go module
├── go.sum                               # Dependencies
├── Makefile                             # Build system
├── cmd/
│   └── atlas-dev/
│       ├── main.go                      # CLI entry point
│       ├── migrate.go                   # Migration commands
│       └── version.go                   # Version command
├── internal/
│   ├── db/
│   │   ├── db.go                        # Database connection + setup
│   │   ├── schema.go                    # Schema creation
│   │   ├── transaction.go               # Transaction handling
│   │   └── queries.go                   # Common queries
│   ├── lock/
│   │   └── filelock.go                  # File locking for git commits
│   ├── output/
│   │   ├── json.go                      # Compact JSON output
│   │   └── human.go                     # Human-readable (optional)
│   ├── audit/
│   │   └── log.go                       # Audit log helper
│   └── version/
│       └── version.go                   # Version info
├── testdata/                            # Test fixtures
│   └── test.db                          # Test database
└── DATABASE-SCHEMA.md                   # Schema reference (already created)
```

---

## Dependencies

Update `go.mod`:

```go
module github.com/atlas-lang/atlas-dev

go 1.22

require (
    github.com/mattn/go-sqlite3 v1.14.18    // SQLite driver
    github.com/spf13/cobra v1.8.0            // CLI framework
)
```

Install dependencies:
```bash
cd tools/atlas-dev
go get github.com/mattn/go-sqlite3@latest
go get github.com/spf13/cobra@latest
go mod tidy
```

---

## Implementation Steps

### Step 1: Database Setup

**File:** `internal/db/db.go`

```go
package db

import (
    "database/sql"
    "fmt"
    "os"
    "path/filepath"

    _ "github.com/mattn/go-sqlite3"
)

var (
    DB *sql.DB
)

// Open opens/creates the database
func Open(path string) error {
    // Create directory if needed
    dir := filepath.Dir(path)
    if err := os.MkdirAll(dir, 0755); err != nil {
        return fmt.Errorf("failed to create db directory: %w", err)
    }

    // Open database
    db, err := sql.Open("sqlite3", path+"?_journal_mode=WAL&_foreign_keys=ON")
    if err != nil {
        return fmt.Errorf("failed to open database: %w", err)
    }

    // Test connection
    if err := db.Ping(); err != nil {
        return fmt.Errorf("database ping failed: %w", err)
    }

    DB = db
    return nil
}

// Close closes the database
func Close() error {
    if DB != nil {
        return DB.Close()
    }
    return nil
}

// InitSchema creates all tables, indexes, triggers, views
func InitSchema() error {
    if DB == nil {
        return fmt.Errorf("database not opened")
    }

    // Execute schema from schema.go
    if err := createTables(); err != nil {
        return fmt.Errorf("failed to create tables: %w", err)
    }

    if err := createIndexes(); err != nil {
        return fmt.Errorf("failed to create indexes: %w", err)
    }

    if err := createTriggers(); err != nil {
        return fmt.Errorf("failed to create triggers: %w", err)
    }

    if err := createViews(); err != nil {
        return fmt.Errorf("failed to create views: %w", err)
    }

    if err := seedData(); err != nil {
        return fmt.Errorf("failed to seed data: %w", err)
    }

    return nil
}

// GetSchemaVersion returns current schema version
func GetSchemaVersion() (string, error) {
    var version string
    err := DB.QueryRow("SELECT value FROM metadata WHERE key = 'schema_version'").Scan(&version)
    return version, err
}
```

---

### Step 2: Schema Creation

**File:** `internal/db/schema.go`

```go
package db

import "database/sql"

func createTables() error {
    tables := []string{
        // Phases table
        `CREATE TABLE IF NOT EXISTS phases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            completed_date TEXT,
            description TEXT,
            test_count INTEGER DEFAULT 0,
            test_target INTEGER,
            acceptance_criteria TEXT,
            blockers TEXT,
            dependencies TEXT,
            files_modified TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )`,

        // Categories table
        `CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            display_name TEXT NOT NULL,
            completed INTEGER NOT NULL DEFAULT 0,
            total INTEGER NOT NULL,
            percentage INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'pending',
            status_notes TEXT,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )`,

        // Decisions table
        `CREATE TABLE IF NOT EXISTS decisions (
            id TEXT PRIMARY KEY,
            component TEXT NOT NULL,
            title TEXT NOT NULL,
            decision TEXT NOT NULL,
            rationale TEXT NOT NULL,
            alternatives TEXT,
            consequences TEXT,
            date TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'accepted',
            superseded_by TEXT,
            related_phases TEXT,
            tags TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )`,

        // Features table
        `CREATE TABLE IF NOT EXISTS features (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            display_name TEXT NOT NULL,
            version TEXT NOT NULL,
            status TEXT NOT NULL,
            description TEXT,
            implementation_notes TEXT,
            related_phases TEXT,
            spec_path TEXT,
            api_path TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )`,

        // Metadata table
        `CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )`,

        // Audit log table
        `CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            action TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            changes TEXT NOT NULL,
            commit_sha TEXT,
            agent TEXT
        )`,

        // Parity checks table
        `CREATE TABLE IF NOT EXISTS parity_checks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            check_type TEXT NOT NULL,
            status TEXT NOT NULL,
            issues_count INTEGER NOT NULL DEFAULT 0,
            issues TEXT,
            summary TEXT,
            duration_ms INTEGER
        )`,

        // Test coverage table
        `CREATE TABLE IF NOT EXISTS test_coverage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            category TEXT NOT NULL,
            test_count INTEGER NOT NULL,
            passing INTEGER NOT NULL,
            failing INTEGER NOT NULL,
            coverage_percent REAL,
            details TEXT
        )`,
    }

    for _, sql := range tables {
        if _, err := DB.Exec(sql); err != nil {
            return err
        }
    }

    return nil
}

func createIndexes() error {
    indexes := []string{
        "CREATE INDEX IF NOT EXISTS idx_phases_category ON phases(category)",
        "CREATE INDEX IF NOT EXISTS idx_phases_status ON phases(status)",
        "CREATE INDEX IF NOT EXISTS idx_phases_completed_date ON phases(completed_date)",
        "CREATE INDEX IF NOT EXISTS idx_decisions_component ON decisions(component)",
        "CREATE INDEX IF NOT EXISTS idx_decisions_date ON decisions(date)",
        "CREATE INDEX IF NOT EXISTS idx_decisions_status ON decisions(status)",
        "CREATE INDEX IF NOT EXISTS idx_features_version ON features(version)",
        "CREATE INDEX IF NOT EXISTS idx_features_status ON features(status)",
        "CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_log(entity_type, entity_id)",
        "CREATE INDEX IF NOT EXISTS idx_parity_timestamp ON parity_checks(timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_parity_status ON parity_checks(status)",
        "CREATE INDEX IF NOT EXISTS idx_test_coverage_timestamp ON test_coverage(timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_test_coverage_category ON test_coverage(category)",
    }

    for _, sql := range indexes {
        if _, err := DB.Exec(sql); err != nil {
            return err
        }
    }

    return nil
}

func createTriggers() error {
    triggers := []string{
        // Auto-update category progress when phase completed
        `CREATE TRIGGER IF NOT EXISTS update_category_progress
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

             UPDATE metadata
             SET value = (SELECT COUNT(*) FROM phases WHERE status = 'completed'),
                 updated_at = datetime('now')
             WHERE key = 'completed_phases';

             UPDATE metadata
             SET value = datetime('now'),
                 updated_at = datetime('now')
             WHERE key = 'last_updated';
         END`,

        // Auto-update timestamps
        `CREATE TRIGGER IF NOT EXISTS update_phases_timestamp
         AFTER UPDATE ON phases
         BEGIN
             UPDATE phases SET updated_at = datetime('now') WHERE id = NEW.id;
         END`,

        `CREATE TRIGGER IF NOT EXISTS update_decisions_timestamp
         AFTER UPDATE ON decisions
         BEGIN
             UPDATE decisions SET updated_at = datetime('now') WHERE id = NEW.id;
         END`,

        `CREATE TRIGGER IF NOT EXISTS update_features_timestamp
         AFTER UPDATE ON features
         BEGIN
             UPDATE features SET updated_at = datetime('now') WHERE id = NEW.id;
         END`,
    }

    for _, sql := range triggers {
        if _, err := DB.Exec(sql); err != nil {
            return err
        }
    }

    return nil
}

func createViews() error {
    views := []string{
        // Current progress summary
        `CREATE VIEW IF NOT EXISTS v_progress AS
         SELECT
             c.name AS category,
             c.display_name,
             c.completed,
             c.total,
             c.percentage,
             c.status,
             c.status_notes
         FROM categories c
         ORDER BY c.id`,

        // Active phases
        `CREATE VIEW IF NOT EXISTS v_active_phases AS
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
         ORDER BY p.category, p.name`,

        // Recent decisions
        `CREATE VIEW IF NOT EXISTS v_recent_decisions AS
         SELECT
             d.id,
             d.component,
             d.title,
             d.date,
             d.status
         FROM decisions d
         ORDER BY d.date DESC
         LIMIT 20`,
    }

    for _, sql := range views {
        if _, err := DB.Exec(sql); err != nil {
            return err
        }
    }

    return nil
}

func seedData() error {
    // Seed categories
    categories := []struct {
        ID          int
        Name        string
        DisplayName string
        Total       int
    }{
        {0, "foundation", "Foundation", 21},
        {1, "stdlib", "Standard Library", 21},
        {2, "bytecode-vm", "Bytecode & VM", 8},
        {3, "frontend", "Frontend", 5},
        {4, "typing", "Type System", 7},
        {5, "interpreter", "Interpreter", 2},
        {6, "cli", "CLI", 6},
        {7, "lsp", "LSP", 5},
        {8, "polish", "Polish & Finalization", 5},
    }

    for _, cat := range categories {
        _, err := DB.Exec(`
            INSERT OR IGNORE INTO categories (id, name, display_name, total, updated_at)
            VALUES (?, ?, ?, ?, datetime('now'))
        `, cat.ID, cat.Name, cat.DisplayName, cat.Total)
        if err != nil {
            return err
        }
    }

    // Seed metadata
    metadata := map[string]string{
        "schema_version":    "1",
        "atlas_version":     "v0.2",
        "total_phases":      "78",
        "completed_phases":  "0",
    }

    for key, value := range metadata {
        _, err := DB.Exec(`
            INSERT OR IGNORE INTO metadata (key, value, updated_at)
            VALUES (?, ?, datetime('now'))
        `, key, value)
        if err != nil {
            return err
        }
    }

    return nil
}
```

---

### Step 3: Transaction Handling

**File:** `internal/db/transaction.go`

```go
package db

import (
    "database/sql"
    "fmt"
)

// Transaction wraps sql.Tx with error handling
type Transaction struct {
    tx *sql.Tx
}

// Begin starts a transaction
func Begin() (*Transaction, error) {
    tx, err := DB.Begin()
    if err != nil {
        return nil, fmt.Errorf("failed to begin transaction: %w", err)
    }
    return &Transaction{tx: tx}, nil
}

// Commit commits the transaction
func (t *Transaction) Commit() error {
    if err := t.tx.Commit(); err != nil {
        return fmt.Errorf("failed to commit transaction: %w", err)
    }
    return nil
}

// Rollback rolls back the transaction
func (t *Transaction) Rollback() error {
    if err := t.tx.Rollback(); err != nil {
        return fmt.Errorf("failed to rollback transaction: %w", err)
    }
    return nil
}

// Exec executes a query within the transaction
func (t *Transaction) Exec(query string, args ...interface{}) (sql.Result, error) {
    return t.tx.Exec(query, args...)
}

// Query executes a query within the transaction
func (t *Transaction) Query(query string, args ...interface{}) (*sql.Rows, error) {
    return t.tx.Query(query, args...)
}

// QueryRow executes a query within the transaction
func (t *Transaction) QueryRow(query string, args ...interface{}) *sql.Row {
    return t.tx.QueryRow(query, args...)
}

// WithTransaction executes a function within a transaction
// Automatically commits on success, rollback on error
func WithTransaction(fn func(*Transaction) error) error {
    tx, err := Begin()
    if err != nil {
        return err
    }

    defer func() {
        if p := recover(); p != nil {
            tx.Rollback()
            panic(p)
        }
    }()

    if err := fn(tx); err != nil {
        tx.Rollback()
        return err
    }

    return tx.Commit()
}
```

---

### Step 4: JSON Output

**File:** `internal/output/json.go`

```go
package output

import (
    "encoding/json"
    "fmt"
    "io"
    "os"
)

// JSON outputs compact JSON to stdout
func JSON(data interface{}) error {
    encoder := json.NewEncoder(os.Stdout)
    encoder.SetIndent("", "")  // Compact (no pretty print)
    return encoder.Encode(data)
}

// JSONTo outputs compact JSON to writer
func JSONTo(w io.Writer, data interface{}) error {
    encoder := json.NewEncoder(w)
    encoder.SetIndent("", "")
    return encoder.Encode(data)
}

// Success outputs success response
func Success(data interface{}) error {
    response := map[string]interface{}{
        "ok": true,
    }

    // Merge data into response if it's a map
    if dataMap, ok := data.(map[string]interface{}); ok {
        for k, v := range dataMap {
            response[k] = v
        }
    } else {
        response["data"] = data
    }

    return JSON(response)
}

// Error outputs error response
func Error(err error, details map[string]interface{}) error {
    response := map[string]interface{}{
        "ok":  false,
        "err": err.Error(),
    }

    if details != nil {
        for k, v := range details {
            response[k] = v
        }
    }

    return JSON(response)
}

// Compact returns compact field names for token efficiency
type Compact map[string]interface{}

// Common abbreviations for token efficiency
const (
    OK          = "ok"          // success
    ERR         = "err"         // error
    MSG         = "msg"         // message
    CAT         = "cat"         // category
    PCT         = "pct"         // percentage
    CNT         = "cnt"         // count
    TOT         = "tot"         // total
    CMP         = "cmp"         // completed
    MOD         = "mod"         // modified
    DEP         = "dep"         // dependencies
    BLK         = "blk"         // blockers
    DESC        = "desc"        // description
    TS          = "ts"          // timestamp
)
```

---

### Step 5: File Locking

**File:** `internal/lock/filelock.go`

```go
package lock

import (
    "fmt"
    "os"
    "path/filepath"
    "time"
)

// FileLock represents a file lock
type FileLock struct {
    path     string
    lockFile *os.File
}

// Acquire acquires a file lock
func Acquire(path string) (*FileLock, error) {
    lockPath := path + ".lock"

    // Try to create lock file (exclusive)
    for i := 0; i < 50; i++ {  // Wait up to 5 seconds
        lockFile, err := os.OpenFile(lockPath, os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0644)
        if err == nil {
            // Write PID to lock file
            fmt.Fprintf(lockFile, "%d\n", os.Getpid())
            return &FileLock{
                path:     path,
                lockFile: lockFile,
            }, nil
        }

        // Lock exists, wait and retry
        time.Sleep(100 * time.Millisecond)
    }

    return nil, fmt.Errorf("failed to acquire lock on %s: timeout", path)
}

// Release releases the file lock
func (l *FileLock) Release() error {
    if l.lockFile != nil {
        l.lockFile.Close()
        lockPath := l.path + ".lock"
        return os.Remove(lockPath)
    }
    return nil
}

// WithLock executes function with file lock
func WithLock(path string, fn func() error) error {
    lock, err := Acquire(path)
    if err != nil {
        return err
    }
    defer lock.Release()

    return fn()
}
```

---

### Step 6: Audit Logging

**File:** `internal/audit/log.go`

```go
package audit

import (
    "encoding/json"
    "fmt"
    "time"

    "github.com/atlas-lang/atlas-dev/internal/db"
)

// Log creates an audit log entry
func Log(action, entityType, entityID string, changes interface{}) error {
    changesJSON, err := json.Marshal(changes)
    if err != nil {
        return fmt.Errorf("failed to marshal changes: %w", err)
    }

    _, err = db.DB.Exec(`
        INSERT INTO audit_log (action, entity_type, entity_id, changes)
        VALUES (?, ?, ?, ?)
    `, action, entityType, entityID, string(changesJSON))

    return err
}

// LogWithCommit creates audit log with git commit SHA
func LogWithCommit(action, entityType, entityID, commitSHA string, changes interface{}) error {
    changesJSON, err := json.Marshal(changes)
    if err != nil {
        return fmt.Errorf("failed to marshal changes: %w", err)
    }

    _, err = db.DB.Exec(`
        INSERT INTO audit_log (action, entity_type, entity_id, changes, commit_sha)
        VALUES (?, ?, ?, ?, ?)
    `, action, entityType, entityID, string(changesJSON), commitSHA)

    return err
}

// GetRecent returns recent audit log entries
func GetRecent(limit int) ([]AuditEntry, error) {
    rows, err := db.DB.Query(`
        SELECT id, timestamp, action, entity_type, entity_id, changes, commit_sha
        FROM audit_log
        ORDER BY timestamp DESC
        LIMIT ?
    `, limit)
    if err != nil {
        return nil, err
    }
    defer rows.Close()

    var entries []AuditEntry
    for rows.Next() {
        var entry AuditEntry
        var commitSHA *string
        err := rows.Scan(
            &entry.ID,
            &entry.Timestamp,
            &entry.Action,
            &entry.EntityType,
            &entry.EntityID,
            &entry.Changes,
            &commitSHA,
        )
        if err != nil {
            return nil, err
        }
        if commitSHA != nil {
            entry.CommitSHA = *commitSHA
        }
        entries = append(entries, entry)
    }

    return entries, nil
}

// AuditEntry represents an audit log entry
type AuditEntry struct {
    ID         int64  `json:"id"`
    Timestamp  string `json:"ts"`
    Action     string `json:"action"`
    EntityType string `json:"entity_type"`
    EntityID   string `json:"entity_id"`
    Changes    string `json:"changes"`
    CommitSHA  string `json:"commit,omitempty"`
}
```

---

### Step 7: CLI Framework

**File:** `cmd/atlas-dev/main.go`

```go
package main

import (
    "fmt"
    "os"
    "path/filepath"

    "github.com/atlas-lang/atlas-dev/internal/db"
    "github.com/atlas-lang/atlas-dev/internal/output"
    "github.com/spf13/cobra"
)

var (
    dbPath     string
    humanMode  bool
)

func main() {
    rootCmd := &cobra.Command{
        Use:           "atlas-dev",
        Short:         "Atlas development automation and docs management",
        Long:          "AI-optimized CLI tool for Atlas development workflow automation.",
        Version:       "1.0.0",
        SilenceUsage:  true,
        SilenceErrors: true,
        PersistentPreRunE: func(cmd *cobra.Command, args []string) error {
            // Open database
            if dbPath == "" {
                // Default: atlas-dev.db in current directory
                dbPath = "atlas-dev.db"
            }

            if err := db.Open(dbPath); err != nil {
                return fmt.Errorf("failed to open database: %w", err)
            }

            return nil
        },
        PersistentPostRunE: func(cmd *cobra.Command, args []string) error {
            return db.Close()
        },
    }

    // Global flags
    rootCmd.PersistentFlags().StringVar(&dbPath, "db", "", "database path (default: atlas-dev.db)")
    rootCmd.PersistentFlags().BoolVar(&humanMode, "human", false, "human-readable output (default: JSON)")

    // Add commands
    rootCmd.AddCommand(versionCmd())
    rootCmd.AddCommand(migrateCmd())

    // Execute
    if err := rootCmd.Execute(); err != nil {
        output.Error(err, nil)
        os.Exit(1)
    }
}

func versionCmd() *cobra.Command {
    return &cobra.Command{
        Use:   "version",
        Short: "Show version information",
        Run: func(cmd *cobra.Command, args []string) {
            if humanMode {
                fmt.Println("atlas-dev v1.0.0")
            } else {
                output.Success(map[string]interface{}{
                    "version": "1.0.0",
                    "schema":  "1",
                })
            }
        },
    }
}
```

---

### Step 8: Migration Command

**File:** `cmd/atlas-dev/migrate.go`

```go
package main

import (
    "fmt"

    "github.com/atlas-lang/atlas-dev/internal/db"
    "github.com/atlas-lang/atlas-dev/internal/output"
    "github.com/spf13/cobra"
)

func migrateCmd() *cobra.Command {
    cmd := &cobra.Command{
        Use:   "migrate",
        Short: "Database migration commands",
    }

    cmd.AddCommand(migrateBootstrapCmd())
    cmd.AddCommand(migrateSchemaCmd())

    return cmd
}

func migrateBootstrapCmd() *cobra.Command {
    return &cobra.Command{
        Use:   "bootstrap",
        Short: "Bootstrap database from STATUS.md (one-time migration)",
        Long:  "Migrate existing STATUS.md and trackers to SQLite database",
        RunE: func(cmd *cobra.Command, args []string) error {
            // See MIGRATION.md for full implementation
            fmt.Println("Running migration from STATUS.md to SQLite...")
            fmt.Println("(Implementation: see MIGRATION.md)")
            return nil
        },
    }
}

func migrateSchemaCmd() *cobra.Command {
    return &cobra.Command{
        Use:   "schema",
        Short: "Create database schema",
        RunE: func(cmd *cobra.Command, args []string) error {
            if err := db.InitSchema(); err != nil {
                return output.Error(err, nil)
            }

            version, err := db.GetSchemaVersion()
            if err != nil {
                return output.Error(err, nil)
            }

            return output.Success(map[string]interface{}{
                "schema_version": version,
                "msg":            "Schema created successfully",
            })
        },
    }
}
```

---

## Testing

### Unit Tests

**File:** `internal/db/db_test.go`

```go
package db

import (
    "os"
    "testing"
)

func TestDatabaseSetup(t *testing.T) {
    // Use in-memory database for tests
    err := Open(":memory:")
    if err != nil {
        t.Fatalf("Failed to open database: %v", err)
    }
    defer Close()

    // Create schema
    err = InitSchema()
    if err != nil {
        t.Fatalf("Failed to create schema: %v", err)
    }

    // Verify schema version
    version, err := GetSchemaVersion()
    if err != nil {
        t.Fatalf("Failed to get schema version: %v", err)
    }

    if version != "1" {
        t.Errorf("Expected schema version 1, got %s", version)
    }

    // Verify categories seeded
    var count int
    err = DB.QueryRow("SELECT COUNT(*) FROM categories").Scan(&count)
    if err != nil {
        t.Fatalf("Failed to count categories: %v", err)
    }

    if count != 9 {
        t.Errorf("Expected 9 categories, got %d", count)
    }
}
```

### Manual Tests

```bash
# Build
make build

# Create database and schema
./bin/atlas-dev migrate schema

# Verify
sqlite3 atlas-dev.db "SELECT * FROM metadata"

# Test version command
./bin/atlas-dev version
# Expected: {"ok":true,"version":"1.0.0","schema":"1"}

# Test version command (human mode)
./bin/atlas-dev version --human
# Expected: atlas-dev v1.0.0
```

---

## Acceptance Criteria

### Functional Requirements
- [ ] Database opens successfully
- [ ] Schema creates all tables (8 tables)
- [ ] Indexes created (14 indexes)
- [ ] Triggers created (4 triggers)
- [ ] Views created (3 views)
- [ ] Categories seeded (9 categories)
- [ ] Metadata seeded (schema_version, atlas_version, etc.)
- [ ] Transactions work (begin, commit, rollback)
- [ ] File locking works (prevents concurrent edits)
- [ ] Audit logging works (insert + query)
- [ ] `atlas-dev version` returns JSON
- [ ] `atlas-dev version --human` returns human text
- [ ] `atlas-dev migrate schema` creates schema
- [ ] All tests pass

### Token Efficiency Requirements
- [ ] `atlas-dev --help` output < 100 tokens
- [ ] `atlas-dev version --help` output < 60 tokens
- [ ] Default output is compact JSON (no pretty print)
- [ ] JSON uses abbreviated field names (ok, err, msg, cat, etc.)
- [ ] No emoji in default JSON output
- [ ] Null fields omitted from JSON

### Database Requirements
- [ ] WAL mode enabled (concurrent reads)
- [ ] Foreign keys enabled
- [ ] Auto-update triggers work correctly
- [ ] Schema version tracked in metadata
- [ ] All indexes created for performance

---

## Next Phase

**Phase 2:** Phase Management System
- Phase CRUD operations (using DB)
- `phase complete` command (updates DB, triggers update categories)
- `phase current` / `phase next` commands
- Git integration (commits after DB updates)

**Ready to start Phase 2 after this phase is 100% complete.**
