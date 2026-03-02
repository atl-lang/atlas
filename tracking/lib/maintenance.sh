#!/bin/bash
# maintenance.sh — Self-maintenance to prevent bloat

# Constants
MAX_DB_SIZE_MB=50
SESSION_TIMEOUT_HOURS=24
ARCHIVE_AFTER_DAYS=90

# Auto-close stale sessions (called by go command)
auto_close_stale_sessions() {
    local cutoff=$(date -u -v-${SESSION_TIMEOUT_HOURS}H +"%Y-%m-%d %H:%M:%S" 2>/dev/null || \
                   date -u -d "${SESSION_TIMEOUT_HOURS} hours ago" +"%Y-%m-%d %H:%M:%S")

    local stale=$(sqlite3 "$DB" "SELECT id FROM sessions WHERE ended_at IS NULL AND started_at < '$cutoff'")
    local count=0

    for sid in $stale; do
        sqlite3 "$DB" "UPDATE sessions SET ended_at=datetime('now'), outcome='abandoned', summary='Session timed out (>${SESSION_TIMEOUT_HOURS}h without close)' WHERE id='$sid'"
        ((count++))
    done

    [[ $count -gt 0 ]] && echo "⚠ Auto-closed $count stale sessions (>${SESSION_TIMEOUT_HOURS}h)"
    return 0
}

# Release orphaned issues from abandoned sessions
release_orphaned_issues() {
    local released=$(sqlite3 "$DB" "
        UPDATE issues
        SET status='open', fixed_by=NULL
        WHERE status='in_progress'
        AND fixed_by IN (SELECT id FROM sessions WHERE outcome='abandoned')
        RETURNING id")

    if [[ -n "$released" ]]; then
        local count=$(echo "$released" | wc -l | tr -d ' ')
        echo "⚠ Released $count orphaned issues back to open"
    fi
    return 0
}

# Check DB size
check_db_size() {
    local size_bytes=$(stat -f%z "$DB" 2>/dev/null || stat -c%s "$DB")
    local size_mb=$((size_bytes / 1024 / 1024))

    if [[ $size_mb -gt $MAX_DB_SIZE_MB ]]; then
        echo "⚠ WARNING: DB size ${size_mb}MB exceeds ${MAX_DB_SIZE_MB}MB limit"
        echo "  Run: atlas-track gc --aggressive"
    fi
    return 0
}

# Garbage collection command
cmd_gc() {
    local mode="${1:---normal}"

    echo "═══ Garbage Collection ═══"

    # Always: close stale sessions
    auto_close_stale_sessions
    release_orphaned_issues

    # Count before
    local sessions_before=$(sqlite3 "$DB" "SELECT COUNT(*) FROM sessions")
    local issues_before=$(sqlite3 "$DB" "SELECT COUNT(*) FROM issues")

    if [[ "$mode" == "--aggressive" ]]; then
        # Archive old resolved issues (>90 days)
        local cutoff=$(date -u -v-${ARCHIVE_AFTER_DAYS}d +"%Y-%m-%d" 2>/dev/null || \
                       date -u -d "${ARCHIVE_AFTER_DAYS} days ago" +"%Y-%m-%d")

        sqlite3 "$DB" "UPDATE issues SET status='archived' WHERE status='resolved' AND closed_date < '$cutoff'"

        # Delete archived issues older than 180 days
        local delete_cutoff=$(date -u -v-180d +"%Y-%m-%d" 2>/dev/null || \
                              date -u -d "180 days ago" +"%Y-%m-%d")
        sqlite3 "$DB" "DELETE FROM issues WHERE status='archived' AND closed_date < '$delete_cutoff'"

        # Delete old sessions (keep last 100)
        sqlite3 "$DB" "DELETE FROM sessions WHERE id NOT IN (SELECT id FROM sessions ORDER BY started_at DESC LIMIT 100)"

        # Vacuum to reclaim space
        sqlite3 "$DB" "VACUUM"

        echo "Mode: aggressive (archive + delete + vacuum)"
    else
        echo "Mode: normal (stale sessions + orphaned issues)"
    fi

    # Count after
    local sessions_after=$(sqlite3 "$DB" "SELECT COUNT(*) FROM sessions")
    local issues_after=$(sqlite3 "$DB" "SELECT COUNT(*) FROM issues")

    echo ""
    echo "Sessions: $sessions_before → $sessions_after"
    echo "Issues: $issues_before → $issues_after"

    # Size
    local size_bytes=$(stat -f%z "$DB" 2>/dev/null || stat -c%s "$DB")
    local size_kb=$((size_bytes / 1024))
    echo "DB size: ${size_kb}KB"
    echo "═══════════════════════════"
}

# Health check (quick)
cmd_health() {
    local size_bytes=$(stat -f%z "$DB" 2>/dev/null || stat -c%s "$DB")
    local size_kb=$((size_bytes / 1024))
    local sessions=$(sqlite3 "$DB" "SELECT COUNT(*) FROM sessions")
    local open_issues=$(sqlite3 "$DB" "SELECT COUNT(*) FROM issues WHERE status IN ('open', 'in_progress')")
    local stale=$(sqlite3 "$DB" "SELECT COUNT(*) FROM sessions WHERE ended_at IS NULL AND started_at < datetime('now', '-${SESSION_TIMEOUT_HOURS} hours')")

    echo "DB: ${size_kb}KB | Sessions: $sessions | Open issues: $open_issues | Stale: $stale"

    [[ $stale -gt 0 ]] && echo "⚠ Run: atlas-track gc"
    [[ $size_kb -gt $((MAX_DB_SIZE_MB * 1024)) ]] && echo "⚠ Run: atlas-track gc --aggressive"
    return 0
}
