# API Contract Testing Guide

This guide documents the API contract testing system for KeyRx Web UI v2. Contract tests validate that the daemon's REST API responses match the frontend's Zod schemas, catching integration issues early.

## Overview

The contract testing system provides:
- Runtime validation of API responses against Zod schemas
- Local testing before commits
- Automated CI validation on pull requests
- Clear error reporting for schema mismatches

## Requirements

- Node.js 18+
- Running keyrx_daemon with web feature enabled
- At least one compiled profile (.krx file)

## Running Contract Tests Locally

### 1. Start the Daemon

First, ensure you have a compiled profile and start the daemon:

```bash
# Compile a profile if you don't have one
cargo build --release -p keyrx_compiler
./target/release/keyrx_compiler compile path/to/profile.rhai -o path/to/profile.krx

# Build and start the daemon
cargo build --release -p keyrx_daemon
./target/release/keyrx_daemon run --config path/to/profile.krx
```

The daemon will start on the default port 9867.

### 2. Run Contract Validation

In a separate terminal, run the validation script:

```bash
cd keyrx_ui
npm run validate:contracts
```

**Expected output (success):**
```
🔍 Validating API contracts against http://localhost:9867

✓ Daemon is running

Validating endpoints:

  ✓ GET /api/devices
  ✓ GET /api/profiles
  ✓ GET /api/profiles/:name/config

────────────────────────────────────────────────────────────

Results: 3 passed, 0 failed, 0 skipped

✅ All contracts valid
```

**Expected output (failure):**
```
🔍 Validating API contracts against http://localhost:9867

✓ Daemon is running

Validating endpoints:

  ✓ GET /api/devices
  ✗ GET /api/profiles
    Error: [
  {
    "code": "invalid_type",
    "expected": "string",
    "received": "number",
    "path": ["profiles", 0, "name"],
    "message": "Expected string, received number"
  }
]

────────────────────────────────────────────────────────────

Results: 1 passed, 1 failed, 0 skipped

❌ Contract validation FAILED

To fix contract mismatches:
1. Check if backend API response format changed
2. Update keyrx_ui/src/api/schemas.ts to match
3. Run npm test -- contracts.test.ts to verify
```

### 3. Custom Daemon URL

If your daemon runs on a different port or host:

```bash
npm run validate:contracts -- --base-url http://localhost:3030
```

Or run the script directly:

```bash
npx tsx scripts/validate-api-contracts.ts --base-url http://localhost:3030
```

## CI/CD Integration

Contract tests run automatically on every pull request via GitHub Actions.

### CI Workflow

The `api-contract-tests` job in `.github/workflows/ci.yml`:

1. **Build daemon and compiler** - Builds release binaries
2. **Setup test profile** - Creates minimal Rhai profile and compiles to .krx
3. **Start daemon** - Launches daemon in background
4. **Wait for readiness** - Polls `/api/status` until daemon responds
5. **Run contract validation** - Executes `npm run validate:contracts`
6. **Upload results** - Archives validation logs as CI artifacts
7. **Cleanup** - Stops daemon process

### Viewing CI Results

When contract tests fail in CI:

1. Go to the GitHub Actions run
2. Click on the "API Contract Tests" job
3. Expand "Run contract validation" step to see inline results
4. Download "contract-validation-results" artifact for full logs

### Debugging CI Failures

**Common failure: "Cannot connect to daemon"**
```
❌ Cannot connect to daemon at http://localhost:9867
   Make sure the daemon is running: keyrx_daemon run --config <path>
```
**Fix:** Check daemon startup logs in CI. Daemon may have failed to compile profile or bind to port.

**Common failure: Schema mismatch**
```
✗ GET /api/profiles
  Error: Expected string, received number
```
**Fix:** Backend API response format changed. Update `keyrx_ui/src/api/schemas.ts` to match.

## Adding New Endpoint Validation

### 1. Define Schema (if needed)

Add or verify the schema exists in `keyrx_ui/src/api/schemas.ts`:

```typescript
export const StatusResponseSchema = z.object({
  version: z.string(),
  uptime: z.number(),
  active_profile: z.string().nullable(),
}).passthrough();
```

**Schema guidelines:**
- Use `.passthrough()` to allow unexpected fields (logs warnings instead of failing)
- Use `.nullable()` for fields that can be null
- Use `.optional()` for fields that may not be present
- Match TypeScript types from backend Rust structs

### 2. Add Validation Call

Edit `keyrx_ui/scripts/validate-api-contracts.ts`:

```typescript
// Import the schema
import { StatusResponseSchema } from '../src/api/schemas';

// In main() function, add validation:
results.push(
  await validateEndpoint('GET', '/api/status', StatusResponseSchema, baseUrl)
);
```

### 3. Test Locally

Run validation to verify the new endpoint:

```bash
npm run validate:contracts
```

### 4. Add Unit Tests

Add tests in `keyrx_ui/src/api/contracts.test.ts`:

```typescript
describe('StatusResponseSchema', () => {
  it('validates valid status response', () => {
    const validData = {
      version: '0.1.0',
      uptime: 12345,
      active_profile: 'Default',
    };

    const result = StatusResponseSchema.safeParse(validData);
    expect(result.success).toBe(true);
  });

  it('rejects invalid status response', () => {
    const invalidData = {
      version: 123, // Wrong type
      uptime: 'invalid',
    };

    const result = StatusResponseSchema.safeParse(invalidData);
    expect(result.success).toBe(false);
  });
});
```

Run the unit tests:

```bash
npm test -- contracts.test.ts
```

## Troubleshooting

### Daemon won't start

**Problem:** `keyrx_daemon` fails to start or crashes immediately.

