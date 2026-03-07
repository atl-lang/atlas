#!/bin/bash
# Atlas Full Audit — Run all battle test programs
# Usage: ./run.sh [--engine interpreter|vm] [--domain 01-primitives]

ATLAS="${ATLAS_BIN:-$(dirname "$0")/../../target/debug/atlas}"
AUDIT_DIR="$(dirname "$0")"
ENGINE="${ENGINE:-interpreter}"
FAILED=0
PASSED=0
TOTAL=0

if [[ ! -x "$ATLAS" ]]; then
    echo "ERROR: atlas binary not found at $ATLAS"
    echo "Run: cargo build -p atlas-cli"
    exit 1
fi

run_program() {
    local file="$1"
    local domain="$2"
    local expected_file="${file%.atlas}.expected"
    TOTAL=$((TOTAL + 1))

    local output
    output=$("$ATLAS" run "$file" 2>&1)
    local exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        echo "  FAIL  $file"
        echo "        ERROR: $output"
        FAILED=$((FAILED + 1))
        return 1
    fi

    if [[ -f "$expected_file" ]]; then
        local expected
        expected=$(cat "$expected_file")
        if [[ "$output" != "$expected" ]]; then
            echo "  FAIL  $file"
            echo "        EXPECTED: $expected"
            echo "        GOT:      $output"
            FAILED=$((FAILED + 1))
            return 1
        fi
    fi

    echo "  PASS  $(basename "$file")"
    PASSED=$((PASSED + 1))
    return 0
}

echo "Atlas Full Audit — $(date)"
echo "Binary: $ATLAS"
echo ""

for domain_dir in "$AUDIT_DIR"/domains/*/; do
    domain=$(basename "$domain_dir")
    if [[ -n "$1" && "$1" != "$domain" && "$1" != "--all" ]]; then
        [[ "$1" == "--domain" && "$2" == "$domain" ]] || continue
    fi

    echo "[$domain]"
    for file in "$domain_dir"*.atlas; do
        [[ -f "$file" ]] || continue
        run_program "$file" "$domain"
    done
    echo ""
done

echo "Results: $PASSED passed, $FAILED failed, $TOTAL total"
[[ $FAILED -eq 0 ]] && exit 0 || exit 1
