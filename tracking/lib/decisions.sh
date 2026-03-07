#!/bin/bash
# decisions.sh — Decision management (add, supersede, deprecate)

cmd_add_decision() {
    local title="$1"
    local component="$2"
    local rule="$3"
    local rationale="${4:-}"

    if [[ ${#title} -lt 5 ]]; then
        echo "❌ ERROR: title must be at least 5 chars"
        exit 1
    fi
    if [[ ${#rule} -lt 10 ]]; then
        echo "❌ ERROR: rule must be at least 10 chars"
        exit 1
    fi

    local session=$(get_session)
    local version=$(sqlite3 "$DB" "SELECT version FROM state")
    local next_id=$(sqlite3 "$DB" "SELECT 'D-' || printf('%03d', COALESCE(MAX(CAST(substr(id, 3) AS INTEGER)), 0) + 1) FROM decisions")

    sqlite3 "$DB" "INSERT INTO decisions (id, title, status, version, session_id, component, rule, rationale) VALUES ('$next_id', '$(sql_escape "$title")', 'active', '$version', '$session', '$component', '$(sql_escape "$rule")', '$(sql_escape "$rationale")')"

    echo "✓ Created: $next_id"
    echo "  Title: $title"
    echo "  Component: $component"
}

cmd_supersede() {
    local old_id="$1"
    local new_id="$2"

    local old_status=$(sqlite3 "$DB" "SELECT status FROM decisions WHERE id='$old_id'")
    if [[ -z "$old_status" ]]; then
        echo "ERROR: Decision $old_id not found"
        exit 1
    fi

    local new_status=$(sqlite3 "$DB" "SELECT status FROM decisions WHERE id='$new_id'")
    if [[ -z "$new_status" ]]; then
        echo "ERROR: Decision $new_id not found"
        exit 1
    fi

    sqlite3 "$DB" "UPDATE decisions SET status='superseded', superseded_by='$new_id' WHERE id='$old_id'"
    sqlite3 "$DB" "UPDATE decisions SET supersedes='$old_id' WHERE id='$new_id'"

    echo "✓ $old_id superseded by $new_id"
}

cmd_update_decision() {
    local id="$1"
    local field="$2"
    local value="$3"

    local current=$(sqlite3 "$DB" "SELECT status FROM decisions WHERE id='$id'")
    if [[ -z "$current" ]]; then
        echo "ERROR: Decision $id not found"
        exit 1
    fi

    case "$field" in
        rule|rationale|title|component)
            sqlite3 "$DB" "UPDATE decisions SET ${field}='$(sql_escape "$value")' WHERE id='$id'"
            echo "✓ Updated $id.$field"
            ;;
        *)
            echo "ERROR: Invalid field '$field'. Valid: rule, rationale, title, component"
            exit 1
            ;;
    esac
}

cmd_deprecate() {
    local id="$1"
    local reason="${2:-}"

    local current=$(sqlite3 "$DB" "SELECT status FROM decisions WHERE id='$id'")
    if [[ -z "$current" ]]; then
        echo "ERROR: Decision $id not found"
        exit 1
    fi

    sqlite3 "$DB" "UPDATE decisions SET status='deprecated' WHERE id='$id'"
    echo "✓ Deprecated: $id"
    [[ -n "$reason" ]] && echo "  Reason: $reason"
}
