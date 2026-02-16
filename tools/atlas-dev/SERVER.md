# Atlas Dev - Monitoring Dashboard Server

## Overview

The `atlas-dev server` command provides a professional, real-time monitoring dashboard for tracking Atlas compiler development progress.

## Features

- **Real-time Updates**: 100% WebSocket-based, zero page refreshes
- **Live Statistics**: Phase completion, decisions, features tracked in real-time
- **Professional UI**: Cyber/hacker-themed dark interface with smooth GSAP animations
- **Responsive Design**: Works on desktop, tablet, and mobile
- **Zero Dependencies**: Single binary with embedded assets (production mode)
- **Battle-Tested Stack**:
  - Backend: Go stdlib `net/http` + `nhooyr.io/websocket`
  - Frontend: Tailwind CSS + GSAP + Vanilla JavaScript
  - Database: SQLite with 100ms polling

## Usage

### Quick Start

```bash
# Development mode (serves files from disk, hot-reload friendly)
./atlas-dev server --dev --port 8080

# Production mode (serves embedded assets from binary)
./atlas-dev server --port 8080

# Custom poll rate
./atlas-dev server --poll 50  # Poll database every 50ms
```

Then open http://localhost:8080 in your browser.

### Command Flags

| Flag | Default | Description |
|------|---------|-------------|
| `-p, --port` | 8080 | HTTP server port |
| `--poll` | 100 | Database poll rate in milliseconds |
| `--dev` | false | Development mode (serve files from disk) |
| `--db` | atlas-dev.db | Database path (inherited from global flag) |

### Examples

```bash
# Run on custom port with faster polling
./atlas-dev server --port 3000 --poll 50

# Development mode with debug logging
./atlas-dev server --dev --debug

# Production mode with specific database
./atlas-dev server --db /path/to/atlas-dev.db
```

## Dashboard Features

### Real-time Panels

1. **System Statistics**
   - Total phases, completed, pending
   - Decisions and features count
   - Completion percentage with animated progress bar

2. **Live Activity Feed**
   - Real-time phase updates
   - Decision additions
   - Feature changes
   - Animated entries with timestamps

3. **System Overview**
   - Total decisions and features
   - Last update timestamp
   - Connected clients count

### Connection Indicator

- 🟢 **Green**: Connected and receiving updates
- 🔴 **Red**: Disconnected, attempting reconnection
- **Auto-reconnect**: Up to 10 attempts with 2-second delays

## Architecture

### Backend

- **HTTP Server**: Go stdlib `net/http`
- **WebSocket Hub**: Manages client connections and broadcasts
- **Database Watcher**: Polls SQLite every 100ms for changes
- **Change Detection**: Tracks `updated_at` timestamps for efficient queries

### Frontend

- **Tailwind CSS**: Utility-first styling (CDN for development)
- **GSAP**: Professional animations (number count-ups, fades, glows)
- **WebSocket Client**: Auto-reconnect, heartbeat every 30s
- **Zero Refresh**: All updates via WebSocket, no polling

### Data Flow

```
SQLite DB → Watcher (100ms poll) → Hub → WebSocket → Browser
                                     ↓
                              Stats broadcast
                              Phase updates
                              Decision updates
                              Feature updates
```

## Message Protocol

### Server → Client

```json
{
  "type": "stats_changed",
  "payload": {
    "total_phases": 78,
    "completed_phases": 31,
    "pending_phases": 47,
    "total_decisions": 25,
    "total_features": 15,
    "completion_rate": 39.7,
    "last_update": "2026-02-16T01:20:00Z"
  }
}
```

```json
{
  "type": "phase_update",
  "payload": {
    "id": 42,
    "name": "phase-07b",
    "category": "stdlib",
    "status": "completed",
    "updated_at": "2026-02-16T01:20:15Z"
  }
}
```

### Client → Server

```json
{
  "type": "ping"
}
```

## Health Check

The server provides a health endpoint for monitoring:

```bash
curl http://localhost:8080/health
# {"status":"ok","clients":2}
```

## Performance

- **Database Queries**: < 1ms (indexed `updated_at` columns)
- **WebSocket Latency**: < 10ms (local network)
- **Memory Usage**: ~15MB (including embedded assets)
- **Binary Size**: ~12MB (with embedded web assets)

## Development

### Directory Structure

```
tools/atlas-dev/
├── cmd/atlas-dev/server.go        # Server command
├── internal/server/
│   ├── hub.go                     # WebSocket hub
│   ├── watcher.go                 # Database change detection
│   ├── assets.go                  # Embedded assets
│   └── web/                       # Embedded web files
└── web/                           # Development web files
    ├── index.html
    └── app.js
```

### Hot Reload (Dev Mode)

```bash
# Terminal 1: Watch and rebuild
while true; do
  inotifywait -e modify cmd/**/*.go internal/**/*.go
  go build -o atlas-dev cmd/atlas-dev/*.go
done

# Terminal 2: Run server
./atlas-dev server --dev
```

## Production Deployment

### Build

```bash
go build -o atlas-dev cmd/atlas-dev/*.go
```

The binary includes embedded web assets (HTML, CSS, JS) and requires no external files.

### Deploy

```bash
# Copy binary to server
scp atlas-dev user@server:/opt/atlas-dev/

# Run as systemd service
cat > /etc/systemd/system/atlas-dev-server.service << EOF
[Unit]
Description=Atlas Dev Monitoring Server
After=network.target

[Service]
Type=simple
User=atlas
WorkingDirectory=/opt/atlas-dev
ExecStart=/opt/atlas-dev/atlas-dev server --port 8080
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl enable atlas-dev-server
systemctl start atlas-dev-server
```

## Security Notes

- **WebSocket CORS**: Currently allows all origins for development
- **Production**: Should add origin validation and/or reverse proxy (nginx/caddy)
- **No Authentication**: Dashboard is open to anyone with network access
- **Recommendation**: Deploy behind VPN or add authentication layer

## Troubleshooting

### Port Already in Use

```bash
# Find process using port
lsof -i :8080

# Kill process
kill -9 <PID>
```

### WebSocket Connection Failed

- Check firewall rules
- Verify server is running: `curl http://localhost:8080/health`
- Check browser console for errors
- Try increasing reconnect attempts

### No Real-time Updates

- Verify database has `updated_at` columns
- Check poll rate: `./atlas-dev server --poll 50`
- Monitor server logs with `--debug` flag

## Future Enhancements

- [ ] Add authentication (JWT/OAuth)
- [ ] HTTPS support with TLS certificates
- [ ] Custom dashboards per user
- [ ] Historical data charts (time-series)
- [ ] Export dashboard to PNG/PDF
- [ ] Slack/Discord webhook notifications
- [ ] Multi-database support
- [ ] GraphQL API for advanced queries

## License

MIT License - Same as Atlas project

## Support

Issues: https://github.com/atlas-lang/atlas-dev/issues
Docs: https://github.com/atlas-lang/atlas-dev/wiki
