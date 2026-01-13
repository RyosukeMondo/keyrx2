#!/bin/bash
# Coverage Trend Tracking Script
# Monitors test coverage trends, detects regressions, and generates coverage badges
#
# Usage:
#   scripts/coverage-trend.sh [options]
#
# Options:
#   --baseline-file FILE  Path to baseline coverage file (default: .coverage-baseline.json)
#   --coverage-file FILE  Path to coverage JSON report (default: keyrx_ui/coverage/coverage-summary.json)
#   --threshold PERCENT   Maximum allowed coverage drop in % (default: 2)
#   --update-baseline     Update the baseline with current coverage
#   --generate-badge      Generate shields.io badge URL
#   --json                Output results as JSON
#   --quiet               Suppress non-error output
#
# Exit codes:
#   0 - Coverage maintained or improved
#   1 - Coverage regression detected
#   2 - Missing files or configuration errors

set -euo pipefail

# Get script directory and source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export COMMON_SH_DIR="$SCRIPT_DIR/lib"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Configuration defaults
BASELINE_FILE=".coverage-baseline.json"
COVERAGE_FILE="keyrx_ui/coverage/coverage-summary.json"
REGRESSION_THRESHOLD=2.0
UPDATE_BASELINE=false
GENERATE_BADGE=false

