#!/bin/bash
# session.sh — Session lifecycle (init, done, start, end, my-issues)

cmd_start_session() {
    local agent="${1:-opus}"
    local model="${2:-claude-opus-4-5}"
    local next_id=$(sqlite3 "$DB" "SELECT 'S-' || printf('%03d', COALESCE(MAX(CAST(substr(id, 3) AS INTEGER)), 0) + 1) FROM sessions")
    sqlite3 "$DB" "INSERT INTO sessions (id, agent, model_id, started_at, version, mode) VALUES ('$next_id', '$agent', '$model', datetime('now'), '0.3.0', (SELECT mode FROM state))"
    sqlite3 "$DB" "UPDATE state SET last_session_id='$next_id'"
    echo "$next_id"
}

cmd_end_session() {
    local id="$1"
    local outcome="$2"
    local summary="$3"
    local next="${4:-}"
    sqlite3 "$DB" "UPDATE sessions SET ended_at=datetime('now'), outcome='$outcome', summary='$(sql_escape "$summary")', next_steps='$(sql_escape "$next")' WHERE id='$id'"
    echo "OK"
}

cmd_init() {
    local agent="${1:-opus}"
    local model="${2:-claude-opus-4-5}"
    local sid=$(cmd_start_session "$agent" "$model")
    echo "Session: $sid"
}

# GO: The ONE command agents need - init + full sitrep
cmd_go() {
    local agent="${1:-opus}"
    local model="${2:-claude-opus-4-5}"

    # Auto-maintenance: close stale sessions, release orphaned issues
    auto_close_stale_sessions
    release_orphaned_issues
    check_db_size

    # Start session
    local sid=$(cmd_start_session "$agent" "$model")

    # Full sitrep with session ID prepended
    echo "═══ SITREP ═══"
    echo "Session: $sid | Agent: $agent"
    echo ""

    # State
    sqlite3 -json "$DB" "SELECT mode, block_work_allowed as work_allowed, current_block, version FROM state" | \
        jq -r '.[0] | "Mode: \(.mode) | Work: \(if .work_allowed == 0 then "BLOCKED" else "OK" end) | Block: B\(.current_block) | v\(.version)"'
    echo ""

    # Handoff from last session
    echo "── Handoff ──"
    sqlite3 -json "$DB" "SELECT id, agent, outcome, summary, next_steps, git_commits, issues_closed FROM sessions WHERE id != '$sid' ORDER BY started_at DESC LIMIT 1" | \
        jq -r 'if length > 0 then .[0] | "From: \(.agent) (\(.id)) → \(.outcome // "?")\nDid: \((.summary // "-") | .[0:80])\nNext: \((.next_steps // "-") | .[0:80])\nCommits: \(.git_commits // "none") | Closed: \(.issues_closed // "none")" else "First session" end'
    echo ""

    # Stale issues (in_progress from previous sessions)
    local stale_json=$(sqlite3 -json "$DB" "SELECT id, component, title FROM issues WHERE status='in_progress' LIMIT 5")
    local stale_count=$(echo "$stale_json" | jq 'length')
    if [[ "$stale_count" -gt 0 ]]; then
        echo "── ⚠ Stale Issues ($stale_count) ──"
        echo "$stale_json" | jq -r '.[] | "\(.id) [\(.component)] \(.title)"'
        echo "ACTION: fix <ID> or abandon <ID>"
        echo ""
    fi

    # P0 blockers
    local p0_json=$(sqlite3 -json "$DB" "SELECT id, component, title FROM issues WHERE status='open' AND priority='P0' ORDER BY component")
    local p0_count=$(echo "$p0_json" | jq 'length')
    echo "── P0 Blockers ($p0_count) ──"
    if [[ "$p0_count" -gt 0 ]]; then
        echo "$p0_json" | jq -r '.[] | "\(.id) [\(.component)] \(.title)"'
    else
        echo "None - ready to unblock"
    fi
    echo ""

    # Git
    echo "── Git ──"
    echo "Branch: $(git branch --show-current 2>/dev/null || echo "unknown")"
    git log --oneline -3 2>/dev/null | sed 's/^/  /' || echo "  (no commits)"
    echo ""

    # Blocks summary (compact)
    echo "── Blocks ──"
    sqlite3 -json "$DB" "SELECT block_num, status, phases_done, phases_total FROM blocks WHERE version='0.3.0'" | \
        jq -r '[.[] | "B\(.block_num):\(.status | .[0:4])"] | join(" ")'

    echo "═══════════"
}

