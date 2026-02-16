package db

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"log/slog"
	"time"
)

// PhaseDetails represents complete phase information
type PhaseDetails struct {
	ID                 int
	Path               string
	Name               string
	Category           string
	Status             string
	CompletedDate      sql.NullString
	Description        sql.NullString
	TestCount          int
	TestTarget         sql.NullInt64
	AcceptanceCriteria sql.NullString
	Blockers           sql.NullString
	Dependencies       sql.NullString
	FilesModified      sql.NullString
	CreatedAt          string
	UpdatedAt          string
}

// PhaseListItem represents a phase in list view
type PhaseListItem struct {
	Path          string
	Name          string
	Category      string
	Status        string
	CompletedDate sql.NullString
}

// CompletePhaseRequest represents parameters for completing a phase
type CompletePhaseRequest struct {
	PhasePath   string
	Description string
	Date        string // ISO 8601 format
	TestCount   int
	DryRun      bool
}

// CompletePhaseResult represents the result of completing a phase
type CompletePhaseResult struct {
	PhaseID          int
	PhaseName        string
	Category         string
	CategoryProgress ProgressInfo
	TotalProgress    ProgressInfo
	NextPhase        *PhaseListItem
	FilesModified    []string
}

// ProgressInfo represents progress statistics
type ProgressInfo struct {
	Completed  int
	Total      int
	Percentage int
}

// CompletePhase atomically marks a phase as complete
func (db *DB) CompletePhase(req CompletePhaseRequest) (*CompletePhaseResult, error) {
	start := time.Now()

	// Get phase by path
	phase, err := db.GetPhaseByPath(req.PhasePath)
	if err != nil {
		return nil, err
	}

	// Check if already completed
	if phase.Status == "completed" {
		return nil, ErrPhaseAlreadyDone
	}

	// If dry-run, return what would happen
	if req.DryRun {
		slog.Debug("dry-run mode, skipping actual update", "phase", phase.Name)
		return db.buildCompleteResult(phase, req)
	}

	var result *CompletePhaseResult

	// Use exclusive lock + transaction for atomic update
	err = db.WithExclusiveLock(func() error {
		return db.WithTransaction(func(tx *Transaction) error {
			// Update phase
			_, err := tx.Exec(`
				UPDATE phases
				SET status = 'completed',
				    completed_date = ?,
				    description = ?,
				    test_count = ?
				WHERE id = ?
			`, req.Date, req.Description, req.TestCount, phase.ID)
			if err != nil {
				return fmt.Errorf("failed to update phase: %w", err)
			}

			slog.Debug("phase updated",
				"phase", phase.Name,
				"duration_ms", time.Since(start).Milliseconds())

			return nil
		})
	})

	if err != nil {
		return nil, err
	}

	// Build result AFTER transaction (triggers have auto-updated categories/metadata)
	result, err = db.buildCompleteResult(phase, req)
	if err != nil {
		return nil, err
	}

	return result, nil
}

// buildCompleteResult builds the completion result with updated progress
func (db *DB) buildCompleteResult(phase *Phase, req CompletePhaseRequest) (*CompletePhaseResult, error) {
	// Get updated category progress
	cat, err := db.GetCategory(phase.Category)
	if err != nil {
		return nil, fmt.Errorf("failed to get category progress: %w", err)
	}

	// Get total progress
	total, err := db.GetTotalProgress()
	if err != nil {
		return nil, fmt.Errorf("failed to get total progress: %w", err)
	}

	// Get next phase in category
	nextPhase, _ := db.GetNextPhaseInCategory(phase.Category)

	result := &CompletePhaseResult{
		PhaseID:   phase.ID,
		PhaseName: phase.Name,
		Category:  phase.Category,
		CategoryProgress: ProgressInfo{
			Completed:  cat.Completed,
			Total:      cat.Total,
			Percentage: cat.Percentage,
		},
		TotalProgress: ProgressInfo{
			Completed:  total.TotalCompleted,
			Total:      total.TotalPhases,
			Percentage: total.Percentage,
		},
		NextPhase:     nextPhase,
		FilesModified: []string{"atlas-dev.db"},
	}

	return result, nil
}

