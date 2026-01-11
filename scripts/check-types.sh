#!/bin/bash
# Type Consistency Check Script
# Verifies that TypeScript types are in sync with Rust types
#
# Usage:
#   scripts/check-types.sh          # Run type consistency check
#   scripts/check-types.sh --fix    # Auto-stage changed types

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GENERATED_TYPES="$PROJECT_ROOT/keyrx_ui/src/types/generated.ts"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
AUTO_FIX=false
if [[ "${1:-}" == "--fix" ]]; then
    AUTO_FIX=true
fi

echo "Checking type consistency..."
echo ""

# Check if typeshare is installed
if ! command -v typeshare &> /dev/null; then
    echo -e "${RED}Error: typeshare is not installed${NC}"
    echo ""
    echo "Install with:"
    echo "  cargo install typeshare-cli"
    echo ""
    exit 1
fi

# Save current types for comparison
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

if [[ -f "$GENERATED_TYPES" ]]; then
    cp "$GENERATED_TYPES" "$TEMP_DIR/generated.ts.old"
else
    touch "$TEMP_DIR/generated.ts.old"
fi

# Regenerate TypeScript types
echo "Regenerating TypeScript types from Rust..."
cd "$PROJECT_ROOT/keyrx_daemon"

if ! typeshare --lang=typescript --output-file=../keyrx_ui/src/types/generated.ts src/ > /dev/null 2>&1; then
    echo -e "${RED}Error: Failed to regenerate types${NC}"
    echo ""
    echo "Run 'typeshare --lang=typescript --output-file=../keyrx_ui/src/types/generated.ts src/' in keyrx_daemon directory to see the error"
    exit 1
fi

echo "✓ Types regenerated successfully"
echo ""

# Check if types changed
if ! diff -q "$TEMP_DIR/generated.ts.old" "$GENERATED_TYPES" > /dev/null 2>&1; then
    echo -e "${RED}✗ Type mismatch detected!${NC}"
    echo ""
    echo "The generated TypeScript types do not match the current Rust types."
    echo ""
    echo "Differences:"
    echo "----------------------------------------"
    diff -u "$TEMP_DIR/generated.ts.old" "$GENERATED_TYPES" || true
    echo "----------------------------------------"
    echo ""

    if [[ "$AUTO_FIX" == true ]]; then
        echo "Auto-staging updated types..."
        git add "$GENERATED_TYPES"
        echo -e "${GREEN}✓ Types staged for commit${NC}"
        echo ""
        echo "The updated types have been automatically staged."
        echo "Please review the changes and commit them."
        exit 0
    else
        echo "To fix this issue:"
        echo "  1. Review the type changes shown above"
        echo "  2. Stage the updated types: git add $GENERATED_TYPES"
        echo "  3. Commit the changes: git commit"
        echo ""
        echo "Or run with --fix to auto-stage:"
        echo "  scripts/check-types.sh --fix"
        echo ""
        exit 1
    fi
else
    echo -e "${GREEN}✓ Types are in sync${NC}"
    echo ""
    exit 0
fi
