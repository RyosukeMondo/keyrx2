#!/usr/bin/env bats
# Integration tests for WASM build, verification, and health check scripts
#
# Tests:
# - build-wasm.sh produces valid WASM output
# - verify-wasm.sh detects integrity issues
# - wasm-health.sh runs successfully and reports status

# Get the directory where the tests are located
BATS_TEST_DIRNAME="$(cd "$(dirname "$BATS_TEST_FILENAME")" && pwd)"
PROJECT_ROOT="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"
BUILD_WASM_SCRIPT="$PROJECT_ROOT/scripts/lib/build-wasm.sh"
VERIFY_WASM_SCRIPT="$PROJECT_ROOT/scripts/verify-wasm.sh"
HEALTH_SCRIPT="$PROJECT_ROOT/scripts/wasm-health.sh"

# WASM output locations
WASM_OUTPUT_DIR="$PROJECT_ROOT/keyrx_ui/src/wasm/pkg"
WASM_FILE="$WASM_OUTPUT_DIR/keyrx_core_bg.wasm"
MANIFEST_FILE="$WASM_OUTPUT_DIR/wasm-manifest.json"

# Setup function - runs before each test
setup() {
    # Create a temporary directory for test artifacts
    TEST_TEMP_DIR="$(mktemp -d)"
    export TEST_LOG_FILE="$TEST_TEMP_DIR/test.log"

    # Save current WASM state for restoration
    BACKUP_DIR="$TEST_TEMP_DIR/wasm_backup"
    if [[ -d "$WASM_OUTPUT_DIR" ]]; then
        mkdir -p "$BACKUP_DIR"
        cp -r "$WASM_OUTPUT_DIR" "$BACKUP_DIR/" 2>/dev/null || true
    fi
}

# Teardown function - runs after each test
teardown() {
    # Restore WASM state
    if [[ -d "$BACKUP_DIR/pkg" ]]; then
        rm -rf "$WASM_OUTPUT_DIR"
        cp -r "$BACKUP_DIR/pkg" "$WASM_OUTPUT_DIR"
    fi

    # Clean up temporary directory
    if [[ -n "$TEST_TEMP_DIR" ]] && [[ -d "$TEST_TEMP_DIR" ]]; then
        rm -rf "$TEST_TEMP_DIR"
    fi
}

# Test: build-wasm.sh exists and is executable
@test "build-wasm.sh exists and is executable" {
    [[ -f "$BUILD_WASM_SCRIPT" ]]
    [[ -x "$BUILD_WASM_SCRIPT" ]]
}

# Test: verify-wasm.sh exists and is executable
@test "verify-wasm.sh exists and is executable" {
    [[ -f "$VERIFY_WASM_SCRIPT" ]]
    [[ -x "$VERIFY_WASM_SCRIPT" ]]
}

# Test: wasm-health.sh exists and is executable
@test "wasm-health.sh exists and is executable" {
    [[ -f "$HEALTH_SCRIPT" ]]
    [[ -x "$HEALTH_SCRIPT" ]]
}

# Test: build-wasm.sh produces WASM output
@test "build-wasm.sh successfully builds WASM module" {
    skip "Skipping slow build test - requires wasm-pack and full build"

    # Clean output directory
    rm -rf "$WASM_OUTPUT_DIR"

    # Run build script
    run "$BUILD_WASM_SCRIPT" --quiet --log-file "$TEST_LOG_FILE"
    echo "Exit code: $status"
    echo "Output: $output"

    # Should succeed
    [[ "$status" -eq 0 ]]
    [[ "$output" =~ "=== accomplished ===" ]]

    # Verify output files exist
    [[ -f "$WASM_FILE" ]]
    [[ -f "$MANIFEST_FILE" ]]

    # Verify WASM file is not empty and has reasonable size (> 100KB)
    [[ -s "$WASM_FILE" ]]
    WASM_SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
    [[ $WASM_SIZE -gt 102400 ]]
}

# Test: build-wasm.sh creates manifest with hash
@test "build-wasm.sh generates manifest with hash" {
    skip "Skipping slow build test - requires wasm-pack and full build"

    # Clean output directory
    rm -rf "$WASM_OUTPUT_DIR"

    # Run build script
    run "$BUILD_WASM_SCRIPT" --quiet --log-file "$TEST_LOG_FILE"
    [[ "$status" -eq 0 ]]

    # Verify manifest exists and is valid JSON
    [[ -f "$MANIFEST_FILE" ]]
    jq empty "$MANIFEST_FILE"

    # Verify manifest has required fields
    jq -e '.hash' "$MANIFEST_FILE" > /dev/null
    jq -e '.size' "$MANIFEST_FILE" > /dev/null
    jq -e '.timestamp' "$MANIFEST_FILE" > /dev/null
    jq -e '.version' "$MANIFEST_FILE" > /dev/null

    # Verify hash is not empty
    HASH=$(jq -r '.hash' "$MANIFEST_FILE")
    [[ -n "$HASH" ]]
    [[ "$HASH" != "null" ]]
}

# Test: verify-wasm.sh passes with valid WASM
@test "verify-wasm.sh succeeds when WASM is valid" {
    # Only run if WASM already exists (from previous build)
    if [[ ! -f "$WASM_FILE" ]] || [[ ! -f "$MANIFEST_FILE" ]]; then
        skip "WASM not built yet - run build-wasm.sh first"
    fi

    # Run verification
    run "$VERIFY_WASM_SCRIPT" --quiet
    echo "Exit code: $status"
    echo "Output: $output"

    # Should succeed
    [[ "$status" -eq 0 ]]
    [[ "$output" =~ "=== accomplished ===" ]]
}

