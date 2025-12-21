#!/bin/bash
# Build script for keyrx workspace
# Supports: --release, --watch, --error, --json, --quiet, --log-file

# Get script directory for sourcing common.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common utilities
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Script-specific variables
RELEASE_MODE=false
WATCH_MODE=false

# Usage information
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Build the keyrx workspace.

OPTIONS:
    --release       Build in release mode (optimized)
    --watch         Watch mode - rebuild on file changes (requires cargo-watch)
    --error         Show only errors
    --json          Output results in JSON format
    --quiet         Suppress non-error output
    --log-file PATH Specify custom log file path
    -h, --help      Show this help message

EXAMPLES:
    $(basename "$0")                    # Debug build
    $(basename "$0") --release          # Release build
    $(basename "$0") --watch            # Watch mode (debug)
    $(basename "$0") --watch --release  # Watch mode (release)
    $(basename "$0") --json             # JSON output

EXIT CODES:
    0 - Build succeeded
    1 - Build failed
    2 - Missing required tool

OUTPUT MARKERS:
    === accomplished === - Build succeeded
    === failed ===       - Build failed
EOF
}

# Parse arguments
parse_args() {
    # Parse common flags first
    parse_common_flags "$@"

    # Parse script-specific flags from remaining args
    set -- "${REMAINING_ARGS[@]}"

    while [[ $# -gt 0 ]]; do
        case $1 in
            --release)
                RELEASE_MODE=true
                shift
                ;;
            --watch)
                WATCH_MODE=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Main build function
do_build() {
    local build_cmd="cargo build --workspace"

    if [[ "$RELEASE_MODE" == "true" ]]; then
        build_cmd="$build_cmd --release"
        log_info "Building in release mode..."
    else
        log_info "Building in debug mode..."
    fi

    # Execute build
    if $build_cmd; then
        log_accomplished
        return 0
    else
        log_failed
        return 1
    fi
}

# Watch mode function
do_watch() {
    # Check if cargo-watch is installed
    if ! require_tool "cargo-watch" "Install cargo-watch: cargo install cargo-watch"; then
        log_failed
        return 2
    fi

    local watch_cmd="cargo watch -x 'build --workspace"

    if [[ "$RELEASE_MODE" == "true" ]]; then
        watch_cmd="$watch_cmd --release"
        log_info "Starting watch mode (release)..."
    else
        log_info "Starting watch mode (debug)..."
    fi

    watch_cmd="$watch_cmd'"

    # Execute watch (this blocks until interrupted)
    eval "$watch_cmd"

    # If we get here, watch was interrupted (Ctrl+C)
    return 0
}

# Main execution
main() {
    local exit_code=0
    local build_type="debug"
    local mode="standard"

    # Parse arguments
    parse_args "$@"

    # Setup log file if not provided via --log-file
    if [[ -z "$LOG_FILE" ]]; then
        setup_log_file "build"
    fi

    # Verify cargo is installed
    if ! require_tool "cargo" "Install Rust from https://rustup.rs"; then
        if [[ "$JSON_MODE" == "true" ]]; then
            output_json "status" "failed" "error" "cargo not found" "exit_code" "2"
        fi
        exit 2
    fi

    separator

    # Determine build type and mode
    if [[ "$RELEASE_MODE" == "true" ]]; then
        build_type="release"
    fi

    if [[ "$WATCH_MODE" == "true" ]]; then
        mode="watch"
    fi

    # Execute appropriate mode
    if [[ "$WATCH_MODE" == "true" ]]; then
        do_watch
        exit_code=$?
    else
        do_build
        exit_code=$?
    fi

    separator

    # JSON output
    if [[ "$JSON_MODE" == "true" ]]; then
        if [[ $exit_code -eq 0 ]]; then
            output_json \
                "status" "success" \
                "build_type" "$build_type" \
                "mode" "$mode" \
                "exit_code" "0"
        else
            output_json \
                "status" "failed" \
                "build_type" "$build_type" \
                "mode" "$mode" \
                "exit_code" "$exit_code"
        fi
    fi

    exit $exit_code
}

# Run main function
main "$@"
