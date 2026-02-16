package server

import (
	"context"
	"log/slog"
	"time"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

// Watcher monitors database changes and broadcasts updates
type Watcher struct {
	db       *db.DB
	hub      *Hub
	interval time.Duration
	state    *watchState
}

// watchState tracks the last known state for change detection
type watchState struct {
	lastPhaseUpdate    time.Time
	lastDecisionUpdate time.Time
	lastFeatureUpdate  time.Time
	lastSpecUpdate     time.Time
	lastAPIUpdate      time.Time
	stats              *Stats
}

// Stats represents current database statistics
type Stats struct {
	TotalPhases      int     `json:"total_phases"`
	CompletedPhases  int     `json:"completed_phases"`
	PendingPhases    int     `json:"pending_phases"`
	TotalDecisions   int     `json:"total_decisions"`
	TotalFeatures    int     `json:"total_features"`
	CompletionRate   float64 `json:"completion_rate"`
	LastUpdate       string  `json:"last_update"`
}

// PhaseUpdate represents a phase update event
type PhaseUpdate struct {
	ID          int    `json:"id"`
	Name        string `json:"name"`
	Category    string `json:"category"`
	Status      string `json:"status"`
	UpdatedAt   string `json:"updated_at"`
}

// DecisionUpdate represents a decision update event
type DecisionUpdate struct {
	ID        string `json:"id"`
	Component string `json:"component"`
	Title     string `json:"title"`
	Status    string `json:"status"`
	Date      string `json:"date"`
}

// FeatureUpdate represents a feature update event
type FeatureUpdate struct {
	ID          int    `json:"id"`
	Name        string `json:"name"`
	DisplayName string `json:"display_name"`
	Status      string `json:"status"`
	Version     string `json:"version"`
	UpdatedAt   string `json:"updated_at"`
}

// NewWatcher creates a new database watcher
func NewWatcher(database *db.DB, hub *Hub, interval time.Duration) *Watcher {
	return &Watcher{
		db:       database,
		hub:      hub,
		interval: interval,
		state: &watchState{
			lastPhaseUpdate:    time.Time{},
			lastDecisionUpdate: time.Time{},
			lastFeatureUpdate:  time.Time{},
			lastSpecUpdate:     time.Time{},
			lastAPIUpdate:      time.Time{},
		},
	}
}

// Run starts the watcher's polling loop
func (w *Watcher) Run(ctx context.Context) {
	slog.Info("database watcher started", "interval", w.interval)

	// Send initial stats immediately (with a small delay to let clients connect)
	go func() {
		time.Sleep(500 * time.Millisecond)
		slog.Info("sending initial stats broadcast")
		w.checkStats(ctx)
	}()

	ticker := time.NewTicker(w.interval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			slog.Info("database watcher shutting down")
			return
		case <-ticker.C:
			w.checkChanges(ctx)
		}
	}
}

// checkChanges checks for database changes and broadcasts updates
func (w *Watcher) checkChanges(ctx context.Context) {
	// Check for phase updates
	if phases, err := w.checkPhaseUpdates(ctx); err == nil && len(phases) > 0 {
		for _, phase := range phases {
			w.hub.Broadcast(MessageTypePhaseUpdate, phase)
		}
	}

	// Check for decision updates
	if decisions, err := w.checkDecisionUpdates(ctx); err == nil && len(decisions) > 0 {
		for _, decision := range decisions {
			w.hub.Broadcast(MessageTypeDecisionAdded, decision)
		}
	}

	// Check for feature updates
	if features, err := w.checkFeatureUpdates(ctx); err == nil && len(features) > 0 {
		for _, feature := range features {
			w.hub.Broadcast(MessageTypeFeatureUpdate, feature)
		}
	}

	// Always check stats
	w.checkStats(ctx)
}

// checkPhaseUpdates queries for phases updated since last check
func (w *Watcher) checkPhaseUpdates(ctx context.Context) ([]PhaseUpdate, error) {
	query := `
		SELECT id, name, category, status, updated_at
		FROM phases
		WHERE updated_at > ?
		ORDER BY updated_at DESC
		LIMIT 10
	`

	rows, err := w.db.Query(query, w.state.lastPhaseUpdate.Format(time.RFC3339))
	if err != nil {
		slog.Error("failed to query phase updates", "error", err)
		return nil, err
	}
	defer rows.Close()

	var updates []PhaseUpdate
	var latestUpdate time.Time

	for rows.Next() {
		var u PhaseUpdate
		var updatedAt string
		if err := rows.Scan(&u.ID, &u.Name, &u.Category, &u.Status, &updatedAt); err != nil {
			continue
		}
		u.UpdatedAt = updatedAt
		updates = append(updates, u)

		if t, err := time.Parse(time.RFC3339, updatedAt); err == nil && t.After(latestUpdate) {
			latestUpdate = t
		}
	}

	if !latestUpdate.IsZero() {
		w.state.lastPhaseUpdate = latestUpdate
	}

	return updates, nil
}

