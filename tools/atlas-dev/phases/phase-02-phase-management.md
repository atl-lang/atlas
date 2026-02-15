# Phase 02: Phase Management System (SQLite)

**Objective:** Implement core phase tracking with pure SQLite - complete, current, next, validate.

**Priority:** CRITICAL
**Depends On:** Phase 1
**Architecture:** Pure SQLite (DB queries only, NO markdown parsing)

---

## Deliverables

1. ✅ `phase complete` command (updates DB, triggers auto-update categories)
2. ✅ `phase current` command (query DB for current phase)
3. ✅ `phase next` command (query DB for next pending phase)
4. ✅ `phase info <path>` command (get phase details from DB)
5. ✅ `phase list` command (list phases by category/status)
6. ✅ `validate` command (validate DB consistency)
7. ✅ Git integration (commit after DB updates)
8. ✅ Audit logging (all changes tracked)
9. ✅ Transaction safety (ACID guarantees)

---

## Implementation Steps

### Step 1: Phase Complete Command

**File:** `cmd/atlas-dev/phase_complete.go`

**Algorithm:**
1. Parse phase path → extract category, name
2. Begin transaction
3. Find phase in DB (validate exists)
4. Update phase status → 'completed'
5. Trigger auto-updates category progress (via SQL trigger)
6. Insert audit log entry
7. Commit transaction
8. Git commit (if --commit flag)
9. Return compact JSON

**Code:**

