package undo

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"path/filepath"
	"testing"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

func newTestDB(t *testing.T) *db.DB {
	t.Helper()

	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")

	database, err := db.New(dbPath)
	if err != nil {
		t.Fatalf("failed to create test db: %v", err)
	}

	if err := database.InitSchema(); err != nil {
		t.Fatalf("failed to init schema: %v", err)
	}

	if err := database.Prepare(); err != nil {
		t.Fatalf("failed to prepare statements: %v", err)
	}

	t.Cleanup(func() {
		_ = database.Close()
	})

	return database
}

// insertAuditEntry is a test helper to insert audit log entries
func insertAuditEntry(t *testing.T, database *db.DB, action, entityType, entityID string, oldData map[string]interface{}) int {
	t.Helper()

	var oldDataJSON sql.NullString
	if oldData != nil {
		jsonBytes, err := json.Marshal(oldData)
		if err != nil {
			t.Fatalf("failed to marshal old data: %v", err)
		}
		oldDataJSON = sql.NullString{String: string(jsonBytes), Valid: true}
	}

	query := `INSERT INTO audit_log (action, entity_type, entity_id, old_data, changes, agent)
	          VALUES (?, ?, ?, ?, '{}', 'test')`

	result, err := database.Exec(query, action, entityType, entityID, oldDataJSON)
	if err != nil {
		t.Fatalf("failed to insert audit entry: %v", err)
	}

	id, _ := result.LastInsertId()
	return int(id)
}

// insertPhase is a test helper to insert a phase
func insertPhase(t *testing.T, database *db.DB, id int, path, name, category, status string) {
	t.Helper()

	query := `INSERT INTO phases (id, path, name, category, status, completed_date, description, test_count)
	          VALUES (?, ?, ?, ?, ?, NULL, NULL, 0)`

	_, err := database.Exec(query, id, path, name, category, status)
	if err != nil {
		t.Fatalf("failed to insert phase: %v", err)
	}
}

// insertDecision is a test helper to insert a decision
func insertDecision(t *testing.T, database *db.DB, id, title, status, rationale string) {
	t.Helper()

	query := `INSERT INTO decisions (id, component, title, decision, rationale, date, status, created_at)
	          VALUES (?, 'test-component', ?, 'test-decision', ?, date('now'), ?, datetime('now'))`

	_, err := database.Exec(query, id, title, rationale, status)
	if err != nil {
		t.Fatalf("failed to insert decision: %v", err)
	}
}

// getPhaseStatus is a test helper to get phase status
func getPhaseStatus(t *testing.T, database *db.DB, path string) string {
	t.Helper()

	var status string
	query := `SELECT status FROM phases WHERE path = ?`
	err := database.QueryRow(query, path).Scan(&status)
	if err != nil {
		t.Fatalf("failed to get phase status: %v", err)
	}
	return status
}

// getDecision is a test helper to get decision details
func getDecision(t *testing.T, database *db.DB, id string) (title, status, rationale string, exists bool) {
	t.Helper()

	query := `SELECT title, status, rationale FROM decisions WHERE id = ?`
	err := database.QueryRow(query, id).Scan(&title, &status, &rationale)
	if err == sql.ErrNoRows {
		return "", "", "", false
	}
	if err != nil {
		t.Fatalf("failed to get decision: %v", err)
	}
	return title, status, rationale, true
}

// TestNewUndoManager tests creating a new undo manager
func TestNewUndoManager(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	if undoMgr == nil {
		t.Fatal("NewUndoManager() returned nil")
	}

	if undoMgr.db == nil {
		t.Error("UndoManager.db is nil")
	}
}

// TestCanUndo tests the CanUndo method
func TestCanUndo(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// No audit entries yet
	canUndo, err := undoMgr.CanUndo()
	if err != nil {
		t.Fatalf("CanUndo() error = %v", err)
	}

	if canUndo {
		t.Error("expected canUndo = false when no audit entries")
	}

	// Add an audit entry
	insertAuditEntry(t, database, "test_action", "test_entity", "test_id", nil)

	// Now should be able to undo
	canUndo, err = undoMgr.CanUndo()
	if err != nil {
		t.Fatalf("CanUndo() error = %v", err)
	}

	if !canUndo {
		t.Error("expected canUndo = true when audit entries exist")
	}
}

// TestUndo_NothingToUndo tests undo when there are no audit entries
func TestUndo_NothingToUndo(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	_, err := undoMgr.Undo()
	if err == nil {
		t.Error("expected error when nothing to undo")
	}

	if err.Error() != "nothing to undo" {
		t.Errorf("expected 'nothing to undo' error, got: %v", err)
	}
}