# Test: verify-wasm.sh detects missing WASM file
@test "verify-wasm.sh fails when WASM file is missing" {
    # Only test if we have a backup to restore
    if [[ ! -d "$BACKUP_DIR/pkg" ]]; then
        skip "No WASM backup to work with"
    fi

    # Delete WASM file but keep manifest
    rm -f "$WASM_FILE"

    # Run verification
    run "$VERIFY_WASM_SCRIPT" --quiet
    echo "Exit code: $status"
    echo "Output: $output"

    # Should fail
    [[ "$status" -ne 0 ]]
    [[ "$output" =~ "failed" ]]
}

# Test: verify-wasm.sh detects missing manifest
@test "verify-wasm.sh fails when manifest is missing" {
    # Only test if we have a backup to restore
    if [[ ! -d "$BACKUP_DIR/pkg" ]]; then
        skip "No WASM backup to work with"
    fi

    # Delete manifest but keep WASM file
    rm -f "$MANIFEST_FILE"

    # Run verification
    run "$VERIFY_WASM_SCRIPT" --quiet
    echo "Exit code: $status"
    echo "Output: $output"

    # Should fail
    [[ "$status" -ne 0 ]]
    [[ "$output" =~ "failed" ]]
}

# Test: verify-wasm.sh detects hash mismatch
@test "verify-wasm.sh fails when WASM file hash doesn't match manifest" {
    # Only test if we have a backup to restore
    if [[ ! -d "$BACKUP_DIR/pkg" ]]; then
        skip "No WASM backup to work with"
    fi

    # Modify WASM file to corrupt hash
    echo "corrupted" >> "$WASM_FILE"

    # Run verification
    run "$VERIFY_WASM_SCRIPT" --quiet
    echo "Exit code: $status"
    echo "Output: $output"

    # Should fail with hash mismatch
    [[ "$status" -ne 0 ]]
    [[ "$output" =~ "failed" ]]
}

# Test: verify-wasm.sh supports JSON output
@test "verify-wasm.sh --json produces valid JSON" {
    # Only run if WASM already exists
    if [[ ! -f "$WASM_FILE" ]] || [[ ! -f "$MANIFEST_FILE" ]]; then
        skip "WASM not built yet - run build-wasm.sh first"
    fi

    # Run verification with JSON output
    run "$VERIFY_WASM_SCRIPT" --json --quiet
    echo "Exit code: $status"
    echo "Output: $output"

    # Extract JSON output (last line that looks like JSON)
    json_output=$(echo "$output" | grep -o '{.*}' | tail -n1)
    echo "JSON: $json_output"

    # Validate JSON structure
    echo "$json_output" | jq -e '.status' > /dev/null

    # Check status value
    [[ "$(echo "$json_output" | jq -r '.status')" == "success" ]]
}

# Test: wasm-health.sh runs without error
@test "wasm-health.sh executes successfully" {
    # Run health check
    run "$HEALTH_SCRIPT" --quiet
    echo "Exit code: $status"
    echo "Output: $output"

    # Should complete (exit 0 if healthy, exit 1 if issues)
    # We accept either since environment may not be fully set up
    [[ "$status" -eq 0 ]] || [[ "$status" -eq 1 ]]

    # Should output status marker
    [[ "$output" =~ "=== accomplished ===" ]] || [[ "$output" =~ "=== failed ===" ]]
}

# Test: wasm-health.sh checks for wasm-pack
@test "wasm-health.sh reports wasm-pack status" {
    # Run health check with full output
    run "$HEALTH_SCRIPT"
    echo "Exit code: $status"
    echo "Output: $output"

    # Should mention wasm-pack
    [[ "$output" =~ "wasm-pack" ]]
}

# Test: wasm-health.sh checks for WASM target
@test "wasm-health.sh reports wasm32-target status" {
    # Run health check with full output
    run "$HEALTH_SCRIPT"
    echo "Exit code: $status"
    echo "Output: $output"

    # Should mention wasm32-target
    [[ "$output" =~ "wasm32-target" ]]
}

# Test: wasm-health.sh checks for keyrx_core config
@test "wasm-health.sh reports keyrx_core configuration status" {
    # Run health check with full output
    run "$HEALTH_SCRIPT"
    echo "Exit code: $status"
    echo "Output: $output"

    # Should mention keyrx_core config
    [[ "$output" =~ "keyrx_core-config" ]]
}

# Test: wasm-health.sh supports JSON output
@test "wasm-health.sh --json produces valid JSON" {
    # Run health check with JSON output
    run "$HEALTH_SCRIPT" --json --quiet
    echo "Exit code: $status"
    echo "Output: $output"

    # Extract JSON output (last line that looks like JSON)
    json_output=$(echo "$output" | grep -o '{.*}' | tail -n1)
    echo "JSON: $json_output"

    # Validate JSON structure
    echo "$json_output" | jq -e '.status' > /dev/null
    echo "$json_output" | jq -e '.checks_total' > /dev/null
    echo "$json_output" | jq -e '.checks_passed' > /dev/null

    # Check that status is either "ok" or "failed"
    STATUS=$(echo "$json_output" | jq -r '.status')
    [[ "$STATUS" == "ok" ]] || [[ "$STATUS" == "failed" ]]
}

# Test: wasm-health.sh reports summary statistics
@test "wasm-health.sh includes summary with check counts" {
    # Run health check
    run "$HEALTH_SCRIPT"
    echo "Exit code: $status"
    echo "Output: $output"

    # Should include summary with totals
    [[ "$output" =~ "Total checks:" ]]
    [[ "$output" =~ "Passed:" ]]
    [[ "$output" =~ "Failed:" ]]
}
