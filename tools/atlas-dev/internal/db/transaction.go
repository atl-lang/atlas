package db

import (
	"database/sql"
	"fmt"
)

// Transaction wraps sql.Tx with proper cleanup
type Transaction struct {
	tx *sql.Tx
}

// WithTransaction executes fn within a transaction
// Commits on success, rollback on error/panic
func (db *DB) WithTransaction(fn func(*Transaction) error) error {
	tx, err := db.conn.Begin()
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	t := &Transaction{tx: tx}

	// Handle panic (rollback and re-panic)
	defer func() {
		if p := recover(); p != nil {
			_ = tx.Rollback() // Best effort on panic
			panic(p)
		}
	}()

	// Execute function
	err = fn(t)
	if err != nil {
		_ = tx.Rollback() // Best effort on error
		return err
	}

	// Commit
	if err := tx.Commit(); err != nil {
		return fmt.Errorf("failed to commit: %w", err)
	}

	return nil
}

// Exec executes query within transaction
func (t *Transaction) Exec(query string, args ...interface{}) (sql.Result, error) {
	return t.tx.Exec(query, args...)
}

// QueryRow queries single row within transaction
func (t *Transaction) QueryRow(query string, args ...interface{}) *sql.Row {
	return t.tx.QueryRow(query, args...)
}

// Query queries multiple rows within transaction
func (t *Transaction) Query(query string, args ...interface{}) (*sql.Rows, error) {
	return t.tx.Query(query, args...)
}
