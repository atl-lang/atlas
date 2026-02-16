package audit

import (
	"encoding/json"
	"fmt"
	"log/slog"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

// Log inserts an audit log entry
func Log(database *db.DB, action, entityType, entityID string, changes interface{}) error {
	// Convert changes to JSON
	changesJSON, err := json.Marshal(changes)
	if err != nil {
		return fmt.Errorf("failed to marshal changes: %w", err)
	}

	err = database.InsertAuditLog(action, entityType, entityID, string(changesJSON), "", "")
	if err != nil {
		slog.Error("failed to insert audit log",
			"action", action,
			"entity_type", entityType,
			"entity_id", entityID,
			"error", err,
		)
		return err
	}

	slog.Debug("audit log recorded",
		"action", action,
		"entity_type", entityType,
		"entity_id", entityID,
	)

	return nil
}

// LogWithCommit inserts an audit log entry with git commit SHA
func LogWithCommit(database *db.DB, action, entityType, entityID string, changes interface{}, commitSHA, agent string) error {
	// Convert changes to JSON
	changesJSON, err := json.Marshal(changes)
	if err != nil {
		return fmt.Errorf("failed to marshal changes: %w", err)
	}

	err = database.InsertAuditLog(action, entityType, entityID, string(changesJSON), commitSHA, agent)
	if err != nil {
		slog.Error("failed to insert audit log",
			"action", action,
			"entity_type", entityType,
			"entity_id", entityID,
			"commit_sha", commitSHA,
			"error", err,
		)
		return err
	}

	slog.Debug("audit log recorded with commit",
		"action", action,
		"entity_type", entityType,
		"entity_id", entityID,
		"commit_sha", commitSHA,
	)

	return nil
}

// GetRecent retrieves last N audit entries
func GetRecent(database *db.DB, limit int) ([]AuditEntry, error) {
	rows, err := database.Query(`
		SELECT id, timestamp, action, entity_type, entity_id, changes, commit_sha, agent
		FROM audit_log
		ORDER BY timestamp DESC
		LIMIT ?
	`, limit)
	if err != nil {
		return nil, err
	}
	defer func() { _ = rows.Close() }()

	var entries []AuditEntry
	for rows.Next() {
		var e AuditEntry
		if err := rows.Scan(
			&e.ID, &e.Timestamp, &e.Action, &e.EntityType,
			&e.EntityID, &e.Changes, &e.CommitSHA, &e.Agent,
		); err != nil {
			return nil, err
		}
		entries = append(entries, e)
	}

	return entries, rows.Err()
}

// AuditEntry represents an audit log entry
type AuditEntry struct {
	ID         int
	Timestamp  string
	Action     string
	EntityType string
	EntityID   string
	Changes    string
	CommitSHA  string
	Agent      string
}