// GetCurrentPhase returns the most recently completed phase
func (db *DB) GetCurrentPhase() (*PhaseDetails, error) {
	query := `
		SELECT id, path, name, category, status, completed_date, description, test_count,
		       test_target, acceptance_criteria, blockers, dependencies, files_modified,
		       created_at, updated_at
		FROM phases
		WHERE status = 'completed'
		ORDER BY completed_date DESC
		LIMIT 1
	`

	var p PhaseDetails
	err := db.conn.QueryRow(query).Scan(
		&p.ID, &p.Path, &p.Name, &p.Category, &p.Status, &p.CompletedDate,
		&p.Description, &p.TestCount, &p.TestTarget, &p.AcceptanceCriteria,
		&p.Blockers, &p.Dependencies, &p.FilesModified, &p.CreatedAt, &p.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, nil // No completed phases yet
	}

	if err != nil {
		return nil, err
	}

	return &p, nil
}

// GetNextPhase returns the next pending phase
func (db *DB) GetNextPhase(categoryFilter string) (*PhaseDetails, error) {
	var query string
	var args []interface{}

	if categoryFilter != "" {
		// Get next in specified category
		query = `
			SELECT id, path, name, category, status, completed_date, description, test_count,
			       test_target, acceptance_criteria, blockers, dependencies, files_modified,
			       created_at, updated_at
			FROM phases
			WHERE status = 'pending' AND category = ?
			ORDER BY id
			LIMIT 1
		`
		args = []interface{}{categoryFilter}
	} else {
		// Get next phase in category of last completed
		current, err := db.GetCurrentPhase()
		if err != nil {
			return nil, err
		}

		if current == nil {
			// No phases completed, get first pending
			query = `
				SELECT id, path, name, category, status, completed_date, description, test_count,
				       test_target, acceptance_criteria, blockers, dependencies, files_modified,
				       created_at, updated_at
				FROM phases
				WHERE status = 'pending'
				ORDER BY id
				LIMIT 1
			`
		} else {
			// Get next in same category
			query = `
				SELECT id, path, name, category, status, completed_date, description, test_count,
				       test_target, acceptance_criteria, blockers, dependencies, files_modified,
				       created_at, updated_at
				FROM phases
				WHERE status = 'pending' AND category = ?
				ORDER BY id
				LIMIT 1
			`
			args = []interface{}{current.Category}
		}
	}

	var p PhaseDetails
	var err error

	if len(args) > 0 {
		err = db.conn.QueryRow(query, args...).Scan(
			&p.ID, &p.Path, &p.Name, &p.Category, &p.Status, &p.CompletedDate,
			&p.Description, &p.TestCount, &p.TestTarget, &p.AcceptanceCriteria,
			&p.Blockers, &p.Dependencies, &p.FilesModified, &p.CreatedAt, &p.UpdatedAt,
		)
	} else {
		err = db.conn.QueryRow(query).Scan(
			&p.ID, &p.Path, &p.Name, &p.Category, &p.Status, &p.CompletedDate,
			&p.Description, &p.TestCount, &p.TestTarget, &p.AcceptanceCriteria,
			&p.Blockers, &p.Dependencies, &p.FilesModified, &p.CreatedAt, &p.UpdatedAt,
		)
	}

	if err == sql.ErrNoRows {
		return nil, nil // All phases complete
	}

	if err != nil {
		return nil, err
	}

	return &p, nil
}

// GetNextPhaseInCategory returns next pending phase in specified category
func (db *DB) GetNextPhaseInCategory(category string) (*PhaseListItem, error) {
	query := `
		SELECT path, name, category, status, completed_date
		FROM phases
		WHERE status = 'pending' AND category = ?
		ORDER BY id
		LIMIT 1
	`

	var p PhaseListItem
	err := db.conn.QueryRow(query, category).Scan(
		&p.Path, &p.Name, &p.Category, &p.Status, &p.CompletedDate,
	)

	if err == sql.ErrNoRows {
		return nil, nil // No more phases in category
	}

	if err != nil {
		return nil, err
	}

	return &p, nil
}

// GetPhaseInfo returns detailed phase information by path
func (db *DB) GetPhaseInfo(phasePath string) (*PhaseDetails, error) {
	query := `
		SELECT id, path, name, category, status, completed_date, description, test_count,
		       test_target, acceptance_criteria, blockers, dependencies, files_modified,
		       created_at, updated_at
		FROM phases
		WHERE path = ?
	`

	var p PhaseDetails
	err := db.conn.QueryRow(query, phasePath).Scan(
		&p.ID, &p.Path, &p.Name, &p.Category, &p.Status, &p.CompletedDate,
		&p.Description, &p.TestCount, &p.TestTarget, &p.AcceptanceCriteria,
		&p.Blockers, &p.Dependencies, &p.FilesModified, &p.CreatedAt, &p.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, ErrPhaseNotFound
	}

	if err != nil {
		return nil, err
	}

	return &p, nil
}

