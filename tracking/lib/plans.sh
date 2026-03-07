#!/bin/bash
# plans.sh — Plan lifecycle (brainstorm outcomes, approach docs linked to issues/decisions)

cmd_plan_add() {
    local title="$1"
    local approach="$2"
    local linked_issues="${3:-}"    # comma-separated H-XXX
    local linked_decisions="${4:-}" # comma-separated D-XXX

    local next_id=$(sqlite3 "$DB" "SELECT 'PL-' || printf('%03d', COALESCE(MAX(CAST(substr(id, 4) AS INTEGER)), 0) + 1) FROM plans")
    local git_commit=$(git rev-parse --short HEAD 2>/dev/null || echo "")
    local session=$(sqlite3 "$DB" "SELECT last_session_id FROM state")

    sqlite3 "$DB" "INSERT INTO plans (id, title, approach, linked_issues, linked_decisions, git_commit, created_by)
        VALUES (
            '$next_id',
            '$(sql_escape "$title")',
            '$(sql_escape "$approach")',
            '$(sql_escape "$linked_issues")',
            '$(sql_escape "$linked_decisions")',
            '$(sql_escape "$git_commit")',
            '$(sql_escape "$session")'
        )"

    echo "✓ Created: $next_id"
    echo "  Title: $title"
    [[ -n "$linked_issues" ]]    && echo "  Issues: $linked_issues"
    [[ -n "$linked_decisions" ]] && echo "  Decisions: $linked_decisions"
    [[ -n "$git_commit" ]]       && echo "  Snapshot: $git_commit"
}

cmd_plan_list() {
    local filter="${1:-}"  # 'open', 'active', 'done', or '' for open+active

    local where="status IN ('open', 'active')"
    [[ -n "$filter" ]] && where="status = '$(sql_escape "$filter")'"

    local results
    results=$(sqlite3 -json "$DB" "SELECT id, title, status, linked_issues, linked_decisions, git_commit, created_at FROM plans WHERE $where ORDER BY created_at DESC")

    local count
    count=$(echo "$results" | jq 'length')

    if [[ "$count" -eq 0 ]]; then
        echo "No plans found (filter: ${filter:-open+active})"
        return
    fi

    echo "$results" | jq -r '.[] | [
        "\(.id) [\(.status)] \(.title)",
        (if .linked_issues != "" and .linked_issues != null then "  Issues: \(.linked_issues)" else empty end),
        (if .linked_decisions != "" and .linked_decisions != null then "  Decisions: \(.linked_decisions)" else empty end),
        "  Snapshot: \(.git_commit // "none") | Created: \(.created_at[:10])"
    ] | join("\n")'
}

cmd_plan_detail() {
    local id="$1"
    local plan
    plan=$(sqlite3 -json "$DB" "SELECT * FROM plans WHERE id = '$(sql_escape "$id")'" | jq -r '.[0] // empty')

    if [[ -z "$plan" ]]; then
        echo "Plan not found: $id"
        return 1
    fi

    echo "$plan" | jq -r '"[\(.id)] \(.title)
Status: \(.status) | Snapshot: \(.git_commit // "none") | Created: \(.created_at[:16])
Created by: \(.created_by // "unknown")

Approach:
\(.approach)
" +
(if .linked_issues != "" and .linked_issues != null then "Linked Issues: \(.linked_issues)\n" else "" end) +
(if .linked_decisions != "" and .linked_decisions != null then "Linked Decisions: \(.linked_decisions)\n" else "" end) +
(if .outcome != null then "\nOutcome:\n\(.outcome)" else "" end)'
}

cmd_plan_done() {
    local id="$1"
    local outcome="$2"

    local exists
    exists=$(sqlite3 "$DB" "SELECT COUNT(*) FROM plans WHERE id = '$(sql_escape "$id")'")
    if [[ "$exists" -eq 0 ]]; then
        echo "Plan not found: $id"
        return 1
    fi

    sqlite3 "$DB" "UPDATE plans SET status='done', outcome='$(sql_escape "$outcome")', updated_at=datetime('now') WHERE id='$(sql_escape "$id")'"
    echo "✓ Closed: $id"
}

cmd_plan_abandon() {
    local id="$1"
    local reason="${2:-no reason given}"

    local exists
    exists=$(sqlite3 "$DB" "SELECT COUNT(*) FROM plans WHERE id = '$(sql_escape "$id")'")
    if [[ "$exists" -eq 0 ]]; then
        echo "Plan not found: $id"
        return 1
    fi

    sqlite3 "$DB" "UPDATE plans SET status='abandoned', outcome='$(sql_escape "$reason")', updated_at=datetime('now') WHERE id='$(sql_escape "$id")'"
    echo "✓ Abandoned: $id"
}

cmd_plan_link() {
    local id="$1"
    local target="$2"  # H-XXX or D-XXX

    local exists
    exists=$(sqlite3 "$DB" "SELECT COUNT(*) FROM plans WHERE id = '$(sql_escape "$id")'")
    if [[ "$exists" -eq 0 ]]; then
        echo "Plan not found: $id"
        return 1
    fi

    if [[ "$target" == H-* ]]; then
        local current
        current=$(sqlite3 "$DB" "SELECT COALESCE(linked_issues, '') FROM plans WHERE id='$(sql_escape "$id")'")
        local updated
        if [[ -z "$current" ]]; then
            updated="$target"
        else
            if echo "$current" | grep -q "$target"; then
                echo "Already linked: $target"
                return 0
            fi
            updated="$current,$target"
        fi
        sqlite3 "$DB" "UPDATE plans SET linked_issues='$(sql_escape "$updated")', updated_at=datetime('now') WHERE id='$(sql_escape "$id")'"
        echo "✓ Linked issue $target to $id"
    elif [[ "$target" == D-* ]]; then
        local current
        current=$(sqlite3 "$DB" "SELECT COALESCE(linked_decisions, '') FROM plans WHERE id='$(sql_escape "$id")'")
        local updated
        if [[ -z "$current" ]]; then
            updated="$target"
        else
            if echo "$current" | grep -q "$target"; then
                echo "Already linked: $target"
                return 0
            fi
            updated="$current,$target"
        fi
        sqlite3 "$DB" "UPDATE plans SET linked_decisions='$(sql_escape "$updated")', updated_at=datetime('now') WHERE id='$(sql_escape "$id")'"
        echo "✓ Linked decision $target to $id"
    else
        echo "Target must be H-XXX (issue) or D-XXX (decision), got: $target"
        return 1
    fi
}