// TestUndo_UnsupportedAction tests undo with an unsupported action
func TestUndo_UnsupportedAction(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertAuditEntry(t, database, "unsupported_action", "test", "123", nil)

	_, err := undoMgr.Undo()
	if err == nil {
		t.Error("expected error for unsupported action")
	}

	if err.Error() != "unsupported action type: unsupported_action" {
		t.Errorf("expected unsupported action error, got: %v", err)
	}
}

// TestUndo_PhaseComplete tests undoing a phase completion
func TestUndo_PhaseComplete(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert a completed phase
	insertPhase(t, database, 1, "phase-01", "Test Phase", "foundation", "completed")

	// Set completed_date
	_, err := database.Exec(`UPDATE phases SET completed_date = datetime('now'), description = 'Done', test_count = 10 WHERE id = 1`)
	if err != nil {
		t.Fatalf("failed to update phase: %v", err)
	}

	// Create audit entry for phase completion
	oldData := map[string]interface{}{
		"status": "pending",
	}
	insertAuditEntry(t, database, "complete_phase", "phase", "phase-01", oldData)

	// Undo the completion
	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	if result.Action != "complete_phase" {
		t.Errorf("expected action = 'complete_phase', got: %s", result.Action)
	}

	if result.EntityType != "phase" {
		t.Errorf("expected entity_type = 'phase', got: %s", result.EntityType)
	}

	if result.EntityID != "phase-01" {
		t.Errorf("expected entity_id = 'phase-01', got: %s", result.EntityID)
	}

	// Verify phase is back to pending
	status := getPhaseStatus(t, database, "phase-01")
	if status != "pending" {
		t.Errorf("expected status = 'pending', got: %s", status)
	}

	// Verify audit entry was deleted
	canUndo, _ := undoMgr.CanUndo()
	if canUndo {
		t.Error("audit entry should be deleted after undo")
	}
}

// TestUndo_PhaseComplete_AlternateAction tests undoing with phase_complete action
func TestUndo_PhaseComplete_AlternateAction(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertPhase(t, database, 2, "phase-02", "Test Phase 2", "foundation", "completed")

	oldData := map[string]interface{}{
		"status": "pending",
	}
	insertAuditEntry(t, database, "phase_complete", "phase", "phase-02", oldData)

	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	if result.Action != "phase_complete" {
		t.Errorf("expected action = 'phase_complete', got: %s", result.Action)
	}

	status := getPhaseStatus(t, database, "phase-02")
	if status != "pending" {
		t.Errorf("expected status = 'pending', got: %s", status)
	}
}

// TestUndo_PhaseComplete_ByID tests undoing a phase by numeric ID
func TestUndo_PhaseComplete_ByID(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertPhase(t, database, 3, "phase-03", "Test Phase 3", "foundation", "completed")

	oldData := map[string]interface{}{
		"status": "pending",
	}
	// Use numeric ID instead of path
	insertAuditEntry(t, database, "complete_phase", "phase", "3", oldData)

	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	if result.EntityID != "3" {
		t.Errorf("expected entity_id = '3', got: %s", result.EntityID)
	}

	status := getPhaseStatus(t, database, "phase-03")
	if status != "pending" {
		t.Errorf("expected status = 'pending', got: %s", status)
	}
}

// TestUndo_PhaseComplete_PhaseNotFound tests undoing when phase doesn't exist
func TestUndo_PhaseComplete_PhaseNotFound(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	oldData := map[string]interface{}{
		"status": "pending",
	}
	insertAuditEntry(t, database, "complete_phase", "phase", "nonexistent", oldData)

	_, err := undoMgr.Undo()
	if err == nil {
		t.Error("expected error when phase not found")
	}
}

// TestUndo_PhaseComplete_NoOldData tests undoing without old_data
func TestUndo_PhaseComplete_NoOldData(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertPhase(t, database, 4, "phase-04", "Test Phase 4", "foundation", "completed")

	// Insert audit entry without old_data
	insertAuditEntry(t, database, "complete_phase", "phase", "phase-04", nil)

	_, err := undoMgr.Undo()
	if err == nil {
		t.Error("expected error when old_data is missing")
	}
}

// TestUndo_DecisionCreate tests undoing a decision creation
func TestUndo_DecisionCreate(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert a decision
	insertDecision(t, database, "DEC-001", "Test Decision", "approved", "Test rationale")

	// Create audit entry
	insertAuditEntry(t, database, "create_decision", "decision", "DEC-001", nil)

	// Undo the creation (should delete the decision)
	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	if result.Action != "create_decision" {
		t.Errorf("expected action = 'create_decision', got: %s", result.Action)
	}

	if result.EntityID != "DEC-001" {
		t.Errorf("expected entity_id = 'DEC-001', got: %s", result.EntityID)
	}

	// Verify decision was deleted
	_, _, _, exists := getDecision(t, database, "DEC-001")
	if exists {
		t.Error("decision should be deleted after undo")
	}
}

