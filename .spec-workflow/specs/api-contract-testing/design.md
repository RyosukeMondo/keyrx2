# Design: API Contract Testing

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Contract Validation Flow                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐     ┌──────────────────┐                  │
│  │ validate-api-    │     │  schemas.ts      │                  │
│  │ contracts.ts     │────▶│  (Zod schemas)   │                  │
│  │ (validation      │     │                  │                  │
│  │  script)         │     └──────────────────┘                  │
│  └────────┬─────────┘                                           │
│           │                                                      │
│           │ HTTP requests                                        │
│           ▼                                                      │
│  ┌──────────────────┐     ┌──────────────────┐                  │
│  │  keyrx_daemon    │────▶│  Rust API        │                  │
│  │  (running)       │     │  handlers        │                  │
│  └──────────────────┘     └──────────────────┘                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                      CI/CD Integration                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐    │
│  │  Build    │─▶│  Start    │─▶│  Validate │─▶│  Report   │    │
│  │  daemon   │  │  daemon   │  │  contracts│  │  results  │    │
│  └───────────┘  └───────────┘  └───────────┘  └───────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### 1. Contract Validation Script (Existing)
**File:** `keyrx_ui/scripts/validate-api-contracts.ts`

Already implemented:
- Connects to daemon API
- Validates responses against Zod schemas
- Reports pass/fail with error details
- Configurable base URL via CLI argument

### 2. CI Workflow Integration (To Implement)
**File:** `.github/workflows/ci.yml`

New job: `api-contract-tests`
- Depends on: `build` job
- Steps:
  1. Build daemon in release mode
  2. Create test profile
  3. Start daemon in background
  4. Wait for daemon to be ready
  5. Run contract validation script
  6. Report results

### 3. Contract Test Documentation
**File:** `keyrx_ui/docs/contract-testing.md`

Document:
- How to run contract tests locally
- How to add new endpoint validation
- Troubleshooting common failures

## Technical Decisions

### D1: Use Existing Zod Schemas
**Decision:** Validate against schemas already defined in `schemas.ts`
**Rationale:** Single source of truth for frontend expectations; changes to schemas automatically affect validation

### D2: Daemon Must Be Running
**Decision:** Tests require live daemon, not mocked responses
**Rationale:** Catches real mismatches between Rust structs and Zod schemas; mocks would hide the problem

### D3: CI Starts Its Own Daemon
**Decision:** CI builds and starts daemon as part of test job
**Rationale:** Ensures tests run against same code being merged; no external dependencies

### D4: Fail Fast on First Error
**Decision:** Continue testing all endpoints even if one fails
**Rationale:** Developer sees all mismatches at once, not just the first one

## API Endpoint Coverage

| Endpoint | Schema | Status |
|----------|--------|--------|
| GET /api/status | - | Health check only |
| GET /api/devices | DeviceListResponseSchema | Validated |
| GET /api/profiles | ProfileListResponseSchema | Validated |
| GET /api/profiles/:name/config | ProfileConfigRpcSchema | Validated |

## CI Job Configuration

```yaml
api-contract-tests:
  runs-on: ubuntu-latest
  needs: [build]
  steps:
    - uses: actions/checkout@v4
    - name: Setup Rust
      uses: dtolnay/rust-action@stable
    - name: Setup Node
      uses: actions/setup-node@v4
      with:
        node-version: '20'
    - name: Build daemon
      run: cargo build -p keyrx_daemon --release
    - name: Setup test profile
      run: |
        mkdir -p ~/.config/keyrx/profiles
        echo 'device_start("*"); device_end();' > ~/.config/keyrx/profiles/Test.rhai
    - name: Start daemon
      run: |
        ./target/release/keyrx_daemon run --config ~/.config/keyrx/profiles/Test.krx &
        sleep 5
    - name: Run contract validation
      working-directory: keyrx_ui
      run: npm run validate:contracts
```

## Error Handling

### Connection Errors
- Exit code 2 when daemon is not reachable
- Clear message: "Cannot connect to daemon at [URL]"

### Validation Errors
- Exit code 1 when schema validation fails
- Shows: endpoint, expected schema, actual response, field path

### Success
- Exit code 0 when all endpoints pass
- Summary: "X passed, 0 failed"

## Security Considerations

- CI runs daemon without keyboard capture (no evdev permissions)
- Daemon web server binds to localhost only
- No sensitive data in contract tests

## Testing Strategy

### Unit Tests
- `contracts.test.ts`: Tests Zod schemas against mock data (40 tests)

### Integration Tests
- `validate-api-contracts.ts`: Tests against live daemon

### CI Tests
- New workflow job validates contracts on every PR
