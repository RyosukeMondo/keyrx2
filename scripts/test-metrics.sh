#!/bin/bash
# Test Metrics Collection Script
# Collects test health metrics from Vitest JSON output for tracking and trending
#
# Usage:
#   scripts/test-metrics.sh [options]
#
# Options:
#   --test-output FILE    Path to Vitest JSON output (default: /tmp/test-output.json)
#   --coverage-file FILE  Path to coverage summary JSON (default: keyrx_ui/coverage/coverage-summary.json)
#   --output FILE         Output path for metrics JSON (default: test-metrics.json)
#   --json                Output results as JSON
#   --quiet               Suppress non-error output
#
# Exit codes:
#   0 - Metrics collected successfully
#   1 - Collection errors (partial data may be available)
#   2 - Missing files or configuration errors
#
# Metrics collected:
#   - Test counts: total, passed, failed, pending
#   - Pass rate percentage
#   - Total test duration
#   - Coverage percentages (lines, branches, functions, statements)
#   - Flaky test count (tests that passed on retry)
#   - Timestamp

set -euo pipefail

# Get script directory and source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export COMMON_SH_DIR="$SCRIPT_DIR/lib"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Configuration defaults
TEST_OUTPUT="/tmp/test-output.json"
COVERAGE_FILE="keyrx_ui/coverage/coverage-summary.json"
OUTPUT_FILE="test-metrics.json"

