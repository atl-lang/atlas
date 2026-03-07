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

# next — Recommended work order for AI agents
# Groups issues by root cause, flags triage-first items, shows chain reasoning
cmd_next() {
    local db="$DB"

    # Count open P0s
    local p0_count=$(sqlite3 "$db" "SELECT COUNT(*) FROM issues WHERE status='open' AND priority='P0'")
    local p1_count=$(sqlite3 "$db" "SELECT COUNT(*) FROM issues WHERE status='open' AND priority='P1'")

    echo "── RECOMMENDED NEXT ACTIONS ──"
    echo "Open: ${p0_count} P0 blockers, ${p1_count} P1 issues"
    echo ""

    # Step 1: Show any P0s that are DELETES (wrong test, not bugs)
    local deletes=$(sqlite3 -json "$db" \
        "SELECT id, title, problem FROM issues
         WHERE status='open' AND priority='P0'
         AND (problem LIKE '%DELETE%' OR problem LIKE '%WRONG TEST%' OR problem LIKE '%WONTFIX%' OR tags LIKE '%wrong-test%')
         ORDER BY id")
    if [[ "$deletes" != "[]" && -n "$deletes" ]]; then
        echo "① DELETE FIRST (wrong tests — correct behavior, not bugs):"
        echo "$deletes" | jq -r '.[] | "   \(.id)  \(.title)"'
        echo ""
    fi

    # Step 2: Show P0s needing triage before touching
    local triage=$(sqlite3 -json "$db" \
        "SELECT id, title FROM issues
         WHERE status='open' AND priority='P0'
         AND (problem LIKE '%TRIAGE%' OR problem LIKE '%must diff%' OR problem LIKE '%check first%')
         ORDER BY id")
    if [[ "$triage" != "[]" && -n "$triage" ]]; then
        echo "② TRIAGE FIRST (diff actual vs expected before deciding fix or delete):"
        echo "$triage" | jq -r '.[] | "   \(.id)  \(.title)"'
        echo ""
    fi

    # Step 3: P0s grouped by component (likely same root cause)
    echo "③ FIX — P0 blockers by component (same component = likely same root cause):"
    sqlite3 -json "$db" \
        "SELECT component, group_concat(id, ', ') as ids, COUNT(*) as cnt
         FROM issues
         WHERE status='open' AND priority='P0'
         AND problem NOT LIKE '%DELETE%'
         AND problem NOT LIKE '%WRONG TEST%'
         AND problem NOT LIKE '%TRIAGE%'
         AND (tags NOT LIKE '%wrong-test%' OR tags IS NULL)
         GROUP BY component
         ORDER BY cnt DESC, component" | \
        jq -r '.[] | "   [\(.component)] \(.ids)  (\(.cnt) issue\(if .cnt == "1" then "" else "s" end))"'
    echo ""

    # Step 4: P1s grouped by component
    if [[ $p1_count -gt 0 ]]; then
        echo "④ AFTER P0s — P1 issues by component:"
        sqlite3 -json "$db" \
            "SELECT component, group_concat(id, ', ') as ids, COUNT(*) as cnt
             FROM issues
             WHERE status='open' AND priority='P1'
             GROUP BY component
             ORDER BY cnt DESC, component" | \
            jq -r '.[] | "   [\(.component)] \(.ids)  (\(.cnt) issue\(if .cnt == "1" then "" else "s" end))"'
        echo ""
    fi

    # Step 5: linked chains (blocks relationships)
    local chains=$(sqlite3 -json "$db" \
        "SELECT id, title, blocks_issues FROM issues
         WHERE status='open' AND blocks_issues IS NOT NULL AND blocks_issues != ''
         ORDER BY priority, id" 2>/dev/null)
    if [[ -n "$chains" && "$chains" != "[]" && "$chains" != "null" ]]; then
        echo "⑤ CHAINS (fix these before their dependents):"
        echo "$chains" | jq -r '.[] | "   \(.id) → blocks \(.blocks_issues)  [\(.title)]"'
        echo ""
    fi

    echo "── Run 'atlas-track issue H-XXX' for full detail on any issue ──"
}
