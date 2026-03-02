#!/bin/bash
# queries.sh — Read-only queries using JSON + jq

# Compact status line
cmd_status() {
    sqlite3 -json "$DB" "SELECT mode, block_work_allowed as blocked, current_block as block FROM state" | \
        jq -r '.[0] | "Mode: \(.mode) | Work: \(if .blocked == 0 then "BLOCKED" else "OK" end) | Block: B\(.block)"'
    sqlite3 -json "$DB" "SELECT group_concat(id) as p0 FROM issues WHERE status='open' AND priority='P0'" | \
        jq -r '.[0] | "P0: \(.p0 // "none")"'
}

# Issues list - compact JSON to text
cmd_issues() {
    local filter="${1:-}"
    local where="status='open'"
    [[ "$filter" =~ ^P[0-3]$ ]] && where="$where AND priority='$filter'"
    [[ -n "$filter" && ! "$filter" =~ ^P[0-3]$ ]] && where="$where AND component='$filter'"

    sqlite3 -json "$DB" "SELECT id, priority, component FROM issues WHERE $where ORDER BY priority, component LIMIT $MAX_ROWS" | \
        jq -r '.[] | "\(.id) \(.priority) \(.component)"'

    local total=$(sqlite3 "$DB" "SELECT COUNT(*) FROM issues WHERE $where")
    [[ $total -gt $MAX_ROWS ]] && echo "(+$((total - MAX_ROWS)) more)"
}

# Decisions list
cmd_decisions() {
    local filter="${1:-}"
    local where="status='active'"
    [[ -n "$filter" && "$filter" != "all" ]] && where="$where AND component='$filter'"
    [[ "$filter" == "all" ]] && where="1=1"

    sqlite3 -json "$DB" "SELECT id, status, component, substr(title,1,35) as title FROM decisions WHERE $where ORDER BY id LIMIT $MAX_ROWS" | \
        jq -r '.[] | "\(.id) \(.status) \(.component) \(.title)"'
}

# Blocks list
cmd_blocks() {
    sqlite3 -json "$DB" "SELECT block_num, status, phases_done, phases_total FROM blocks WHERE version='0.3.0' ORDER BY block_num" | \
        jq -r '.[] | "B\(.block_num) \(.status) \(.phases_done)/\(.phases_total)"'
}

# Sessions list
cmd_sessions() {
    sqlite3 -json "$DB" "SELECT id, agent, outcome FROM sessions ORDER BY started_at DESC LIMIT $MAX_ROWS" | \
        jq -r '.[] | "\(.id) \(.agent) \(.outcome // "active")"'
}

# Issue detail - structured output
cmd_issue_detail() {
    local id="$1"
    sqlite3 -json "$DB" "SELECT id, title, status, priority, severity, component, problem, fix_required, root_cause, fix_applied, files, fixed_by FROM issues WHERE id='$id'" | \
        jq -r '.[0] | if .status == "resolved" then
            "[\(.id)] \(.title)\nStatus: \(.status) | \(.priority) | \(.component)\nProblem: \(.problem[:150] // "none")\n─ Resolution ─\nCause: \(.root_cause[:150] // "none")\nFix: \(.fix_applied[:150] // "none")\nFiles: \(.files // "none")\nBy: \(.fixed_by // "none")"
        else
            "[\(.id)] \(.title)\nStatus: \(.status) | \(.priority) | \(.component)\nProblem: \(.problem[:200] // "none")\nNeeded: \(.fix_required[:200] // "none")"
        end'
}

# Decision detail
cmd_decision_detail() {
    local id="$1"
    sqlite3 -json "$DB" "SELECT id, title, status, component, rule, rationale FROM decisions WHERE id='$id'" | \
        jq -r '.[0] | "[\(.id)] \(.title)\nStatus: \(.status) | \(.component)\nRule: \(.rule[:200] // "none")\nRationale: \(.rationale[:150] // "none")"'
}

# Session detail
cmd_session_detail() {
    local id="$1"
    sqlite3 -json "$DB" "SELECT id, agent, model_id, outcome, summary, next_steps, git_commits, issues_closed FROM sessions WHERE id='$id'" | \
        jq -r '.[0] | "[\(.id)] \(.agent) (\(.model_id // "?"))\nOutcome: \(.outcome // "active")\nSummary: \(.summary[:200] // "none")\nNext: \(.next_steps[:150] // "none")\nCommits: \(.git_commits // "none")\nClosed: \(.issues_closed // "none")"'
}

# Block detail
cmd_block_detail() {
    local num="$1"
    sqlite3 -json "$DB" "SELECT block_num, name, status, phases_done, phases_total, tests_at_start, tests_at_end, blockers FROM blocks WHERE version='0.3.0' AND block_num=$num" | \
        jq -r '.[0] | "Block \(.block_num): \(.name)\nStatus: \(.status) | Phases: \(.phases_done)/\(.phases_total)\nTests: \(.tests_at_start // "?") → \(.tests_at_end // "?")\nBlockers: \(.blockers // "none")"'
}