```go
package main

import (
    "database/sql"
    "encoding/json"
    "fmt"
    "path/filepath"
    "strings"
    "time"

    "github.com/atlas-lang/atlas-dev/internal/audit"
    "github.com/atlas-lang/atlas-dev/internal/db"
    "github.com/atlas-lang/atlas-dev/internal/git"
    "github.com/atlas-lang/atlas-dev/internal/lock"
    "github.com/atlas-lang/atlas-dev/internal/output"
    "github.com/spf13/cobra"
)

func phaseCompleteCmd() *cobra.Command {
    var (
        description string
        date        string
        commit      bool
        dryRun      bool
        testCount   int
    )

    cmd := &cobra.Command{
        Use:   "complete <phase-path>",
        Short: "Mark phase complete, update DB",
        Args:  cobra.ExactArgs(1),
        RunE: func(cmd *cobra.Command, args []string) error {
            phasePath := args[0]

            // Parse date (default: today)
            if date == "" {
                date = time.Now().Format("2006-01-02")
            }

            // Dry run mode
            if dryRun {
                return dryRunPhaseComplete(phasePath, description, date, testCount)
            }

            // Execute phase completion
            result, err := executePhaseComplete(phasePath, description, date, testCount, commit)
            if err != nil {
                return output.Error(err, nil)
            }

            return output.Success(result)
        },
    }

    cmd.Flags().StringVarP(&description, "desc", "d", "", "Phase completion description (required)")
    cmd.Flags().StringVar(&date, "date", "", "Completion date (default: today, YYYY-MM-DD)")
    cmd.Flags().BoolVarP(&commit, "commit", "c", false, "Auto-commit changes")
    cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Preview changes without modifying DB")
    cmd.Flags().IntVarP(&testCount, "tests", "t", 0, "Test count")
    cmd.MarkFlagRequired("desc")

    return cmd
}

func executePhaseComplete(phasePath, description, date string, testCount int, shouldCommit bool) (map[string]interface{}, error) {
    // Extract category from path: "phases/stdlib/phase-07b.md" → "stdlib"
    category := extractCategory(phasePath)
    phaseName := filepath.Base(phasePath)
    phaseName = strings.TrimSuffix(phaseName, filepath.Ext(phaseName))

    // Acquire lock (prevent concurrent updates)
    return lock.WithLock("atlas-dev.db", func() (map[string]interface{}, error) {
        // Begin transaction
        return db.WithTransaction(func(tx *db.Transaction) (map[string]interface{}, error) {
            // 1. Find phase in DB
            var phaseID int64
            var currentStatus string
            err := tx.QueryRow(`
                SELECT id, status FROM phases WHERE path = ?
            `, phasePath).Scan(&phaseID, &currentStatus)

            if err == sql.ErrNoRows {
                return nil, fmt.Errorf("phase not found: %s", phasePath)
            }
            if err != nil {
                return nil, fmt.Errorf("failed to find phase: %w", err)
            }

            // 2. Validate not already completed
            if currentStatus == "completed" {
                return nil, fmt.Errorf("phase already completed: %s", phasePath)
            }

            // 3. Update phase to completed
            completedDate := date + "T12:00:00Z"  // ISO 8601
            _, err = tx.Exec(`
                UPDATE phases
                SET status = 'completed',
                    completed_date = ?,
                    description = ?,
                    test_count = ?
                WHERE id = ?
            `, completedDate, description, testCount, phaseID)
            if err != nil {
                return nil, fmt.Errorf("failed to update phase: %w", err)
            }

            // 4. Get updated progress (triggers already updated categories)
            var catCompleted, catTotal, catPercent int
            err = tx.QueryRow(`
                SELECT completed, total, percentage
                FROM categories
                WHERE name = ?
            `, category).Scan(&catCompleted, &catTotal, &catPercent)
            if err != nil {
                return nil, fmt.Errorf("failed to get category progress: %w", err)
            }

            // 5. Get total progress
            var totalCompleted, totalPhases int
            err = tx.QueryRow(`
                SELECT
                    (SELECT COUNT(*) FROM phases WHERE status = 'completed'),
                    (SELECT COUNT(*) FROM phases)
            `).Scan(&totalCompleted, &totalPhases)
            if err != nil {
                return nil, fmt.Errorf("failed to get total progress: %w", err)
            }
            totalPercent := int(float64(totalCompleted) / float64(totalPhases) * 100)

            // 6. Find next phase
            var nextPath, nextName sql.NullString
            tx.QueryRow(`
                SELECT path, name
                FROM phases
                WHERE category = ? AND status = 'pending'
                ORDER BY id
                LIMIT 1
            `, category).Scan(&nextPath, &nextName)

            // 7. Insert audit log
            changes := map[string]interface{}{
                "phase":       phasePath,
                "status":      "completed",
                "description": description,
                "test_count":  testCount,
                "date":        date,
            }
            changesJSON, _ := json.Marshal(changes)

            _, err = tx.Exec(`
                INSERT INTO audit_log (action, entity_type, entity_id, changes)
                VALUES ('phase_complete', 'phase', ?, ?)
            `, fmt.Sprintf("%d", phaseID), string(changesJSON))
            if err != nil {
                return nil, fmt.Errorf("failed to insert audit log: %w", err)
            }

            // 8. Build response (compact JSON)
            result := map[string]interface{}{
                "phase": phaseName,
                "cat":   category,
                "progress": map[string]interface{}{
                    "cat": []int{catCompleted, catTotal, catPercent},
                    "tot": []int{totalCompleted, totalPhases, totalPercent},
                },
            }

            if nextPath.Valid {
                result["next"] = nextName.String
            }

            return result, nil
        })
    })
}

func dryRunPhaseComplete(phasePath, description, date string, testCount int) error {
    category := extractCategory(phasePath)
    phaseName := filepath.Base(phasePath)

    // Query what would change
    var phaseID int64
    var currentStatus string
    err := db.DB.QueryRow(`
        SELECT id, status FROM phases WHERE path = ?
    `, phasePath).Scan(&phaseID, &currentStatus)

    if err == sql.ErrNoRows {
        return fmt.Errorf("phase not found: %s", phasePath)
    }
    if err != nil {
        return fmt.Errorf("failed to find phase: %w", err)
    }

    if currentStatus == "completed" {
        return fmt.Errorf("phase already completed")
    }

    // Show what would happen
    result := map[string]interface{}{
        "dry_run":     true,
        "phase":       phaseName,
        "cat":         category,
        "changes":     "Set status='completed', description='" + description + "'",
        "note":        "No changes made (dry-run mode)",
    }

    return output.Success(result)
}

func extractCategory(phasePath string) string {
    // "phases/stdlib/phase-07b.md" → "stdlib"
    parts := strings.Split(filepath.ToSlash(phasePath), "/")
    if len(parts) >= 2 {
        return parts[1]
    }
    return ""
}
```

