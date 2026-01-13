#!/bin/bash
# WASM Health Check - Diagnostics for WASM build environment
#
# Purpose:
#   Performs comprehensive health checks on the WASM build environment
#   and build artifacts to diagnose issues quickly
#
# Checks performed:
#   1. wasm-pack installation and version
#   2. Rust WASM target (wasm32-unknown-unknown) installed
#   3. keyrx_core Cargo.toml configuration (cdylib)
#   4. WASM build artifacts exist
#   5. wasm-bindgen CLI installation
#
# Usage:
#   ./scripts/wasm-health.sh [--quiet] [--json] [--log-file PATH]
#
# Exit codes:
#   0 - All checks passed (WASM environment healthy)
#   1 - One or more checks failed
#   2 - Critical tool missing (cannot perform checks)

set -euo pipefail

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Store the project root directory
PROJECT_ROOT="$(pwd)"

# Parse command line arguments
parse_common_flags "$@"
setup_log_file "wasm_health"

# Convert LOG_FILE to absolute path if it's relative
if [[ -n "$LOG_FILE" ]] && [[ ! "$LOG_FILE" = /* ]]; then
    LOG_FILE="$PROJECT_ROOT/$LOG_FILE"
fi

log_header "WASM Health Check"
log_info "Checking WASM build environment..."
separator

# Define paths
CORE_TOML="$PROJECT_ROOT/keyrx_core/Cargo.toml"
OUTPUT_DIR="$PROJECT_ROOT/keyrx_ui/src/wasm/pkg"
WASM_FILE="$OUTPUT_DIR/keyrx_core_bg.wasm"
MANIFEST_FILE="$OUTPUT_DIR/wasm-manifest.json"

# Track health status
HEALTH_OK=true
FAILED_CHECKS=()
WARNINGS=()
CHECKS_PASSED=0
TOTAL_CHECKS=0

# Helper function to record check result
record_check() {
    local check_name="$1"
    local status="$2"  # "pass", "fail", or "warn"
    local message="$3"

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    if [[ "$status" == "pass" ]]; then
        log_success "✓ $check_name: $message"
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    elif [[ "$status" == "fail" ]]; then
        log_error "✗ $check_name: $message"
        FAILED_CHECKS+=("$check_name")
        HEALTH_OK=false
    elif [[ "$status" == "warn" ]]; then
        log_warn "⚠ $check_name: $message"
        WARNINGS+=("$check_name")
    fi
}

# Check 1: wasm-pack installation
log_info "Check 1: wasm-pack installation..."
if command -v wasm-pack >/dev/null 2>&1; then
    WASM_PACK_VERSION=$(wasm-pack --version | awk '{print $2}')
    record_check "wasm-pack" "pass" "Installed (version $WASM_PACK_VERSION)"
else
    record_check "wasm-pack" "fail" "Not installed"
    log_error "  Fix: cargo install wasm-pack"
fi

# Check 2: Rust WASM target installed
log_info "Check 2: Rust WASM target..."
if command -v rustup >/dev/null 2>&1; then
    if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
        record_check "wasm32-target" "pass" "wasm32-unknown-unknown installed"
    else
        record_check "wasm32-target" "fail" "wasm32-unknown-unknown not installed"
        log_error "  Fix: rustup target add wasm32-unknown-unknown"
    fi
else
    record_check "wasm32-target" "fail" "rustup not found (cannot check target)"
    log_error "  Fix: Install rustup from https://rustup.rs"
fi

# Check 3: keyrx_core Cargo.toml configuration
log_info "Check 3: keyrx_core configuration..."
if [[ ! -f "$CORE_TOML" ]]; then
    record_check "keyrx_core-config" "fail" "Cargo.toml not found at $CORE_TOML"
else
    # Check for cdylib crate-type
    if grep -q 'crate-type.*=.*\[.*"cdylib"' "$CORE_TOML"; then
        record_check "keyrx_core-config" "pass" "cdylib crate-type configured"
    else
        record_check "keyrx_core-config" "fail" "cdylib crate-type not found in Cargo.toml"
        log_error "  Fix: Add crate-type = [\"cdylib\", \"rlib\"] to [lib] section"
    fi
fi

# Check 4: wasm-bindgen dependency
log_info "Check 4: wasm-bindgen dependency..."
if [[ -f "$CORE_TOML" ]]; then
    if grep -q "wasm-bindgen" "$CORE_TOML"; then
        WASM_BINDGEN_VERSION=$(grep "wasm-bindgen" "$CORE_TOML" | grep -o '"[^"]*"' | head -1 | tr -d '"')
        record_check "wasm-bindgen-dep" "pass" "Configured (version $WASM_BINDGEN_VERSION)"
    else
        record_check "wasm-bindgen-dep" "fail" "wasm-bindgen dependency not found"
        log_error "  Fix: Add wasm-bindgen to [dependencies] in keyrx_core/Cargo.toml"
    fi
fi

# Check 5: wasm-bindgen CLI installation
log_info "Check 5: wasm-bindgen CLI..."
if command -v wasm-bindgen >/dev/null 2>&1; then
    WASM_BINDGEN_CLI_VERSION=$(wasm-bindgen --version | awk '{print $2}')
    record_check "wasm-bindgen-cli" "pass" "Installed (version $WASM_BINDGEN_CLI_VERSION)"

    # Check version compatibility if we have both CLI and dependency
    if [[ -n "${WASM_BINDGEN_VERSION:-}" ]]; then
        CLI_MAJOR=$(echo "$WASM_BINDGEN_CLI_VERSION" | cut -d. -f1)
        DEP_MAJOR=$(echo "$WASM_BINDGEN_VERSION" | cut -d. -f1)

        if [[ "$CLI_MAJOR" != "$DEP_MAJOR" ]]; then
            record_check "wasm-bindgen-compat" "warn" "Version mismatch (CLI: $WASM_BINDGEN_CLI_VERSION, Dep: $WASM_BINDGEN_VERSION)"
            log_warn "  Recommend: cargo install wasm-bindgen-cli --version $WASM_BINDGEN_VERSION"
        else
            record_check "wasm-bindgen-compat" "pass" "CLI and dependency versions compatible"
        fi
    fi
else
    record_check "wasm-bindgen-cli" "warn" "Not installed (optional but recommended)"
    log_warn "  Install: cargo install wasm-bindgen-cli"
fi

# Check 6: WASM build artifacts
log_info "Check 6: WASM build artifacts..."
if [[ -d "$OUTPUT_DIR" ]]; then
    if [[ -f "$WASM_FILE" ]]; then
        WASM_SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null || echo "0")
        WASM_SIZE_KB=$((WASM_SIZE / 1024))

        if [[ $WASM_SIZE -gt 102400 ]]; then  # > 100KB
            record_check "wasm-artifacts" "pass" "WASM file exists (${WASM_SIZE_KB}KB)"
        else
            record_check "wasm-artifacts" "warn" "WASM file exists but seems too small (${WASM_SIZE_KB}KB)"
            log_warn "  Recommend: npm run rebuild:wasm"
        fi

        # Check manifest
        if [[ -f "$MANIFEST_FILE" ]]; then
            if jq empty "$MANIFEST_FILE" 2>/dev/null; then
                record_check "wasm-manifest" "pass" "Manifest exists and is valid JSON"
            else
                record_check "wasm-manifest" "warn" "Manifest exists but is invalid JSON"
                log_warn "  Fix: npm run rebuild:wasm"
            fi
        else
            record_check "wasm-manifest" "warn" "Manifest not found"
            log_warn "  Fix: npm run build:wasm"
        fi
    else
        record_check "wasm-artifacts" "fail" "WASM file not found at $WASM_FILE"
        log_error "  Fix: npm run build:wasm"
    fi
else
    record_check "wasm-artifacts" "fail" "Output directory not found: $OUTPUT_DIR"
    log_error "  Fix: npm run build:wasm"
fi

separator

# Summary
log_info "Health Check Summary:"
log_info "  Total checks: $TOTAL_CHECKS"
log_info "  Passed: $CHECKS_PASSED"
log_info "  Failed: ${#FAILED_CHECKS[@]}"
log_info "  Warnings: ${#WARNINGS[@]}"

separator

# Final result
if [[ "$HEALTH_OK" == "true" ]]; then
    if [[ ${#WARNINGS[@]} -eq 0 ]]; then
        log_success "WASM Health: OK ✓"
        log_success "All checks passed - WASM environment is healthy"
        log_accomplished
    else
        log_success "WASM Health: OK (with warnings)"
        log_warn "Warnings detected (non-critical):"
        for warning in "${WARNINGS[@]}"; do
            log_warn "  - $warning"
        done
        log_accomplished
    fi

    # Output JSON if requested
    if [[ "$JSON_MODE" == "true" ]]; then
        # Build warnings array
        WARNINGS_JSON="["
        for i in "${!WARNINGS[@]}"; do
            if [[ $i -gt 0 ]]; then
                WARNINGS_JSON+=","
            fi
            WARNINGS_JSON+="\"${WARNINGS[$i]}\""
        done
        WARNINGS_JSON+="]"

        output_json \
            "status" "ok" \
            "checks_total" "$TOTAL_CHECKS" \
            "checks_passed" "$CHECKS_PASSED" \
            "warnings_count" "${#WARNINGS[@]}" \
            "warnings" "$WARNINGS_JSON"
    fi

    exit 0
else
    log_error "WASM Health: FAILED ✗"
    log_error "Failed checks:"
    for check in "${FAILED_CHECKS[@]}"; do
        log_error "  - $check"
    done

    if [[ ${#WARNINGS[@]} -gt 0 ]]; then
        log_warn "Warnings:"
        for warning in "${WARNINGS[@]}"; do
            log_warn "  - $warning"
        done
    fi

    log_failed

    # Output JSON if requested
    if [[ "$JSON_MODE" == "true" ]]; then
        # Build failed checks array
        FAILED_JSON="["
        for i in "${!FAILED_CHECKS[@]}"; do
            if [[ $i -gt 0 ]]; then
                FAILED_JSON+=","
            fi
            FAILED_JSON+="\"${FAILED_CHECKS[$i]}\""
        done
        FAILED_JSON+="]"

        # Build warnings array
        WARNINGS_JSON="["
        for i in "${!WARNINGS[@]}"; do
            if [[ $i -gt 0 ]]; then
                WARNINGS_JSON+=","
            fi
            WARNINGS_JSON+="\"${WARNINGS[$i]}\""
        done
        WARNINGS_JSON+="]"

        echo "{\"status\":\"failed\",\"checks_total\":$TOTAL_CHECKS,\"checks_passed\":$CHECKS_PASSED,\"failed_count\":${#FAILED_CHECKS[@]},\"failed_checks\":$FAILED_JSON,\"warnings_count\":${#WARNINGS[@]},\"warnings\":$WARNINGS_JSON}"
    fi

    exit 1
fi