# Parse arguments
parse_script_args() {
    # Parse common flags first
    parse_common_flags "$@"

    # Parse script-specific flags
    local args=("${REMAINING_ARGS[@]}")
    REMAINING_ARGS=()

    while [[ ${#args[@]} -gt 0 ]]; do
        case ${args[0]} in
            --test-output)
                if [[ ${#args[@]} -lt 2 ]]; then
                    log_error "--test-output requires a path argument"
                    return 1
                fi
                TEST_OUTPUT="${args[1]}"
                args=("${args[@]:2}")
                ;;
            --coverage-file)
                if [[ ${#args[@]} -lt 2 ]]; then
                    log_error "--coverage-file requires a path argument"
                    return 1
                fi
                COVERAGE_FILE="${args[1]}"
                args=("${args[@]:2}")
                ;;
            --output)
                if [[ ${#args[@]} -lt 2 ]]; then
                    log_error "--output requires a path argument"
                    return 1
                fi
                OUTPUT_FILE="${args[1]}"
                args=("${args[@]:2}")
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: ${args[0]}"
                show_usage
                return 1
                ;;
        esac
    done
}

show_usage() {
    cat << EOF
Test Metrics Collection Script

Usage:
  scripts/test-metrics.sh [options]

Options:
  --test-output FILE    Path to Vitest JSON output (default: /tmp/test-output.json)
  --coverage-file FILE  Path to coverage summary JSON (default: keyrx_ui/coverage/coverage-summary.json)
  --output FILE         Output path for metrics JSON (default: test-metrics.json)
  --json                Output results as JSON
  --quiet               Suppress non-error output

Exit codes:
  0 - Metrics collected successfully
  1 - Collection errors (partial data may be available)
  2 - Missing files or configuration errors

Metrics collected:
  - Test counts: total, passed, failed, pending
  - Pass rate percentage
  - Total test duration
  - Coverage percentages (lines, branches, functions, statements)
  - Flaky test count (tests that passed on retry)
  - Timestamp
EOF
}

# Check if jq is available, recommend installation if not
check_jq() {
    if ! command_exists jq; then
        log_error "jq is required for JSON parsing"
        log_error "Install jq: sudo apt-get install jq"
        return 2
    fi
}

# Extract test metrics from Vitest JSON output
extract_test_metrics() {
    local test_output_path="$1"

    if [[ ! -f "$test_output_path" ]]; then
        log_error "Test output file not found: $test_output_path" >&2
        log_error "Run tests with: npx vitest run --reporter=json --outputFile.json=$test_output_path" >&2
        return 2
    fi

    log_info "Reading test results from: $test_output_path" >&2

    # Extract basic test counts
    local total_tests passed_tests failed_tests pending_tests success
    total_tests=$(jq -r '.numTotalTests // 0' "$test_output_path")
    passed_tests=$(jq -r '.numPassedTests // 0' "$test_output_path")
    failed_tests=$(jq -r '.numFailedTests // 0' "$test_output_path")
    pending_tests=$(jq -r '.numPendingTests // 0' "$test_output_path")
    success=$(jq -r '.success // false' "$test_output_path")

    # Calculate pass rate
    local pass_rate=0
    if [[ "$total_tests" -gt 0 ]]; then
        pass_rate=$(awk "BEGIN {printf \"%.2f\", ($passed_tests / $total_tests) * 100}")
    fi

    # Calculate total duration (sum of all test file durations)
    local total_duration
    total_duration=$(jq '[.testResults[] | (.endTime - .startTime)] | add // 0' "$test_output_path")

    # Detect flaky tests (tests that initially failed but passed on retry)
    # In Vitest JSON, we look for tests with multiple attempts where final status is passed
    local flaky_count
    flaky_count=$(jq '[.testResults[].assertionResults[] | select(.status == "passed" and .retryReasons != null and (.retryReasons | length) > 0)] | length // 0' "$test_output_path" 2>/dev/null || echo "0")

    # Build test metrics JSON
    cat <<EOF
{
  "tests": {
    "total": $total_tests,
    "passed": $passed_tests,
    "failed": $failed_tests,
    "pending": $pending_tests,
    "passRate": $pass_rate,
    "flakyCount": $flaky_count
  },
  "duration": {
    "total": $total_duration,
    "unit": "milliseconds"
  },
  "success": $success
}
EOF
}

# Extract coverage metrics from coverage-summary.json
extract_coverage_metrics() {
    local coverage_path="$SCRIPT_DIR/../$COVERAGE_FILE"

    if [[ ! -f "$coverage_path" ]]; then
        log_warn "Coverage file not found: $coverage_path" >&2
        log_warn "Coverage metrics will be unavailable" >&2
        # Return empty coverage object
        echo '{"lines":0,"branches":0,"functions":0,"statements":0}'
        return 0
    fi

    log_info "Reading coverage from: $coverage_path" >&2

    # Extract coverage percentages
    local lines_pct branches_pct functions_pct statements_pct
    lines_pct=$(jq -r '.total.lines.pct // 0' "$coverage_path")
    branches_pct=$(jq -r '.total.branches.pct // 0' "$coverage_path")
    functions_pct=$(jq -r '.total.functions.pct // 0' "$coverage_path")
    statements_pct=$(jq -r '.total.statements.pct // 0' "$coverage_path")

    # Build coverage JSON
    cat <<EOF
{
  "lines": $lines_pct,
  "branches": $branches_pct,
  "functions": $functions_pct,
  "statements": $statements_pct
}
EOF
}

# Combine all metrics into final JSON
combine_metrics() {
    local test_metrics="$1"
    local coverage_metrics="$2"
    local timestamp
    timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    # Validate JSON inputs
    if ! echo "$test_metrics" | jq empty 2>/dev/null; then
        log_error "Invalid test metrics JSON"
        return 1
    fi

    if ! echo "$coverage_metrics" | jq empty 2>/dev/null; then
        log_error "Invalid coverage metrics JSON"
        return 1
    fi

    # Combine into final metrics object
    echo "$test_metrics" | jq \
        --argjson coverage "$coverage_metrics" \
        --arg timestamp "$timestamp" \
        '{
            timestamp: $timestamp,
            tests: .tests,
            duration: .duration,
            coverage: $coverage,
            success: .success
        }'
}

# Output metrics summary to console
output_metrics_summary() {
    local metrics="$1"

    if [[ "$QUIET_MODE" == "true" ]]; then
        return 0
    fi

    separator
    log_info "Test Metrics Summary:"
    log_info ""

    # Test counts
    local total passed failed pending pass_rate flaky
    total=$(echo "$metrics" | jq -r '.tests.total')
    passed=$(echo "$metrics" | jq -r '.tests.passed')
    failed=$(echo "$metrics" | jq -r '.tests.failed')
    pending=$(echo "$metrics" | jq -r '.tests.pending')
    pass_rate=$(echo "$metrics" | jq -r '.tests.passRate')
    flaky=$(echo "$metrics" | jq -r '.tests.flakyCount')

    log_info "  Tests:"
    log_info "    Total:     $total"
    log_info "    Passed:    $passed"
    log_info "    Failed:    $failed"
    log_info "    Pending:   $pending"
    log_info "    Pass Rate: ${pass_rate}%"
    if [[ "$flaky" -gt 0 ]]; then
        log_warn "    Flaky:     $flaky (tests that passed on retry)"
    fi

    # Duration
    local duration_ms
    duration_ms=$(echo "$metrics" | jq -r '.duration.total')
    local duration_sec
    duration_sec=$(awk "BEGIN {printf \"%.2f\", $duration_ms / 1000}")
    log_info ""
    log_info "  Duration: ${duration_sec}s (${duration_ms}ms)"

    # Coverage
    local cov_lines cov_branches cov_functions cov_statements
    cov_lines=$(echo "$metrics" | jq -r '.coverage.lines')
    cov_branches=$(echo "$metrics" | jq -r '.coverage.branches')
    cov_functions=$(echo "$metrics" | jq -r '.coverage.functions')
    cov_statements=$(echo "$metrics" | jq -r '.coverage.statements')

    log_info ""
    log_info "  Coverage:"
    log_info "    Lines:      ${cov_lines}%"
    log_info "    Branches:   ${cov_branches}%"
    log_info "    Functions:  ${cov_functions}%"
    log_info "    Statements: ${cov_statements}%"

    separator
}

# Main execution
main() {
    # Setup logging
    setup_log_file "test-metrics"

    # Parse arguments
    if ! parse_script_args "$@"; then
        return 2
    fi

    log_header "Test Metrics Collection"

    # Check for jq
    if ! check_jq; then
        return 2
    fi

    # Extract test metrics
    log_info "Step 1: Extracting test metrics..."
    local test_metrics
    test_metrics=$(extract_test_metrics "$TEST_OUTPUT")
    if [[ $? -ne 0 ]]; then
        log_failed
        return 2
    fi

    # Extract coverage metrics
    log_info "Step 2: Extracting coverage metrics..."
    local coverage_metrics
    coverage_metrics=$(extract_coverage_metrics)

    # Combine metrics
    log_info "Step 3: Combining metrics..."
    local final_metrics
    final_metrics=$(combine_metrics "$test_metrics" "$coverage_metrics")

    # Write output file
    log_info "Step 4: Writing metrics to: $OUTPUT_FILE"
    local output_path
    if [[ "$OUTPUT_FILE" == /* ]]; then
        # Absolute path
        output_path="$OUTPUT_FILE"
    else
        # Relative path from project root
        output_path="$SCRIPT_DIR/../$OUTPUT_FILE"
    fi
    echo "$final_metrics" > "$output_path"
    log_success "Metrics written to: $output_path"

    # Output summary
    output_metrics_summary "$final_metrics"

    # Output JSON if requested
    if [[ "$JSON_MODE" == "true" ]]; then
        echo "$final_metrics"
    fi

    log_accomplished
    return 0
}

# Run main function
main "$@"