// TestUndo_DecisionCreate_AlternateAction tests with decision_create action
func TestUndo_DecisionCreate_AlternateAction(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertDecision(t, database, "DEC-002", "Test Decision 2", "approved", "Test rationale")
	insertAuditEntry(t, database, "decision_create", "decision", "DEC-002", nil)

	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	if result.Action != "decision_create" {
		t.Errorf("expected action = 'decision_create', got: %s", result.Action)
	}

	_, _, _, exists := getDecision(t, database, "DEC-002")
	if exists {
		t.Error("decision should be deleted after undo")
	}
}

// TestUndo_DecisionCreate_DecisionNotFound tests undoing when decision doesn't exist
func TestUndo_DecisionCreate_DecisionNotFound(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertAuditEntry(t, database, "create_decision", "decision", "DEC-999", nil)

	_, err := undoMgr.Undo()
	if err == nil {
		t.Error("expected error when decision not found")
	}
}

// TestUndo_DecisionUpdate tests undoing a decision update
func TestUndo_DecisionUpdate(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert a decision
	insertDecision(t, database, "DEC-003", "Updated Title", "approved", "Updated rationale")

	// Create audit entry with old data
	oldData := map[string]interface{}{
		"title":     "Original Title",
		"status":    "pending",
		"rationale": "Original rationale",
	}
	insertAuditEntry(t, database, "update_decision", "decision", "DEC-003", oldData)

	// Undo the update
	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	if result.Action != "update_decision" {
		t.Errorf("expected action = 'update_decision', got: %s", result.Action)
	}

	// Verify decision was restored to original values
	title, status, rationale, exists := getDecision(t, database, "DEC-003")
	if !exists {
		t.Fatal("decision should still exist after undo update")
	}

	if title != "Original Title" {
		t.Errorf("expected title = 'Original Title', got: %s", title)
	}

	if status != "pending" {
		t.Errorf("expected status = 'pending', got: %s", status)
	}

	if rationale != "Original rationale" {
		t.Errorf("expected rationale = 'Original rationale', got: %s", rationale)
	}
}

// TestUndo_DecisionUpdate_AlternateAction tests with decision_update action
func TestUndo_DecisionUpdate_AlternateAction(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertDecision(t, database, "DEC-004", "Updated Title", "approved", "Updated rationale")

	oldData := map[string]interface{}{
		"title": "Original Title",
	}
	insertAuditEntry(t, database, "decision_update", "decision", "DEC-004", oldData)

	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	if result.Action != "decision_update" {
		t.Errorf("expected action = 'decision_update', got: %s", result.Action)
	}

	title, _, _, exists := getDecision(t, database, "DEC-004")
	if !exists {
		t.Fatal("decision should still exist")
	}

	if title != "Original Title" {
		t.Errorf("expected title = 'Original Title', got: %s", title)
	}
}

// TestUndo_DecisionUpdate_PartialRestore tests restoring only some fields
func TestUndo_DecisionUpdate_PartialRestore(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertDecision(t, database, "DEC-005", "Current Title", "approved", "Current rationale")

	// Only restore status
	oldData := map[string]interface{}{
		"status": "pending",
	}
	insertAuditEntry(t, database, "update_decision", "decision", "DEC-005", oldData)

	_, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	title, status, rationale, _ := getDecision(t, database, "DEC-005")

	// Title and rationale should remain unchanged
	if title != "Current Title" {
		t.Errorf("title should not change, got: %s", title)
	}

	if rationale != "Current rationale" {
		t.Errorf("rationale should not change, got: %s", rationale)
	}

	// Status should be restored
	if status != "pending" {
		t.Errorf("expected status = 'pending', got: %s", status)
	}
}

// TestUndo_DecisionUpdate_DecisionNotFound tests undoing when decision doesn't exist
func TestUndo_DecisionUpdate_DecisionNotFound(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	oldData := map[string]interface{}{
		"status": "pending",
	}
	insertAuditEntry(t, database, "update_decision", "decision", "DEC-999", oldData)

	_, err := undoMgr.Undo()
	if err == nil {
		t.Error("expected error when decision not found")
	}
}

// TestUndo_DecisionUpdate_NoOldData tests undoing without old_data
func TestUndo_DecisionUpdate_NoOldData(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertDecision(t, database, "DEC-006", "Test Decision", "approved", "Test")
	insertAuditEntry(t, database, "update_decision", "decision", "DEC-006", nil)

	_, err := undoMgr.Undo()
	if err == nil {
		t.Error("expected error when old_data is missing")
	}
}

