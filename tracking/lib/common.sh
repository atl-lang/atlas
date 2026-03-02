#!/bin/bash
# common.sh — Shared variables and helpers

DB="tracking/atlas.db"
MAX_ROWS=5

# Get current session ID
get_session() {
    sqlite3 "$DB" "SELECT last_session_id FROM state"
}

# Require active session
require_session() {
    local session=$(get_session)
    if [[ -z "$session" || "$session" == "null" ]]; then
        echo "ERROR: No active session. Run 'atlas-track init' first."
        exit 1
    fi
    echo "$session"
}

# Escape single quotes for SQL
sql_escape() {
    echo "$1" | sed "s/'/''/g"
}
