package db

import (
	"testing"
	"time"
)

func TestCompletePhase(t *testing.T) {
	db := NewTestDB(t)

	// Seed category and phase
	SeedTestCategory(t, db, 90, "test", "Test Category", 3)
	id := SeedTestPhase(t, db, "phases/test/phase-01.md", "test", "phase-01")

	// Complete phase
	result, err := db.CompletePhase(CompletePhaseRequest{
		PhasePath:   "phases/test/phase-01.md",
		Description: "Test phase complete",
		Date:        time.Now().Format(time.RFC3339),
		TestCount:   10,
		DryRun:      false,
	})

	if err != nil {
		t.Fatalf("CompletePhase() error: %v", err)
	}

	if result.PhaseName != "phase-01" {
		t.Errorf("expected phase name = 'phase-01', got %q", result.PhaseName)
	}

	if result.Category != "test" {
		t.Errorf("expected category = 'test', got %q", result.Category)
	}

	// Verify phase was updated
	phase, err := db.GetPhase(int(id))
	if err != nil {
		t.Fatalf("GetPhase() error: %v", err)
	}

	if phase.Status != "completed" {
		t.Errorf("expected status = 'completed', got %q", phase.Status)
	}

	if phase.TestCount != 10 {
		t.Errorf("expected test count = 10, got %d", phase.TestCount)
	}
}

func TestCompletePhaseAlreadyDone(t *testing.T) {
	db := NewTestDB(t)

	// Seed category and completed phase
	SeedTestCategory(t, db, 91, "test", "Test", 1)
	_, err := db.Exec(`
		INSERT INTO phases (path, name, category, status, completed_date)
		VALUES ('test.md', 'test', 'test', 'completed', ?)
	`, time.Now().Format(time.RFC3339))
	if err != nil {
		t.Fatal(err)
	}

	// Try to complete again
	_, err = db.CompletePhase(CompletePhaseRequest{
		PhasePath:   "test.md",
		Description: "Test",
		Date:        time.Now().Format(time.RFC3339),
	})

	if err != ErrPhaseAlreadyDone {
		t.Errorf("expected ErrPhaseAlreadyDone, got %v", err)
	}
}

func TestCompletePhaseDryRun(t *testing.T) {
	db := NewTestDB(t)

	// Seed category and phase
	SeedTestCategory(t, db, 92, "test", "Test", 1)
	id := SeedTestPhase(t, db, "test.md", "test", "test")

	// Dry run complete
	_, err := db.CompletePhase(CompletePhaseRequest{
		PhasePath:   "test.md",
		Description: "Test",
		Date:        time.Now().Format(time.RFC3339),
		DryRun:      true,
	})

	if err != nil {
		t.Fatalf("CompletePhase() dry-run error: %v", err)
	}

	// Verify phase was NOT updated
	phase, err := db.GetPhase(int(id))
	if err != nil {
		t.Fatalf("GetPhase() error: %v", err)
	}

	if phase.Status != "pending" {
		t.Errorf("dry-run should not update status, got %q", phase.Status)
	}
}

func TestGetCurrentPhase(t *testing.T) {
	db := NewTestDB(t)

	// Initially no completed phases
	phase, err := db.GetCurrentPhase()
	if err != nil {
		t.Fatalf("GetCurrentPhase() error: %v", err)
	}

	if phase != nil {
		t.Error("expected nil when no phases completed")
	}

	// Add completed phase
	_, err = db.Exec(`
		INSERT INTO phases (path, name, category, status, completed_date)
		VALUES ('test1.md', 'test1', 'test', 'completed', ?)
	`, time.Now().Add(-1*time.Hour).Format(time.RFC3339))
	if err != nil {
		t.Fatal(err)
	}

	// Add more recent completed phase
	_, err = db.Exec(`
		INSERT INTO phases (path, name, category, status, completed_date)
		VALUES ('test2.md', 'test2', 'test', 'completed', ?)
	`, time.Now().Format(time.RFC3339))
	if err != nil {
		t.Fatal(err)
	}

	// Should get most recent
	phase, err = db.GetCurrentPhase()
	if err != nil {
		t.Fatalf("GetCurrentPhase() error: %v", err)
	}

	if phase == nil {
		t.Fatal("expected phase, got nil")
	}

	if phase.Name != "test2" {
		t.Errorf("expected most recent phase 'test2', got %q", phase.Name)
	}
}

func TestGetNextPhase(t *testing.T) {
	db := NewTestDB(t)

	// Initially all pending
	phase, err := db.GetNextPhase("")
	if err != nil {
		t.Fatalf("GetNextPhase() error: %v", err)
	}

	if phase != nil {
		// No phases in db yet, should return nil
		t.Log("Expected nil when no pending phases")
	}

	// Add phases
	SeedTestCategory(t, db, 93, "test", "Test", 2)
	SeedTestPhase(t, db, "test1.md", "test", "test1")
	SeedTestPhase(t, db, "test2.md", "test", "test2")

	// Get next
	phase, err = db.GetNextPhase("")
	if err != nil {
		t.Fatalf("GetNextPhase() error: %v", err)
	}

	if phase == nil {
		t.Fatal("expected phase, got nil")
	}

	if phase.Name != "test1" {
		t.Errorf("expected first pending 'test1', got %q", phase.Name)
	}
}

