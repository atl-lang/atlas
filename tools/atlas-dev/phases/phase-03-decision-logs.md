# Phase 03: Decision Log Integration (SQLite)

**Objective:** Implement decision log management with pure SQLite - create, list, search, track decisions.

**Priority:** HIGH
**Depends On:** Phase 2
**Architecture:** Pure SQLite (decisions table)

---

## Deliverables

1. ✅ `decision create` command (insert into DB)
2. ✅ `decision list` command (query DB)
3. ✅ `decision search` command (full-text search)
4. ✅ `decision read <id>` command (get decision details)
5. ✅ `decision next-id <component>` command (auto-increment DR-XXX)
6. ✅ `decision update <id>` command (update status/supersede)
7. ✅ Auto-generate decision IDs (DR-001, DR-002, etc.)
8. ✅ Audit logging
9. ✅ Optional markdown export (`decision export`)

---

## Implementation

### Step 1: Decision Create Command

```go
func decisionCreateCmd() *cobra.Command {
    var (
        component    string
        title        string
        decision     string
        rationale    string
        alternatives string
        status       string
    )

    cmd := &cobra.Command{
        Use:   "create",
        Short: "Create decision log",
        RunE: func(cmd *cobra.Command, args []string) error {
            // Auto-generate ID
            id, err := getNextDecisionID(component)
            if err != nil {
                return output.Error(err, nil)
            }

            date := time.Now().Format("2006-01-02")

            // Insert into DB
            return db.WithTransaction(func(tx *db.Transaction) error {
                _, err := tx.Exec(`
                    INSERT INTO decisions (
                        id, component, title, decision, rationale,
                        alternatives, date, status, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
                `, id, component, title, decision, rationale, alternatives, date, status)

                if err != nil {
                    return err
                }

                // Audit log
                audit.Log("decision_create", "decision", id, map[string]interface{}{
                    "component": component,
                    "title":     title,
                })

                return output.Success(map[string]interface{}{
                    "id":   id,
                    "comp": component,
                    "date": date,
                })
            })
        },
    }

    cmd.Flags().StringVar(&component, "component", "", "Component (required)")
    cmd.Flags().StringVar(&title, "title", "", "Decision title (required)")
    cmd.Flags().StringVar(&decision, "decision", "", "Decision text")
    cmd.Flags().StringVar(&rationale, "rationale", "", "Rationale")
    cmd.Flags().StringVar(&alternatives, "alternatives", "", "Alternatives considered")
    cmd.Flags().StringVar(&status, "status", "accepted", "Status (accepted|rejected|proposed)")
    cmd.MarkFlagRequired("component")
    cmd.MarkFlagRequired("title")

    return cmd
}

func getNextDecisionID(component string) (string, error) {
    var maxID int
    err := db.DB.QueryRow(`
        SELECT COALESCE(MAX(CAST(SUBSTR(id, 4) AS INTEGER)), 0)
        FROM decisions
    `).Scan(&maxID)

    if err != nil {
        return "", err
    }

    return fmt.Sprintf("DR-%03d", maxID+1), nil
}
```

### Step 2: Decision List/Search

```go
func decisionListCmd() *cobra.Command {
    var (
        component string
        status    string
        limit     int
    )

    cmd := &cobra.Command{
        Use:   "list",
        Short: "List decisions",
        RunE: func(cmd *cobra.Command, args []string) error {
            query := "SELECT id, component, title, date, status FROM decisions WHERE 1=1"
            var args []interface{}

            if component != "" {
                query += " AND component = ?"
                args = append(args, component)
            }

            if status != "" {
                query += " AND status = ?"
                args = append(args, status)
            }

            query += " ORDER BY date DESC LIMIT ?"
            args = append(args, limit)

            rows, err := db.DB.Query(query, args...)
            if err != nil {
                return output.Error(err, nil)
            }
            defer rows.Close()

            var decisions [][]string
            for rows.Next() {
                var id, comp, title, date, stat string
                rows.Scan(&id, &comp, &title, &date, &stat)
                decisions = append(decisions, []string{id, comp, title, date, stat})
            }

            return output.Success(map[string]interface{}{
                "decisions": decisions,
                "cnt":       len(decisions),
            })
        },
    }

    cmd.Flags().StringVarP(&component, "component", "c", "", "Filter by component")
    cmd.Flags().StringVarP(&status, "status", "s", "accepted", "Filter by status")
    cmd.Flags().IntVarP(&limit, "limit", "n", 20, "Limit results")

    return cmd
}
```

### Step 3: Decision Search

```go
func decisionSearchCmd() *cobra.Command {
    return &cobra.Command{
        Use:   "search <query>",
        Short: "Search decisions",
        Args:  cobra.ExactArgs(1),
        RunE: func(cmd *cobra.Command, args []string) error {
            searchQuery := "%" + args[0] + "%"

            rows, err := db.DB.Query(`
                SELECT id, component, title, date
                FROM decisions
                WHERE title LIKE ? OR decision LIKE ? OR rationale LIKE ?
                ORDER BY date DESC
                LIMIT 20
            `, searchQuery, searchQuery, searchQuery)

            if err != nil {
                return output.Error(err, nil)
            }
            defer rows.Close()

            var results [][]string
            for rows.Next() {
                var id, comp, title, date string
                rows.Scan(&id, &comp, &title, &date)
                results = append(results, []string{id, comp, title, date})
            }

            return output.Success(map[string]interface{}{
                "results": results,
                "cnt":     len(results),
            })
        },
    }
}
```

---

## Acceptance Criteria

- [ ] `decision create` inserts into DB
- [ ] Auto-generates DR-XXX IDs
- [ ] `decision list` filters by component/status
- [ ] `decision search` performs full-text search
- [ ] `decision read` returns full details
- [ ] Audit log tracks all changes
- [ ] JSON output < 80 tokens for list
- [ ] JSON output < 150 tokens for details

---

## Token Efficiency

```bash
# List decisions: ~60 tokens
{"ok":true,"decisions":[["DR-001","stdlib","Hash function","2026-01-15","accepted"]],"cnt":1}

# Create decision: ~30 tokens
{"ok":true,"id":"DR-007","comp":"stdlib","date":"2026-02-15"}
```

---

## Next Phase

**Phase 4:** Progress Analytics & Validation