---

### Step 2: Phase Current Command

**File:** `cmd/atlas-dev/phase_current.go`

```go
package main

import (
    "database/sql"

    "github.com/atlas-lang/atlas-dev/internal/db"
    "github.com/atlas-lang/atlas-dev/internal/output"
    "github.com/spf13/cobra"
)

func phaseCurrentCmd() *cobra.Command {
    return &cobra.Command{
        Use:   "current",
        Short: "Show current phase",
        RunE: func(cmd *cobra.Command, args []string) error {
            // Query most recently completed phase
            var path, name, category, description string
            var completedDate sql.NullString

            err := db.DB.QueryRow(`
                SELECT path, name, category, description, completed_date
                FROM phases
                WHERE status = 'completed'
                ORDER BY completed_date DESC
                LIMIT 1
            `).Scan(&path, &name, &category, &description, &completedDate)

            if err == sql.ErrNoRows {
                return output.Success(map[string]interface{}{
                    "msg": "No phases completed yet",
                })
            }
            if err != nil {
                return output.Error(err, nil)
            }

            result := map[string]interface{}{
                "last_completed": name,
                "cat":            category,
                "path":           path,
            }

            if completedDate.Valid {
                result["date"] = completedDate.String[:10]  // YYYY-MM-DD
            }

            return output.Success(result)
        },
    }
}
```

---

### Step 3: Phase Next Command

**File:** `cmd/atlas-dev/phase_next.go`

```go
package main

import (
    "database/sql"

    "github.com/atlas-lang/atlas-dev/internal/db"
    "github.com/atlas-lang/atlas-dev/internal/output"
    "github.com/spf13/cobra"
)

func phaseNextCmd() *cobra.Command {
    var category string

    cmd := &cobra.Command{
        Use:   "next",
        Short: "Show next phase",
        RunE: func(cmd *cobra.Command, args []string) error {
            query := `
                SELECT path, name, category, description
                FROM phases
                WHERE status = 'pending'
            `

            var queryArgs []interface{}
            if category != "" {
                query += " AND category = ?"
                queryArgs = append(queryArgs, category)
            }

            query += " ORDER BY id LIMIT 1"

            var path, name, cat, description sql.NullString
            err := db.DB.QueryRow(query, queryArgs...).Scan(&path, &name, &cat, &description)

            if err == sql.ErrNoRows {
                return output.Success(map[string]interface{}{
                    "msg": "No pending phases",
                })
            }
            if err != nil {
                return output.Error(err, nil)
            }

            result := map[string]interface{}{
                "next": name.String,
                "cat":  cat.String,
                "path": path.String,
            }

            if description.Valid && description.String != "" {
                result["desc"] = description.String
            }

            return output.Success(result)
        },
    }

    cmd.Flags().StringVarP(&category, "category", "c", "", "Filter by category")

    return cmd
}
```

---

### Step 4: Phase Info Command

**File:** `cmd/atlas-dev/phase_info.go`

