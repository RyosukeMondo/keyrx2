#!/usr/bin/env bash
# Synchronize version across all project files (SSOT: Cargo.toml)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Extract version from Cargo.toml (workspace version)
VERSION=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')

if [[ -z "$VERSION" ]]; then
    echo "Error: Could not extract version from Cargo.toml"
    exit 1
fi

echo "Synchronizing version to: $VERSION"
echo ""

# Update package.json
PACKAGE_JSON="$PROJECT_ROOT/keyrx_ui/package.json"
if [[ -f "$PACKAGE_JSON" ]]; then
    # Use jq if available, otherwise sed
    if command -v jq &> /dev/null; then
        TMP=$(mktemp)
        jq ".version = \"$VERSION\"" "$PACKAGE_JSON" > "$TMP"
        mv "$TMP" "$PACKAGE_JSON"
        echo "✓ Updated keyrx_ui/package.json to $VERSION"
    else
        sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$PACKAGE_JSON"
        echo "✓ Updated keyrx_ui/package.json to $VERSION (using sed)"
    fi
else
    echo "✗ keyrx_ui/package.json not found"
fi

# Update Python tray version
TRAY_SCRIPT="$PROJECT_ROOT/keyrx_tray/keyrx-tray.py"
if [[ -f "$TRAY_SCRIPT" ]]; then
    sed -i "s/set_version(\"[^\"]*\")/set_version(\"$VERSION\")/" "$TRAY_SCRIPT"
    echo "✓ Updated keyrx_tray/keyrx-tray.py to $VERSION"
fi

# Generate version.ts
if [[ -f "$SCRIPT_DIR/generate-version.js" ]]; then
    node "$SCRIPT_DIR/generate-version.js"
    echo "✓ Generated keyrx_ui/src/version.ts"
else
    echo "⚠  generate-version.js not found, skipping version.ts generation"
fi

echo ""
echo "Version synchronization complete!"
echo "All project files now use version: $VERSION"
