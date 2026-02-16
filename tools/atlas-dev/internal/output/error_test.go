package output

import (
	"os"
	"testing"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

func TestErrorWithCode(t *testing.T) {
	tests := []struct {
		name     string
		err      error
		wantCode int
	}{
		{"phase not found", db.ErrPhaseNotFound, ExitNotFound},
		{"category not found", db.ErrCategoryNotFound, ExitNotFound},
		{"decision not found", db.ErrDecisionNotFound, ExitNotFound},
		{"feature not found", db.ErrFeatureNotFound, ExitNotFound},
		{"phase already done", db.ErrPhaseAlreadyDone, ExitValidationFail},
		{"invalid status", db.ErrInvalidStatus, ExitValidationFail},
		{"invalid schema", db.ErrInvalidSchema, ExitValidationFail},
		{"database locked", db.ErrDatabaseLocked, ExitPermDenied},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Suppress output for test
			old := os.Stdout
			os.Stdout, _ = os.Open(os.DevNull)
			defer func() { os.Stdout = old }()

			code := ErrorWithCode(tt.err)
			if code != tt.wantCode {
				t.Errorf("ErrorWithCode() = %d, want %d", code, tt.wantCode)
			}
		})
	}
}

func TestExitCodes(t *testing.T) {
	tests := []struct {
		name string
		code int
	}{
		{"success", ExitSuccess},
		{"invalid args", ExitInvalidArgs},
		{"not found", ExitNotFound},
		{"validation fail", ExitValidationFail},
		{"git failed", ExitGitFailed},
		{"cache error", ExitCacheError},
		{"perm denied", ExitPermDenied},
	}

	expectedCodes := []int{0, 1, 2, 3, 4, 5, 6}

	for i, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.code != expectedCodes[i] {
				t.Errorf("%s code = %d, want %d", tt.name, tt.code, expectedCodes[i])
			}
		})
	}
}
