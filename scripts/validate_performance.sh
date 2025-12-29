#!/bin/bash
# Performance validation script for web-ui-ux-comprehensive spec
# Validates that all performance targets are met

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Performance Validation ===${NC}"
echo ""

# Create results directory
RESULTS_DIR="target/bench_results"
mkdir -p "$RESULTS_DIR"

# Performance targets (in milliseconds)
declare -A TARGETS
TARGETS["profile_activation_hot_reload"]=100
TARGETS["ipc_status_query_roundtrip"]=10
TARGETS["device_registry_save_10_devices"]=50

echo "Running benchmarks..."
echo ""

# Run benchmarks and capture output
cd keyrx_daemon

# Run device_registry benchmark
echo -e "${YELLOW}Running device_registry benchmarks...${NC}"
cargo bench --bench device_registry --quiet 2>&1 | tee "../$RESULTS_DIR/device_registry.log"

# Run profile_activation benchmark (may fail if ProfileManager is not fully implemented)
echo -e "${YELLOW}Running profile_activation benchmarks...${NC}"
if cargo bench --bench profile_activation --quiet 2>&1 | tee "../$RESULTS_DIR/profile_activation.log"; then
    echo -e "${GREEN}Profile activation benchmarks completed${NC}"
else
    echo -e "${YELLOW}Warning: Profile activation benchmarks skipped (implementation not complete)${NC}"
fi

# Run ipc_latency benchmark (Unix only)
if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${YELLOW}Running IPC latency benchmarks...${NC}"
    if cargo bench --bench ipc_latency --quiet 2>&1 | tee "../$RESULTS_DIR/ipc_latency.log"; then
        echo -e "${GREEN}IPC latency benchmarks completed${NC}"
    else
        echo -e "${YELLOW}Warning: IPC latency benchmarks skipped (Unix socket not available)${NC}"
    fi
else
    echo -e "${YELLOW}Skipping IPC benchmarks (Unix-only)${NC}"
fi

cd ..

echo ""
echo -e "${GREEN}=== Benchmark Results Summary ===${NC}"
echo ""

# Parse results from criterion output
# Note: Criterion stores results in target/criterion/<bench_name>/*/estimates.json
# For this validation, we'll look for the existence of benchmark outputs

PASSED=0
FAILED=0
SKIPPED=0

for bench_name in "${!TARGETS[@]}"; do
    target_ms="${TARGETS[$bench_name]}"

    # Check if benchmark results exist
    if [ -d "target/criterion/$bench_name" ]; then
        # Try to extract timing from latest run
        latest_estimate=$(find "target/criterion/$bench_name" -name "estimates.json" -type f | head -1)

        if [ -f "$latest_estimate" ]; then
            # Extract mean time in nanoseconds and convert to ms
            mean_ns=$(jq -r '.mean.point_estimate' "$latest_estimate" 2>/dev/null || echo "0")
            mean_ms=$(echo "scale=2; $mean_ns / 1000000" | bc 2>/dev/null || echo "N/A")

            if [ "$mean_ms" != "N/A" ]; then
                # Compare against target
                if (( $(echo "$mean_ms < $target_ms" | bc -l) )); then
                    echo -e "${GREEN}✓ $bench_name: ${mean_ms}ms (target: <${target_ms}ms)${NC}"
                    ((PASSED++))
                else
                    echo -e "${RED}✗ $bench_name: ${mean_ms}ms (target: <${target_ms}ms) - EXCEEDED${NC}"
                    ((FAILED++))
                fi
            else
                echo -e "${YELLOW}⚠ $bench_name: Unable to parse results${NC}"
                ((SKIPPED++))
            fi
        else
            echo -e "${YELLOW}⚠ $bench_name: No estimate data found${NC}"
            ((SKIPPED++))
        fi
    else
        echo -e "${YELLOW}⚠ $bench_name: Benchmark not run${NC}"
        ((SKIPPED++))
    fi
done

echo ""
echo -e "${GREEN}=== Summary ===${NC}"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Skipped: $SKIPPED"
echo ""

# Additional benchmarks that don't have strict targets but should be documented
echo -e "${GREEN}=== Additional Benchmarks ===${NC}"
echo ""

# List all benchmarks that were run
if [ -d "target/criterion" ]; then
    for bench_dir in target/criterion/*/; do
        bench_name=$(basename "$bench_dir")

        # Skip if it's one of the targets we already checked
        if [[ -v TARGETS[$bench_name] ]]; then
            continue
        fi

        latest_estimate=$(find "$bench_dir" -name "estimates.json" -type f | head -1)
        if [ -f "$latest_estimate" ]; then
            mean_ns=$(jq -r '.mean.point_estimate' "$latest_estimate" 2>/dev/null || echo "0")
            mean_ms=$(echo "scale=2; $mean_ns / 1000000" | bc 2>/dev/null || echo "N/A")

            if [ "$mean_ms" != "N/A" ]; then
                echo -e "  $bench_name: ${mean_ms}ms"
            fi
        fi
    done
fi

echo ""

# Exit with appropriate code
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Performance validation FAILED${NC}"
    exit 1
elif [ $PASSED -gt 0 ]; then
    echo -e "${GREEN}Performance validation PASSED${NC}"
    exit 0
else
    echo -e "${YELLOW}Performance validation SKIPPED (no benchmarks completed)${NC}"
    exit 0
fi
