#!/bin/bash
# queries.sh — Read-only queries using JSON + jq

# Compact status line
cmd_status() {
    sqlite3 -json "$DB" "SELECT mode, block_work_allowed as blocked, current_block as block FROM state" | \
        jq -r '.[0] | "Mode: \(.mode) | Work: \(if .blocked == 0 then "BLOCKED" else "OK" end) | Block: B\(.block)"'
    sqlite3 -json "$DB" "SELECT group_concat(id) as p0 FROM issues WHERE status='open' AND priority='P0'" | \
        jq -r '.[0] | "P0: \(.p0 // "none")"'
}

# Ultra-compact dashboard — no session needed, single-call orientation
cmd_context() {
    local p0=$(sqlite3 "$DB" "SELECT COUNT(*) FROM issues WHERE status='open' AND priority='P0'")
    local p1=$(sqlite3 "$DB" "SELECT COUNT(*) FROM issues WHERE status='open' AND priority='P1'")
    local p2=$(sqlite3 "$DB" "SELECT COUNT(*) FROM issues WHERE status='open' AND priority='P2'")
    local inprog=$(sqlite3 "$DB" "SELECT COUNT(*) FROM issues WHERE status='in_progress'")
    local decisions=$(sqlite3 "$DB" "SELECT COUNT(*) FROM decisions WHERE status='active'")
    local session_count=$(sqlite3 "$DB" "SELECT COUNT(*) FROM sessions")
    local last_session=$(sqlite3 -json "$DB" "SELECT id, agent, outcome FROM sessions ORDER BY started_at DESC LIMIT 1" | jq -r '.[0] | "\(.id) \(.agent)/\(.outcome // "active")"')
    local block_info=$(sqlite3 -json "$DB" "SELECT b.block_num, b.name, b.status, b.phases_done, b.phases_total FROM blocks b JOIN state s ON b.block_num = s.current_block AND b.version = '0.3.0' LIMIT 1" | jq -r '.[0] | "B\(.block_num) \(.status) (\(.phases_done)/\(.phases_total)) \(.name)"')
    local ci_file="$(dirname "$DB")/ci-status.json"
    local ci_status="no run"
    if [[ -f "$ci_file" ]]; then
        ci_status=$(jq -r '"\(if .status == "pass" then "✓ PASS" else "✗ FAIL" end) \(.run_at[:16])"' "$ci_file")
    fi

    echo "Issues: ${p0} P0, ${p1} P1, ${p2} P2 | ${inprog} in_progress"
    echo "Decisions: ${decisions} active | Sessions: ${session_count} (last: ${last_session})"
    echo "Block: ${block_info:-none}"
    echo "CI: ${ci_status}"
}

# Issues list — compact with titles and status
cmd_issues() {
    local filter="${1:-}"
    local where="status IN ('open','in_progress')"
    [[ "$filter" =~ ^P[0-3]$ ]] && where="status IN ('open','in_progress') AND priority='$filter'"
    [[ -n "$filter" && ! "$filter" =~ ^P[0-3]$ && "$filter" != "all" ]] && where="status IN ('open','in_progress') AND component='$filter'"
    [[ "$filter" == "all" ]] && where="status IN ('open','in_progress')"

    local limit="LIMIT $MAX_ROWS"
    local total=$(sqlite3 "$DB" "SELECT COUNT(*) FROM issues WHERE $where")

    sqlite3 -json "$DB" "SELECT id, priority, component, status, substr(title,1,55) as title FROM issues WHERE $where ORDER BY priority, component $limit" | \
        jq -r 'if length == 0 then "No issues found." else .[] | "\(.id) \(.priority) \(.component) [\(.status)] \(.title)" end'

    if [[ $total -gt $MAX_ROWS ]]; then echo "(+$((total - MAX_ROWS)) more — use 'issues all' or 'search <keyword>')"; fi
}

