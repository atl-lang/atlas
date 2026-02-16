package db

import (
	"testing"
)

func TestValidate(t *testing.T) {
	db := NewTestDB(t)

	report, err := db.Validate()
	if err != nil {
		t.Fatalf("Validate() error: %v", err)
	}

	if report == nil {
		t.Fatal("expected report, got nil")
	}

	if report.ChecksRun != 7 {
		t.Errorf("expected 7 checks run, got %d", report.ChecksRun)
	}
}

func TestValidateCategoryCounts(t *testing.T) {
	db := NewTestDB(t)

	// Add category with incorrect count
	SeedTestCategory(t, db, 98, "test", "Test", 5)

	// Add only 2 completed phases
	_, _ = db.Exec("INSERT INTO phases (path, name, category, status) VALUES ('t1.md', 't1', 'test', 'completed')")
	_, _ = db.Exec("INSERT INTO phases (path, name, category, status) VALUES ('t2.md', 't2', 'test', 'completed')")

	// Manually set wrong count
	_, _ = db.Exec("UPDATE categories SET completed = 3 WHERE name = 'test'")

	report, err := db.Validate()
	if err != nil {
		t.Fatalf("Validate() error: %v", err)
	}

	if report.OK {
		t.Error("expected validation to fail with incorrect count")
	}

	if report.ErrorsFound == 0 {
		t.Error("expected errors found")
	}
}

func TestValidateCategoryPercentages(t *testing.T) {
	db := NewTestDB(t)

	// Add category
	SeedTestCategory(t, db, 99, "test", "Test", 10)

	// Add 5 completed phases
	for i := 0; i < 5; i++ {
		_, _ = db.Exec("INSERT INTO phases (path, name, category, status) VALUES (?, ?, 'test', 'completed')",
			"t"+string(rune(i))+".md", "t"+string(rune(i)))
	}

	// Manually set wrong percentage
	_, _ = db.Exec("UPDATE categories SET completed = 5, percentage = 99 WHERE name = 'test'")

	report := &ValidationReport{Issues: []ValidationIssue{}}
	err := db.validateCategoryPercentages(report)
	if err != nil {
		t.Fatalf("validateCategoryPercentages() error: %v", err)
	}

	if len(report.Issues) == 0 {
		t.Error("expected percentage issue to be detected")
	}
}

func TestValidateOrphanedPhases(t *testing.T) {
	db := NewTestDB(t)

	// Add phase with invalid category
	_, _ = db.Exec("INSERT INTO phases (path, name, category, status) VALUES ('orphan.md', 'orphan', 'nonexistent', 'pending')")

	report := &ValidationReport{Issues: []ValidationIssue{}}
	err := db.validateOrphanedPhases(report)
	if err != nil {
		t.Fatalf("validateOrphanedPhases() error: %v", err)
	}

	if len(report.Issues) == 0 {
		t.Error("expected orphaned phase to be detected")
	}
}

func TestValidateInvalidStatuses(t *testing.T) {
	db := NewTestDB(t)

	// Add phase with invalid status
	_, _ = db.Exec("INSERT INTO phases (path, name, category, status) VALUES ('bad.md', 'bad', 'test', 'invalid_status')")

	report := &ValidationReport{Issues: []ValidationIssue{}}
	err := db.validateInvalidStatuses(report)
	if err != nil {
		t.Fatalf("validateInvalidStatuses() error: %v", err)
	}

	if len(report.Issues) == 0 {
		t.Error("expected invalid status to be detected")
	}
}

func TestValidateTriggers(t *testing.T) {
	db := NewTestDB(t)

	report := &ValidationReport{Issues: []ValidationIssue{}}
	err := db.validateTriggers(report)
	if err != nil {
		t.Fatalf("validateTriggers() error: %v", err)
	}

	// All triggers should exist in a fresh DB
	if len(report.Issues) > 0 {
		t.Errorf("expected no missing triggers, got %d issues", len(report.Issues))
	}
}