// ListPhasesOptions represents filtering options for listing phases
type ListPhasesOptions struct {
	Category string
	Status   string
	Limit    int
	Offset   int
}

// ListPhases returns phases matching the filter criteria
func (db *DB) ListPhases(opts ListPhasesOptions) ([]*PhaseListItem, error) {
	query := `
		SELECT path, name, category, status, completed_date
		FROM phases
		WHERE 1=1
	`
	args := []interface{}{}

	// Add filters
	if opts.Category != "" {
		query += " AND category = ?"
		args = append(args, opts.Category)
	}

	if opts.Status != "" {
		query += " AND status = ?"
		args = append(args, opts.Status)
	}

	// Order by category, then ID
	query += " ORDER BY category, id"

	// Add pagination
	if opts.Limit > 0 {
		query += " LIMIT ?"
		args = append(args, opts.Limit)
	}

	if opts.Offset > 0 {
		query += " OFFSET ?"
		args = append(args, opts.Offset)
	}

	rows, err := db.conn.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer func() { _ = rows.Close() }()

	var phases []*PhaseListItem
	for rows.Next() {
		var p PhaseListItem
		if err := rows.Scan(&p.Path, &p.Name, &p.Category, &p.Status, &p.CompletedDate); err != nil {
			return nil, err
		}
		phases = append(phases, &p)
	}

	return phases, rows.Err()
}

// CountPhases returns the count of phases matching the filter criteria (no data fetch)
func (db *DB) CountPhases(opts ListPhasesOptions) (int, error) {
	start := time.Now()

	query := `
		SELECT COUNT(*)
		FROM phases
		WHERE 1=1
	`
	args := []interface{}{}

	// Add filters (same as ListPhases)
	if opts.Category != "" {
		query += " AND category = ?"
		args = append(args, opts.Category)
	}

	if opts.Status != "" {
		query += " AND status = ?"
		args = append(args, opts.Status)
	}

	var count int
	err := db.conn.QueryRow(query, args...).Scan(&count)
	if err != nil {
		return 0, err
	}

	duration := time.Since(start)
	slog.Debug("query completed",
		"query", "countPhases",
		"category", opts.Category,
		"status", opts.Status,
		"count", count,
		"duration_ms", duration.Milliseconds(),
	)

	return count, nil
}

// InsertPhase inserts a new phase into the database
func (db *DB) InsertPhase(path, name, category string) (int64, error) {
	result, err := db.Exec(`
		INSERT INTO phases (path, name, category, status)
		VALUES (?, ?, ?, 'pending')
	`, path, name, category)
	if err != nil {
		return 0, err
	}

	return result.LastInsertId()
}

// ToCompactJSON converts PhaseDetails to compact JSON map
func (p *PhaseDetails) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"id":   p.ID,
		"path": p.Path,
		"name": p.Name,
		"cat":  p.Category,
		"sts":  p.Status,
	}

	if p.CompletedDate.Valid {
		result["date"] = p.CompletedDate.String
	}

	if p.Description.Valid {
		result["desc"] = p.Description.String
	}

	if p.TestCount > 0 {
		result["tests"] = p.TestCount
	}

	if p.TestTarget.Valid && p.TestTarget.Int64 > 0 {
		result["target"] = p.TestTarget.Int64
	}

	if p.AcceptanceCriteria.Valid {
		var criteria []string
		if err := json.Unmarshal([]byte(p.AcceptanceCriteria.String), &criteria); err == nil {
			result["accept"] = criteria
		}
	}

	if p.Blockers.Valid {
		var blockers []string
		if err := json.Unmarshal([]byte(p.Blockers.String), &blockers); err == nil && len(blockers) > 0 {
			result["blk"] = blockers
		}
	}

	if p.Dependencies.Valid {
		var deps []string
		if err := json.Unmarshal([]byte(p.Dependencies.String), &deps); err == nil && len(deps) > 0 {
			result["dep"] = deps
		}
	}

	return result
}
