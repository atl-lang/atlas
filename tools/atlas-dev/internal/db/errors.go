package db

import "errors"

// Sentinel errors for database operations
var (
	ErrPhaseNotFound     = errors.New("phase not found")
	ErrPhaseAlreadyDone  = errors.New("phase already completed")
	ErrInvalidStatus     = errors.New("invalid status")
	ErrCategoryNotFound  = errors.New("category not found")
	ErrDecisionNotFound  = errors.New("decision not found")
	ErrFeatureNotFound   = errors.New("feature not found")
	ErrDatabaseLocked    = errors.New("database is locked")
	ErrInvalidSchema     = errors.New("invalid schema version")
)