```go
package main

import (
    "database/sql"
    "encoding/json"

    "github.com/atlas-lang/atlas-dev/internal/db"
    "github.com/atlas-lang/atlas-dev/internal/output"
    "github.com/spf13/cobra"
)

func phaseInfoCmd() *cobra.Command {
    return &cobra.Command{
        Use:   "info <phase-path>",
        Short: "Get phase details",
        Args:  cobra.ExactArgs(1),
        RunE: func(cmd *cobra.Command, args []string) error {
            phasePath := args[0]

            var (
                id                                    int64
                name, category, status, description   string
                completedDate                         sql.NullString
                testCount, testTarget                 sql.NullInt64
                acceptanceCriteria, blockers, deps    sql.NullString
            )

            err := db.DB.QueryRow(`
                SELECT
                    id, name, category, status, description,
                    completed_date, test_count, test_target,
                    acceptance_criteria, blockers, dependencies
                FROM phases
                WHERE path = ?
            `, phasePath).Scan(
                &id, &name, &category, &status, &description,
                &completedDate, &testCount, &testTarget,
                &acceptanceCriteria, &blockers, &deps,
            )

            if err == sql.ErrNoRows {
                return output.Error(fmt.Errorf("phase not found: %s", phasePath), nil)
            }
            if err != nil {
                return output.Error(err, nil)
            }

            result := map[string]interface{}{
                "id":     id,
                "name":   name,
                "cat":    category,
                "status": status,
            }

            if description != "" {
                result["desc"] = description
            }

            if completedDate.Valid {
                result["completed"] = completedDate.String[:10]
            }

            if testCount.Valid {
                result["tests"] = []int{int(testCount.Int64), int(testTarget.Int64)}
            }

            if acceptanceCriteria.Valid {
                var criteria []string
                json.Unmarshal([]byte(acceptanceCriteria.String), &criteria)
                result["acceptance"] = criteria
            }

            if blockers.Valid {
                var blockersList []string
                json.Unmarshal([]byte(blockers.String), &blockersList)
                result["blk"] = blockersList
            }

            if deps.Valid {
                var depsList []string
                json.Unmarshal([]byte(deps.String), &depsList)
                result["dep"] = depsList
            }

            return output.Success(result)
        },
    }
}
```

---

### Step 5: Phase List Command

**File:** `cmd/atlas-dev/phase_list.go`

```go
package main

import (
    "github.com/atlas-lang/atlas-dev/internal/db"
    "github.com/atlas-lang/atlas-dev/internal/output"
    "github.com/spf13/cobra"
)

func phaseListCmd() *cobra.Command {
    var (
        category string
        status   string
        limit    int
    )

    cmd := &cobra.Command{
        Use:   "list",
        Short: "List phases",
        RunE: func(cmd *cobra.Command, args []string) error {
            query := "SELECT name, category, status FROM phases WHERE 1=1"
            var queryArgs []interface{}

            if category != "" {
                query += " AND category = ?"
                queryArgs = append(queryArgs, category)
            }

            if status != "" {
                query += " AND status = ?"
                queryArgs = append(queryArgs, status)
            }

            query += " ORDER BY id"

            if limit > 0 {
                query += " LIMIT ?"
                queryArgs = append(queryArgs, limit)
            }

            rows, err := db.DB.Query(query, queryArgs...)
            if err != nil {
                return output.Error(err, nil)
            }
            defer rows.Close()

            var phases [][]string
            for rows.Next() {
                var name, cat, stat string
                if err := rows.Scan(&name, &cat, &stat); err != nil {
                    return output.Error(err, nil)
                }
                phases = append(phases, []string{name, cat, stat})
            }

            return output.Success(map[string]interface{}{
                "phases": phases,
                "cnt":    len(phases),
            })
        },
    }

    cmd.Flags().StringVarP(&category, "category", "c", "", "Filter by category")
    cmd.Flags().StringVarP(&status, "status", "s", "", "Filter by status (pending|completed|blocked)")
    cmd.Flags().IntVarP(&limit, "limit", "n", 0, "Limit results")

    return cmd
}
```

---

### Step 6: Validate Command

**File:** `cmd/atlas-dev/validate.go`

