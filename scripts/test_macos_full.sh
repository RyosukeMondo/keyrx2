#!/usr/bin/env bash
# macOS comprehensive test runner
# Orchestrates mock tests, E2E tests, and benchmarks with permission checking

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Track test results
MOCK_TESTS_PASSED=false
E2E_TESTS_PASSED=false
E2E_TESTS_SKIPPED=false
BENCHMARKS_PASSED=false
MANUAL_TESTS_RUN=false
OVERALL_EXIT_CODE=0

# Color codes
readonly COLOR_RESET='\033[0m'
readonly COLOR_RED='\033[0;31m'
readonly COLOR_YELLOW='\033[1;33m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_BLUE='\033[0;34m'

# Cleanup handler
cleanup() {
    # Additional cleanup can be added here if needed
    :
}

# Trap Ctrl+C and errors
trap cleanup EXIT
trap 'echo -e "\n${COLOR_YELLOW}⚠️  Test run interrupted${COLOR_RESET}"; exit 130' INT

# Print section header
print_section() {
    echo ""
    echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
    echo -e "${COLOR_BLUE}$1${COLOR_RESET}"
    echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
    echo ""
}

# Print step indicator
print_step() {
    echo -e "${COLOR_BLUE}▶ $1${COLOR_RESET}"
}

# Print success
print_success() {
    echo -e "${COLOR_GREEN}✅ $1${COLOR_RESET}"
}

# Print warning
print_warning() {
    echo -e "${COLOR_YELLOW}⚠️  $1${COLOR_RESET}"
}

# Print error
print_error() {
    echo -e "${COLOR_RED}❌ $1${COLOR_RESET}"
}

# Change to project root
cd "$PROJECT_ROOT"

print_section "macOS Test Suite"

#
# Phase 1: Mock Tests (Required)
#
print_section "Phase 1: Mock Tests"
print_step "Running mock tests (no permissions required)..."

if cargo test --package keyrx_daemon --test macos_mock_tests -- --test-threads=1; then
    print_success "Mock tests passed"
    MOCK_TESTS_PASSED=true
else
    print_error "Mock tests failed"
    OVERALL_EXIT_CODE=1
    # Mock test failure is critical - stop execution
    echo ""
    echo -e "${COLOR_RED}========================================${COLOR_RESET}"
    echo -e "${COLOR_RED}CRITICAL: Mock tests failed${COLOR_RESET}"
    echo -e "${COLOR_RED}========================================${COLOR_RESET}"
    echo ""
    exit 1
fi

#
# Phase 2: Permission Check
#
print_section "Phase 2: Accessibility Permission Check"
print_step "Checking Accessibility permission..."

if "$SCRIPT_DIR/check_macos_permission.sh"; then
    print_success "Accessibility permission granted"
    HAS_PERMISSION=true
else
    print_warning "Accessibility permission not granted"
    HAS_PERMISSION=false
    echo ""
    echo "E2E tests will be skipped. To run E2E tests:"
    echo "  1. Open System Settings > Privacy & Security > Accessibility"
    echo "  2. Grant permission to Terminal (or your IDE)"
    echo "  3. Re-run this script"
    echo ""
fi

#
# Phase 3: E2E Tests (Permission Required)
#
print_section "Phase 3: E2E Tests"

if [ "$HAS_PERMISSION" = true ]; then
    print_step "Running E2E tests (permission granted)..."

    if cargo test --package keyrx_daemon --test e2e_macos_basic --test e2e_macos_multidevice -- --test-threads=1; then
        print_success "E2E tests passed"
        E2E_TESTS_PASSED=true
    else
        print_error "E2E tests failed"
        E2E_TESTS_PASSED=false
        OVERALL_EXIT_CODE=1
    fi
else
    print_warning "E2E tests skipped (no permission)"
    E2E_TESTS_SKIPPED=true
fi

#
# Phase 4: Benchmarks
#
print_section "Phase 4: Benchmarks"
print_step "Running benchmarks..."

if cargo bench --package keyrx_daemon --bench macos_latency; then
    print_success "Benchmarks completed"
    BENCHMARKS_PASSED=true
else
    print_warning "Benchmarks failed or not available"
    BENCHMARKS_PASSED=false
    # Benchmark failure is not critical
fi

#
# Phase 5: Manual Test Prompt (Interactive Only)
#
# Only prompt if:
# - stdin is a terminal (interactive mode)
# - Accessibility permission is granted
if [ -t 0 ] && [ "$HAS_PERMISSION" = true ]; then
    print_section "Phase 5: Manual Testing (Optional)"

    echo "Automated tests complete. Would you like to run manual tests?"
    echo "Manual tests require physical keyboard interaction to verify:"
    echo "  • Key remapping accuracy"
    echo "  • Tap-hold timing behavior"
    echo "  • Multi-device discrimination"
    echo ""

    # Prompt with default to No (safety)
    read -r -p "Run manual tests? [y/N] " response || {
        # Handle EOF gracefully (e.g., Ctrl+D)
        echo ""
        print_warning "Manual tests skipped (EOF received)"
        response="n"
    }

    echo ""

    case "$response" in
        [yY][eE][sS]|[yY])
            print_step "Manual testing instructions:"
            echo ""
            echo "1. Ensure keyrx daemon is running with a test config"
            echo "2. Test basic remapping (e.g., A → B)"
            echo "3. Test tap-hold behavior with timing variations"
            echo "4. If multi-device, test device-specific configs"
            echo "5. Verify latency is imperceptible (<1ms)"
            echo ""
            echo "Refer to docs/development/MACOS_TESTING_GUIDE.md for detailed procedures"
            echo ""
            read -r -p "Press Enter when manual testing is complete..." || true
            echo ""
            print_success "Manual tests completed"
            MANUAL_TESTS_RUN=true
            ;;
        *)
            print_warning "Manual tests skipped by user"
            ;;
    esac
fi

#
# Summary
#
print_section "Test Summary"

echo "Results:"
echo ""

if [ "$MOCK_TESTS_PASSED" = true ]; then
    print_success "Mock tests: PASSED"
else
    print_error "Mock tests: FAILED"
fi

if [ "$E2E_TESTS_SKIPPED" = true ]; then
    print_warning "E2E tests: SKIPPED (no permission)"
elif [ "$E2E_TESTS_PASSED" = true ]; then
    print_success "E2E tests: PASSED"
else
    print_error "E2E tests: FAILED"
fi

if [ "$BENCHMARKS_PASSED" = true ]; then
    print_success "Benchmarks: COMPLETED"
else
    print_warning "Benchmarks: FAILED/UNAVAILABLE"
fi

if [ "$MANUAL_TESTS_RUN" = true ]; then
    print_success "Manual tests: COMPLETED"
elif [ -t 0 ] && [ "$HAS_PERMISSION" = true ]; then
    print_warning "Manual tests: SKIPPED BY USER"
else
    echo -e "${COLOR_BLUE}ℹ️  Manual tests: NOT PROMPTED (non-interactive or no permission)${COLOR_RESET}"
fi

echo ""

if [ $OVERALL_EXIT_CODE -eq 0 ]; then
    print_section "✅ Test suite completed successfully"
else
    print_section "❌ Test suite completed with failures"
fi

exit $OVERALL_EXIT_CODE
