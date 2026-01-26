#!/usr/bin/env bash
# Validate package build system
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Validating KeyRx Package Build System"
echo "======================================"
echo ""

FAILED=0

# Check required tools
echo "Checking build tools..."
TOOLS=(cargo npm wasm-pack dpkg-deb tar)
for tool in "${TOOLS[@]}"; do
    if command -v "$tool" &> /dev/null; then
        echo "  ✓ $tool found"
    else
        echo "  ✗ $tool not found"
        FAILED=1
    fi
done
echo ""

# Check scripts are executable
echo "Checking script permissions..."
SCRIPTS=(build-deb.sh build-tarball.sh build-all.sh)
for script in "${SCRIPTS[@]}"; do
    if [[ -x "$SCRIPT_DIR/$script" ]]; then
        echo "  ✓ $script is executable"
    else
        echo "  ✗ $script is not executable"
        FAILED=1
    fi
done
echo ""

# Check GitHub workflow exists
echo "Checking GitHub Actions workflow..."
if [[ -f "$PROJECT_ROOT/.github/workflows/release.yml" ]]; then
    echo "  ✓ release.yml exists"

    # Check for correct UI path
    if grep -q "keyrx_ui/" "$PROJECT_ROOT/.github/workflows/release.yml"; then
        echo "  ✓ Uses correct keyrx_ui path"
    else
        echo "  ✗ May have incorrect UI path"
        FAILED=1
    fi

    # Check for package building steps
    if grep -q "build-deb.sh" "$PROJECT_ROOT/.github/workflows/release.yml" && \
       grep -q "build-tarball.sh" "$PROJECT_ROOT/.github/workflows/release.yml"; then
        echo "  ✓ Includes package build steps"
    else
        echo "  ✗ Missing package build steps"
        FAILED=1
    fi
else
    echo "  ✗ release.yml not found"
    FAILED=1
fi
echo ""

# Check Makefile targets
echo "Checking Makefile targets..."
if [[ -f "$PROJECT_ROOT/Makefile" ]]; then
    TARGETS=(package package-deb package-tar release)
    for target in "${TARGETS[@]}"; do
        if grep -q "^$target:" "$PROJECT_ROOT/Makefile"; then
            echo "  ✓ $target target exists"
        else
            echo "  ✗ $target target missing"
            FAILED=1
        fi
    done
else
    echo "  ✗ Makefile not found"
    FAILED=1
fi
echo ""

# Check documentation
echo "Checking documentation..."
DOCS=(
    "docs/RELEASE.md"
    "scripts/package/README.md"
    "INSTALLER_SETUP.md"
)
for doc in "${DOCS[@]}"; do
    if [[ -f "$PROJECT_ROOT/$doc" ]]; then
        echo "  ✓ $doc exists"
    else
        echo "  ✗ $doc missing"
        FAILED=1
    fi
done
echo ""

# Validate scripts syntax
echo "Validating shell script syntax..."
for script in "$SCRIPT_DIR"/*.sh; do
    if bash -n "$script" 2>&1; then
        echo "  ✓ $(basename "$script") syntax OK"
    else
        echo "  ✗ $(basename "$script") syntax error"
        FAILED=1
    fi
done
echo ""

# Summary
echo "======================================"
if [[ $FAILED -eq 0 ]]; then
    echo "✅ All validations passed!"
    echo ""
    echo "Ready to:"
    echo "  1. Test build: make package"
    echo "  2. Commit: git add -A && git commit -m 'feat: add Linux installers'"
    echo "  3. Create release: git tag v0.2.0 && git push --tags"
    exit 0
else
    echo "❌ Some validations failed!"
    echo ""
    echo "Please fix the issues above before proceeding."
    exit 1
fi
