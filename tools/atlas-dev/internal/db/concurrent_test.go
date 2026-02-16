package db

import (
	"sync"
	"testing"
)

func TestWithExclusiveLock(t *testing.T) {
	db := NewTestDB(t)

	// Test exclusive lock works
	counter := 0
	var wg sync.WaitGroup

	for i := 0; i < 5; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			db.WithExclusiveLock(func() error {
				// This should be executed serially due to lock
				temp := counter
				temp++
				counter = temp
				return nil
			})
		}()
	}

	wg.Wait()

	if counter != 5 {
		t.Errorf("expected counter = 5, got %d (race condition detected)", counter)
	}
}

func TestQuery(t *testing.T) {
	db := NewTestDB(t)

	rows, err := db.Query("SELECT * FROM categories LIMIT 2")
	if err != nil {
		t.Fatalf("Query() error: %v", err)
	}
	defer rows.Close()

	count := 0
	for rows.Next() {
		count++
	}

	if count != 2 {
		t.Errorf("expected 2 rows, got %d", count)
	}
}

func TestQueryRow(t *testing.T) {
	db := NewTestDB(t)

	var count int
	row := db.QueryRow("SELECT COUNT(*) FROM categories")
	err := row.Scan(&count)
	if err != nil {
		t.Fatalf("QueryRow() error: %v", err)
	}

	if count != 9 {
		t.Errorf("expected 9 categories, got %d", count)
	}
}

func TestExec(t *testing.T) {
	db := NewTestDB(t)

	result, err := db.Exec(`
		INSERT INTO phases (path, name, category, status)
		VALUES ('test.md', 'test', 'test', 'pending')
	`)
	if err != nil {
		t.Fatalf("Exec() error: %v", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		t.Fatalf("LastInsertId() error: %v", err)
	}

	if id == 0 {
		t.Error("expected non-zero ID")
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		t.Fatalf("RowsAffected() error: %v", err)
	}

	if rowsAffected != 1 {
		t.Errorf("expected 1 row affected, got %d", rowsAffected)
	}
}

func TestInsertAuditLog(t *testing.T) {
	db := NewTestDB(t)

	err := db.InsertAuditLog(
		"test_action",
		"phase",
		"1",
		`{"before":"pending","after":"completed"}`,
		"abc123",
		"test-agent",
	)

	if err != nil {
		t.Fatalf("InsertAuditLog() error: %v", err)
	}

	// Verify audit log was inserted
	var count int
	err = db.QueryRow("SELECT COUNT(*) FROM audit_log WHERE action = 'test_action'").Scan(&count)
	if err != nil {
		t.Fatalf("failed to query audit log: %v", err)
	}

	if count != 1 {
		t.Errorf("expected 1 audit log entry, got %d", count)
	}
}
