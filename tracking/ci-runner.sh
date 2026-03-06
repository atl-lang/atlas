#!/bin/bash
# ══════════════════════════════════════════════════════════════════════════════
# Atlas Nightly CI Runner
# ══════════════════════════════════════════════════════════════════════════════
#
# Runs full validation suite and writes results to tracking/ci-status.json
# Run via launchd plist (nightly 2am) or manually: atlas-track run-ci
#
# Output:
#   tracking/ci-status.json  — machine-readable results (read by atlas-track go)
#   stdout                   — human-readable summary
# ══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

# ── Paths ──────────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
STATUS_FILE="$PROJECT_ROOT/tracking/ci-status.json"
DB="$PROJECT_ROOT/tracking/atlas.db"

# ── Colors ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ── Helpers ───────────────────────────────────────────────────────────────────
log() { echo -e "${CYAN}[CI]${NC} $*"; }
pass() { echo -e "  ${GREEN}PASS${NC} $*"; }
fail() { echo -e "  ${RED}FAIL${NC} $*"; }

START_TIME=$(date +%s)
RUN_AT=$(date -u +"%Y-%m-%dT%H:%M:%S")
GIT_COMMIT=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo "none")

echo ""
echo -e "${BOLD}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}              ATLAS NIGHTLY CI RUNNER${NC}"
echo -e "${BOLD}══════════════════════════════════════════════════════════════════${NC}"
echo -e "  Commit: $GIT_COMMIT  |  Started: $RUN_AT"
echo ""

cd "$PROJECT_ROOT"

# ── Per-check state ───────────────────────────────────────────────────────────
FMT_STATUS="pass"
CLIPPY_STATUS="pass"
TESTS_STATUS="pass"
PARITY_STATUS="pass"
BATTLE_STATUS="pass"
CORPUS_STATUS="pass"

TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED_NAMES="[]"

OVERALL_STATUS="pass"
P0_CREATED=""

# ── Check 1: cargo fmt ────────────────────────────────────────────────────────
log "cargo fmt --check"
if cargo fmt --check --quiet 2>/dev/null; then
  pass "fmt"
else
  fail "fmt — run 'cargo fmt' to fix"
  FMT_STATUS="fail"
  OVERALL_STATUS="fail"
fi

# ── Check 2: cargo clippy ─────────────────────────────────────────────────────
log "cargo clippy --workspace"
CLIPPY_OUT=$(cargo clippy --workspace --quiet -- -D warnings 2>&1) || {
  fail "clippy"
  echo "$CLIPPY_OUT" | head -20
  CLIPPY_STATUS="fail"
  OVERALL_STATUS="fail"
}
if [[ "$CLIPPY_STATUS" == "pass" ]]; then
  pass "clippy"
fi

# ── Check 3: cargo nextest --workspace ───────────────────────────────────────
log "cargo nextest run --workspace --no-fail-fast"
NEXTEST_OUT_FILE=$(mktemp)
if cargo nextest run --workspace --no-fail-fast 2>&1 | tee "$NEXTEST_OUT_FILE"; then
  pass "tests"
  TESTS_TOTAL=$(grep -E '^test run' "$NEXTEST_OUT_FILE" | grep -oE '[0-9]+ tests?' | head -1 | grep -oE '[0-9]+' || echo "0")
  TESTS_PASSED=$TESTS_TOTAL
else
  fail "tests"
  TESTS_STATUS="fail"
  OVERALL_STATUS="fail"
  # Extract failing test names
  FAILED_LIST=$(grep -E '^\s+FAIL ' "$NEXTEST_OUT_FILE" | sed 's/^\s*FAIL //' | head -20 || true)
  TESTS_TOTAL=$(grep -E 'tests? run' "$NEXTEST_OUT_FILE" | grep -oE '[0-9]+' | head -1 || echo "0")
  FAIL_COUNT=$(echo "$FAILED_LIST" | grep -c . || echo "0")
  TESTS_PASSED=$(( TESTS_TOTAL - FAIL_COUNT ))
  # Build JSON array of failed test names
  if [[ -n "$FAILED_LIST" ]]; then
    TESTS_FAILED_NAMES=$(echo "$FAILED_LIST" | jq -R . | jq -s .)
  fi
fi
rm -f "$NEXTEST_OUT_FILE"

# ── Check 4: parity sweep ─────────────────────────────────────────────────────
log "cargo nextest run -p atlas-runtime -E 'test(parity)' --no-fail-fast"
PARITY_OUT=$(cargo nextest run -p atlas-runtime -E 'test(parity)' --no-fail-fast 2>&1) || {
  fail "parity sweep"
  PARITY_STATUS="fail"
  OVERALL_STATUS="fail"
}
if [[ "$PARITY_STATUS" == "pass" ]]; then
  pass "parity"
fi

# ── Check 5: battle tests ─────────────────────────────────────────────────────
log "battle tests (battle-test/hydra-v2/)"
BATTLE_DIR="$PROJECT_ROOT/battle-test/hydra-v2"
BATTLE_FAILED_FILES=""
if [[ -d "$BATTLE_DIR" ]]; then
  BATTLE_FILES=$(find "$BATTLE_DIR" -name "*.atlas" 2>/dev/null || true)
  if [[ -n "$BATTLE_FILES" ]]; then
    while IFS= read -r f; do
      if ! atlas run "$f" >/dev/null 2>&1; then
        BATTLE_STATUS="fail"
        OVERALL_STATUS="fail"
        BATTLE_FAILED_FILES="$BATTLE_FAILED_FILES $f"
        fail "battle: $(basename "$f")"
      fi
    done <<< "$BATTLE_FILES"
    if [[ "$BATTLE_STATUS" == "pass" ]]; then
      pass "battle tests"
    fi
  else
    log "  No battle test files found — skipping"
  fi