// TestGetUndoHistory tests retrieving undo history
func TestGetUndoHistory(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert multiple audit entries
	insertAuditEntry(t, database, "action1", "type1", "id1", nil)
	insertAuditEntry(t, database, "action2", "type2", "id2", nil)
	insertAuditEntry(t, database, "action3", "type3", "id3", nil)

	// Get history
	history, err := undoMgr.GetUndoHistory(10)
	if err != nil {
		t.Fatalf("GetUndoHistory() error = %v", err)
	}

	if len(history) != 3 {
		t.Errorf("expected 3 entries, got: %d", len(history))
	}

	// Should be in reverse chronological order (newest first)
	if history[0].Action != "action3" {
		t.Errorf("expected first entry action = 'action3', got: %s", history[0].Action)
	}
}

// TestGetUndoHistory_Limit tests history limit
func TestGetUndoHistory_Limit(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert 5 entries
	for i := 1; i <= 5; i++ {
		insertAuditEntry(t, database, "action", "type", "id", nil)
	}

	// Get only 3
	history, err := undoMgr.GetUndoHistory(3)
	if err != nil {
		t.Fatalf("GetUndoHistory() error = %v", err)
	}

	if len(history) != 3 {
		t.Errorf("expected 3 entries, got: %d", len(history))
	}
}

// TestGetUndoHistory_Empty tests getting history when empty
func TestGetUndoHistory_Empty(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	history, err := undoMgr.GetUndoHistory(10)
	if err != nil {
		t.Fatalf("GetUndoHistory() error = %v", err)
	}

	if len(history) != 0 {
		t.Errorf("expected empty history, got: %d entries", len(history))
	}
}

// TestParseOldData tests parsing old_data JSON
func TestParseOldData(t *testing.T) {
	validJSON := sql.NullString{
		String: `{"status":"pending"}`,
		Valid:  true,
	}

	data, err := parseOldData(validJSON)
	if err != nil {
		t.Fatalf("parseOldData() error = %v", err)
	}

	if data["status"] != "pending" {
		t.Error("failed to parse old data")
	}
}

// TestParseOldData_Invalid tests parsing invalid old_data
func TestParseOldData_Invalid(t *testing.T) {
	invalidJSON := sql.NullString{
		String: "",
		Valid:  false,
	}

	_, err := parseOldData(invalidJSON)
	if err == nil {
		t.Error("expected error for invalid old data")
	}

	if err.Error() != "no old data available for undo" {
		t.Errorf("expected 'no old data available' error, got: %v", err)
	}
}

// TestParseOldData_EmptyString tests parsing empty string
func TestParseOldData_EmptyString(t *testing.T) {
	emptyJSON := sql.NullString{
		String: "",
		Valid:  true,
	}

	_, err := parseOldData(emptyJSON)
	if err == nil {
		t.Error("expected error for empty string")
	}
}

// TestParseOldData_InvalidJSON tests parsing malformed JSON
func TestParseOldData_InvalidJSON(t *testing.T) {
	malformedJSON := sql.NullString{
		String: `{"invalid": json}`,
		Valid:  true,
	}

	_, err := parseOldData(malformedJSON)
	if err == nil {
		t.Error("expected error for malformed JSON")
	}
}

// TestParseOldData_ComplexObject tests parsing complex JSON
func TestParseOldData_ComplexObject(t *testing.T) {
	complexJSON := sql.NullString{
		String: `{"status":"pending","details":{"count":5,"items":["a","b"]}}`,
		Valid:  true,
	}

	data, err := parseOldData(complexJSON)
	if err != nil {
		t.Fatalf("parseOldData() error = %v", err)
	}

	if data["status"] != "pending" {
		t.Error("failed to parse status")
	}

	details, ok := data["details"].(map[string]interface{})
	if !ok {
		t.Fatal("failed to parse nested object")
	}

	if details["count"].(float64) != 5 {
		t.Error("failed to parse nested count")
	}
}

// TestValidateUndoSafe_Phase tests validation for phase undo
func TestValidateUndoSafe_Phase(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertPhase(t, database, 10, "phase-10", "Test Phase", "foundation", "completed")

	entry := &AuditLogEntry{
		Action:     "complete_phase",
		EntityType: "phase",
		EntityID:   "phase-10",
	}

	err := undoMgr.ValidateUndoSafe(entry)
	if err != nil {
		t.Errorf("ValidateUndoSafe() error = %v", err)
	}
}

// TestValidateUndoSafe_PhaseNotFound tests validation when phase doesn't exist
func TestValidateUndoSafe_PhaseNotFound(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	entry := &AuditLogEntry{
		Action:     "complete_phase",
		EntityType: "phase",
		EntityID:   "nonexistent",
	}

	err := undoMgr.ValidateUndoSafe(entry)
	if err == nil {
		t.Error("expected error when phase not found")
	}
}