# What's in flight right now — continuity check before claiming new work
cmd_in_progress() {
    local results=$(sqlite3 -json "$DB" \
        "SELECT i.id, i.priority, i.component, i.title, i.fixed_by,
                s.agent
         FROM issues i
         LEFT JOIN sessions s ON i.fixed_by = s.id
         WHERE i.status = 'in_progress'
         ORDER BY i.priority, i.id")

    local count=$(echo "$results" | jq 'length')
    if [[ "$count" == "0" || -z "$results" || "$results" == "[]" ]]; then
        echo "Nothing in progress."
        return 0
    fi

    echo "── IN PROGRESS ($count) ──"
    echo "$results" | jq -r '.[] | "\(.id) [\(.priority)/\(.component)] \(.agent // "?")/\(.fixed_by // "?") — \(.title)"'
}

# Decisions list — no cap when "all" is requested
cmd_decisions() {
    local filter="${1:-}"
    local where="status='active'"
    local limit="LIMIT $MAX_ROWS"

    if [[ "$filter" == "all" ]]; then
        where="1=1"
        limit=""  # No cap — this is the source of truth, agents need it all
    elif [[ -n "$filter" ]]; then
        where="status='active' AND component='$filter'"
    fi

    sqlite3 -json "$DB" "SELECT id, status, component, substr(title,1,55) as title FROM decisions WHERE $where ORDER BY id $limit" | \
        jq -r '.[] | "\(.id) \(.status) \(.component) \(.title)"'

    if [[ -z "$filter" ]]; then
        local total=$(sqlite3 "$DB" "SELECT COUNT(*) FROM decisions WHERE status='active'")
        if [[ $total -gt $MAX_ROWS ]]; then echo "(+$((total - MAX_ROWS)) more — use 'decisions all' to see everything)"; fi
    fi
}

# Blocks list — with name
cmd_blocks() {
    sqlite3 -json "$DB" "SELECT block_num, status, phases_done, phases_total, name FROM blocks WHERE version='0.3.0' ORDER BY block_num" | \
        jq -r '.[] | "B\(.block_num) \(.status) \(.phases_done)/\(.phases_total) \(.name)"'
}

# Sessions list
cmd_sessions() {
    sqlite3 -json "$DB" "SELECT id, agent, outcome, substr(summary,1,60) as summary FROM sessions ORDER BY started_at DESC LIMIT $MAX_ROWS" | \
        jq -r '.[] | "\(.id) \(.agent) \(.outcome // "active") \(.summary // "")"'
}

# Issue detail — full text, no truncation
cmd_issue_detail() {
    local id="$1"
    sqlite3 -json "$DB" "SELECT id, title, status, priority, severity, component, problem, fix_required, root_cause, fix_applied, files, fixed_by FROM issues WHERE id='$id'" | \
        jq -r '.[0] | if . == null then "Issue not found: '"$id"'"
        elif .status == "resolved" then
            "[\(.id)] \(.title)\nStatus: \(.status) | \(.priority) | \(.component)\nProblem: \(.problem // "none")\n─ Resolution ─\nCause: \(.root_cause // "none")\nFix: \(.fix_applied // "none")\nFiles: \(.files // "none")\nBy: \(.fixed_by // "none")"
        else
            "[\(.id)] \(.title)\nStatus: \(.status) | \(.priority) | \(.component)\nProblem: \(.problem // "none")\nNeeded: \(.fix_required // "none")"
        end'
}

# Decision detail — full text, no truncation
cmd_decision_detail() {
    local id="$1"
    sqlite3 -json "$DB" "SELECT id, title, status, component, rule, rationale FROM decisions WHERE id='$id'" | \
        jq -r '.[0] | if . == null then "Decision not found: '"$id"'"
        else
            "[\(.id)] \(.title)\nStatus: \(.status) | \(.component)\nRule: \(.rule // "none")\nRationale: \(.rationale // "none")"
        end'
}

# Session detail
cmd_session_detail() {
    local id="$1"
    sqlite3 -json "$DB" "SELECT id, agent, model_id, outcome, summary, next_steps, git_commits, issues_closed FROM sessions WHERE id='$id'" | \
        jq -r '.[0] | "[\(.id)] \(.agent) (\(.model_id // "?"))\nOutcome: \(.outcome // "active")\nSummary: \(.summary // "none")\nNext: \(.next_steps // "none")\nCommits: \(.git_commits // "none")\nClosed: \(.issues_closed // "none")"'
}

# Block detail — accept both "8" and "B8", include acceptance criteria
cmd_block_detail() {
    local num="${1#[Bb]}"  # Strip leading B or b
    sqlite3 -json "$DB" "SELECT block_num, name, status, phases_done, phases_total, tests_at_start, tests_at_end, blockers, acceptance_criteria FROM blocks WHERE version='0.3.0' AND block_num=$num" | \
        jq -r '.[0] | if . == null then "Block not found: '"$1"'"
        else
            "Block \(.block_num): \(.name)\nStatus: \(.status) | Phases: \(.phases_done)/\(.phases_total)\nTests: \(.tests_at_start // "?") → \(.tests_at_end // "?")\nBlockers: \(.blockers // "none")\nAC: \(.acceptance_criteria // "none")"
        end'
}

# phase-done — increment phases_done for a block (e.g. atlas-track phase-done B8)
cmd_phase_done() {
    local num="${1#[Bb]}"
    local row
    row=$(sqlite3 -json "$DB" "SELECT block_num, name, phases_done, phases_total, status FROM blocks WHERE version='0.3.0' AND block_num=$num" | jq -r '.[0] // empty')
    if [[ -z "$row" ]]; then echo "Block not found: $1"; return 1; fi
    local done total status name
    done=$(echo "$row" | jq -r '.phases_done')
    total=$(echo "$row" | jq -r '.phases_total')
    status=$(echo "$row" | jq -r '.status')
    name=$(echo "$row" | jq -r '.name')
    if [[ "$status" == "complete" ]]; then echo "Block B$num already complete — use complete-block to reopen or verify."; return 0; fi
    local new_done=$((done + 1))
    sqlite3 "$DB" "UPDATE blocks SET phases_done=$new_done, status='in_progress' WHERE version='0.3.0' AND block_num=$num"
    echo "✓ B$num phase $new_done/$total done: $name"
    if [[ $new_done -ge $total ]]; then echo "  → All phases done. Run: atlas-track complete-block B$num \"notes\""; fi
}

# complete-block — mark a block complete with optional notes
cmd_complete_block() {
    local num="${1#[Bb]}"
    local notes="${2:-}"
    local session_id
    session_id=$(sqlite3 "$DB" "SELECT id FROM sessions WHERE outcome IS NULL ORDER BY started_at DESC LIMIT 1" 2>/dev/null || echo "")
    local row
    row=$(sqlite3 -json "$DB" "SELECT block_num, name, phases_done, phases_total FROM blocks WHERE version='0.3.0' AND block_num=$num" | jq -r '.[0] // empty')
    if [[ -z "$row" ]]; then echo "Block not found: $1"; return 1; fi
    local name phases_done phases_total
    name=$(echo "$row" | jq -r '.name')
    phases_done=$(echo "$row" | jq -r '.phases_done')
    phases_total=$(echo "$row" | jq -r '.phases_total')
    local today
    today=$(date +%Y-%m-%d)
    local escaped_notes="${notes//\'/\'\'}"
    sqlite3 "$DB" "UPDATE blocks SET status='complete', completed_date='$today', completed_by='${session_id:-manual}', notes='$escaped_notes' WHERE version='0.3.0' AND block_num=$num"
    echo "✓ B$num complete ($phases_done/$phases_total phases): $name"
    if [[ -n "$notes" ]]; then echo "  Notes: $notes"; fi
}

# CI status — proper function (was broken inline `local` in case block)
cmd_ci_status() {
    local ci_file
    ci_file="$(cd "$(dirname "$DB")" && pwd)/ci-status.json"
    if [[ ! -f "$ci_file" ]]; then
        echo "No CI run recorded yet. Run: atlas-track run-ci"
        return 0
    fi

    local status run_at duration
    status=$(jq -r '.status' "$ci_file")
    run_at=$(jq -r '.run_at' "$ci_file")
    duration=$(jq -r '.duration_seconds' "$ci_file")

    if [[ "$status" == "pass" ]]; then
        echo "✓ CI PASS — ${run_at} (${duration}s)"
    else
        echo "✗ CI FAIL — ${run_at} (${duration}s)"
        echo ""
        echo "Failed checks:"
        jq -r '.checks | to_entries[] | select(.value.status == "fail") | "  ✗ \(.key)"' "$ci_file"
        local failed_tests
        failed_tests=$(jq -r '.checks.tests.failed // [] | .[]' "$ci_file" 2>/dev/null | head -20)
        if [[ -n "$failed_tests" ]]; then
            echo ""
            echo "Failed tests (first 20):"
            echo "$failed_tests" | sed 's/^/  /'
        fi
        local p0
        p0=$(jq -r '.checks | to_entries[] | select(.value.status == "fail") | .value.p0_issue // empty' "$ci_file" 2>/dev/null)
        local p0_created
        p0_created=$(jq -r '.p0_created // empty' "$ci_file" 2>/dev/null)
        [[ -n "$p0_created" ]] && echo "" && echo "P0 filed: $p0_created"
    fi
}

# Search issues — with component, status, priority
cmd_search_issues() {
    local keyword="$1"
    local escaped=$(sql_escape "$keyword")

    echo "Issues matching '${keyword}':"
    sqlite3 -json "$DB" \
        "SELECT id, priority, component, status, substr(title,1,55) as title
         FROM issues
         WHERE title LIKE '%${escaped}%' OR problem LIKE '%${escaped}%' OR fix_required LIKE '%${escaped}%'
         ORDER BY priority, id
         LIMIT 10" | \
        jq -r '.[] | "\(.id) \(.priority) \(.component) [\(.status)] \(.title)"'

    local total=$(sqlite3 "$DB" \
        "SELECT COUNT(*) FROM issues WHERE title LIKE '%${escaped}%' OR problem LIKE '%${escaped}%' OR fix_required LIKE '%${escaped}%'")
    if [[ $total -gt 10 ]]; then echo "(+$((total - 10)) more — refine your search keyword)"; fi
}

# next — Recommended work order for AI agents
# Groups issues by root cause, flags triage-first items, shows chain reasoning
cmd_next() {
    local db="$DB"

    # Count open P0s
    local p0_count=$(sqlite3 "$db" "SELECT COUNT(*) FROM issues WHERE status='open' AND priority='P0'")
    local p1_count=$(sqlite3 "$db" "SELECT COUNT(*) FROM issues WHERE status='open' AND priority='P1'")
    local inprog_count=$(sqlite3 "$db" "SELECT COUNT(*) FROM issues WHERE status='in_progress'")

    echo "── RECOMMENDED NEXT ACTIONS ──"
    echo "Open: ${p0_count} P0 blockers, ${p1_count} P1 issues | ${inprog_count} in_progress"
    echo ""

    # Show in-progress if any (don't duplicate work)
    if [[ $inprog_count -gt 0 ]]; then
        echo "⚡ IN PROGRESS (check before claiming new work):"
        sqlite3 -json "$db" \
            "SELECT i.id, i.priority, i.component, i.title, s.agent
             FROM issues i LEFT JOIN sessions s ON i.fixed_by = s.id
             WHERE i.status = 'in_progress' ORDER BY i.priority, i.id" | \
            jq -r '.[] | "   \(.id) [\(.priority)/\(.component)] \(.agent // "?") — \(.title)"'
        echo ""
    fi

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
