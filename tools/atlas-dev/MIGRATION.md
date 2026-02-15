# Migration from Markdown to SQLite

**One-time migration to populate `atlas-dev.db` from existing STATUS.md system.**

---

## Overview

**Current state:**
- `STATUS.md` (dashboard)
- `status/trackers/*.md` (9 tracker files)
- Completed: 30/78 phases

**Target state:**
- `atlas-dev.db` (all tracking data)
- `phases/**/*.md` (instructions only - unchanged)
- Delete: `STATUS.md`, `status/trackers/*.md` (no longer needed)

---

## Migration Strategy

### Phase 0: One-Time Bootstrap (Before Phase 1)

**Run once to populate initial DB from markdown files.**

```bash
cd tools/atlas-dev
atlas-dev migrate bootstrap
```

**What it does:**
1. Create `atlas-dev.db` with full schema
2. Parse `STATUS.md` → extract metadata
3. Parse `status/trackers/*.md` → extract all phases
4. Insert all data into DB
5. Validate counts match
6. Backup original markdown files
7. Generate audit log entry

---

## Migration Script

**File:** `cmd/atlas-dev/migrate.go`

```go
package main

import (
    "database/sql"
    "fmt"
    "os"
    "path/filepath"
    "regexp"
    "strings"
    "time"

    _ "github.com/mattn/go-sqlite3"
)

type MigrationData struct {
    Phases     []Phase
    Categories []Category
    Metadata   map[string]string
}

func runMigration() error {
    // 1. Create database
    db, err := sql.Open("sqlite3", "atlas-dev.db")
    if err != nil {
        return fmt.Errorf("failed to create database: %w", err)
    }
    defer db.Close()

    // 2. Create schema
    if err := createSchema(db); err != nil {
        return fmt.Errorf("failed to create schema: %w", err)
    }

    // 3. Parse STATUS.md
    status, err := parseStatusMd("../../STATUS.md")
    if err != nil {
        return fmt.Errorf("failed to parse STATUS.md: %w", err)
    }

    // 4. Parse all tracker files
    trackers, err := parseTrackers("../../status/trackers")
    if err != nil {
        return fmt.Errorf("failed to parse trackers: %w", err)
    }

    // 5. Build migration data
    data := buildMigrationData(status, trackers)

    // 6. Insert into database
    if err := insertMigrationData(db, data); err != nil {
        return fmt.Errorf("failed to insert data: %w", err)
    }

    // 7. Validate
    if err := validateMigration(db, data); err != nil {
        return fmt.Errorf("validation failed: %w", err)
    }

    // 8. Backup original markdown files
    if err := backupMarkdownFiles(); err != nil {
        return fmt.Errorf("backup failed: %w", err)
    }

    // 9. Success message
    fmt.Println("✅ Migration complete!")
    fmt.Printf("   Phases: %d\n", len(data.Phases))
    fmt.Printf("   Categories: %d\n", len(data.Categories))
    fmt.Println("\n✅ Database: atlas-dev.db created")
    fmt.Println("✅ Backup: markdown files backed up to .migration-backup/")
    fmt.Println("\nNext: Start Phase 1 implementation")

    return nil
}

func parseStatusMd(path string) (*StatusData, error) {
    content, err := os.ReadFile(path)
    if err != nil {
        return nil, err
    }

    data := &StatusData{}

    // Extract metadata
    data.Version = extractPattern(string(content), `\*\*Version:\*\* (.+)`)
    data.LastUpdated = extractPattern(string(content), `\*\*Last Updated:\*\* (.+)`)
    data.LastCompleted = extractPattern(string(content), `\*\*Last Completed:\*\* (.+)`)
    data.NextPhase = extractPattern(string(content), `\*\*Next Phase:\*\* (.+)`)

    // Extract progress
    progressStr := extractPattern(string(content), `\*\*Real Progress:\*\* (\d+)/(\d+) phases`)
    fmt.Sscanf(progressStr, "%d/%d", &data.CompletedPhases, &data.TotalPhases)

    return data, nil
}

func parseTrackers(dir string) ([]TrackerData, error) {
    files, err := filepath.Glob(filepath.Join(dir, "*.md"))
    if err != nil {
        return nil, err
    }

    var trackers []TrackerData
    for _, file := range files {
        tracker, err := parseTrackerFile(file)
        if err != nil {
            return nil, fmt.Errorf("failed to parse %s: %w", file, err)
        }
        trackers = append(trackers, tracker)
    }

    return trackers, nil
}

func parseTrackerFile(path string) (TrackerData, error) {
    content, err := os.ReadFile(path)
    if err != nil {
        return TrackerData{}, err
    }

    data := TrackerData{
        FilePath: path,
    }

    // Extract category from filename: "1-stdlib.md" → stdlib, tracker_num=1
    filename := filepath.Base(path)
    parts := strings.SplitN(strings.TrimSuffix(filename, ".md"), "-", 2)
    if len(parts) == 2 {
        fmt.Sscanf(parts[0], "%d", &data.TrackerNum)
        data.Category = parts[1]
    }

    // Parse completed phases
    completedRe := regexp.MustCompile(`(?m)^- ✅ (.+?)\.md \*\*\[(.+?), (\d{4}-\d{2}-\d{2})\]\*\*`)
    matches := completedRe.FindAllStringSubmatch(string(content), -1)
    for _, match := range matches {
        if len(match) >= 4 {
            data.CompletedPhases = append(data.CompletedPhases, PhaseData{
                Name:          match[1] + ".md",
                Description:   match[2],
                CompletedDate: match[3],
                Status:        "completed",
            })
        }
    }

    // Parse pending phases
    pendingRe := regexp.MustCompile(`(?m)^- ⬜ (.+?)\.md \*\*\[(.+?)\]\*\*`)
    matches = pendingRe.FindAllStringSubmatch(string(content), -1)
    for _, match := range matches {
        if len(match) >= 3 {
            data.PendingPhases = append(data.PendingPhases, PhaseData{
                Name:        match[1] + ".md",
                Description: match[2],
                Status:      "pending",
            })
        }
    }

    // Parse blocked phases
    blockedRe := regexp.MustCompile(`(?m)^- 🚨 (.+?)\.md \*\*\[BLOCKED: (.+?)\]\*\*`)
    matches = blockedRe.FindAllStringSubmatch(string(content), -1)
    for _, match := range matches {
        if len(match) >= 3 {
            data.PendingPhases = append(data.PendingPhases, PhaseData{
                Name:        match[1] + ".md",
                Description: "BLOCKED: " + match[2],
                Status:      "blocked",
            })
        }
    }

    data.Completed = len(data.CompletedPhases)
    data.Total = len(data.CompletedPhases) + len(data.PendingPhases)
    data.Percentage = int(float64(data.Completed) / float64(data.Total) * 100)

    return data, nil
}

func buildMigrationData(status *StatusData, trackers []TrackerData) *MigrationData {
    data := &MigrationData{
        Metadata: map[string]string{
            "atlas_version":            status.Version,
            "total_phases":             fmt.Sprintf("%d", status.TotalPhases),
            "completed_phases":         fmt.Sprintf("%d", status.CompletedPhases),
            "last_updated":             status.LastUpdated,
        },
    }

    // Build phases and categories from trackers
    phaseID := 1
    for _, tracker := range trackers {
        // Add category
        data.Categories = append(data.Categories, Category{
            ID:          tracker.TrackerNum,
            Name:        tracker.Category,
            DisplayName: categoryDisplayName(tracker.Category),
            Completed:   tracker.Completed,
            Total:       tracker.Total,
            Percentage:  tracker.Percentage,
            Status:      categoryStatus(tracker.Percentage),
        })

        // Add completed phases
        for _, p := range tracker.CompletedPhases {
            data.Phases = append(data.Phases, Phase{
                ID:            phaseID,
                Path:          fmt.Sprintf("phases/%s/%s", tracker.Category, p.Name),
                Name:          strings.TrimSuffix(p.Name, ".md"),
                Category:      tracker.Category,
                Status:        p.Status,
                CompletedDate: p.CompletedDate,
                Description:   p.Description,
            })
            phaseID++
        }

        // Add pending/blocked phases
        for _, p := range tracker.PendingPhases {
            data.Phases = append(data.Phases, Phase{
                ID:          phaseID,
                Path:        fmt.Sprintf("phases/%s/%s", tracker.Category, p.Name),
                Name:        strings.TrimSuffix(p.Name, ".md"),
                Category:    tracker.Category,
                Status:      p.Status,
                Description: p.Description,
            })
            phaseID++
        }
    }

    return data
}

func insertMigrationData(db *sql.DB, data *MigrationData) error {
    tx, err := db.Begin()
    if err != nil {
        return err
    }
    defer tx.Rollback()

    // Insert categories
    for _, cat := range data.Categories {
        _, err := tx.Exec(`
            INSERT INTO categories (id, name, display_name, completed, total, percentage, status, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))
        `, cat.ID, cat.Name, cat.DisplayName, cat.Completed, cat.Total, cat.Percentage, cat.Status)
        if err != nil {
            return fmt.Errorf("failed to insert category %s: %w", cat.Name, err)
        }
    }

    // Insert phases
    for _, phase := range data.Phases {
        completedDate := sql.NullString{String: phase.CompletedDate, Valid: phase.CompletedDate != ""}
        _, err := tx.Exec(`
            INSERT INTO phases (id, path, name, category, status, completed_date, description, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
        `, phase.ID, phase.Path, phase.Name, phase.Category, phase.Status, completedDate, phase.Description)
        if err != nil {
            return fmt.Errorf("failed to insert phase %s: %w", phase.Name, err)
        }
    }

    // Insert metadata
    for key, value := range data.Metadata {
        _, err := tx.Exec(`
            INSERT OR REPLACE INTO metadata (key, value, updated_at)
            VALUES (?, ?, datetime('now'))
        `, key, value)
        if err != nil {
            return fmt.Errorf("failed to insert metadata %s: %w", key, err)
        }
    }

    // Insert audit log entry
    _, err = tx.Exec(`
        INSERT INTO audit_log (action, entity_type, entity_id, changes)
        VALUES ('migration_bootstrap', 'system', 'initial', ?)
    `, fmt.Sprintf(`{"phases_migrated": %d, "categories_migrated": %d}`,
        len(data.Phases), len(data.Categories)))
    if err != nil {
        return fmt.Errorf("failed to insert audit log: %w", err)
    }

    return tx.Commit()
}

func validateMigration(db *sql.DB, data *MigrationData) error {
    // Count phases in DB
    var dbPhaseCount int
    err := db.QueryRow("SELECT COUNT(*) FROM phases").Scan(&dbPhaseCount)
    if err != nil {
        return err
    }

    if dbPhaseCount != len(data.Phases) {
        return fmt.Errorf("phase count mismatch: expected %d, got %d", len(data.Phases), dbPhaseCount)
    }

    // Count completed phases
    var dbCompletedCount int
    err = db.QueryRow("SELECT COUNT(*) FROM phases WHERE status = 'completed'").Scan(&dbCompletedCount)
    if err != nil {
        return err
    }

    expectedCompleted := 0
    for _, p := range data.Phases {
        if p.Status == "completed" {
            expectedCompleted++
        }
    }

    if dbCompletedCount != expectedCompleted {
        return fmt.Errorf("completed count mismatch: expected %d, got %d", expectedCompleted, dbCompletedCount)
    }

    // Validate categories
    var dbCategoryCount int
    err = db.QueryRow("SELECT COUNT(*) FROM categories").Scan(&dbCategoryCount)
    if err != nil {
        return err
    }

    if dbCategoryCount != len(data.Categories) {
        return fmt.Errorf("category count mismatch: expected %d, got %d", len(data.Categories), dbCategoryCount)
    }

    return nil
}

func backupMarkdownFiles() error {
    backupDir := "../../.migration-backup"
    if err := os.MkdirAll(backupDir, 0755); err != nil {
        return err
    }

    // Backup STATUS.md
    if err := copyFile("../../STATUS.md", filepath.Join(backupDir, "STATUS.md")); err != nil {
        return err
    }

    // Backup trackers
    trackerBackupDir := filepath.Join(backupDir, "status/trackers")
    if err := os.MkdirAll(trackerBackupDir, 0755); err != nil {
        return err
    }

    files, _ := filepath.Glob("../../status/trackers/*.md")
    for _, file := range files {
        dest := filepath.Join(trackerBackupDir, filepath.Base(file))
        if err := copyFile(file, dest); err != nil {
            return err
        }
    }

    return nil
}

func categoryDisplayName(name string) string {
    displayNames := map[string]string{
        "foundation":  "Foundation",
        "stdlib":      "Standard Library",
        "bytecode-vm": "Bytecode & VM",
        "frontend":    "Frontend",
        "typing":      "Type System",
        "interpreter": "Interpreter",
        "cli":         "CLI",
        "lsp":         "LSP",
        "polish":      "Polish & Finalization",
    }
    if display, ok := displayNames[name]; ok {
        return display
    }
    return strings.Title(name)
}

func categoryStatus(percentage int) string {
    if percentage == 0 {
        return "pending"
    } else if percentage == 100 {
        return "complete"
    }
    return "active"
}
```

---

## Running the Migration

```bash
# 1. Build atlas-dev with migration command
cd tools/atlas-dev
go build -o atlas-dev cmd/atlas-dev/*.go

# 2. Run migration
./atlas-dev migrate bootstrap

# Output:
# Parsing STATUS.md...
# Parsing trackers...
# Creating database schema...
# Inserting 78 phases...
# Inserting 9 categories...
# Validating migration...
# Backing up markdown files...
# ✅ Migration complete!
#    Phases: 78
#    Categories: 9
#    Completed: 30
#
# ✅ Database: atlas-dev.db created
# ✅ Backup: markdown files backed up to .migration-backup/

# 3. Verify migration
./atlas-dev validate

# Output:
# ✅ Database validation passed
#    Phases: 78 (30 completed, 48 pending)
#    Categories: 9
#    Progress: 30/78 (38%)

# 4. Test queries
./atlas-dev phase current
# {"path":"phases/stdlib/phase-07b-hashset.md","cat":"stdlib"}

./atlas-dev summary
# {"cats":[["foundation",21,21,100],["stdlib",9,21,43],...]}
```

---

## Post-Migration Cleanup

**After successful migration and validation:**

```bash
# 1. Delete old markdown files (backed up in .migration-backup/)
rm STATUS.md
rm -rf status/

# 2. Update .gitignore
echo "atlas-dev.db" >> .gitignore  # Or commit it - your choice
echo ".migration-backup/" >> .gitignore

# 3. Commit
git add .
git commit -m "Migrate to pure SQLite tracking system

- Migrated STATUS.md → atlas-dev.db
- Migrated status/trackers/*.md → phases + categories tables
- 78 phases, 30 completed, 9 categories
- Backup saved to .migration-backup/
- Single source of truth: atlas-dev.db

Old markdown tracking files removed (backed up)"
```

---

## Rollback (If Needed)

```bash
# Restore from backup
cp .migration-backup/STATUS.md .
cp -r .migration-backup/status/ .

# Delete database
rm atlas-dev.db

# Migration can be re-run after fixes
```

---

## Validation Checklist

After migration:

- [ ] Database created: `atlas-dev.db` exists
- [ ] Schema version: `SELECT value FROM metadata WHERE key = 'schema_version'` returns `"1"`
- [ ] Phase count: `SELECT COUNT(*) FROM phases` returns `78`
- [ ] Completed count: `SELECT COUNT(*) FROM phases WHERE status = 'completed'` returns `30`
- [ ] Category count: `SELECT COUNT(*) FROM categories` returns `9`
- [ ] Progress matches: `SELECT completed, total FROM categories WHERE name = 'stdlib'` returns `9, 21`
- [ ] Backup exists: `.migration-backup/STATUS.md` and `.migration-backup/status/trackers/*.md` exist
- [ ] Commands work: `atlas-dev phase current`, `atlas-dev summary`, `atlas-dev validate` all return valid JSON

---

## Next Steps

After successful migration:

1. ✅ Delete old markdown files (backed up)
2. ✅ Proceed to Phase 1 implementation
3. ✅ Use pure SQLite for all operations
4. ✅ Markdown export available later (`atlas-dev export markdown`)
