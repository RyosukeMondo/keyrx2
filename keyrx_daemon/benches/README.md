# Performance Benchmarks

This directory contains Criterion-based benchmarks for validating performance targets specified in the web-ui-ux-comprehensive specification.

## Benchmarks

### 1. Device Registry (`device_registry.rs`)

Benchmarks for the DeviceRegistry component that manages persistent device metadata.

**Benchmarks:**
- `device_registry_save_10_devices`: Atomic write operation for 10 devices
  - **Target:** <50ms
  - **Tests:** JSON serialization + atomic file write (write to .tmp, then rename)

- `device_registry_load_50_devices`: Load and parse from disk
  - **No strict target:** Measures deserialization overhead

- `device_registry_get_lookup`: HashMap lookup by device ID
  - **No strict target:** Should be O(1), typically <1μs

- `device_registry_list_100_devices`: List all devices
  - **No strict target:** Measures iteration overhead

- `device_registry_rename`: Rename with validation
  - **No strict target:** Measures validation + HashMap update

### 2. Profile Activation (`profile_activation.rs`)

Benchmarks for the ProfileManager hot-reload functionality.

**Benchmarks:**
- `profile_activation_hot_reload`: Hot-reload (excluding compilation)
  - **Target:** <100ms
  - **Tests:** Load .krx from disk + atomic config swap
  - **Note:** Compilation time is excluded as it's a separate operation

- `profile_creation_blank_template`: Create new profile from template
  - **No strict target:** Measures file I/O + template generation

- `profile_list_10_profiles`: List all available profiles
  - **No strict target:** Measures filesystem scanning

### 3. IPC Latency (`ipc_latency.rs`)

Benchmarks for Unix socket IPC communication between CLI and daemon.

**Benchmarks:**
- `ipc_status_query_roundtrip`: Status query request + response
  - **Target:** <10ms
  - **Tests:** Unix socket send + receive with small JSON payload

- `ipc_latency_metrics_roundtrip`: Latency metrics query
  - **No strict target:** Slightly larger JSON payload than status

- `ipc_state_query_roundtrip`: State query (255-element boolean array)
  - **No strict target:** Largest payload, tests serialization overhead

**Note:** IPC benchmarks only run on Unix platforms (Linux, macOS).

## Running Benchmarks

### Run all benchmarks:
```bash
cargo bench
```

### Run specific benchmark suite:
```bash
cargo bench --bench device_registry
cargo bench --bench profile_activation
cargo bench --bench ipc_latency
```

### Run specific benchmark function:
```bash
cargo bench --bench device_registry -- device_registry_save
```

### Validate against performance targets:
```bash
../scripts/validate_performance.sh
```

## Performance Targets Summary

From `web-ui-ux-comprehensive` specification:

| Operation | Target | Rationale |
|-----------|--------|-----------|
| Profile activation (hot-reload) | <100ms | User-perceivable delay threshold |
| IPC status query | <10ms | CLI responsiveness |
| Device registry save | <50ms | Non-blocking file I/O |
| Simulation determinism | Variance = 0 | Same seed must produce identical output |

## Benchmark Results

Results are stored in `target/criterion/<benchmark_name>/` directory:
- `report/index.html` - Interactive HTML report
- `*/estimates.json` - Statistical estimates (mean, std dev, percentiles)
- `base/` - Baseline for regression detection

### Viewing Results

Open the HTML report:
```bash
open target/criterion/<benchmark_name>/report/index.html
```

Or check the summary in terminal output after running benchmarks.

## CI Integration

These benchmarks are run in CI to detect performance regressions:
- Benchmarks run on each PR
- Significant regressions (>10% slowdown) fail the build
- Results are archived as CI artifacts

## Notes

- Benchmarks use `tempfile` for isolated test environments
- Each benchmark iteration runs with fresh state to avoid caching effects
- IPC benchmarks use a mock server to avoid daemon dependency
- Profile activation benchmark pre-compiles profiles to isolate hot-reload measurement
- All benchmarks use `criterion::black_box` to prevent compiler optimizations

## Troubleshooting

### ProfileManager benchmarks fail
ProfileManager may not be fully implemented yet. The benchmark will be skipped automatically.

### IPC benchmarks don't run
IPC benchmarks are Unix-only. On Windows, they compile to no-op benchmarks.

### Results show high variance
Try increasing the measurement time:
```bash
cargo bench -- --measurement-time 10
```

Or increase sample size:
```bash
cargo bench -- --sample-size 1000
```