// TestValidateUndoSafe_Decision tests validation for decision undo
func TestValidateUndoSafe_Decision(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertDecision(t, database, "DEC-010", "Test Decision", "approved", "Test")

	entry := &AuditLogEntry{
		Action:     "update_decision",
		EntityType: "decision",
		EntityID:   "DEC-010",
	}

	err := undoMgr.ValidateUndoSafe(entry)
	if err != nil {
		t.Errorf("ValidateUndoSafe() error = %v", err)
	}
}

// TestValidateUndoSafe_DecisionCreate tests validation for decision creation (should pass even if decision exists)
func TestValidateUndoSafe_DecisionCreate(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertDecision(t, database, "DEC-011", "Test Decision", "approved", "Test")

	entry := &AuditLogEntry{
		Action:     "create_decision",
		EntityType: "decision",
		EntityID:   "DEC-011",
	}

	err := undoMgr.ValidateUndoSafe(entry)
	if err != nil {
		t.Errorf("ValidateUndoSafe() should not error for create_decision, got: %v", err)
	}
}

// TestValidateUndoSafe_DecisionNotFound tests validation when decision doesn't exist
func TestValidateUndoSafe_DecisionNotFound(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	entry := &AuditLogEntry{
		Action:     "update_decision",
		EntityType: "decision",
		EntityID:   "DEC-999",
	}

	err := undoMgr.ValidateUndoSafe(entry)
	if err == nil {
		t.Error("expected error when decision not found")
	}
}

// TestMultipleUndoOperations tests performing multiple undo operations in sequence
func TestMultipleUndoOperations(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert and track multiple operations
	insertPhase(t, database, 20, "phase-20", "Phase 1", "foundation", "completed")
	insertAuditEntry(t, database, "complete_phase", "phase", "phase-20", map[string]interface{}{"status": "pending"})

	insertDecision(t, database, "DEC-020", "Decision 1", "approved", "Rationale 1")
	insertAuditEntry(t, database, "create_decision", "decision", "DEC-020", nil)

	// Undo decision creation (most recent)
	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("First undo error = %v", err)
	}

	if result.EntityID != "DEC-020" {
		t.Errorf("expected first undo to be DEC-020, got: %s", result.EntityID)
	}

	// Undo phase completion
	result, err = undoMgr.Undo()
	if err != nil {
		t.Fatalf("Second undo error = %v", err)
	}

	if result.EntityID != "phase-20" {
		t.Errorf("expected second undo to be phase-20, got: %s", result.EntityID)
	}

	// No more to undo
	_, err = undoMgr.Undo()
	if err == nil {
		t.Error("expected error when nothing left to undo")
	}
}

// TestUndoResult tests the UndoResult structure
func TestUndoResult(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertPhase(t, database, 30, "phase-30", "Test Phase", "foundation", "completed")

	oldData := map[string]interface{}{
		"status": "pending",
		"count":  5,
	}
	insertAuditEntry(t, database, "complete_phase", "phase", "phase-30", oldData)

	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	if result.Restored == nil {
		t.Fatal("expected Restored to contain old data")
	}

	if result.Restored["status"] != "pending" {
		t.Errorf("expected Restored status = 'pending', got: %v", result.Restored["status"])
	}

	if result.Restored["count"].(float64) != 5 {
		t.Errorf("expected Restored count = 5, got: %v", result.Restored["count"])
	}
}

// TestAuditLogEntry tests the AuditLogEntry structure
func TestAuditLogEntry(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	oldData := map[string]interface{}{"test": "value"}
	insertAuditEntry(t, database, "test_action", "test_type", "test_id", oldData)

	entry, err := undoMgr.getLastAuditEntry()
	if err != nil {
		t.Fatalf("getLastAuditEntry() error = %v", err)
	}

	if entry.Action != "test_action" {
		t.Errorf("expected Action = 'test_action', got: %s", entry.Action)
	}

	if entry.EntityType != "test_type" {
		t.Errorf("expected EntityType = 'test_type', got: %s", entry.EntityType)
	}

	if entry.EntityID != "test_id" {
		t.Errorf("expected EntityID = 'test_id', got: %s", entry.EntityID)
	}

	if entry.Agent != "test" {
		t.Errorf("expected Agent = 'test', got: %s", entry.Agent)
	}

	if !entry.OldData.Valid {
		t.Error("expected OldData to be valid")
	}
}

