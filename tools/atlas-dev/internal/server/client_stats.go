package server

import (
	"encoding/json"
	"log/slog"
	"time"
)

// SendStatsToClient sends current stats to a specific client
func (w *Watcher) SendStatsToClient(client *Client) {
	// Get current stats
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
		slog.Error("failed to query phase stats for client", "error", err)
		return
	}

	// Get decision count
	if err := w.db.QueryRow("SELECT COUNT(*) FROM decisions").Scan(&stats.TotalDecisions); err != nil {
		slog.Debug("failed to query decision count", "error", err)
		stats.TotalDecisions = 0
	}

	// Get feature count
	if err := w.db.QueryRow("SELECT COUNT(*) FROM features").Scan(&stats.TotalFeatures); err != nil {
		slog.Debug("failed to query feature count", "error", err)
		stats.TotalFeatures = 0
	}

	// Calculate completion rate
	if stats.TotalPhases > 0 {
		stats.CompletionRate = float64(stats.CompletedPhases) / float64(stats.TotalPhases) * 100
	} else {
		stats.CompletionRate = 0
	}

	// Send directly to this client
	msg := Message{
		Type: MessageTypeStatsChanged,
	}

	data, err := jsonMarshal(stats)
	if err != nil {
		slog.Error("failed to marshal stats for client", "error", err)
		return
	}
	msg.Payload = data

	select {
	case client.send <- msg:
		slog.Debug("sent initial stats to new client", "total", stats.TotalPhases, "completed", stats.CompletedPhases)
	default:
		slog.Debug("client send channel full, skipping initial stats")
	}
}

// Helper to avoid circular import
func jsonMarshal(v interface{}) ([]byte, error) {
	return json.Marshal(v)
}
