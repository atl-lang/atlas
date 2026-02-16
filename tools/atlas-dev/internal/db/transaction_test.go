package db

import (
	"errors"
	"testing"
)

func TestTransactionCommit(t *testing.T) {
	db := NewTestDB(t)

	err := db.WithTransaction(func(tx *Transaction) error {
		_, err := tx.Exec(`
			INSERT INTO phases (path, name, category, status)
			VALUES ('test.md', 'test', 'test', 'pending')
		`)
		return err
	})

	if err != nil {
		t.Fatalf("WithTransaction() error: %v", err)
	}

	// Verify data was committed
	var count int
	err = db.conn.QueryRow("SELECT COUNT(*) FROM phases WHERE name = 'test'").Scan(&count)
	if err != nil {
		t.Fatalf("failed to query: %v", err)
	}
	if count != 1 {
		t.Errorf("expected 1 row, got %d", count)
	}
}

func TestTransactionRollback(t *testing.T) {
	db := NewTestDB(t)

	testErr := errors.New("test error")

	err := db.WithTransaction(func(tx *Transaction) error {
		_, err := tx.Exec(`
			INSERT INTO phases (path, name, category, status)
			VALUES ('test.md', 'test', 'test', 'pending')
		`)
		if err != nil {
			return err
		}

		// Return error to trigger rollback
		return testErr
	})

	if err != testErr {
		t.Fatalf("expected test error, got: %v", err)
	}

	// Verify data was rolled back
	var count int
	err = db.conn.QueryRow("SELECT COUNT(*) FROM phases WHERE name = 'test'").Scan(&count)
	if err != nil {
		t.Fatalf("failed to query: %v", err)
	}
	if count != 0 {
		t.Errorf("expected 0 rows (rollback), got %d", count)
	}
}

func TestTransactionPanic(t *testing.T) {
	db := NewTestDB(t)

	defer func() {
		if r := recover(); r == nil {
			t.Error("expected panic to be re-raised")
		}
	}()

	db.WithTransaction(func(tx *Transaction) error {
		_, err := tx.Exec(`
			INSERT INTO phases (path, name, category, status)
			VALUES ('test.md', 'test', 'test', 'pending')
		`)
		if err != nil {
			return err
		}

		// Panic should trigger rollback
		panic("test panic")
	})
}

func TestTransactionNestedOperations(t *testing.T) {
	db := NewTestDB(t)

	err := db.WithTransaction(func(tx *Transaction) error {
		// Insert phase
		result, err := tx.Exec(`
			INSERT INTO phases (path, name, category, status)
			VALUES ('test.md', 'test', 'stdlib', 'pending')
		`)
		if err != nil {
			return err
		}

		// Get inserted ID
		id, err := result.LastInsertId()
		if err != nil {
			return err
		}

		// Update phase in same transaction
		_, err = tx.Exec(`
			UPDATE phases SET status = 'completed' WHERE id = ?
		`, id)
		if err != nil {
			return err
		}

		// Query in same transaction
		var status string
		err = tx.QueryRow("SELECT status FROM phases WHERE id = ?", id).Scan(&status)
		if err != nil {
			return err
		}

		if status != "completed" {
			t.Errorf("expected status = 'completed', got %q", status)
		}

		return nil
	})

	if err != nil {
		t.Fatalf("WithTransaction() error: %v", err)
	}
}

func TestConcurrentTransactions(t *testing.T) {
	db := NewTestDB(t)

	// SQLite with single MaxOpenConns serializes transactions
	// This test verifies they don't deadlock

	done := make(chan error, 2)

	go func() {
		err := db.WithTransaction(func(tx *Transaction) error {
			_, err := tx.Exec(`
				INSERT INTO phases (path, name, category, status)
				VALUES ('test1.md', 'test1', 'test', 'pending')
			`)
			return err
		})
		done <- err
	}()

	go func() {
		err := db.WithTransaction(func(tx *Transaction) error {
			_, err := tx.Exec(`
				INSERT INTO phases (path, name, category, status)
				VALUES ('test2.md', 'test2', 'test', 'pending')
			`)
			return err
		})
		done <- err
	}()

	// Wait for both
	for i := 0; i < 2; i++ {
		if err := <-done; err != nil {
			t.Errorf("transaction %d failed: %v", i+1, err)
		}
	}

	// Verify both committed
	var count int
	db.conn.QueryRow("SELECT COUNT(*) FROM phases WHERE name LIKE 'test%'").Scan(&count)
	if count != 2 {
		t.Errorf("expected 2 rows, got %d", count)
	}
}
