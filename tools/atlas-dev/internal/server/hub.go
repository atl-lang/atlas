package server

import (
	"context"
	"encoding/json"
	"log/slog"
	"sync"
	"time"

	"nhooyr.io/websocket"
)

// MessageType represents the type of WebSocket message
type MessageType string

const (
	// Data update events
	MessageTypePhaseUpdate    MessageType = "phase_update"
	MessageTypeDecisionAdded  MessageType = "decision_added"
	MessageTypeFeatureUpdate  MessageType = "feature_update"
	MessageTypeStatsChanged   MessageType = "stats_changed"
	MessageTypeSpecUpdate     MessageType = "spec_update"
	MessageTypeAPIUpdate      MessageType = "api_update"

	// System events
	MessageTypeConnected      MessageType = "connected"
	MessageTypePing           MessageType = "ping"
	MessageTypePong           MessageType = "pong"
)

// Message represents a WebSocket message
type Message struct {
	Type    MessageType     `json:"type"`
	Payload json.RawMessage `json:"payload"`
}

// Client represents a WebSocket client connection
type Client struct {
	conn *websocket.Conn
	send chan Message
}

// Hub manages WebSocket connections and broadcasts
type Hub struct {
	clients    map[*Client]bool
	broadcast  chan Message
	register   chan *Client
	unregister chan *Client
	mu         sync.RWMutex
}

// NewHub creates a new WebSocket hub
func NewHub() *Hub {
	return &Hub{
		clients:    make(map[*Client]bool),
		broadcast:  make(chan Message, 256),
		register:   make(chan *Client),
		unregister: make(chan *Client),
	}
}

// Run starts the hub's main loop
func (h *Hub) Run(ctx context.Context) {
	slog.Info("WebSocket hub started")

	for {
		select {
		case <-ctx.Done():
			slog.Info("WebSocket hub shutting down")
			h.closeAll()
			return

		case client := <-h.register:
			h.mu.Lock()
			h.clients[client] = true
			h.mu.Unlock()
			slog.Info("client connected", "total", len(h.clients))

		case client := <-h.unregister:
			h.mu.Lock()
			if _, ok := h.clients[client]; ok {
				delete(h.clients, client)
				close(client.send)
				slog.Info("client disconnected", "total", len(h.clients))
			} else {
				slog.Debug("client already unregistered, skipping")
			}
			h.mu.Unlock()

		case message := <-h.broadcast:
			h.mu.RLock()
			for client := range h.clients {
				select {
				case client.send <- message:
				default:
					// Client's send channel is full, disconnect
					go func(c *Client) {
						h.unregister <- c
					}(client)
				}
			}
			h.mu.RUnlock()
		}
	}
}

// Broadcast sends a message to all connected clients
func (h *Hub) Broadcast(msgType MessageType, payload interface{}) error {
	data, err := json.Marshal(payload)
	if err != nil {
		slog.Error("failed to marshal broadcast payload", "type", msgType, "error", err)
		return err
	}

	h.broadcast <- Message{
		Type:    msgType,
		Payload: data,
	}
	return nil
}

// RegisterClient adds a new client to the hub
func (h *Hub) RegisterClient(conn *websocket.Conn) *Client {
	client := &Client{
		conn: conn,
		send: make(chan Message, 256),
	}
	h.register <- client
	return client
}

// UnregisterClient removes a client from the hub
func (h *Hub) UnregisterClient(client *Client) {
	h.unregister <- client
}

// WritePump handles writing messages to the WebSocket connection
func (c *Client) WritePump(ctx context.Context) {
	defer c.conn.Close(websocket.StatusNormalClosure, "")

	for {
		select {
		case <-ctx.Done():
			return
		case message, ok := <-c.send:
			if !ok {
				return
			}

			data, err := json.Marshal(message)
			if err != nil {
				slog.Error("failed to marshal message", "error", err, "type", message.Type)
				continue
			}

			// Set write deadline to prevent hanging
			writeCtx, cancel := context.WithTimeout(ctx, 5*time.Second)
			err = c.conn.Write(writeCtx, websocket.MessageText, data)
			cancel()

			if err != nil {
				slog.Debug("failed to write message", "error", err)
				return
			}
		}
	}
}

// ReadPump handles reading messages from the WebSocket connection
func (c *Client) ReadPump(ctx context.Context, hub *Hub) {
	defer func() {
		hub.UnregisterClient(c)
		c.conn.Close(websocket.StatusNormalClosure, "")
	}()

	for {
		_, msg, err := c.conn.Read(ctx)
		if err != nil {
			if websocket.CloseStatus(err) != websocket.StatusNormalClosure {
				slog.Debug("websocket read closed", "status", websocket.CloseStatus(err))
			}
			return
		}

		// Handle ping/pong
		var message Message
		if err := json.Unmarshal(msg, &message); err != nil {
			slog.Debug("failed to unmarshal client message", "error", err)
			continue
		}

		if message.Type == MessageTypePing {
			select {
			case c.send <- Message{Type: MessageTypePong}:
			default:
				slog.Debug("client send channel full, skipping pong")
			}
		}
	}
}

// closeAll closes all client connections
func (h *Hub) closeAll() {
	h.mu.Lock()
	defer h.mu.Unlock()

	for client := range h.clients {
		client.conn.Close(websocket.StatusNormalClosure, "server shutting down")
		close(client.send)
		delete(h.clients, client)
	}
}

// ClientCount returns the number of connected clients
func (h *Hub) ClientCount() int {
	h.mu.RLock()
	defer h.mu.RUnlock()
	return len(h.clients)
}