// checkDecisionUpdates queries for decisions updated since last check
func (w *Watcher) checkDecisionUpdates(ctx context.Context) ([]DecisionUpdate, error) {
	query := `
		SELECT id, component, title, status, date, updated_at
		FROM decisions
		WHERE updated_at > ?
		ORDER BY updated_at DESC
		LIMIT 10
	`

	rows, err := w.db.Query(query, w.state.lastDecisionUpdate.Format(time.RFC3339))
	if err != nil {
		slog.Error("failed to query decision updates", "error", err)
		return nil, err
	}
	defer rows.Close()

	var updates []DecisionUpdate
	var latestUpdate time.Time

	for rows.Next() {
		var u DecisionUpdate
		var updatedAt string
		if err := rows.Scan(&u.ID, &u.Component, &u.Title, &u.Status, &u.Date, &updatedAt); err != nil {
			continue
		}
		updates = append(updates, u)

		if t, err := time.Parse(time.RFC3339, updatedAt); err == nil && t.After(latestUpdate) {
			latestUpdate = t
		}
	}

	if !latestUpdate.IsZero() {
		w.state.lastDecisionUpdate = latestUpdate
	}

	return updates, nil
}

// checkFeatureUpdates queries for features updated since last check
func (w *Watcher) checkFeatureUpdates(ctx context.Context) ([]FeatureUpdate, error) {
	query := `
		SELECT id, name, display_name, status, version, updated_at
		FROM features
		WHERE updated_at > ?
		ORDER BY updated_at DESC
		LIMIT 10
	`

	rows, err := w.db.Query(query, w.state.lastFeatureUpdate.Format(time.RFC3339))
	if err != nil {
		slog.Error("failed to query feature updates", "error", err)
		return nil, err
	}
	defer rows.Close()

	var updates []FeatureUpdate
	var latestUpdate time.Time

	for rows.Next() {
		var u FeatureUpdate
		var updatedAt string
		if err := rows.Scan(&u.ID, &u.Name, &u.DisplayName, &u.Status, &u.Version, &updatedAt); err != nil {
			continue
		}
		u.UpdatedAt = updatedAt
		updates = append(updates, u)

		if t, err := time.Parse(time.RFC3339, updatedAt); err == nil && t.After(latestUpdate) {
			latestUpdate = t
		}
	}

	if !latestUpdate.IsZero() {
		w.state.lastFeatureUpdate = latestUpdate
	}

	return updates, nil
}

// checkStats calculates and broadcasts current stats
func (w *Watcher) checkStats(ctx context.Context) {
	stats := &Stats{
		LastUpdate: time.Now().Format(time.RFC3339),
	}

	// Get phase stats
	query := `
		SELECT
			COUNT(*) as total,
			COALESCE(SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END), 0) as completed,
			COALESCE(SUM(CASE WHEN status != 'completed' THEN 1 ELSE 0 END), 0) as pending
		FROM phases
	`
	if err := w.db.QueryRow(query).Scan(&stats.TotalPhases, &stats.CompletedPhases, &stats.PendingPhases); err != nil {
		slog.Error("failed to query phase stats", "error", err)
		return
	}

	// Get decision count
	if err := w.db.QueryRow("SELECT COUNT(*) FROM decisions").Scan(&stats.TotalDecisions); err != nil {
		slog.Error("failed to query decision count", "error", err)
		stats.TotalDecisions = 0
	}

	// Get feature count
	if err := w.db.QueryRow("SELECT COUNT(*) FROM features").Scan(&stats.TotalFeatures); err != nil {
		slog.Error("failed to query feature count", "error", err)
		stats.TotalFeatures = 0
	}

	// Calculate completion rate
	if stats.TotalPhases > 0 {
		stats.CompletionRate = float64(stats.CompletedPhases) / float64(stats.TotalPhases) * 100
	} else {
		stats.CompletionRate = 0
	}

	// Only broadcast if stats changed (reduce noise)
	if w.state.stats == nil || statsChanged(w.state.stats, stats) {
		if err := w.hub.Broadcast(MessageTypeStatsChanged, stats); err != nil {
			slog.Error("failed to broadcast stats", "error", err)
			return
		}
		slog.Debug("stats broadcast", "total", stats.TotalPhases, "completed", stats.CompletedPhases)
		w.state.stats = stats
	}
}

// statsChanged checks if stats have changed
func statsChanged(old, new *Stats) bool {
	return old.TotalPhases != new.TotalPhases ||
		old.CompletedPhases != new.CompletedPhases ||
		old.PendingPhases != new.PendingPhases ||
		old.TotalDecisions != new.TotalDecisions ||
		old.TotalFeatures != new.TotalFeatures
}