else
  log "  battle-test/hydra-v2/ not found — skipping"
fi

# ── Check 6: corpus tests ─────────────────────────────────────────────────────
log "cargo nextest run -p atlas-runtime -E 'test(corpus)' --no-fail-fast"
CORPUS_OUT=$(cargo nextest run -p atlas-runtime -E 'test(corpus)' --no-fail-fast 2>&1) || {
  fail "corpus"
  CORPUS_STATUS="fail"
  OVERALL_STATUS="fail"
}
if [[ "$CORPUS_STATUS" == "pass" ]]; then
  pass "corpus"
fi

# ── Timing ────────────────────────────────────────────────────────────────────
END_TIME=$(date +%s)
DURATION=$(( END_TIME - START_TIME ))

# ── Write ci-status.json ──────────────────────────────────────────────────────
jq -n \
  --arg run_at "$RUN_AT" \
  --arg status "$OVERALL_STATUS" \
  --argjson duration "$DURATION" \
  --arg fmt "$FMT_STATUS" \
  --arg clippy "$CLIPPY_STATUS" \
  --arg tests "$TESTS_STATUS" \
  --argjson tests_failed "$TESTS_FAILED_NAMES" \
  --argjson tests_total "$TESTS_TOTAL" \
  --argjson tests_passed "$TESTS_PASSED" \
  --arg parity "$PARITY_STATUS" \
  --arg battle "$BATTLE_STATUS" \
  --arg corpus "$CORPUS_STATUS" \
  --arg commit "$GIT_COMMIT" \
  --arg p0 "$P0_CREATED" \
  '{
    run_at: $run_at,
    status: $status,
    duration_seconds: $duration,
    checks: {
      fmt:    { status: $fmt },
      clippy: { status: $clippy },
      tests:  { status: $tests, failed: $tests_failed, total: $tests_total, passed: $tests_passed },
      parity: { status: $parity },
      battle: { status: $battle },
      corpus: { status: $corpus }
    },
    git_commit: $commit,
    p0_created: $p0
  }' > "$STATUS_FILE"

# ── Create P0 issues for failures ─────────────────────────────────────────────
create_p0_if_needed() {
  local check="$1"
  local check_status="$2"

  if [[ "$check_status" == "fail" ]]; then
    local title="CI: $check failed (nightly)"
    # Check if a CI P0 already exists for this check (don't duplicate)
    local existing
    existing=$(sqlite3 "$DB" "SELECT id FROM issues WHERE title LIKE 'CI: $check%' AND status IN ('open','in_progress') LIMIT 1" 2>/dev/null || echo "")
    if [[ -z "$existing" ]]; then
      local new_id
      new_id=$(sqlite3 "$DB" "SELECT 'H-' || printf('%03d', COALESCE(MAX(CAST(substr(id, 3) AS INTEGER)), 0) + 1) FROM issues" 2>/dev/null || echo "")
      if [[ -n "$new_id" ]]; then
        sqlite3 "$DB" "INSERT INTO issues (id, title, status, priority, severity, component, version, source, problem, fix_required, found_by, tags) VALUES ('$new_id', '$title', 'open', 'P0', 'critical', 'infra', '0.3.0', 'nightly-ci', 'Nightly CI detected $check failure at $RUN_AT on commit $GIT_COMMIT', 'Fix $check failures before continuing new work', 'nightly-ci', 'ci')" 2>/dev/null || true
        echo "$new_id"
        log "  Created P0 issue: $new_id ($title)"
      fi
    else
      log "  P0 already exists for $check: $existing (not duplicating)"
    fi
  fi
}

if [[ "$OVERALL_STATUS" == "fail" ]] && [[ -f "$DB" ]]; then
  log "Creating P0 issues for failures..."
  for check_name in fmt clippy tests parity battle corpus; do
    var="${check_name^^}_STATUS"
    # indirect variable reference
    check_val="${!var}"
    p0_id=$(create_p0_if_needed "$check_name" "$check_val")
    if [[ -n "$p0_id" ]] && [[ -z "$P0_CREATED" ]]; then
      P0_CREATED="$p0_id"
      # Update the status file with the P0 id
      jq --arg p0 "$P0_CREATED" '.p0_created = $p0' "$STATUS_FILE" > "${STATUS_FILE}.tmp" && mv "${STATUS_FILE}.tmp" "$STATUS_FILE"
    fi
  done
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}══════════════════════════════════════════════════════════════════${NC}"
if [[ "$OVERALL_STATUS" == "pass" ]]; then
  echo -e "${GREEN}${BOLD}✓ CI PASS${NC} — All checks passed in ${DURATION}s"
else
  echo -e "${RED}${BOLD}✗ CI FAIL${NC} — Failures detected (${DURATION}s)"
  echo ""
  [[ "$FMT_STATUS" == "fail" ]]    && echo -e "  ${RED}✗${NC} fmt"
  [[ "$CLIPPY_STATUS" == "fail" ]] && echo -e "  ${RED}✗${NC} clippy"
  [[ "$TESTS_STATUS" == "fail" ]]  && echo -e "  ${RED}✗${NC} tests"
  [[ "$PARITY_STATUS" == "fail" ]] && echo -e "  ${RED}✗${NC} parity"
  [[ "$BATTLE_STATUS" == "fail" ]] && echo -e "  ${RED}✗${NC} battle"
  [[ "$CORPUS_STATUS" == "fail" ]] && echo -e "  ${RED}✗${NC} corpus"
  echo ""
  echo -e "  Run 'atlas-track ci-status' for details."
  echo -e "  CI failures = P0 blockers — fix before new work."
fi
echo -e "${BOLD}══════════════════════════════════════════════════════════════════${NC}"
echo ""
