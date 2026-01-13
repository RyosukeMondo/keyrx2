#!/bin/bash
# Verify WASM build integrity and version matching
#
# Purpose:
#   Validates WASM build artifacts and ensures version consistency
#   between keyrx_core, WASM module, and manifest
#
# Checks performed:
#   1. WASM manifest exists and is valid JSON
#   2. WASM file hash matches manifest
#   3. keyrx_core version matches WASM package version
#   4. wasm-bindgen version compatibility
#
# Usage:
#   ./scripts/verify-wasm.sh [--quiet] [--json] [--log-file PATH]
#
# Exit codes:
#   0 - All checks passed
#   1 - Verification failed
#   2 - Missing files or dependencies

set -euo pipefail

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Store the project root directory
PROJECT_ROOT="$(pwd)"

# Parse command line arguments
parse_common_flags "$@"
setup_log_file "verify_wasm"

# Convert LOG_FILE to absolute path if it's relative
if [[ -n "$LOG_FILE" ]] && [[ ! "$LOG_FILE" = /* ]]; then
    LOG_FILE="$PROJECT_ROOT/$LOG_FILE"
fi

log_info "Verifying WASM build integrity..."
separator

# Define paths
OUTPUT_DIR="$PROJECT_ROOT/keyrx_ui/src/wasm/pkg"
MANIFEST_FILE="$OUTPUT_DIR/wasm-manifest.json"
WASM_FILE="$OUTPUT_DIR/keyrx_core_bg.wasm"
PACKAGE_JSON="$OUTPUT_DIR/package.json"
WORKSPACE_TOML="$PROJECT_ROOT/Cargo.toml"

# Track verification status
VERIFICATION_FAILED=false
ISSUES=()

# Check 1: Verify manifest file exists
log_info "Check 1: Verifying manifest file exists..."
if [[ ! -f "$MANIFEST_FILE" ]]; then
    log_error "WASM manifest not found: $MANIFEST_FILE"
    log_error "Run 'npm run build:wasm' to build WASM module"
    VERIFICATION_FAILED=true
    ISSUES+=("manifest_missing")
else
    log_success "Manifest found"
fi

# Check 2: Verify manifest is valid JSON
if [[ "$VERIFICATION_FAILED" == "false" ]]; then
    log_info "Check 2: Verifying manifest is valid JSON..."
    if ! jq empty "$MANIFEST_FILE" 2>/dev/null; then
        log_error "Manifest is not valid JSON: $MANIFEST_FILE"
        VERIFICATION_FAILED=true
        ISSUES+=("manifest_invalid_json")
    else
        log_success "Manifest is valid JSON"
    fi
fi

# Check 3: Verify WASM file exists
if [[ "$VERIFICATION_FAILED" == "false" ]]; then
    log_info "Check 3: Verifying WASM file exists..."
    if [[ ! -f "$WASM_FILE" ]]; then
        log_error "WASM file not found: $WASM_FILE"
        log_error "Run 'npm run build:wasm' to build WASM module"
        VERIFICATION_FAILED=true
        ISSUES+=("wasm_file_missing")
    else
        log_success "WASM file found"
    fi
fi

# Check 4: Verify WASM file hash matches manifest
if [[ "$VERIFICATION_FAILED" == "false" ]]; then
    log_info "Check 4: Verifying WASM file hash..."

    # Extract hash from manifest
    MANIFEST_HASH=$(jq -r '.hash' "$MANIFEST_FILE")

    if [[ -z "$MANIFEST_HASH" ]] || [[ "$MANIFEST_HASH" == "null" ]]; then
        log_error "No hash found in manifest"
        VERIFICATION_FAILED=true
        ISSUES+=("manifest_no_hash")
    else
        log_debug "Manifest hash: $MANIFEST_HASH"

        # Calculate current WASM file hash
        if command -v sha256sum >/dev/null 2>&1; then
            CURRENT_HASH=$(sha256sum "$WASM_FILE" | awk '{print $1}')
        elif command -v shasum >/dev/null 2>&1; then
            CURRENT_HASH=$(shasum -a 256 "$WASM_FILE" | awk '{print $1}')
        else
            log_error "Neither sha256sum nor shasum found - cannot verify hash"
            VERIFICATION_FAILED=true
            ISSUES+=("hash_tool_missing")
            CURRENT_HASH=""
        fi

        if [[ -n "$CURRENT_HASH" ]]; then
            log_debug "Current hash:  $CURRENT_HASH"

            if [[ "$MANIFEST_HASH" != "$CURRENT_HASH" ]]; then
                log_error "WASM file hash mismatch!"
                log_error "  Expected: $MANIFEST_HASH"
                log_error "  Actual:   $CURRENT_HASH"
                log_error "WASM file may have been modified after build"
                log_error "Run 'npm run rebuild:wasm' to rebuild"
                VERIFICATION_FAILED=true
                ISSUES+=("hash_mismatch")
            else
                log_success "Hash verified"
            fi
        fi
    fi
fi

# Check 5: Verify package.json exists
if [[ "$VERIFICATION_FAILED" == "false" ]]; then
    log_info "Check 5: Verifying package.json exists..."
    if [[ ! -f "$PACKAGE_JSON" ]]; then
        log_error "WASM package.json not found: $PACKAGE_JSON"
        VERIFICATION_FAILED=true
        ISSUES+=("package_json_missing")
    else
        log_success "package.json found"
    fi
fi

# Check 6: Verify version matching
if [[ "$VERIFICATION_FAILED" == "false" ]]; then
    log_info "Check 6: Verifying version matching..."

    # Extract version from workspace Cargo.toml
    if [[ ! -f "$WORKSPACE_TOML" ]]; then
        log_error "Workspace Cargo.toml not found: $WORKSPACE_TOML"
        VERIFICATION_FAILED=true
        ISSUES+=("workspace_toml_missing")
    else
        CARGO_VERSION=$(grep -E '^\s*version\s*=' "$WORKSPACE_TOML" | head -1 | sed -E 's/.*version\s*=\s*"([^"]+)".*/\1/')
        log_debug "Cargo workspace version: $CARGO_VERSION"

        # Extract version from WASM manifest
        MANIFEST_VERSION=$(jq -r '.version' "$MANIFEST_FILE")
        log_debug "Manifest version: $MANIFEST_VERSION"

        # Extract version from WASM package.json
        PACKAGE_VERSION=$(jq -r '.version' "$PACKAGE_JSON")
        log_debug "Package version: $PACKAGE_VERSION"

        # Compare versions
        if [[ "$CARGO_VERSION" != "$MANIFEST_VERSION" ]]; then
            log_error "Version mismatch: Cargo.toml ($CARGO_VERSION) != manifest ($MANIFEST_VERSION)"
            VERIFICATION_FAILED=true
            ISSUES+=("version_mismatch_cargo_manifest")
        fi

        if [[ "$CARGO_VERSION" != "$PACKAGE_VERSION" ]]; then
            log_error "Version mismatch: Cargo.toml ($CARGO_VERSION) != package.json ($PACKAGE_VERSION)"
            VERIFICATION_FAILED=true
            ISSUES+=("version_mismatch_cargo_package")
        fi

        if [[ "$MANIFEST_VERSION" != "$PACKAGE_VERSION" ]]; then
            log_error "Version mismatch: manifest ($MANIFEST_VERSION) != package.json ($PACKAGE_VERSION)"
            VERIFICATION_FAILED=true
            ISSUES+=("version_mismatch_manifest_package")
        fi

        if [[ "$VERIFICATION_FAILED" == "false" ]]; then
            log_success "All versions match: $CARGO_VERSION"
        fi
    fi
fi

# Check 7: Verify wasm-bindgen version compatibility
if [[ "$VERIFICATION_FAILED" == "false" ]]; then
    log_info "Check 7: Verifying wasm-bindgen compatibility..."

    # Check if wasm-bindgen CLI is installed
    if ! command -v wasm-bindgen >/dev/null 2>&1; then
        log_warn "wasm-bindgen CLI not found - skipping version check"
        log_warn "Install with: cargo install wasm-bindgen-cli"
    else
        WASM_BINDGEN_CLI_VERSION=$(wasm-bindgen --version | awk '{print $2}')
        log_debug "wasm-bindgen CLI version: $WASM_BINDGEN_CLI_VERSION"

        # Extract wasm-bindgen dependency version from workspace Cargo.toml
        WASM_BINDGEN_DEP_VERSION=$(grep -E '^\s*wasm-bindgen\s*=' "$WORKSPACE_TOML" | sed -E 's/.*version\s*=\s*"([^"]+)".*/\1/' | head -1)

        if [[ -z "$WASM_BINDGEN_DEP_VERSION" ]]; then
            log_warn "Could not extract wasm-bindgen dependency version"
        else
            log_debug "wasm-bindgen dependency version: $WASM_BINDGEN_DEP_VERSION"

            # Check if major versions match
            CLI_MAJOR=$(echo "$WASM_BINDGEN_CLI_VERSION" | cut -d. -f1)
            DEP_MAJOR=$(echo "$WASM_BINDGEN_DEP_VERSION" | cut -d. -f1)

            if [[ "$CLI_MAJOR" != "$DEP_MAJOR" ]]; then
                log_error "wasm-bindgen version mismatch!"
                log_error "  CLI version:        $WASM_BINDGEN_CLI_VERSION (major: $CLI_MAJOR)"
                log_error "  Dependency version: $WASM_BINDGEN_DEP_VERSION (major: $DEP_MAJOR)"
                log_error "Update CLI with: cargo install wasm-bindgen-cli --version $WASM_BINDGEN_DEP_VERSION"
                VERIFICATION_FAILED=true
                ISSUES+=("wasm_bindgen_version_mismatch")
            else
                log_success "wasm-bindgen versions compatible"
            fi
        fi
    fi
fi

separator

# Final result
if [[ "$VERIFICATION_FAILED" == "true" ]]; then
    log_error "WASM verification FAILED"
    log_error "Issues found: ${#ISSUES[@]}"
    for issue in "${ISSUES[@]}"; do
        log_error "  - $issue"
    done
    log_failed

    # Output JSON if requested
    if [[ "$JSON_MODE" == "true" ]]; then
        # Build issues array
        ISSUES_JSON="["
        for i in "${!ISSUES[@]}"; do
            if [[ $i -gt 0 ]]; then
                ISSUES_JSON+=","
            fi
            ISSUES_JSON+="\"${ISSUES[$i]}\""
        done
        ISSUES_JSON+="]"

        echo "{\"status\":\"failed\",\"issues\":$ISSUES_JSON,\"issues_count\":${#ISSUES[@]}}"
    fi

    exit 1
else
    log_success "WASM verification PASSED"
    log_accomplished

    # Output JSON if requested
    if [[ "$JSON_MODE" == "true" ]]; then
        output_json \
            "status" "success" \
            "version" "$CARGO_VERSION" \
            "hash" "$MANIFEST_HASH" \
            "wasm_file" "$WASM_FILE" \
            "manifest_file" "$MANIFEST_FILE"
    fi

    exit 0
fi