# Parse arguments
parse_script_args() {
    # Parse common flags first
    parse_common_flags "$@"

    # Parse script-specific flags
    local args=("${REMAINING_ARGS[@]}")
    REMAINING_ARGS=()

    while [[ ${#args[@]} -gt 0 ]]; do
        case ${args[0]} in
            --baseline-file)
                if [[ ${#args[@]} -lt 2 ]]; then
                    log_error "--baseline-file requires a path argument"
                    return 1
                fi
                BASELINE_FILE="${args[1]}"
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
            --threshold)
                if [[ ${#args[@]} -lt 2 ]]; then
                    log_error "--threshold requires a numeric argument"
                    return 1
                fi
                REGRESSION_THRESHOLD="${args[1]}"
                args=("${args[@]:2}")
                ;;
            --update-baseline)
                UPDATE_BASELINE=true
                args=("${args[@]:1}")
                ;;
            --generate-badge)
                GENERATE_BADGE=true
                args=("${args[@]:1}")
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
Coverage Trend Tracking Script

Usage:
  scripts/coverage-trend.sh [options]

Options:
  --baseline-file FILE  Path to baseline coverage file (default: .coverage-baseline.json)
  --coverage-file FILE  Path to coverage JSON report (default: keyrx_ui/coverage/coverage-summary.json)
  --threshold PERCENT   Maximum allowed coverage drop in % (default: 2)
  --update-baseline     Update the baseline with current coverage
  --generate-badge      Generate shields.io badge URL
  --json                Output results as JSON
  --quiet               Suppress non-error output

Exit codes:
  0 - Coverage maintained or improved
  1 - Coverage regression detected
  2 - Missing files or configuration errors
EOF
}

# Extract coverage percentage from JSON
# Usage: extract_coverage_pct "path.to.field" < coverage.json
extract_coverage_pct() {
    local field="$1"
    # Use jq if available, otherwise use grep/sed fallback
    if command_exists jq; then
        jq -r "$field" 2>/dev/null || echo "0"
    else
        # Simple JSON parsing fallback (less robust but works for coverage-summary.json)
        grep -oP "\"pct\":\s*\K[0-9.]+" | head -1 || echo "0"
    fi
}

# Check if jq is available, recommend installation if not
check_jq() {
    if ! command_exists jq; then
        log_warn "jq not found - using basic JSON parsing (less accurate)"
        log_warn "Install jq for better JSON parsing: sudo apt-get install jq"
    fi
}

# Read current coverage from Vitest JSON output
read_current_coverage() {
    local coverage_path="$SCRIPT_DIR/../$COVERAGE_FILE"

    if [[ ! -f "$coverage_path" ]]; then
        log_error "Coverage file not found: $coverage_path"
        log_error "Run 'npm run test:coverage' in keyrx_ui directory first"
        return 2
    fi

    log_info "Reading coverage from: $coverage_path" >&2

    if command_exists jq; then
        # Use jq for accurate parsing - build JSON directly
        local lines_pct branches_pct functions_pct statements_pct timestamp
        lines_pct=$(jq -r '.total.lines.pct' "$coverage_path" 2>/dev/null || echo "0")
        branches_pct=$(jq -r '.total.branches.pct' "$coverage_path" 2>/dev/null || echo "0")
        functions_pct=$(jq -r '.total.functions.pct' "$coverage_path" 2>/dev/null || echo "0")
        statements_pct=$(jq -r '.total.statements.pct' "$coverage_path" 2>/dev/null || echo "0")
        timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

        # Build JSON object directly without helper function
        cat <<EOF
{"lines":"$lines_pct","branches":"$branches_pct","functions":"$functions_pct","statements":"$statements_pct","timestamp":"$timestamp"}
EOF
    else
        # Fallback: parse without jq (less accurate)
        log_warn "Using fallback JSON parsing - results may be inaccurate" >&2
        local pct timestamp
        pct=$(grep -oP '"lines":\s*\{[^}]*"pct":\s*\K[0-9.]+' "$coverage_path" | head -1 || echo "0")
        timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        cat <<EOF
{"lines":"$pct","branches":"0","functions":"0","statements":"0","timestamp":"$timestamp"}
EOF
    fi
}

# Read baseline coverage
read_baseline_coverage() {
    local baseline_path="$SCRIPT_DIR/../$BASELINE_FILE"

    if [[ ! -f "$baseline_path" ]]; then
        log_warn "No baseline file found: $baseline_path" >&2
        log_warn "Current coverage will become the baseline" >&2
        echo "{}"
        return 0
    fi

    log_info "Reading baseline from: $baseline_path" >&2
    cat "$baseline_path"
}

# Write baseline coverage
write_baseline_coverage() {
    local coverage_json="$1"
    local baseline_path="$SCRIPT_DIR/../$BASELINE_FILE"

    echo "$coverage_json" > "$baseline_path"
    log_success "Baseline updated: $baseline_path"
}

# Compare coverage and detect regression
# Returns: 0 if OK, 1 if regression detected
compare_coverage() {
    local current_json="$1"
    local baseline_json="$2"

    # If no baseline, current becomes baseline
    if [[ "$baseline_json" == "{}" ]]; then
        log_info "No baseline - current coverage will be the new baseline"
        return 0
    fi

    # Extract values
    local current_lines baseline_lines
    if command_exists jq; then
        current_lines=$(echo "$current_json" | jq -r '.lines')
        baseline_lines=$(echo "$baseline_json" | jq -r '.lines')
    else
        current_lines=$(echo "$current_json" | grep -oP '"lines":\s*"\K[0-9.]+' || echo "0")
        baseline_lines=$(echo "$baseline_json" | grep -oP '"lines":\s*"\K[0-9.]+' || echo "0")
    fi

    # Calculate difference
    local diff
    diff=$(awk "BEGIN {print $baseline_lines - $current_lines}")

    log_info "Coverage comparison:"
    log_info "  Baseline: ${baseline_lines}%"
    log_info "  Current:  ${current_lines}%"
    log_info "  Change:   ${diff}%"

    # Check if regression exceeds threshold
    local regression_detected=false
    if (( $(awk "BEGIN {print ($diff > $REGRESSION_THRESHOLD)}") )); then
        log_error "Coverage regression detected: -${diff}%"
        log_error "Threshold: ${REGRESSION_THRESHOLD}%"
        regression_detected=true
    else
        if (( $(awk "BEGIN {print ($diff > 0)}") )); then
            log_warn "Coverage decreased by ${diff}% (within threshold)"
        elif (( $(awk "BEGIN {print ($diff < 0)}") )); then
            log_success "Coverage improved by $(awk "BEGIN {print -1 * $diff}")%"
        else
            log_success "Coverage maintained at ${current_lines}%"
        fi
    fi

    if [[ "$regression_detected" == "true" ]]; then
        return 1
    fi

    return 0
}

# Generate shields.io badge URL
generate_badge_url() {
    local coverage_pct="$1"

    # Determine badge color based on coverage
    local color
    if (( $(awk "BEGIN {print ($coverage_pct >= 80)}") )); then
        color="brightgreen"
    elif (( $(awk "BEGIN {print ($coverage_pct >= 70)}") )); then
        color="green"
    elif (( $(awk "BEGIN {print ($coverage_pct >= 60)}") )); then
        color="yellow"
    elif (( $(awk "BEGIN {print ($coverage_pct >= 50)}") )); then
        color="orange"
    else
        color="red"
    fi

    # Round to 1 decimal place
    local rounded
    rounded=$(awk "BEGIN {printf \"%.1f\", $coverage_pct}")

    local badge_url="https://img.shields.io/badge/coverage-${rounded}%25-${color}"

    if [[ "$GENERATE_BADGE" == "true" ]]; then
        log_info "Coverage badge URL:"
        echo "$badge_url"
    fi

    echo "$badge_url"
}

# Output JSON results
output_results_json() {
    local current_json="$1"
    local baseline_json="$2"
    local regression_detected="$3"
    local badge_url="$4"

    local status="pass"
    if [[ "$regression_detected" == "true" ]]; then
        status="fail"
    fi

    json_object \
        "status" "$status" \
        "current" "$current_json" \
        "baseline" "$baseline_json" \
        "threshold" "$REGRESSION_THRESHOLD" \
        "badgeUrl" "$badge_url"
}

# Main execution
main() {
    # Setup logging
    setup_log_file "coverage-trend"

    # Parse arguments
    if ! parse_script_args "$@"; then
        return 2
    fi

    log_header "Coverage Trend Tracking"

    # Check for jq
    check_jq

    # Read current coverage
    log_info "Step 1: Reading current coverage..."
    local current_coverage
    current_coverage=$(read_current_coverage)
    if [[ $? -ne 0 ]]; then
        log_failed
        return 2
    fi

    # Extract lines coverage percentage for badge
    local current_lines_pct
    if command_exists jq; then
        current_lines_pct=$(echo "$current_coverage" | jq -r '.lines')
    else
        current_lines_pct=$(echo "$current_coverage" | grep -oP '"lines":\s*"\K[0-9.]+' || echo "0")
    fi

    # Generate badge URL
    local badge_url
    badge_url=$(generate_badge_url "$current_lines_pct")

    # Update baseline if requested
    if [[ "$UPDATE_BASELINE" == "true" ]]; then
        log_info "Step 2: Updating baseline..."
        write_baseline_coverage "$current_coverage"
        log_accomplished
        return 0
    fi

    # Read baseline
    log_info "Step 2: Reading baseline coverage..."
    local baseline_coverage
    baseline_coverage=$(read_baseline_coverage)

    # Compare coverage
    log_info "Step 3: Comparing coverage..."
    local regression_detected=false
    if ! compare_coverage "$current_coverage" "$baseline_coverage"; then
        regression_detected=true
    fi

    # Output results
    if [[ "$JSON_MODE" == "true" ]]; then
        output_results_json "$current_coverage" "$baseline_coverage" "$regression_detected" "$badge_url"
    fi

    # Final status
    separator
    if [[ "$regression_detected" == "true" ]]; then
        log_failed
        return 1
    else
        log_accomplished
        return 0
    fi
}

# Run main function
main "$@"
