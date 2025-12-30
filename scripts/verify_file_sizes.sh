#!/bin/bash
# File size compliance verification script
# Verifies all source files are ≤500 lines (excluding comments and blank lines)
# Usage: ./verify_file_sizes.sh [--verbose]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MAX_LINES=500
VERBOSE=false

# Parse arguments
if [[ "${1:-}" == "--verbose" ]]; then
    VERBOSE=true
fi

# Check if tokei is installed
if ! command -v tokei &> /dev/null; then
    echo "Error: tokei not found. Install with: cargo install tokei"
    exit 1
fi

echo "=== File Size Compliance Verification ==="
echo "Maximum allowed lines: $MAX_LINES (excluding comments and blank lines)"
echo "Project root: $PROJECT_ROOT"
echo

# Create temporary directory for results
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

# Generate tokei report for the entire project
cd "$PROJECT_ROOT"
tokei --output json > "$TMPDIR/tokei.json"

# Process the JSON output to find files exceeding the limit
# We'll use a Python script for easier JSON processing
python3 - "$TMPDIR/tokei.json" "$MAX_LINES" "$VERBOSE" <<'PYTHON_SCRIPT'
import json
import sys
from pathlib import Path

json_file = sys.argv[1]
max_lines = int(sys.argv[2])
verbose = sys.argv[3] == 'True'

with open(json_file) as f:
    data = json.load(f)

# Collect all files with their line counts
all_files = []
violations = []

# Process each language
for lang, lang_data in data.items():
    if lang in ['Total', 'SUM']:
        continue

    if 'reports' in lang_data:
        for report in lang_data['reports']:
            if 'name' in report and 'stats' in report:
                filepath = report['name']
                stats = report['stats']
                code_lines = stats['code']

                # Filter to only source files we care about
                if any(filepath.endswith(ext) for ext in ['.rs', '.ts', '.tsx', '.js', '.jsx']):
                    # Exclude certain patterns
                    if any(pattern in filepath for pattern in [
                        'node_modules/',
                        'target/',
                        'dist/',
                        '.next/',
                        'build/',
                        'tests/testUtils', # Test utilities may be exempt
                        'fuzz/',
                    ]):
                        continue

                    all_files.append((filepath, code_lines))

                    if code_lines > max_lines:
                        violations.append((filepath, code_lines))

# Sort by line count descending
all_files.sort(key=lambda x: x[1], reverse=True)
violations.sort(key=lambda x: x[1], reverse=True)

# Print results
print(f"Total source files checked: {len(all_files)}")
print(f"Files exceeding {max_lines} lines: {len(violations)}")
print()

if violations:
    print("=== VIOLATIONS ===")
    print(f"{'File':<80} {'Lines':>10}")
    print("-" * 92)
    for filepath, lines in violations:
        print(f"{filepath:<80} {lines:>10}")
    print()

if verbose or len(all_files) <= 50:
    print("=== ALL FILES (sorted by size) ===")
    print(f"{'File':<80} {'Lines':>10} {'Status':>10}")
    print("-" * 102)
    for filepath, lines in all_files:
        status = "✓ OK" if lines <= max_lines else "✗ FAIL"
        print(f"{filepath:<80} {lines:>10} {status:>10}")
    print()

# Exit with error if violations found
if violations:
    print("❌ File size compliance check FAILED")
    print(f"   {len(violations)} file(s) exceed the {max_lines} line limit")
    sys.exit(1)
else:
    print("✅ File size compliance check PASSED")
    print(f"   All {len(all_files)} files are within the {max_lines} line limit")
    sys.exit(0)
PYTHON_SCRIPT