```go
package main

import (
    "github.com/atlas-lang/atlas-dev/internal/db"
    "github.com/atlas-lang/atlas-dev/internal/output"
    "github.com/spf13/cobra"
)

func validateCmd() *cobra.Command {
    return &cobra.Command{
        Use:   "validate",
        Short: "Validate DB consistency",
        RunE: func(cmd *cobra.Command, args []string) error {
            // 1. Verify total completed matches sum of categories
            var totalFromPhases, totalFromMetadata int

            db.DB.QueryRow("SELECT COUNT(*) FROM phases WHERE status = 'completed'").Scan(&totalFromPhases)
            db.DB.QueryRow("SELECT CAST(value AS INTEGER) FROM metadata WHERE key = 'completed_phases'").Scan(&totalFromMetadata)

            if totalFromPhases != totalFromMetadata {
                return output.Error(
                    fmt.Errorf("completed count mismatch"),
                    map[string]interface{}{
                        "phases":   totalFromPhases,
                        "metadata": totalFromMetadata,
                    },
                )
            }

            // 2. Verify each category completed count matches phases table
            type CategoryCheck struct {
                Name      string
                Completed int
                Total     int
            }

            rows, err := db.DB.Query(`
                SELECT name, completed, total FROM categories ORDER BY id
            `)
            if err != nil {
                return output.Error(err, nil)
            }
            defer rows.Close()

            var categories []CategoryCheck
            var totalCompleted, totalPhases int

            for rows.Next() {
                var cat CategoryCheck
                rows.Scan(&cat.Name, &cat.Completed, &cat.Total)

                // Verify against phases table
                var actualCompleted int
                db.DB.QueryRow(`
                    SELECT COUNT(*) FROM phases
                    WHERE category = ? AND status = 'completed'
                `, cat.Name).Scan(&actualCompleted)

                if actualCompleted != cat.Completed {
                    return output.Error(
                        fmt.Errorf("category %s mismatch", cat.Name),
                        map[string]interface{}{
                            "cat":      cat.Name,
                            "expected": actualCompleted,
                            "actual":   cat.Completed,
                        },
                    )
                }

                categories = append(categories, cat)
                totalCompleted += cat.Completed
                totalPhases += cat.Total
            }

            // 3. All checks passed
            return output.Success(map[string]interface{}{
                "completed": totalCompleted,
                "total":     totalPhases,
                "pct":       int(float64(totalCompleted) / float64(totalPhases) * 100),
                "cats":      len(categories),
            })
        },
    }
}
```

---

### Step 7: Git Integration

**File:** `internal/git/commit.go`

```go
package git

import (
    "fmt"
    "os"
    "os/exec"
    "strings"
)

// Commit creates a git commit
func Commit(message string) (string, error) {
    // Stage atlas-dev.db
    cmd := exec.Command("git", "add", "atlas-dev.db")
    if output, err := cmd.CombinedOutput(); err != nil {
        return "", fmt.Errorf("git add failed: %s", output)
    }

    // Commit
    cmd = exec.Command("git", "commit", "-m", message)
    if output, err := cmd.CombinedOutput(); err != nil {
        // Check if it's "nothing to commit"
        if strings.Contains(string(output), "nothing to commit") {
            return "", fmt.Errorf("nothing to commit")
        }
        return "", fmt.Errorf("git commit failed: %s", output)
    }

    // Get commit SHA
    cmd = exec.Command("git", "rev-parse", "HEAD")
    output, err := cmd.Output()
    if err != nil {
        return "", fmt.Errorf("failed to get commit SHA: %w", err)
    }

    sha := strings.TrimSpace(string(output))
    return sha[:7], nil  // Short SHA
}

// CommitWithCoAuthor creates a commit with co-author
func CommitWithCoAuthor(message string) (string, error) {
    fullMessage := fmt.Sprintf("%s\n\nCo-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>", message)
    return Commit(fullMessage)
}
```

---

### Step 8: Update Main CLI

**File:** `cmd/atlas-dev/main.go` (add phase commands)

```go
// Add to main.go

func main() {
    rootCmd := &cobra.Command{
        Use:   "atlas-dev",
        Short: "Atlas development automation",
        // ... (existing setup)
    }

    // Phase commands
    phaseCmd := &cobra.Command{
        Use:   "phase",
        Short: "Phase management",
    }
    phaseCmd.AddCommand(phaseCompleteCmd())
    phaseCmd.AddCommand(phaseCurrentCmd())
    phaseCmd.AddCommand(phaseNextCmd())
    phaseCmd.AddCommand(phaseInfoCmd())
    phaseCmd.AddCommand(phaseListCmd())

    rootCmd.AddCommand(phaseCmd)
    rootCmd.AddCommand(validateCmd())
    rootCmd.AddCommand(versionCmd())
    rootCmd.AddCommand(migrateCmd())

    // Execute
    if err := rootCmd.Execute(); err != nil {
        output.Error(err, nil)
        os.Exit(1)
    }
}
```

