#!/usr/bin/env bash
# Build all release packages
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Building all release packages..."
echo ""

# Build Debian package
echo "=== Building Debian package ==="
"$SCRIPT_DIR/build-deb.sh"
echo ""

# Build tarball
echo "=== Building tarball ==="
"$SCRIPT_DIR/build-tarball.sh"
echo ""

echo "✓ All packages built successfully!"
echo ""
echo "Packages created in target/:"
ls -lh "$SCRIPT_DIR/../../target/debian/"*.deb 2>/dev/null || true
ls -lh "$SCRIPT_DIR/../../target/tarball/"*.tar.gz 2>/dev/null || true