func TestGetNextPhaseInCategory(t *testing.T) {
	db := NewTestDB(t)

	// Add phases in different categories
	SeedTestCategory(t, db, 94, "test1", "Test1", 1)
	SeedTestCategory(t, db, 95, "test2", "Test2", 1)
	SeedTestPhase(t, db, "test1.md", "test1", "test1-phase")
	SeedTestPhase(t, db, "test2.md", "test2", "test2-phase")

	// Get next in specific category
	phase, err := db.GetNextPhaseInCategory("test2")
	if err != nil {
		t.Fatalf("GetNextPhaseInCategory() error: %v", err)
	}

	if phase == nil {
		t.Fatal("expected phase, got nil")
	}

	if phase.Category != "test2" {
		t.Errorf("expected category 'test2', got %q", phase.Category)
	}
}

func TestGetPhaseInfo(t *testing.T) {
	db := NewTestDB(t)

	// Add phase
	SeedTestPhase(t, db, "test.md", "test", "test-phase")

	// Get info
	phase, err := db.GetPhaseInfo("test.md")
	if err != nil {
		t.Fatalf("GetPhaseInfo() error: %v", err)
	}

	if phase == nil {
		t.Fatal("expected phase, got nil")
	}

	if phase.Name != "test-phase" {
		t.Errorf("expected name 'test-phase', got %q", phase.Name)
	}

	// Non-existent phase
	_, err = db.GetPhaseInfo("nonexistent.md")
	if err != ErrPhaseNotFound {
		t.Errorf("expected ErrPhaseNotFound, got %v", err)
	}
}

func TestListPhases(t *testing.T) {
	db := NewTestDB(t)

	// Add phases
	SeedTestCategory(t, db, 96, "test1", "Test1", 2)
	SeedTestCategory(t, db, 97, "test2", "Test2", 1)
	SeedTestPhase(t, db, "test1-a.md", "test1", "test1-a")
	SeedTestPhase(t, db, "test1-b.md", "test1", "test1-b")
	SeedTestPhase(t, db, "test2-a.md", "test2", "test2-a")

	// List all
	phases, err := db.ListPhases(ListPhasesOptions{})
	if err != nil {
		t.Fatalf("ListPhases() error: %v", err)
	}

	if len(phases) < 3 {
		t.Errorf("expected at least 3 phases, got %d", len(phases))
	}

	// Filter by category
	phases, err = db.ListPhases(ListPhasesOptions{
		Category: "test1",
	})
	if err != nil {
		t.Fatalf("ListPhases() with category error: %v", err)
	}

	if len(phases) != 2 {
		t.Errorf("expected 2 phases in test1, got %d", len(phases))
	}

	// Filter by status
	phases, err = db.ListPhases(ListPhasesOptions{
		Status: "pending",
	})
	if err != nil {
		t.Fatalf("ListPhases() with status error: %v", err)
	}

	for _, p := range phases {
		if p.Status != "pending" {
			t.Errorf("expected all pending, got status %q", p.Status)
		}
	}

	// Test pagination
	phases, err = db.ListPhases(ListPhasesOptions{
		Limit:  2,
		Offset: 1,
	})
	if err != nil {
		t.Fatalf("ListPhases() with pagination error: %v", err)
	}

	if len(phases) != 2 {
		t.Errorf("expected 2 phases with limit=2, got %d", len(phases))
	}
}

func TestInsertPhase(t *testing.T) {
	db := NewTestDB(t)

	id, err := db.InsertPhase("new.md", "new-phase", "test")
	if err != nil {
		t.Fatalf("InsertPhase() error: %v", err)
	}

	if id == 0 {
		t.Error("expected non-zero ID")
	}

	// Verify inserted
	phase, err := db.GetPhase(int(id))
	if err != nil {
		t.Fatalf("GetPhase() error: %v", err)
	}

	if phase.Name != "new-phase" {
		t.Errorf("expected name 'new-phase', got %q", phase.Name)
	}

	if phase.Status != "pending" {
		t.Errorf("expected status 'pending', got %q", phase.Status)
	}
}

func TestToCompactJSON(t *testing.T) {
	phase := &PhaseDetails{
		ID:       1,
		Path:     "test.md",
		Name:     "test",
		Category: "stdlib",
		Status:   "completed",
	}

	json := phase.ToCompactJSON()

	if json["id"] != 1 {
		t.Errorf("expected id=1, got %v", json["id"])
	}

	if json["cat"] != "stdlib" {
		t.Errorf("expected cat='stdlib', got %v", json["cat"])
	}

	if json["sts"] != "completed" {
		t.Errorf("expected sts='completed', got %v", json["sts"])
	}
}
