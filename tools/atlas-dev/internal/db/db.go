package db

import (
	"database/sql"
	"fmt"
	"log/slog"
	"sync"
	"time"

	_ "github.com/mattn/go-sqlite3"
)

// DB is the database handle with prepared statements
type DB struct {
	conn  *sql.DB
	stmts map[string]*sql.Stmt
	mu    sync.RWMutex
}

// New creates a new database connection
func New(path string) (*DB, error) {
	// Open with WAL mode + foreign keys + busy timeout
	conn, err := sql.Open("sqlite3",
		path+"?_journal_mode=WAL&_foreign_keys=ON&_busy_timeout=5000")
	if err != nil {
		return nil, fmt.Errorf("failed to open database: %w", err)
	}

	// SQLite: single writer (prevents lock contention)
	conn.SetMaxOpenConns(1)

	// Test connection
	if err := conn.Ping(); err != nil {
		return nil, fmt.Errorf("database ping failed: %w", err)
	}

	db := &DB{
		conn:  conn,
		stmts: make(map[string]*sql.Stmt),
	}

	// Check if schema exists (look for phases table)
	var count int
	err = conn.QueryRow("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='phases'").Scan(&count)
	if err == nil && count > 0 {
		// Schema exists, prepare statements
		if err := db.prepare(); err != nil {
			_ = conn.Close() // Best effort cleanup
			return nil, err
		}
		slog.Debug("prepared statements cached", "count", len(db.stmts))
	}

	slog.Debug("database opened successfully", "path", path)

	return db, nil
}

// Prepare prepares all common queries (call after schema initialization)
func (db *DB) Prepare() error {
	return db.prepare()
}

// prepare caches common queries as prepared statements
func (db *DB) prepare() error {
	queries := map[string]string{
		"getPhase": `
			SELECT id, path, name, category, status, completed_date, description, test_count
			FROM phases WHERE id = ?`,
		"getPhaseByPath": `
			SELECT id, path, name, category, status, completed_date, description, test_count
			FROM phases WHERE path = ?`,
		"updatePhaseStatus": `
			UPDATE phases
			SET status = ?, completed_date = ?, description = ?, test_count = ?
			WHERE id = ?`,
		"listPhases": `
			SELECT id, path, name, category, status
			FROM phases
			WHERE category = ?
			ORDER BY id`,
		"listAllPhases": `
			SELECT id, path, name, category, status
			FROM phases
			ORDER BY category, id`,
		"getCategory": `
			SELECT completed, total, percentage
			FROM categories WHERE name = ?`,
		"getCategoryProgress": `
			SELECT id, name, display_name, completed, total, percentage, status
			FROM categories
			WHERE name = ?`,
		"getTotalProgress": `
			SELECT
				SUM(completed) as total_completed,
				SUM(total) as total_phases,
				ROUND(CAST(SUM(completed) AS REAL) / SUM(total) * 100) as percentage
			FROM categories`,
		"getMetadata": `
			SELECT value FROM metadata WHERE key = ?`,
		"setMetadata": `
			INSERT OR REPLACE INTO metadata (key, value, updated_at)
			VALUES (?, ?, datetime('now'))`,
		"insertAuditLog": `
			INSERT INTO audit_log (action, entity_type, entity_id, changes, commit_sha, agent)
			VALUES (?, ?, ?, ?, ?, ?)`,
	}

	for name, query := range queries {
		stmt, err := db.conn.Prepare(query)
		if err != nil {
			return fmt.Errorf("failed to prepare %s: %w", name, err)
		}
		db.stmts[name] = stmt
	}

	slog.Debug("prepared statements cached", "count", len(db.stmts))

	return nil
}

// Close closes database and prepared statements
func (db *DB) Close() error {
	// Close prepared statements
	for name, stmt := range db.stmts {
		if err := stmt.Close(); err != nil {
			slog.Warn("failed to close prepared statement", "name", name, "error", err)
		}
	}

	// Close connection
	if db.conn != nil {
		slog.Debug("closing database connection")
		return db.conn.Close()
	}
	return nil
}

// WithExclusiveLock ensures single writer
func (db *DB) WithExclusiveLock(fn func() error) error {
	db.mu.Lock()
	defer db.mu.Unlock()

	return fn()
}

// Phase represents a phase record
type Phase struct {
	ID            int
	Path          string
	Name          string
	Category      string
	Status        string
	CompletedDate sql.NullString
	Description   sql.NullString
	TestCount     int
}

// GetPhase retrieves phase by ID (uses prepared statement)
func (db *DB) GetPhase(id int) (*Phase, error) {
	start := time.Now()
	stmt := db.stmts["getPhase"]

	var p Phase
	err := stmt.QueryRow(id).Scan(
		&p.ID, &p.Path, &p.Name, &p.Category,
		&p.Status, &p.CompletedDate, &p.Description, &p.TestCount,
	)

	duration := time.Since(start)
	slog.Debug("query completed",
		"query", "getPhase",
		"id", id,
		"duration_ms", duration.Milliseconds(),
		"found", err == nil,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return nil, ErrPhaseNotFound
		}
		return nil, err
	}

	return &p, nil
}