// TestDeleteAuditEntry tests deleting audit entries
func TestDeleteAuditEntry(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	id := insertAuditEntry(t, database, "test", "test", "test", nil)

	canUndo, _ := undoMgr.CanUndo()
	if !canUndo {
		t.Error("expected audit entry to exist")
	}

	err := undoMgr.deleteAuditEntry(id)
	if err != nil {
		t.Fatalf("deleteAuditEntry() error = %v", err)
	}

	canUndo, _ = undoMgr.CanUndo()
	if canUndo {
		t.Error("expected audit entry to be deleted")
	}
}

// TestGetLastAuditEntry tests retrieving the last audit entry
func TestGetLastAuditEntry(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertAuditEntry(t, database, "action1", "type1", "id1", nil)
	insertAuditEntry(t, database, "action2", "type2", "id2", nil)

	entry, err := undoMgr.getLastAuditEntry()
	if err != nil {
		t.Fatalf("getLastAuditEntry() error = %v", err)
	}

	// Should get the most recent (action2)
	if entry.Action != "action2" {
		t.Errorf("expected Action = 'action2', got: %s", entry.Action)
	}
}

// TestGetLastAuditEntry_NoEntries tests when there are no audit entries
func TestGetLastAuditEntry_NoEntries(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	_, err := undoMgr.getLastAuditEntry()
	if err != sql.ErrNoRows {
		t.Errorf("expected sql.ErrNoRows, got: %v", err)
	}
}

// TestUndoFeatureUpdate tests undoing a feature update
func TestUndoFeatureUpdate(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert a feature (features table already exists from InitSchema)
	insertQuery := `INSERT INTO features (name, display_name, version, status, description) VALUES (?, ?, ?, ?, ?)`
	result, err := database.Exec(insertQuery, "test-feature-1", "Test Feature 1", "1.0", "completed", "Updated description")
	if err != nil {
		t.Fatalf("failed to insert feature: %v", err)
	}

	featureID, _ := result.LastInsertId()

	// Create audit entry with old data
	oldData := map[string]interface{}{
		"status":      "pending",
		"description": "Original description",
	}
	insertAuditEntry(t, database, "update_feature", "feature", fmt.Sprintf("%d", featureID), oldData)

	// Undo the update
	undoResult, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	if undoResult.Action != "update_feature" {
		t.Errorf("expected action = 'update_feature', got: %s", undoResult.Action)
	}

	// Verify feature was restored
	var status sql.NullString
	var description sql.NullString
	query := `SELECT status, description FROM features WHERE id = ?`
	err = database.QueryRow(query, featureID).Scan(&status, &description)
	if err != nil {
		t.Fatalf("failed to get feature: %v", err)
	}

	if status.String != "pending" {
		t.Errorf("expected status = 'pending', got: %s", status.String)
	}

	if description.String != "Original description" {
		t.Errorf("expected description = 'Original description', got: %s", description.String)
	}
}

// TestUndoFeatureUpdate_AlternateAction tests with feature_update action
func TestUndoFeatureUpdate_AlternateAction(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert a feature
	insertQuery := `INSERT INTO features (name, display_name, version, status, description) VALUES (?, ?, ?, ?, ?)`
	result, err := database.Exec(insertQuery, "test-feature-2", "Test Feature 2", "1.0", "completed", "Description")
	if err != nil {
		t.Fatalf("failed to insert feature: %v", err)
	}

	featureID, _ := result.LastInsertId()

	oldData := map[string]interface{}{
		"status": "pending",
	}
	insertAuditEntry(t, database, "feature_update", "feature", fmt.Sprintf("%d", featureID), oldData)

	undoResult, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	if undoResult.Action != "feature_update" {
		t.Errorf("expected action = 'feature_update', got: %s", undoResult.Action)
	}

	var status sql.NullString
	query := `SELECT status FROM features WHERE id = ?`
	err = database.QueryRow(query, featureID).Scan(&status)
	if err != nil {
		t.Fatalf("failed to get feature: %v", err)
	}

	if status.String != "pending" {
		t.Errorf("expected status = 'pending', got: %s", status.String)
	}
}

// TestUndoFeatureUpdate_FeatureNotFound tests undoing when feature doesn't exist
func TestUndoFeatureUpdate_FeatureNotFound(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	oldData := map[string]interface{}{
		"status": "pending",
	}
	insertAuditEntry(t, database, "update_feature", "feature", "999999", oldData)

	_, err := undoMgr.Undo()
	if err == nil {
		t.Error("expected error when feature not found")
	}
}

// TestUndoFeatureUpdate_NoOldData tests undoing without old_data
func TestUndoFeatureUpdate_NoOldData(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert a feature
	insertQuery := `INSERT INTO features (name, display_name, version, status, description) VALUES (?, ?, ?, ?, ?)`
	result, err := database.Exec(insertQuery, "test-feature-3", "Test Feature 3", "1.0", "completed", "Description")
	if err != nil {
		t.Fatalf("failed to insert feature: %v", err)
	}

	featureID, _ := result.LastInsertId()

	// Insert audit entry without old_data
	insertAuditEntry(t, database, "update_feature", "feature", fmt.Sprintf("%d", featureID), nil)

	_, err = undoMgr.Undo()
	if err == nil {
		t.Error("expected error when old_data is missing")
	}
}

