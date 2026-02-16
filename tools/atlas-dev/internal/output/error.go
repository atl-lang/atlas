package output

import (
	"errors"
	"os"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

// Exit codes as defined in AI-OPTIMIZATION.md
const (
	ExitSuccess        = 0
	ExitInvalidArgs    = 1
	ExitNotFound       = 2
	ExitValidationFail = 3
	ExitGitFailed      = 4
	ExitCacheError     = 5
	ExitPermDenied     = 6
)

// ErrorWithCode outputs structured JSON error and returns exit code
func ErrorWithCode(err error) int {
	code := ExitInvalidArgs // Default: invalid arguments
	details := make(map[string]interface{})

	// Map error to exit code
	switch {
	case errors.Is(err, db.ErrPhaseNotFound):
		code = ExitNotFound
	case errors.Is(err, db.ErrCategoryNotFound):
		code = ExitNotFound
	case errors.Is(err, db.ErrDecisionNotFound):
		code = ExitNotFound
	case errors.Is(err, db.ErrFeatureNotFound):
		code = ExitNotFound
	case errors.Is(err, db.ErrPhaseAlreadyDone):
		code = ExitValidationFail
	case errors.Is(err, db.ErrInvalidStatus):
		code = ExitValidationFail
	case errors.Is(err, db.ErrInvalidSchema):
		code = ExitValidationFail
	case errors.Is(err, db.ErrDatabaseLocked):
		code = ExitPermDenied
	}

	// Output JSON
	_ = Error(err, details) // Best effort

	return code
}

// Fatal outputs error and exits with appropriate code
func Fatal(err error) {
	code := ErrorWithCode(err)
	os.Exit(code)
}