**Solutions:**
1. Check if you have required permissions (root/admin for keyboard capture)
2. Verify profile compiles: `keyrx_compiler compile profile.rhai -o profile.krx`
3. Check daemon logs for error messages
4. Ensure web feature is enabled (default in release builds)

### Connection refused

**Problem:** Script reports "Cannot connect to daemon at http://localhost:9867"

**Solutions:**
1. Verify daemon is running: `ps aux | grep keyrx_daemon`
2. Check daemon is listening: `curl http://localhost:9867/api/status`
3. Verify port 9867 is not blocked by firewall
4. Check if daemon bound to different port (see daemon startup logs)

### Schema validation fails

**Problem:** Endpoint returns 200 OK but schema validation fails.

**Solutions:**
1. Inspect actual response data (printed in error output)
2. Check if backend API format changed
3. Update schema in `keyrx_ui/src/api/schemas.ts` to match backend
4. Verify schema uses `.passthrough()` for extensibility
5. Check if field should be `.nullable()` or `.optional()`

**Example fix:**
```typescript
// Before (fails if 'layout' is null)
export const DeviceSchema = z.object({
  id: z.string(),
  name: z.string(),
  layout: z.string(),
});

// After (accepts null)
export const DeviceSchema = z.object({
  id: z.string(),
  name: z.string(),
  layout: z.string().nullable(),
});
```

### 404 Not Found

**Problem:** Endpoint returns 404, validation shows "skip" status.

**Expected behavior:** Some endpoints return 404 when no data exists (e.g., no profiles created yet). This is treated as a "skip" and doesn't fail validation.

**If unexpected:**
1. Verify endpoint path is correct in script
2. Check if endpoint requires authentication or special setup
3. Ensure daemon has necessary data (profiles, devices, etc.)

### Test data setup

**Problem:** Need specific test data for validation.

**Solutions:**

1. **For profiles:** Create test profile before validation:
```bash
cat > test.rhai << 'EOF'
device_start("*");
device_end();
EOF
keyrx_compiler compile test.rhai -o test.krx
keyrx_daemon run --config test.krx
```

2. **For devices:** Devices are auto-detected by daemon. On CI (no keyboards), list may be empty - this is expected.

3. **For parameterized endpoints:** Script dynamically fetches data. See `/api/profiles/:name/config` validation for example.

## Script Exit Codes

The validation script uses specific exit codes for automation:

- **0** - All contracts valid (success)
- **1** - Contract validation failed (schema mismatch)
- **2** - Connection error (daemon not running)

Use in scripts:

```bash
#!/bin/bash
npm run validate:contracts
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
  echo "All contracts valid"
elif [ $EXIT_CODE -eq 1 ]; then
  echo "Contract mismatch - update schemas"
elif [ $EXIT_CODE -eq 2 ]; then
  echo "Daemon not running - start daemon first"
fi
```

## Best Practices

### When to Run Contract Tests

- **Before committing** - Catch breaking changes early
- **After modifying schemas** - Verify changes work with live daemon
- **After backend API changes** - Ensure frontend schemas stay in sync
- **In CI pipeline** - Automatic validation on all PRs

### Schema Design Guidelines

1. **Use permissive schemas:**
   - Add `.passthrough()` to allow unexpected fields
   - Backend may add fields without breaking frontend

2. **Handle optional data:**
   - Use `.nullable()` for fields that can be null
   - Use `.optional()` for fields that may not exist
   - Example: `serial: z.string().nullable().optional()`

3. **Document backend types:**
   - Add comments linking schemas to Rust structs
   - Example: `// Matches DeviceResponse in keyrx_daemon/src/web/api/devices.rs`

4. **Test edge cases:**
   - Empty arrays
   - Null values
   - Missing optional fields
   - Unexpected extra fields

### Maintaining Contract Tests

1. **Keep schemas in sync:** When backend changes, update schemas immediately
2. **Add tests for new endpoints:** Every REST endpoint should have validation
3. **Document breaking changes:** Update CHANGELOG.md when API format changes
4. **Version API if needed:** Consider API versioning for major breaking changes

## Technical Details

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│ CI Pipeline                                             │
│                                                         │
│  ┌──────────┐    ┌───────────┐    ┌────────────────┐  │
│  │  Build   │───▶│  Start    │───▶│   Validate     │  │
│  │  Daemon  │    │  Daemon   │    │   Contracts    │  │
│  └──────────┘    └───────────┘    └────────────────┘  │
│                         │                   │          │
│                         ▼                   ▼          │
│                  Port 9867           Zod Schemas       │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  Artifacts:     │
                    │  - Logs         │
                    │  - Results      │
                    └─────────────────┘
```

### Validated Endpoints

Currently validated endpoints (as of tasks.md completion):

- `GET /api/devices` - DeviceListResponseSchema
- `GET /api/profiles` - ProfileListResponseSchema
- `GET /api/profiles/:name/config` - ProfileConfigResponseSchema

Additional endpoints available for validation:

- `GET /api/status` - Daemon status and version
- `PATCH /api/devices/:id` - Update device configuration

### Schema Validation Flow

```typescript
1. Fetch API response
2. Parse JSON
3. Schema.safeParse(data)
   ├── Success: return validated data
   └── Failure: log error + exit(1)
```

## References

- **Schemas:** `keyrx_ui/src/api/schemas.ts`
- **Validation script:** `keyrx_ui/scripts/validate-api-contracts.ts`
- **CI workflow:** `.github/workflows/ci.yml`
- **Unit tests:** `keyrx_ui/src/api/contracts.test.ts`
- **Spec:** `.spec-workflow/specs/api-contract-testing/`
