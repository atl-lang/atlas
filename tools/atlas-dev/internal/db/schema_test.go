package db

import (
	"testing"
)

func TestSchemaCreation(t *testing.T) {
	db := NewTestDB(t)

	// Verify indexes exist
	indexes := []string{
		"idx_phases_category",
		"idx_phases_status",
		"idx_phases_completed_date",
		"idx_decisions_component",
		"idx_decisions_date",
		"idx_decisions_status",
		"idx_features_version",
		"idx_features_status",
		"idx_specs_section",
		"idx_audit_timestamp",
		"idx_audit_entity",
		"idx_parity_timestamp",
		"idx_parity_status",
		"idx_test_coverage_timestamp",
		"idx_test_coverage_category",
	}

	for _, index := range indexes {
		var count int
		err := db.conn.QueryRow(
			"SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?",
			index,
		).Scan(&count)
		if err != nil {
			t.Errorf("failed to check index %s: %v", index, err)
		}
		if count != 1 {
			t.Errorf("index %s not found", index)
		}
	}
}

func TestTriggers(t *testing.T) {
	db := NewTestDB(t)

	triggers := []string{
		"update_category_progress",
		"update_phases_timestamp",
		"update_decisions_timestamp",
		"update_features_timestamp",
	}

	for _, trigger := range triggers {
		var count int
		err := db.conn.QueryRow(
			"SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name=?",
			trigger,
		).Scan(&count)
		if err != nil {
			t.Errorf("failed to check trigger %s: %v", trigger, err)
		}
		if count != 1 {
			t.Errorf("trigger %s not found", trigger)
		}
	}
}

func TestViews(t *testing.T) {
	db := NewTestDB(t)

	views := []string{
		"v_progress",
		"v_active_phases",
		"v_recent_decisions",
		"v_parity_summary",
	}

	for _, view := range views {
		var count int
		err := db.conn.QueryRow(
			"SELECT COUNT(*) FROM sqlite_master WHERE type='view' AND name=?",
			view,
		).Scan(&count)
		if err != nil {
			t.Errorf("failed to check view %s: %v", view, err)
		}
		if count != 1 {
			t.Errorf("view %s not found", view)
		}
	}
}

func TestCategoryProgressTrigger(t *testing.T) {
	db := NewTestDB(t)

	// Seed test category
	SeedTestCategory(t, db, 99, "test", "Test Category", 3)

	// Seed test phases
	id1 := SeedTestPhase(t, db, "test1.md", "test", "test1")
	id2 := SeedTestPhase(t, db, "test2.md", "test", "test2")
	SeedTestPhase(t, db, "test3.md", "test", "test3")

	// Initially all pending
	cat, err := db.GetCategory("test")
	if err != nil {
		t.Fatalf("GetCategory() error: %v", err)
	}
	if cat.Completed != 0 {
		t.Errorf("expected completed = 0, got %d", cat.Completed)
	}
	if cat.Percentage != 0 {
		t.Errorf("expected percentage = 0, got %d", cat.Percentage)
	}

	// Complete first phase
	_, err = db.conn.Exec(`
		UPDATE phases SET status = 'completed' WHERE id = ?
	`, id1)
	if err != nil {
		t.Fatalf("failed to update phase: %v", err)
	}

	// Check trigger updated category
	cat, err = db.GetCategory("test")
	if err != nil {
		t.Fatalf("GetCategory() error: %v", err)
	}
	if cat.Completed != 1 {
		t.Errorf("expected completed = 1, got %d", cat.Completed)
	}
	if cat.Percentage != 33 { // 1/3 = 33%
		t.Errorf("expected percentage = 33, got %d", cat.Percentage)
	}
	if cat.Status != "active" {
		t.Errorf("expected status = 'active', got %q", cat.Status)
	}

	// Complete second phase
	_, err = db.conn.Exec(`
		UPDATE phases SET status = 'completed' WHERE id = ?
	`, id2)
	if err != nil {
		t.Fatalf("failed to update phase: %v", err)
	}

	// Check trigger updated again
	cat, err = db.GetCategory("test")
	if err != nil {
		t.Fatalf("GetCategory() error: %v", err)
	}
	if cat.Completed != 2 {
		t.Errorf("expected completed = 2, got %d", cat.Completed)
	}
	if cat.Percentage != 67 { // 2/3 = 67%
		t.Errorf("expected percentage = 67, got %d", cat.Percentage)
	}
}

func TestMetadataUpdateTrigger(t *testing.T) {
	db := NewTestDB(t)

	// Seed test category and phase
	SeedTestCategory(t, db, 99, "test", "Test", 1)
	id := SeedTestPhase(t, db, "test.md", "test", "test")

	// Get initial completed count
	initial, err := db.GetMetadata("completed_phases")
	if err != nil {
		t.Fatalf("GetMetadata() error: %v", err)
	}

	// Complete phase
	_, err = db.conn.Exec("UPDATE phases SET status = 'completed' WHERE id = ?", id)
	if err != nil {
		t.Fatalf("failed to update phase: %v", err)
	}

	// Check metadata was updated by trigger
	after, err := db.GetMetadata("completed_phases")
	if err != nil {
		t.Fatalf("GetMetadata() error: %v", err)
	}

	if after == initial {
		t.Error("expected metadata to be updated by trigger")
	}
}

func TestProgressView(t *testing.T) {
	db := NewTestDB(t)

	rows, err := db.conn.Query("SELECT * FROM v_progress")
	if err != nil {
		t.Fatalf("failed to query v_progress: %v", err)
	}
	defer rows.Close()

	count := 0
	for rows.Next() {
		count++
	}

	if count != 9 {
		t.Errorf("expected 9 categories in v_progress, got %d", count)
	}
}

func TestActivePhasesView(t *testing.T) {
	db := NewTestDB(t)

	// Seed phase with in_progress status
	_, err := db.conn.Exec(`
		INSERT INTO phases (path, name, category, status)
		VALUES ('test.md', 'test', 'test', 'in_progress')
	`)
	if err != nil {
		t.Fatalf("failed to insert: %v", err)
	}

	rows, err := db.conn.Query("SELECT * FROM v_active_phases")
	if err != nil {
		t.Fatalf("failed to query v_active_phases: %v", err)
	}
	defer rows.Close()

	count := 0
	for rows.Next() {
		count++
	}

	if count != 1 {
		t.Errorf("expected 1 active phase, got %d", count)
	}
}
