package main

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"time"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/server"
	"nhooyr.io/websocket"
)

// runServer starts the monitoring dashboard server
func runServer(port int) error {
	// Verify database is initialized
	if database == nil {
		return fmt.Errorf("database not initialized")
	}

	slog.Info("starting monitoring dashboard", "port", port, "db", dbPath)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Create WebSocket hub
	hub := server.NewHub()
	go hub.Run(ctx)

	// Create database watcher (1 second poll rate - reasonable for dev monitoring)
	watcher := server.NewWatcher(database, hub, 1*time.Second)
	go watcher.Run(ctx)

	// Create HTTP server
	mux := http.NewServeMux()

	// WebSocket endpoint
	mux.HandleFunc("/ws", func(w http.ResponseWriter, r *http.Request) {
		handleWebSocket(w, r, hub, watcher)
	})

	// Serve static files from disk (like xcalibr does)
	webDir := "tools/atlas-dev/web"
	if _, err := os.Stat(webDir); os.IsNotExist(err) {
		// Try relative path if absolute doesn't work
		webDir = "web"
	}
	slog.Info("serving static files", "dir", webDir)
	mux.Handle("/", http.FileServer(http.Dir(webDir)))

	// Health check endpoint
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		fmt.Fprintf(w, `{"status":"ok","clients":%d}`, hub.ClientCount())
	})

	addr := fmt.Sprintf(":%d", port)
	srv := &http.Server{
		Addr:    addr,
		Handler: mux,
	}

	// Handle shutdown gracefully
	sigint := make(chan os.Signal, 1)
	signal.Notify(sigint, os.Interrupt)

	go func() {
		<-sigint
		slog.Info("shutting down server...")
		cancel()
		shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer shutdownCancel()
		if err := srv.Shutdown(shutdownCtx); err != nil {
			slog.Error("server shutdown error", "error", err)
		}
	}()

	// Start server
	if err := output.Success(map[string]interface{}{
		"server": "started",
		"port":   port,
		"url":    fmt.Sprintf("http://localhost:%d", port),
	}); err != nil {
		return err
	}

	slog.Info("monitoring dashboard started", "url", fmt.Sprintf("http://localhost:%d", port))

	if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
		return fmt.Errorf("server error: %w", err)
	}

	return nil
}

// handleWebSocket upgrades HTTP connection to WebSocket and handles client communication
func handleWebSocket(w http.ResponseWriter, r *http.Request, hub *server.Hub, watcher *server.Watcher) {
	conn, err := websocket.Accept(w, r, &websocket.AcceptOptions{
		OriginPatterns: []string{"*"}, // Allow all origins
	})
	if err != nil {
		slog.Error("websocket accept failed", "error", err)
		return
	}

	client := hub.RegisterClient(conn)
	ctx := r.Context()

	// Send current stats immediately to new client
	go watcher.SendStatsToClient(client)

	// Start read and write pumps
	go client.WritePump(ctx)
	client.ReadPump(ctx, hub)
}
