#!/usr/bin/env bash
# UAT.sh - User Acceptance Test script for keyrx daemon
#
# Usage: ./scripts/UAT.sh
#
# This script manages the keyrx daemon for testing:
# - If daemon is running: stops it
# - If daemon is not running: compiles layout and starts daemon

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LAYOUT_RHAI="$PROJECT_ROOT/examples/user_layout.rhai"
LAYOUT_KRX="/tmp/user_layout_uat.krx"
DAEMON_NAME="keyrx_daemon"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if daemon is running
check_daemon_running() {
    pgrep -f "$DAEMON_NAME" > /dev/null 2>&1
}

# Stop daemon
stop_daemon() {
    log_info "Stopping keyrx daemon..."
    sudo pkill -f "$DAEMON_NAME" || true
    sleep 1

    if check_daemon_running; then
        log_warn "Daemon still running, force killing..."
        sudo pkill -9 -f "$DAEMON_NAME" || true
        sleep 1
    fi

    if check_daemon_running; then
        log_error "Failed to stop daemon"
        exit 1
    fi

    log_info "Daemon stopped successfully"
}

# Compile layout
compile_layout() {
    log_info "Compiling $LAYOUT_RHAI..."

    if [[ ! -f "$LAYOUT_RHAI" ]]; then
        log_error "Layout file not found: $LAYOUT_RHAI"
        exit 1
    fi

    cargo run -p keyrx_compiler --quiet -- compile "$LAYOUT_RHAI" -o "$LAYOUT_KRX"

    if [[ ! -f "$LAYOUT_KRX" ]]; then
        log_error "Compilation failed"
        exit 1
    fi

    log_info "Compiled to $LAYOUT_KRX"
}

# Start daemon
start_daemon() {
    log_info "Starting keyrx daemon..."
    log_info "Config: $LAYOUT_KRX"
    echo ""
    echo -e "${YELLOW}Press Ctrl+C to stop the daemon${NC}"
    echo ""

    # Run daemon with sudo (needs linux feature)
    sudo "$PROJECT_ROOT/target/debug/keyrx_daemon" run --config "$LAYOUT_KRX"
}

# Build daemon with linux feature
build_daemon() {
    log_info "Building keyrx_daemon with linux feature..."
    cargo build -p keyrx_daemon --features linux --quiet
}

# Main
main() {
    cd "$PROJECT_ROOT"

    echo "========================================"
    echo "  KeyRx UAT (User Acceptance Test)"
    echo "========================================"
    echo ""

    if check_daemon_running; then
        log_warn "Daemon is currently running"
        stop_daemon
        echo ""
        log_info "Daemon stopped. Run this script again to start."
    else
        log_info "Daemon is not running"

        # Build daemon binary
        build_daemon

        # Compile layout
        compile_layout

        # Start daemon
        start_daemon
    fi
}

main "$@"
