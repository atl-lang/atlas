package db

import (
	"testing"
)

// NewTestDB creates in-memory database for tests
func NewTestDB(t *testing.T) *DB {
	t.Helper()

	// Use :memory: for speed
	db, err := New(":memory:")
	if err != nil {
		t.Fatalf("failed to create test db: %v", err)
	}

	// Create schema
	if err := db.InitSchema(); err != nil {
		t.Fatalf("failed to init schema: %v", err)
	}

	// Auto-cleanup
	t.Cleanup(func() {
		_ = db.Close()
	})

	return db
}

// SeedTestPhase inserts test phase
func SeedTestPhase(t *testing.T, db *DB, path, category, name string) int64 {
	t.Helper()

	result, err := db.conn.Exec(`
		INSERT INTO phases (path, name, category, status)
		VALUES (?, ?, ?, 'pending')
	`, path, name, category)
	if err != nil {
		t.Fatalf("failed to seed phase: %v", err)
	}

	id, _ := result.LastInsertId()
	return id
}

// SeedTestCategory ensures category exists with specified total
func SeedTestCategory(t *testing.T, db *DB, id int, name, displayName string, total int) {
	t.Helper()

	_, err := db.conn.Exec(`
		INSERT OR REPLACE INTO categories (id, name, display_name, total, completed, percentage, status)
		VALUES (?, ?, ?, ?, 0, 0, 'pending')
	`, id, name, displayName, total)
	if err != nil {
		t.Fatalf("failed to seed category: %v", err)
	}
}
