#!/bin/bash
# Atlas Phase Validation Script
# Purpose: Verify phase file references actual files and commands work

set -e  # Exit on error

PHASE_FILE="$1"

if [ -z "$PHASE_FILE" ]; then
    echo "Usage: $0 <phase-file>"
    echo "Example: $0 phases/stdlib/phase-07d-collection-integration.md"
    exit 1
fi

if [ ! -f "$PHASE_FILE" ]; then
    echo "❌ ERROR: Phase file not found: $PHASE_FILE"
    exit 1
fi

echo "🔍 Validating Phase: $PHASE_FILE"
echo "=================================="
echo ""

# Extract project root (assumes script is in tools/ directory)
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

# Track validation status
ERRORS=0

# Function to check if file exists
check_file() {
    local file="$1"
    if [ -f "$file" ]; then
        echo "✅ File exists: $file"
    else
        echo "❌ File missing: $file"
        ((ERRORS++))
    fi
}

# Function to check if directory exists
check_dir() {
    local dir="$1"
    if [ -d "$dir" ]; then
        echo "✅ Directory exists: $dir"
    else
        echo "❌ Directory missing: $dir"
        ((ERRORS++))
    fi
}

echo "📋 Phase File Validation"
echo "------------------------"

# Extract referenced files from phase file
# Look for patterns like: crates/atlas-runtime/src/...
# This is a simple heuristic - adjust as needed

# Check core runtime files (always referenced)
echo ""
echo "Checking core runtime files..."
check_file "crates/atlas-runtime/src/value.rs"
check_file "crates/atlas-runtime/src/interpreter/expr.rs"
check_file "crates/atlas-runtime/src/vm/mod.rs"
check_file "crates/atlas-runtime/src/stdlib/mod.rs"

# Check memory system
echo ""
echo "Checking memory system..."
check_dir "memory"
check_file "memory/MEMORY.md"
check_file "memory/patterns.md"
check_file "memory/testing-patterns.md"
check_dir "memory/patterns"

# Check if phase references specific files
echo ""
echo "Checking phase-specific references..."

# Extract file paths from phase file (simple grep)
REFERENCED_FILES=$(grep -oE 'crates/[a-zA-Z0-9_/-]+\.(rs|md)' "$PHASE_FILE" | sort -u || true)

if [ -n "$REFERENCED_FILES" ]; then
    while IFS= read -r file; do
        check_file "$file"
    done <<< "$REFERENCED_FILES"
else
    echo "ℹ️  No specific file references found in phase (or pattern didn't match)"
fi

# Validate build commands
echo ""
echo "🔨 Build Validation"
echo "-------------------"

echo "Running: cargo check -p atlas-runtime"
if cargo check -p atlas-runtime --quiet 2>&1 | grep -q "error"; then
    echo "❌ cargo check failed"
    ((ERRORS++))
else
    echo "✅ cargo check passed"
fi

# Final summary
echo ""
echo "=================================="
if [ $ERRORS -eq 0 ]; then
    echo "✅ Validation PASSED - Phase file is accurate"
    exit 0
else
    echo "❌ Validation FAILED - $ERRORS error(s) found"
    echo ""
    echo "Fix these issues before proceeding:"
    echo "1. Ensure all referenced files exist"
    echo "2. Update phase file to reference correct paths"
    echo "3. Fix any build errors"
    exit 1
fi
