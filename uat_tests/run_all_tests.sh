#!/bin/bash
# UAT Test Runner
# Compiles and verifies all test configurations

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

COMPILER="../target/release/keyrx_compiler"
TESTS_PASSED=0
TESTS_FAILED=0

echo "=================================================="
echo "KeyRx UAT - User Acceptance Testing"
echo "=================================================="
echo ""

# Check if compiler exists
if [ ! -f "$COMPILER" ]; then
    echo -e "${RED}Error: Compiler not found${NC}"
    echo "Please run: cargo build --release"
    exit 1
fi

# Test function
run_test() {
    local test_name=$1
    local config_file=$2
    local output_file="${config_file%.rhai}.krx"

    echo "----------------------------------------"
    echo "Test: $test_name"
    echo "Config: $config_file"
    echo ""

    # Compile
    echo -n "  Compiling... "
    if $COMPILER compile "$config_file" -o "$output_file" 2>&1 | grep -q "error"; then
        echo -e "${RED}FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    echo -e "${GREEN}OK${NC}"

    # Verify
    echo -n "  Verifying... "
    if $COMPILER verify "$output_file" 2>&1 | grep -q "error"; then
        echo -e "${RED}FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    echo -e "${GREEN}OK${NC}"

    # Parse (check for errors in source file)
    echo -n "  Parsing... "
    if $COMPILER parse "$config_file" > /dev/null 2>&1; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi

    echo -e "${GREEN}✓ PASS${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    return 0
}

# Run tests
run_test "Simple Remapping" "test1_simple.rhai"
run_test "Modifier Mappings" "test2_modifiers.rhai"
run_test "Lock Mappings" "test3_locks.rhai"
run_test "Modified Output" "test4_chords.rhai"
run_test "Complex Multi-Layer" "test5_vim.rhai"
run_test "Multiple Devices" "test9_multidevice.rhai"

# Summary
echo ""
echo "=================================================="
echo "UAT Summary"
echo "=================================================="
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    echo "Configuration compiler is working correctly!"
    exit 0
else
    echo ""
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    echo "Please review errors above"
    exit 1
fi