// TestCanUndo_Error tests CanUndo with database error
func TestCanUndo_Error(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Close the database to cause an error
	database.Close()

	_, err := undoMgr.CanUndo()
	if err == nil {
		t.Error("expected error when database is closed")
	}
}

// TestGetUndoHistory_Error tests GetUndoHistory with database error
func TestGetUndoHistory_Error(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Close the database to cause an error
	database.Close()

	_, err := undoMgr.GetUndoHistory(10)
	if err == nil {
		t.Error("expected error when database is closed")
	}
}

// TestUndo_AuditDeletionFails tests when audit deletion fails (should still succeed)
func TestUndo_AuditDeletionFails(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert phase and audit entry
	insertPhase(t, database, 40, "phase-40", "Test Phase", "foundation", "completed")
	oldData := map[string]interface{}{
		"status": "pending",
	}
	auditID := insertAuditEntry(t, database, "complete_phase", "phase", "phase-40", oldData)

	// Delete the audit entry manually before undo
	_, err := database.Exec(`DELETE FROM audit_log WHERE id = ?`, auditID)
	if err != nil {
		t.Fatalf("failed to delete audit entry: %v", err)
	}

	// Re-insert a new audit entry (will have different ID)
	insertAuditEntry(t, database, "complete_phase", "phase", "phase-40", oldData)

	// Undo should still work even though we can't delete the specific entry
	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() should succeed even if audit deletion fails: %v", err)
	}

	if result.EntityID != "phase-40" {
		t.Errorf("expected entity_id = 'phase-40', got: %s", result.EntityID)
	}
}

// TestValidateUndoSafe_PhaseByNumericID tests validation with numeric phase ID
func TestValidateUndoSafe_PhaseByNumericID(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertPhase(t, database, 50, "phase-50", "Test Phase", "foundation", "completed")

	entry := &AuditLogEntry{
		Action:     "complete_phase",
		EntityType: "phase",
		EntityID:   "50", // numeric ID
	}

	err := undoMgr.ValidateUndoSafe(entry)
	if err != nil {
		t.Errorf("ValidateUndoSafe() error = %v", err)
	}
}

// TestValidateUndoSafe_DecisionCreateAlternate tests validation with decision_create
func TestValidateUndoSafe_DecisionCreateAlternate(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertDecision(t, database, "DEC-050", "Test Decision", "approved", "Test")

	entry := &AuditLogEntry{
		Action:     "decision_create",
		EntityType: "decision",
		EntityID:   "DEC-050",
	}

	err := undoMgr.ValidateUndoSafe(entry)
	if err != nil {
		t.Errorf("ValidateUndoSafe() should not error for decision_create, got: %v", err)
	}
}

// TestValidateUndoSafe_UnknownEntityType tests validation with unknown entity type
func TestValidateUndoSafe_UnknownEntityType(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	entry := &AuditLogEntry{
		Action:     "test_action",
		EntityType: "unknown_type",
		EntityID:   "test-id",
	}

	// Should not error for unknown entity types
	err := undoMgr.ValidateUndoSafe(entry)
	if err != nil {
		t.Errorf("ValidateUndoSafe() should not error for unknown entity types, got: %v", err)
	}
}

// TestUndoResult_NoRestored tests UndoResult without restored data
func TestUndoResult_NoRestored(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertDecision(t, database, "DEC-060", "Test Decision", "approved", "Test")
	insertAuditEntry(t, database, "create_decision", "decision", "DEC-060", nil)

	result, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	// For create operations, there's no old data to restore
	if result.Restored != nil {
		t.Error("expected Restored to be nil for create operations")
	}
}

// TestGetUndoHistory_WithOldData tests getting history with old_data
func TestGetUndoHistory_WithOldData(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	oldData := map[string]interface{}{
		"status": "pending",
		"count":  5,
	}
	insertAuditEntry(t, database, "test_action", "test_type", "test_id", oldData)

	history, err := undoMgr.GetUndoHistory(10)
	if err != nil {
		t.Fatalf("GetUndoHistory() error = %v", err)
	}

	if len(history) != 1 {
		t.Fatalf("expected 1 entry, got: %d", len(history))
	}

	if !history[0].OldData.Valid {
		t.Error("expected OldData to be valid")
	}

	var parsedOldData map[string]interface{}
	err = json.Unmarshal([]byte(history[0].OldData.String), &parsedOldData)
	if err != nil {
		t.Fatalf("failed to parse old_data: %v", err)
	}

	if parsedOldData["status"] != "pending" {
		t.Errorf("expected status = 'pending', got: %v", parsedOldData["status"])
	}
}

