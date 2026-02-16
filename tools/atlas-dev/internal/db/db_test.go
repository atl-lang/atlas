package db

import (
	"testing"
)

func TestNew(t *testing.T) {
	tests := []struct {
		name    string
		path    string
		wantErr bool
	}{
		{"memory database", ":memory:", false},
		{"file database", "test.db", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			db, err := New(tt.path)
			if (err != nil) != tt.wantErr {
				t.Errorf("New() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if db != nil {
				defer db.Close()
			}
		})
	}
}

func TestInitSchema(t *testing.T) {
	db := NewTestDB(t)

	// Verify tables exist
	tables := []string{
		"phases", "categories", "decisions", "features",
		"specs", "api_docs", "metadata", "audit_log",
		"parity_checks", "test_coverage",
	}

	for _, table := range tables {
		var count int
		err := db.conn.QueryRow(
			"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
			table,
		).Scan(&count)
		if err != nil {
			t.Errorf("failed to check table %s: %v", table, err)
		}
		if count != 1 {
			t.Errorf("table %s not found", table)
		}
	}
}

func TestCategoriesSeeded(t *testing.T) {
	db := NewTestDB(t)

	categories, err := db.GetAllCategories()
	if err != nil {
		t.Fatalf("GetAllCategories() error: %v", err)
	}

	if len(categories) != 9 {
		t.Errorf("expected 9 categories, got %d", len(categories))
	}

	// Verify first category
	if categories[0].Name != "foundation" {
		t.Errorf("expected first category to be 'foundation', got %q", categories[0].Name)
	}
	if categories[0].Total != 21 {
		t.Errorf("expected foundation total to be 21, got %d", categories[0].Total)
	}
}

func TestMetadataSeeded(t *testing.T) {
	db := NewTestDB(t)

	tests := []struct {
		key      string
		expected string
	}{
		{"schema_version", "1"},
		{"atlas_version", "v0.2"},
		{"total_phases", "78"},
		{"completed_phases", "0"},
	}

	for _, tt := range tests {
		t.Run(tt.key, func(t *testing.T) {
			value, err := db.GetMetadata(tt.key)
			if err != nil {
				t.Errorf("GetMetadata(%q) error: %v", tt.key, err)
				return
			}
			if value != tt.expected {
				t.Errorf("GetMetadata(%q) = %q, want %q", tt.key, value, tt.expected)
			}
		})
	}
}

func TestGetPhase(t *testing.T) {
	db := NewTestDB(t)

	// Seed test phase
	id := SeedTestPhase(t, db, "phases/test/phase-01.md", "test", "phase-01")

	tests := []struct {
		name    string
		id      int
		wantErr error
	}{
		{"valid phase", int(id), nil},
		{"invalid phase", 999, ErrPhaseNotFound},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			phase, err := db.GetPhase(tt.id)

			if tt.wantErr != nil {
				if err != tt.wantErr {
					t.Errorf("GetPhase() error = %v, want %v", err, tt.wantErr)
				}
				return
			}

			if err != nil {
				t.Errorf("GetPhase() unexpected error: %v", err)
				return
			}

			if phase.ID != tt.id {
				t.Errorf("GetPhase() ID = %d, want %d", phase.ID, tt.id)
			}
		})
	}
}

func TestGetPhaseByPath(t *testing.T) {
	db := NewTestDB(t)

	// Seed test phase
	SeedTestPhase(t, db, "phases/test/phase-01.md", "test", "phase-01")

	tests := []struct {
		name    string
		path    string
		wantErr error
	}{
		{"valid path", "phases/test/phase-01.md", nil},
		{"invalid path", "phases/test/nonexistent.md", ErrPhaseNotFound},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			phase, err := db.GetPhaseByPath(tt.path)

			if tt.wantErr != nil {
				if err != tt.wantErr {
					t.Errorf("GetPhaseByPath() error = %v, want %v", err, tt.wantErr)
				}
				return
			}

			if err != nil {
				t.Errorf("GetPhaseByPath() unexpected error: %v", err)
				return
			}

			if phase.Path != tt.path {
				t.Errorf("GetPhaseByPath() path = %q, want %q", phase.Path, tt.path)
			}
		})
	}
}

func TestGetCategory(t *testing.T) {
	db := NewTestDB(t)

	tests := []struct {
		name         string
		categoryName string
		wantErr      error
	}{
		{"valid category", "stdlib", nil},
		{"invalid category", "nonexistent", ErrCategoryNotFound},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			category, err := db.GetCategory(tt.categoryName)

			if tt.wantErr != nil {
				if err != tt.wantErr {
					t.Errorf("GetCategory() error = %v, want %v", err, tt.wantErr)
				}
				return
			}

			if err != nil {
				t.Errorf("GetCategory() unexpected error: %v", err)
				return
			}

			if category.Name != tt.categoryName {
				t.Errorf("GetCategory() name = %q, want %q", category.Name, tt.categoryName)
			}
		})
	}
}

func TestGetTotalProgress(t *testing.T) {
	db := NewTestDB(t)

	progress, err := db.GetTotalProgress()
	if err != nil {
		t.Fatalf("GetTotalProgress() error: %v", err)
	}

	// Initial state - should have at least 78 phases from seeded categories
	if progress.TotalPhases < 78 {
		t.Errorf("expected total phases >= 78, got %d", progress.TotalPhases)
	}

	if progress.TotalCompleted != 0 {
		t.Errorf("expected completed = 0, got %d", progress.TotalCompleted)
	}

	if progress.Percentage != 0 {
		t.Errorf("expected percentage = 0, got %d", progress.Percentage)
	}
}

func TestSetMetadata(t *testing.T) {
	db := NewTestDB(t)

	err := db.SetMetadata("test_key", "test_value")
	if err != nil {
		t.Fatalf("SetMetadata() error: %v", err)
	}

	value, err := db.GetMetadata("test_key")
	if err != nil {
		t.Fatalf("GetMetadata() error: %v", err)
	}

	if value != "test_value" {
		t.Errorf("expected value = 'test_value', got %q", value)
	}
}

func TestClose(t *testing.T) {
	db, err := New(":memory:")
	if err != nil {
		t.Fatalf("New() error: %v", err)
	}

	if err := db.InitSchema(); err != nil {
		t.Fatalf("InitSchema() error: %v", err)
	}

	if err := db.Close(); err != nil {
		t.Errorf("Close() error: %v", err)
	}

	// Verify database is closed (should fail to query)
	_, err = db.GetMetadata("schema_version")
	if err == nil {
		t.Error("expected error querying closed database, got nil")
	}
}

func TestPreparedStatements(t *testing.T) {
	db := NewTestDB(t)

	// Verify prepared statements exist
	expectedStmts := []string{
		"getPhase",
		"getPhaseByPath",
		"updatePhaseStatus",
		"listPhases",
		"listAllPhases",
		"getCategory",
		"getCategoryProgress",
		"getTotalProgress",
		"getMetadata",
		"setMetadata",
		"insertAuditLog",
	}

	for _, name := range expectedStmts {
		if _, ok := db.stmts[name]; !ok {
			t.Errorf("prepared statement %q not found", name)
		}
	}
}

func TestConnectionPoolConfig(t *testing.T) {
	db := NewTestDB(t)

	stats := db.conn.Stats()

	// SQLite should have MaxOpenConns = 1
	if stats.MaxOpenConnections != 1 {
		t.Errorf("expected MaxOpenConnections = 1, got %d", stats.MaxOpenConnections)
	}
}
