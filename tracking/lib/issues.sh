#!/bin/bash
# issues.sh — Issue management (claim, fix, abandon, reopen, open, close)

cmd_claim() {
    local issue_id="$1"
    local session=$(require_session)

    local current_status=$(sqlite3 "$DB" "SELECT status FROM issues WHERE id='$issue_id'")
    if [[ -z "$current_status" ]]; then
        echo "ERROR: Issue $issue_id not found"
        exit 1
    fi
    if [[ "$current_status" != "open" ]]; then
        echo "ERROR: Issue $issue_id is '$current_status', not 'open'"
        exit 1
    fi

    sqlite3 "$DB" "UPDATE issues SET status='in_progress', fixed_by='$session' WHERE id='$issue_id'"
    echo "✓ Claimed: $issue_id (now in_progress)"
    echo "  When done: atlas-track fix $issue_id 'cause' 'fix'"
}

cmd_close_issue() {
    local id="$1"
    local session="$2"
    local root_cause="$3"
    local fix="$4"

    # Auto-capture commit SHA and changed files from git
    local commit_sha=$(git rev-parse --short HEAD 2>/dev/null || echo "")
    local files_changed=$(git diff --name-only HEAD~1 2>/dev/null | head -5 | tr '\n' ',' | sed 's/,$//' || echo "")

    sqlite3 "$DB" "UPDATE issues SET status='resolved', closed_date=datetime('now'), fixed_by='$session', root_cause='$(sql_escape "$root_cause")', fix_applied='$(sql_escape "$fix")', files='$files_changed' WHERE id='$id'"

    # Update session's issues_closed and git_commits
    local current_issues=$(sqlite3 "$DB" "SELECT COALESCE(issues_closed, '') FROM sessions WHERE id='$session'")
    local current_commits=$(sqlite3 "$DB" "SELECT COALESCE(git_commits, '') FROM sessions WHERE id='$session'")

    if [[ -n "$current_issues" ]]; then
        sqlite3 "$DB" "UPDATE sessions SET issues_closed='$current_issues,$id' WHERE id='$session'"
    else
        sqlite3 "$DB" "UPDATE sessions SET issues_closed='$id' WHERE id='$session'"
    fi

    if [[ -n "$commit_sha" ]]; then
        if [[ -n "$current_commits" && ! "$current_commits" =~ "$commit_sha" ]]; then
            sqlite3 "$DB" "UPDATE sessions SET git_commits='$current_commits,$commit_sha' WHERE id='$session'"
        elif [[ -z "$current_commits" ]]; then
            sqlite3 "$DB" "UPDATE sessions SET git_commits='$commit_sha' WHERE id='$session'"
        fi
    fi
}

cmd_fix() {
    local issue_id="$1"
    local root_cause="$2"
    local fix_applied="$3"

    # Validate
    if [[ ${#root_cause} -lt 10 ]]; then
        echo "❌ ERROR: root_cause must be meaningful (min 10 chars)"
        echo "Example: 'FFI calls bypassed policy.check_permission()'"
        exit 1
    fi
    if [[ ${#fix_applied} -lt 10 ]]; then
        echo "❌ ERROR: fix_applied must be meaningful (min 10 chars)"
        echo "Example: 'Added check_permission() call before ffi_dispatch()'"
        exit 1
    fi

    local session=$(require_session)

    # Capture git info before closing
    local commit_sha=$(git rev-parse --short HEAD 2>/dev/null || echo "none")
    local files_changed=$(git diff --name-only HEAD~1 2>/dev/null | head -3 | tr '\n' ' ' || echo "none")

    cmd_close_issue "$issue_id" "$session" "$root_cause" "$fix_applied"

    echo "✓ Closed: $issue_id"
    echo "  Cause: $root_cause"
    echo "  Fix: $fix_applied"
    echo "  Commit: $commit_sha"
    [[ "$files_changed" != "none" ]] && echo "  Files: $files_changed"

    local p0_count=$(sqlite3 "$DB" "SELECT COUNT(*) FROM issues WHERE status='open' AND priority='P0'")
    if [[ $p0_count -gt 0 ]]; then
        echo "P0 remaining: $(sqlite3 "$DB" "SELECT group_concat(id, ',') FROM issues WHERE status='open' AND priority='P0'")"
    else
        echo "P0 remaining: none — ready to unblock"
    fi
}

cmd_abandon() {
    local issue_id="$1"
    local reason="${2:-}"

    local current_status=$(sqlite3 "$DB" "SELECT status FROM issues WHERE id='$issue_id'")
    if [[ -z "$current_status" ]]; then
        echo "ERROR: Issue $issue_id not found"
        exit 1
    fi
    if [[ "$current_status" != "in_progress" ]]; then
        echo "ERROR: Issue $issue_id is '$current_status', not 'in_progress'"
        exit 1
    fi

    sqlite3 "$DB" "UPDATE issues SET status='open', fixed_by=NULL WHERE id='$issue_id'"
    echo "✓ Abandoned: $issue_id (back to open)"
    [[ -n "$reason" ]] && echo "  Reason: $reason"
}

cmd_reopen() {
    local issue_id="$1"
    sqlite3 "$DB" "UPDATE issues SET status='open', closed_date=NULL WHERE id='$issue_id'"
    echo "✓ Reopened: $issue_id"
}

cmd_add_issue() {
    # Simple interface: add "Title" P0|P1|P2 "problem description"
    local title="$1"
    local priority="$2"
    local problem="$3"

    # Validate priority
    if [[ ! "$priority" =~ ^P[0-2]$ ]]; then
        echo "ERROR: Priority must be P0, P1, or P2"
        exit 1
    fi

    # Derive severity from priority
    local severity
    case "$priority" in
        P0) severity="critical" ;;
        P1) severity="high" ;;
        P2) severity="medium" ;;
    esac

    local next_id=$(sqlite3 "$DB" "SELECT 'H-' || printf('%03d', COALESCE(MAX(CAST(substr(id, 3) AS INTEGER)), 0) + 1) FROM issues")
    local session=$(get_session)

    sqlite3 "$DB" "INSERT INTO issues (id, title, status, priority, severity, component, version, source, problem, fix_required, found_by, tags) VALUES ('$next_id', '$(sql_escape "$title")', 'open', '$priority', '$severity', 'unknown', '0.3.0', 'agent', '$(sql_escape "$problem")', 'TBD', '$session', '')"

    echo "✓ Created: $next_id"
    echo "  Title: $title"
    echo "  Priority: $priority ($severity)"
    echo "  Next: atlas-track claim $next_id"
}

cmd_open_issue() {
    # Verbose interface for full control (6 params)
    local title="$1"
    local priority="$2"
    local severity="$3"
    local component="$4"
    local problem="$5"
    local fix="$6"

    local next_id=$(sqlite3 "$DB" "SELECT 'H-' || printf('%03d', COALESCE(MAX(CAST(substr(id, 3) AS INTEGER)), 0) + 1) FROM issues")
    local session=$(get_session)

    sqlite3 "$DB" "INSERT INTO issues (id, title, status, priority, severity, component, version, source, problem, fix_required, found_by, tags) VALUES ('$next_id', '$(sql_escape "$title")', 'open', '$priority', '$severity', '$component', '0.3.0', 'agent', '$(sql_escape "$problem")', '$(sql_escape "$fix")', '$session', '')"
    echo "$next_id"
}
