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

    # Enforce meaningful problem description (min 30 chars)
    if [[ ${#problem} -lt 30 ]]; then
        echo "❌ ERROR: problem description too short (${#problem} chars, min 30)"
        echo "   Include: what breaks, where it occurs, workaround if known."
        echo "   Example: 'hashMapHas() typechecks to any not bool. Breaks if hashMapHas(m,k). Workaround: str(hashMapHas(m,k)) == \"true\"'"
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

# ═══════════════════════════════════════════════════════════════════════════════
# NEW FEATURES (2026-03-05)
# ═══════════════════════════════════════════════════════════════════════════════

cmd_update_issue() {
    # Update issue field: update H-001 priority P0
    local issue_id="$1"
    local field="$2"
    local value="$3"

    # Validate issue exists
    local exists=$(sqlite3 "$DB" "SELECT id FROM issues WHERE id='$issue_id'")
    if [[ -z "$exists" ]]; then
        echo "ERROR: Issue $issue_id not found"
        exit 1
    fi

    # Validate field
    case "$field" in
        priority)
            if [[ ! "$value" =~ ^P[0-3]$ ]]; then
                echo "ERROR: Priority must be P0, P1, P2, or P3"
                exit 1
            fi
            # Also update severity to match
            local severity
            case "$value" in
                P0) severity="critical" ;;
                P1) severity="high" ;;
                P2) severity="medium" ;;
                P3) severity="low" ;;
            esac
            sqlite3 "$DB" "UPDATE issues SET priority='$value', severity='$severity' WHERE id='$issue_id'"
            ;;
        component)
            sqlite3 "$DB" "UPDATE issues SET component='$value' WHERE id='$issue_id'"
            ;;
        title)
            sqlite3 "$DB" "UPDATE issues SET title='$(sql_escape "$value")' WHERE id='$issue_id'"
            ;;
        problem)
            sqlite3 "$DB" "UPDATE issues SET problem='$(sql_escape "$value")' WHERE id='$issue_id'"
            ;;
        *)
            echo "ERROR: Unknown field '$field'. Valid: priority, component, title, problem"
            exit 1
            ;;
    esac

    # Log to history
    local session=$(get_session)
    cmd_log_history "$issue_id" "update" "$field=$value" "$session"

    echo "✓ Updated: $issue_id.$field = $value"
}

cmd_search_issues() {
    # Search issues by keyword: search "closure"
    local keyword="$1"
    local results=$(sqlite3 "$DB" "SELECT id, priority, title FROM issues WHERE title LIKE '%$keyword%' OR problem LIKE '%$keyword%' OR fix_required LIKE '%$keyword%' LIMIT 10")

    if [[ -z "$results" ]]; then
        echo "No issues found matching '$keyword'"
        return
    fi

    echo "Issues matching '$keyword':"
    echo "$results" | while IFS='|' read -r id priority title; do
        printf "  [%s] %s: %s\n" "$id" "$priority" "$title"
    done
}