// GetPhaseByPath retrieves phase by path
func (db *DB) GetPhaseByPath(path string) (*Phase, error) {
	start := time.Now()
	stmt := db.stmts["getPhaseByPath"]

	var p Phase
	err := stmt.QueryRow(path).Scan(
		&p.ID, &p.Path, &p.Name, &p.Category,
		&p.Status, &p.CompletedDate, &p.Description, &p.TestCount,
	)

	duration := time.Since(start)
	slog.Debug("query completed",
		"query", "getPhaseByPath",
		"path", path,
		"duration_ms", duration.Milliseconds(),
		"found", err == nil,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return nil, ErrPhaseNotFound
		}
		return nil, err
	}

	return &p, nil
}

// Category represents a category record
type Category struct {
	ID          int
	Name        string
	DisplayName string
	Completed   int
	Total       int
	Percentage  int
	Status      string
}

// GetCategory retrieves category by name
func (db *DB) GetCategory(name string) (*Category, error) {
	start := time.Now()
	stmt := db.stmts["getCategoryProgress"]

	var c Category
	err := stmt.QueryRow(name).Scan(
		&c.ID, &c.Name, &c.DisplayName,
		&c.Completed, &c.Total, &c.Percentage, &c.Status,
	)

	duration := time.Since(start)
	slog.Debug("query completed",
		"query", "getCategory",
		"name", name,
		"duration_ms", duration.Milliseconds(),
		"found", err == nil,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return nil, ErrCategoryNotFound
		}
		return nil, err
	}

	return &c, nil
}

// GetAllCategories retrieves all categories
func (db *DB) GetAllCategories() ([]*Category, error) {
	start := time.Now()

	rows, err := db.conn.Query(`
		SELECT id, name, display_name, completed, total, percentage, status
		FROM categories
		ORDER BY id
	`)
	if err != nil {
		return nil, err
	}
	defer func() { _ = rows.Close() }()

	var categories []*Category
	for rows.Next() {
		var c Category
		if err := rows.Scan(
			&c.ID, &c.Name, &c.DisplayName,
			&c.Completed, &c.Total, &c.Percentage, &c.Status,
		); err != nil {
			return nil, err
		}
		categories = append(categories, &c)
	}

	duration := time.Since(start)
	slog.Debug("query completed",
		"query", "getAllCategories",
		"count", len(categories),
		"duration_ms", duration.Milliseconds(),
	)

	return categories, rows.Err()
}

// TotalProgress represents overall progress
type TotalProgress struct {
	TotalCompleted int
	TotalPhases    int
	Percentage     int
}

// GetTotalProgress retrieves overall progress
func (db *DB) GetTotalProgress() (*TotalProgress, error) {
	start := time.Now()
	stmt := db.stmts["getTotalProgress"]

	var p TotalProgress
	err := stmt.QueryRow().Scan(&p.TotalCompleted, &p.TotalPhases, &p.Percentage)

	duration := time.Since(start)
	slog.Debug("query completed",
		"query", "getTotalProgress",
		"duration_ms", duration.Milliseconds(),
	)

	if err != nil {
		return nil, err
	}

	return &p, nil
}

// GetMetadata retrieves a metadata value
func (db *DB) GetMetadata(key string) (string, error) {
	stmt := db.stmts["getMetadata"]

	var value string
	err := stmt.QueryRow(key).Scan(&value)
	if err != nil {
		if err == sql.ErrNoRows {
			return "", fmt.Errorf("metadata key not found: %s", key)
		}
		return "", err
	}

	return value, nil
}

// SetMetadata sets a metadata value
func (db *DB) SetMetadata(key, value string) error {
	stmt := db.stmts["setMetadata"]

	_, err := stmt.Exec(key, value)
	return err
}

// InsertAuditLog inserts an audit log entry
func (db *DB) InsertAuditLog(action, entityType, entityID, changes, commitSHA, agent string) error {
	stmt := db.stmts["insertAuditLog"]

	_, err := stmt.Exec(action, entityType, entityID, changes, commitSHA, agent)
	return err
}

// Query executes a query that returns rows
func (db *DB) Query(query string, args ...interface{}) (*sql.Rows, error) {
	return db.conn.Query(query, args...)
}

// QueryRow executes a query that returns a single row
func (db *DB) QueryRow(query string, args ...interface{}) *sql.Row {
	return db.conn.QueryRow(query, args...)
}

// Exec executes a query that doesn't return rows
func (db *DB) Exec(query string, args ...interface{}) (sql.Result, error) {
	return db.conn.Exec(query, args...)
}