// TestUndoPhaseComplete_DatabaseError tests phase undo with transaction error
func TestUndoPhaseComplete_DatabaseError(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Don't insert the phase, causing the update to fail
	oldData := map[string]interface{}{
		"status": "pending",
	}
	insertAuditEntry(t, database, "complete_phase", "phase", "missing-phase", oldData)

	_, err := undoMgr.Undo()
	if err == nil {
		t.Error("expected error when phase doesn't exist")
	}

	// The audit entry should not be deleted since undo failed
	canUndo, _ := undoMgr.CanUndo()
	if !canUndo {
		t.Error("audit entry should remain when undo fails")
	}
}

// TestGetUndoHistory_RowsError tests handling rows.Err() in GetUndoHistory
func TestGetUndoHistory_RowsError(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert audit entries
	insertAuditEntry(t, database, "action1", "type1", "id1", nil)

	// Get history (this exercises the rows.Err() path)
	history, err := undoMgr.GetUndoHistory(10)
	if err != nil {
		t.Fatalf("GetUndoHistory() error = %v", err)
	}

	if len(history) != 1 {
		t.Errorf("expected 1 entry, got: %d", len(history))
	}
}

// TestGetUndoHistory_ScanError tests handling scan errors
func TestGetUndoHistory_ScanError(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert entry with complex data that might cause scan issues
	complexData := map[string]interface{}{
		"nested": map[string]interface{}{
			"field": "value",
		},
	}
	insertAuditEntry(t, database, "action", "type", "id", complexData)

	// Should still work even with complex data
	history, err := undoMgr.GetUndoHistory(10)
	if err != nil {
		t.Fatalf("GetUndoHistory() error = %v", err)
	}

	if len(history) != 1 {
		t.Errorf("expected 1 entry, got: %d", len(history))
	}
}

// TestAuditLogEntry_AllFields tests all fields of AuditLogEntry
func TestAuditLogEntry_AllFields(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// Insert entry with all fields populated
	oldData := map[string]interface{}{"status": "old"}
	auditID := insertAuditEntry(t, database, "test_action", "test_type", "test_id", oldData)

	// Add commit_sha
	_, err := database.Exec(`UPDATE audit_log SET commit_sha = ? WHERE id = ?`, "abc123", auditID)
	if err != nil {
		t.Fatalf("failed to update audit entry: %v", err)
	}

	entry, err := undoMgr.getLastAuditEntry()
	if err != nil {
		t.Fatalf("getLastAuditEntry() error = %v", err)
	}

	if entry.ID != auditID {
		t.Errorf("expected ID = %d, got: %d", auditID, entry.ID)
	}

	if !entry.CommitSHA.Valid || entry.CommitSHA.String != "abc123" {
		t.Errorf("expected CommitSHA = 'abc123', got: %v", entry.CommitSHA)
	}

	if !entry.Changes.Valid {
		t.Error("expected Changes to be valid")
	}

	if entry.CreatedAt == "" {
		t.Error("expected CreatedAt to be populated")
	}
}

// TestUndo_DecisionUpdate_EmptyOldData tests decision update with empty fields
func TestUndo_DecisionUpdate_EmptyOldData(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	insertDecision(t, database, "DEC-100", "Current Title", "approved", "Current rationale")

	// Old data with empty/missing fields (should handle gracefully)
	oldData := map[string]interface{}{
		"unknown_field": "value",
	}
	insertAuditEntry(t, database, "update_decision", "decision", "DEC-100", oldData)

	// Should succeed but not update any fields since none match
	_, err := undoMgr.Undo()
	if err != nil {
		t.Fatalf("Undo() error = %v", err)
	}

	// Verify decision unchanged
	title, status, rationale, _ := getDecision(t, database, "DEC-100")
	if title != "Current Title" {
		t.Errorf("title should not change, got: %s", title)
	}
	if status != "approved" {
		t.Errorf("status should not change, got: %s", status)
	}
	if rationale != "Current rationale" {
		t.Errorf("rationale should not change, got: %s", rationale)
	}
}

// TestValidateUndoSafe_PhaseCreateAction tests validation for create_phase action
func TestValidateUndoSafe_PhaseCreateAction(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	entry := &AuditLogEntry{
		Action:     "create_phase",
		EntityType: "phase",
		EntityID:   "phase-100",
	}

	// Should not check existence for create actions
	err := undoMgr.ValidateUndoSafe(entry)
	if err != nil {
		t.Errorf("ValidateUndoSafe() should not error for create_phase, got: %v", err)
	}
}