---

## Testing

### Manual Tests

```bash
# Build
make build

# Complete a phase
./bin/atlas-dev phase complete "phases/stdlib/phase-07b-hashset.md" \
  -d "HashSet with 25 tests, 100% parity" \
  --tests 25 \
  --commit

# Expected output (compact JSON):
# {"ok":true,"phase":"phase-07b-hashset","cat":"stdlib","progress":{"cat":[10,21,48],"tot":[31,78,40]},"next":"phase-07c-queue-stack"}

# Get current phase
./bin/atlas-dev phase current
# {"ok":true,"last_completed":"phase-07b-hashset","cat":"stdlib","path":"phases/stdlib/phase-07b-hashset.md","date":"2026-02-15"}

# Get next phase
./bin/atlas-dev phase next
# {"ok":true,"next":"phase-07c-queue-stack","cat":"stdlib","path":"phases/stdlib/phase-07c-queue-stack.md"}

# Get phase info
./bin/atlas-dev phase info "phases/stdlib/phase-07b-hashset.md"
# {"ok":true,"id":32,"name":"phase-07b-hashset","cat":"stdlib","status":"completed","desc":"HashSet with 25 tests","completed":"2026-02-15","tests":[25,25]}

# List phases
./bin/atlas-dev phase list -c stdlib -s pending
# {"ok":true,"phases":[["phase-07c-queue-stack","stdlib","pending"],...],"cnt":11}

# Validate
./bin/atlas-dev validate
# {"ok":true,"completed":31,"total":78,"pct":40,"cats":9}
```

---

## Acceptance Criteria

### Functional Requirements
- [ ] `phase complete` updates DB atomically
- [ ] Triggers auto-update category progress
- [ ] `phase complete --commit` creates git commit
- [ ] `phase complete --dry-run` shows preview without changes
- [ ] `phase current` returns last completed phase
- [ ] `phase next` returns next pending phase
- [ ] `phase info` returns full phase details
- [ ] `phase list` filters by category/status
- [ ] `validate` detects inconsistencies
- [ ] All commands use transactions (ACID)
- [ ] File locking prevents concurrent corruption
- [ ] Audit log tracks all changes
- [ ] Git commits reference phase completion

### Token Efficiency Requirements
- [ ] `phase complete` output < 120 tokens
- [ ] `phase current` output < 60 tokens
- [ ] `phase next` output < 60 tokens
- [ ] `phase info` output < 150 tokens
- [ ] JSON uses compact notation (abbreviated fields)
- [ ] Arrays used for tuples: `[10,21,48]`
- [ ] Null/empty fields omitted

### Database Requirements
- [ ] All updates use transactions
- [ ] Triggers fire correctly (auto-update categories)
- [ ] No race conditions (file locking)
- [ ] Audit log contains all phase completions
- [ ] Database remains consistent (validated)

---

## Token Efficiency Examples

### phase complete
```bash
# Old (markdown): ~150 tokens
✅ Phase marked complete: phase-07b-hashset.md
📊 Progress:
   Category: stdlib (10/21 = 48%)
   Total: 31/78 (40%)
📝 Updates: status/trackers/1-stdlib.md, STATUS.md
⏭️  Next: phase-07c-queue-stack.md

# New (SQLite): ~40 tokens
{"ok":true,"phase":"phase-07b-hashset","cat":"stdlib","progress":{"cat":[10,21,48],"tot":[31,78,40]},"next":"phase-07c-queue-stack"}
```

**73% token reduction** ✅

---

## Next Phase

**Phase 3:** Decision Log Integration
- Decision CRUD operations (DB-based)
- `decision create` / `list` / `search` commands
- Auto-generate DR-XXX IDs from DB sequence
