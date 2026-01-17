#!/bin/bash
# uat-monitor.sh - Launch daemon for UAT and monitor logs
#
# Usage:
#   ./scripts/uat-monitor.sh              # Full UAT with log monitoring
#   ./scripts/uat-monitor.sh --release    # Release build with monitoring
#   ./scripts/uat-monitor.sh --rebuild    # Clean rebuild with monitoring

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/tmp/keyrx_daemon.log"
UAT_LOG="/tmp/keyrx_uat_$(date +%s).log"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}===================================================${NC}"
echo -e "${BLUE}  KeyRX UAT Monitor${NC}"
echo -e "${BLUE}===================================================${NC}"
echo ""

# Clear old daemon log
if [[ -f "$LOG_FILE" ]]; then
    echo -e "${YELLOW}Clearing old daemon log: $LOG_FILE${NC}"
    rm -f "$LOG_FILE"
fi

# Run UAT script and capture output
echo -e "${BLUE}Starting UAT build and launch...${NC}"
echo -e "${YELLOW}UAT output will be saved to: $UAT_LOG${NC}"
echo ""

if "$SCRIPT_DIR/uat.sh" "$@" 2>&1 | tee "$UAT_LOG"; then
    echo ""
    echo -e "${GREEN}===================================================${NC}"
    echo -e "${GREEN}  UAT Launch Successful${NC}"
    echo -e "${GREEN}===================================================${NC}"
    echo ""
    echo -e "${BLUE}Daemon logs: $LOG_FILE${NC}"
    echo -e "${BLUE}UAT logs: $UAT_LOG${NC}"
    echo ""
    echo -e "${YELLOW}Monitoring daemon logs (Ctrl+C to stop monitoring)...${NC}"
    echo -e "${BLUE}===================================================${NC}"
    echo ""

    # Wait a moment for daemon to start writing logs
    sleep 2

    # Tail the daemon log file
    if [[ -f "$LOG_FILE" ]]; then
        tail -f "$LOG_FILE"
    else
        echo -e "${YELLOW}Warning: Daemon log file not yet created. Waiting...${NC}"
        # Wait up to 10 seconds for log file to appear
        for i in {1..10}; do
            sleep 1
            if [[ -f "$LOG_FILE" ]]; then
                echo -e "${GREEN}Log file appeared. Starting monitoring...${NC}"
                tail -f "$LOG_FILE"
                break
            fi
        done
        if [[ ! -f "$LOG_FILE" ]]; then
            echo -e "${YELLOW}Error: Daemon log file never appeared at $LOG_FILE${NC}"
            echo -e "${YELLOW}Check if daemon started correctly in: $UAT_LOG${NC}"
            exit 1
        fi
    fi
else
    echo ""
    echo -e "${YELLOW}===================================================${NC}"
    echo -e "${YELLOW}  UAT Launch Failed${NC}"
    echo -e "${YELLOW}===================================================${NC}"
    echo ""
    echo -e "${YELLOW}Check the UAT log for details: $UAT_LOG${NC}"
    exit 1
fi