cmd_done() {
    local id="$1"
    local outcome="$2"
    local summary="$3"
    local next="${4:-}"

    # Audit unclosed issues
    local unclosed=$(sqlite3 "$DB" "SELECT id FROM issues WHERE status='in_progress' AND fixed_by='$id'")
    if [[ -n "$unclosed" ]]; then
        echo "❌ BLOCKED: You have unclosed in_progress issues:"
        echo "$unclosed"
        echo ""
        echo "Fix each one: atlas-track fix <ID> 'cause' 'fix'"
        echo "Or abandon:   atlas-track abandon <ID> 'reason'"
        return 1
    fi

    cmd_end_session "$id" "$outcome" "$summary" "$next"
    echo "---"
    cmd_status
}

cmd_my_issues() {
    local session=$(require_session)

    echo "Session: $session"
    echo "---"

    local in_progress=$(sqlite3 "$DB" "SELECT id || ' ' || priority || ' ' || component FROM issues WHERE status='in_progress' AND fixed_by='$session'")
    local closed=$(sqlite3 "$DB" "SELECT id || ' ' || priority || ' ' || component FROM issues WHERE status='resolved' AND fixed_by='$session'")

    if [[ -n "$in_progress" ]]; then
        echo "⚠️  In Progress (MUST close before done):"
        echo "$in_progress"
    fi
    if [[ -n "$closed" ]]; then
        echo "✓ Closed this session:"
        echo "$closed"
    fi
    if [[ -z "$in_progress" && -z "$closed" ]]; then
        echo "No issues touched this session"
    fi
}

# Full situational report - everything an agent needs in ONE call
cmd_sitrep() {
    # Build complete JSON sitrep with safe defaults
    local state_json=$(sqlite3 -json "$DB" "SELECT mode, block_work_allowed as work_allowed, current_block, version FROM state" | jq -c '.[0] // {mode:"unknown",work_allowed:0,current_block:0,version:"?"}')
    local last_session=$(sqlite3 -json "$DB" "SELECT id, agent, outcome, summary, next_steps, git_commits, issues_closed FROM sessions ORDER BY started_at DESC LIMIT 1" | jq -c 'if length > 0 then .[0] else {} end')
    local p0_issues=$(sqlite3 -json "$DB" "SELECT id, component, title FROM issues WHERE status='open' AND priority='P0' ORDER BY component LIMIT 10" | jq -c 'if . == null then [] else . end')
    local stale_issues=$(sqlite3 -json "$DB" "SELECT id, component, title FROM issues WHERE status='in_progress' LIMIT 5" | jq -c 'if . == null then [] else . end')
    local blocks=$(sqlite3 -json "$DB" "SELECT block_num, status, phases_done, phases_total FROM blocks WHERE version='0.3.0' ORDER BY block_num" | jq -c 'if . == null then [] else . end')

    # Git info
    local branch=$(git branch --show-current 2>/dev/null || echo "unknown")
    local commits=$(git log --oneline -3 2>/dev/null || echo "")

    # Format output - optimized for AI readability
    echo "═══ SITREP ═══"
    echo "$state_json" | jq -r '"Mode: \(.mode) | Work: \(if .work_allowed == 0 then "BLOCKED" else "OK" end) | Block: B\(.current_block) | v\(.version)"'
    echo ""

    echo "── Handoff ──"
    echo "$last_session" | jq -r 'if . == {} then "No previous session" else "From: \(.agent // "?") (\(.id // "?")) → \(.outcome // "?")\nDid: \((.summary // "none") | .[0:80])\nNext: \((.next_steps // "none") | .[0:80])\nCommits: \(.git_commits // "none") | Closed: \(.issues_closed // "none")" end'
    echo ""

    local p0_count=$(echo "$p0_issues" | jq 'length')
    echo "── P0 Blockers ($p0_count) ──"
    if [[ "$p0_count" -gt 0 ]]; then
        echo "$p0_issues" | jq -r '.[] | "\(.id) [\(.component)] \(.title)"'
    else
        echo "None - ready to unblock"
    fi
    echo ""

    local stale_count=$(echo "$stale_issues" | jq 'length')
    if [[ "$stale_count" -gt 0 ]]; then
        echo "── Stale ($stale_count) ──"
        echo "$stale_issues" | jq -r '.[] | "⚠ \(.id) [\(.component)] \(.title)"'
        echo ""
    fi

    echo "── Git ──"
    echo "Branch: $branch"
    [[ -n "$commits" ]] && echo "$commits" | head -3 | sed 's/^/  /'
    echo ""

    echo "── Blocks ──"
    echo "$blocks" | jq -r '.[] | "B\(.block_num) \(.status) \(.phases_done)/\(.phases_total)"'
    echo "═══════════"
}
