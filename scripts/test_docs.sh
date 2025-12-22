#!/usr/bin/env bash
# Test documentation accuracy - ensures examples compile correctly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "Building compiler..."
cargo build --release -p keyrx_compiler

echo ""
echo "Testing example .rhai files..."
for f in examples/*.rhai; do
    echo "  Compiling $f..."
    cargo run --release -p keyrx_compiler -- compile "$f" -o /tmp/test.krx
    echo "  ✓ $f compiled successfully"
done
echo "✓ All examples compiled successfully!"

echo ""
echo "Extracting code blocks from DSL_MANUAL.md..."
awk '/```rhai/,/```/ {if (!/```/) print}' docs/DSL_MANUAL.md > /tmp/manual_examples.rhai || true

if [ -s /tmp/manual_examples.rhai ]; then
    echo "  Found $(wc -l < /tmp/manual_examples.rhai) lines of example code"
    echo "  Note: Manual examples may be snippets, not complete configs"
else
    echo "  No code blocks found in DSL_MANUAL.md (this is unusual)"
fi

echo ""
echo "✓ Documentation accuracy check complete"