cmd_fix_batch() {
    # Close multiple issues: fix-batch H-001,H-002,H-003 "cause" "fix"
    local ids="$1"
    local root_cause="$2"
    local fix_applied="$3"

    # Validate
    if [[ ${#root_cause} -lt 10 ]]; then
        echo "ERROR: root_cause must be meaningful (min 10 chars)"
        exit 1
    fi
    if [[ ${#fix_applied} -lt 10 ]]; then
        echo "ERROR: fix_applied must be meaningful (min 10 chars)"
        exit 1
    fi

    local session=$(require_session)
    local commit_sha=$(git rev-parse --short HEAD 2>/dev/null || echo "none")
    local count=0

    IFS=',' read -ra ID_ARRAY <<< "$ids"
    for issue_id in "${ID_ARRAY[@]}"; do
        issue_id=$(echo "$issue_id" | tr -d ' ')
        local exists=$(sqlite3 "$DB" "SELECT id FROM issues WHERE id='$issue_id'")
        if [[ -n "$exists" ]]; then
            cmd_close_issue "$issue_id" "$session" "$root_cause" "$fix_applied"
            echo "✓ Closed: $issue_id"
            ((count++))
        else
            echo "⚠ Skipped: $issue_id (not found)"
        fi
    done

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Batch complete: $count issues closed"
    echo "Commit: $commit_sha"
}

cmd_components() {
    # List valid components with issue counts
    cat << 'EOF'
Valid components (compiler domains):
  parser      - Lexer, parser, AST
  binder      - Name resolution, scopes
  typechecker - Type inference, checking
  interpreter - Tree-walking interpreter
  vm          - Bytecode VM
  codegen     - Code generation
  jit         - JIT compilation
  runtime     - Runtime support
  stdlib      - Standard library
  lsp         - Language server
  cli         - Command-line interface
  infra       - Build, CI, tooling
  docs        - Documentation
  unknown     - Unclassified

EOF

    echo "Current issue distribution:"
    sqlite3 "$DB" "SELECT component, COUNT(*) as count FROM issues WHERE status IN ('open', 'in_progress') GROUP BY component ORDER BY count DESC" | while IFS='|' read -r comp count; do
        printf "  %-12s %s\n" "$comp" "$count"
    done
}

cmd_link_issues() {
    # Link issues: link H-001 blocks H-002
    local issue_id="$1"
    local relation="$2"
    local target_id="$3"

    # Validate both exist
    for id in "$issue_id" "$target_id"; do
        local exists=$(sqlite3 "$DB" "SELECT id FROM issues WHERE id='$id'")
        if [[ -z "$exists" ]]; then
            echo "ERROR: Issue $id not found"
            exit 1
        fi
    done

    # Ensure links table exists
    sqlite3 "$DB" "CREATE TABLE IF NOT EXISTS issue_links (
        id INTEGER PRIMARY KEY,
        source_id TEXT NOT NULL,
        relation TEXT NOT NULL,
        target_id TEXT NOT NULL,
        created_at TEXT DEFAULT (datetime('now')),
        UNIQUE(source_id, relation, target_id)
    )"

    case "$relation" in
        blocks)
            sqlite3 "$DB" "INSERT OR IGNORE INTO issue_links (source_id, relation, target_id) VALUES ('$issue_id', 'blocks', '$target_id')"
            echo "✓ $issue_id blocks $target_id"
            ;;
        blocked-by)
            sqlite3 "$DB" "INSERT OR IGNORE INTO issue_links (source_id, relation, target_id) VALUES ('$target_id', 'blocks', '$issue_id')"
            echo "✓ $issue_id is blocked by $target_id"
            ;;
        related)
            sqlite3 "$DB" "INSERT OR IGNORE INTO issue_links (source_id, relation, target_id) VALUES ('$issue_id', 'related', '$target_id')"
            echo "✓ $issue_id related to $target_id"
            ;;
        *)
            echo "ERROR: Unknown relation '$relation'. Valid: blocks, blocked-by, related"
            exit 1
            ;;
    esac
}

cmd_issue_links() {
    # Show links for an issue
    local issue_id="$1"

    # Check if links table exists
    local table_exists=$(sqlite3 "$DB" "SELECT name FROM sqlite_master WHERE type='table' AND name='issue_links'")
    if [[ -z "$table_exists" ]]; then
        echo "No links found for $issue_id"
        return
    fi

    echo "Links for $issue_id:"

    local blocks=$(sqlite3 "$DB" "SELECT target_id FROM issue_links WHERE source_id='$issue_id' AND relation='blocks'")
    if [[ -n "$blocks" ]]; then
        echo "  Blocks: $blocks"
    fi

    local blocked_by=$(sqlite3 "$DB" "SELECT source_id FROM issue_links WHERE target_id='$issue_id' AND relation='blocks'")
    if [[ -n "$blocked_by" ]]; then
        echo "  Blocked by: $blocked_by"
    fi

    local related=$(sqlite3 "$DB" "SELECT target_id FROM issue_links WHERE source_id='$issue_id' AND relation='related'")
    if [[ -n "$related" ]]; then
        echo "  Related: $related"
    fi

    if [[ -z "$blocks" && -z "$blocked_by" && -z "$related" ]]; then
        echo "  (none)"
    fi
}

cmd_log_history() {
    # Internal: log action to history table
    local issue_id="$1"
    local action="$2"
    local details="$3"
    local session="${4:-$(get_session)}"

    # Ensure history table exists
    sqlite3 "$DB" "CREATE TABLE IF NOT EXISTS issue_history (
        id INTEGER PRIMARY KEY,
        issue_id TEXT NOT NULL,
        action TEXT NOT NULL,
        details TEXT,
        session_id TEXT,
        timestamp TEXT DEFAULT (datetime('now'))
    )"

    sqlite3 "$DB" "INSERT INTO issue_history (issue_id, action, details, session_id) VALUES ('$issue_id', '$action', '$(sql_escape "$details")', '$session')"
}

cmd_history() {
    # Show history for an issue: history H-001
    local issue_id="$1"

    # Check if history table exists
    local table_exists=$(sqlite3 "$DB" "SELECT name FROM sqlite_master WHERE type='table' AND name='issue_history'")
    if [[ -z "$table_exists" ]]; then
        echo "No history found for $issue_id"
        return
    fi

    echo "History for $issue_id:"
    sqlite3 "$DB" "SELECT timestamp, action, details, session_id FROM issue_history WHERE issue_id='$issue_id' ORDER BY timestamp DESC LIMIT 10" | while IFS='|' read -r ts action details session; do
        printf "  %s | %-10s | %s | %s\n" "$ts" "$action" "$details" "$session"
    done
}
