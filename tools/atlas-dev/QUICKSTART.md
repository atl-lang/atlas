# Atlas Dev - Quick Start

## Installation

```bash
cd tools/atlas-dev
go build -o atlas-dev cmd/atlas-dev/*.go
cp atlas-dev ../../
```

## Usage

### Start Monitoring Dashboard

```bash
./atlas-dev --server
```

Then open **http://localhost:8080** in your browser.

That's it. No config, no setup, just works.

---

### Custom Port (Optional)

```bash
./atlas-dev --server --port 3000
```

---

### CLI Commands (For AI Agents)

```bash
# Complete a phase
./atlas-dev phase complete phases/stdlib/phase-07b.md

# View stats
./atlas-dev stats

# Search decisions
./atlas-dev decision search "performance"

# Export data
./atlas-dev export json > data.json
```

See `./atlas-dev --help` for full command list.

---

## What You Get

**Real-time Dashboard:**
- Live statistics (phases, decisions, features)
- Activity feed with smooth animations
- WebSocket updates (zero page refreshes)
- Professional cyber/hacker theme
- Works from anywhere (embedded assets)

**Single Binary:**
- 15MB with embedded web assets
- Zero runtime dependencies
- SQLite database for state
- Production ready

---

## Database Location

Default: `atlas-dev.db` in current directory

Change with `--db` flag:
```bash
./atlas-dev --server --db /path/to/custom.db
```

---

## Troubleshooting

**Port already in use?**
```bash
./atlas-dev --server --port 3000
```

**Can't connect?**
```bash
curl http://localhost:8080/health
# Should return: {"status":"ok","clients":0}
```

**Need debug logs?**
```bash
./atlas-dev --server --debug
```

---

That's it. Simple, professional, works everywhere. 🚀
