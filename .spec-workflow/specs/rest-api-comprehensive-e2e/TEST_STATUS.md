# REST API E2E Test Status

**Last Updated**: 2026-01-22
**Current Status**: 27/83 tests passing (32.5%)
**Target**: 100% pass rate

## Recent Fixes (Commits)

### 1. Template and Endpoint Corrections (3a638453)
- Changed invalid 'empty' template to 'blank' (valid template name)
- Fixed profile config endpoints from `/profiles/:name` to `/profiles/:name/config`
- **Impact**: Fixed 1-2 test failures

### 2. Error Handling Structure (f29f21ee)
- Corrected error handling in layouts, macros, and simulator tests
- Changed from `error.response?.status` to `error.statusCode` (ApiClientError structure)
- Changed from `error.response?.data` to `error.response`
- **Impact**: Fixed error handling logic, but tests still fail due to API behavior

### 3. Config Test Assertions (c6c9bcdb)
- Fixed config test error handling to use ApiClientError pattern
- Fixed assertions to unwrap data from `{status, data}` response structure
- **Impact**: Fixed "Cannot read properties of undefined" errors

## Remaining Test Failures (56 tests)

### Category 1: SOCKET_NOT_CONNECTED / HTTP 503 (10 tests)
**Root Cause**: Tests require an active profile with socket connection

Tests affected:
- Profile config GET/PUT operations
- Device management operations
- Config/mapping operations
- Several workflow tests

**Fix Required**:
- Tests need to create and activate a profile with proper socket setup
- May need to mock/stub socket connection or use test daemon mode

### Category 2: GENERATOR_ERROR - Device block not found (6 tests)
**Root Cause**: Config tests don't set up proper device blocks

Tests affected:
- POST /api/config/key-mappings (all variants)
- DELETE /api/config/key-mappings
- GET /api/layers

**Fix Required**:
- Tests must create a valid config with device blocks before adding mappings
- Example: Set up `[device.keyboard-0]` block with base_layer

### Category 3: HTTP 400 - Bad Request (15 tests)
**Root Cause**: Test request data doesn't match API expectations

Common issues:
- Invalid request body structure
- Missing required fields
- Invalid field values
- Template/config syntax errors

**Fix Required**:
- Review each failing test's request payload
- Align with actual API endpoint requirements
- Check backend validation logic

### Category 4: HTTP 404 - Not Found (4 tests)
**Root Cause**: Tests reference resources that don't exist

Tests affected:
- Device operations on nonexistent devices
- Profile operations on nonexistent profiles

**Fix Required**:
- Ensure setup creates required resources
- Fix test data to use correct IDs/names

### Category 5: WebSocket Subscription Timeout (4 tests)
**Root Cause**: WebSocket subscription mechanism not responding

Tests affected:
- websocket-002: Subscribe to channel
- websocket-003: Device event notification
- websocket-004: Profile event notification
- websocket-005: Reconnection

**Fix Required**:
- Investigate WebSocket server subscription handling
- Check if subscription acknowledgment is being sent
- May need to fix backend WebSocket implementation

### Category 6: Expected Status Mismatch (4 tests)
**Root Cause**: API returns different status than expected

Tests affected:
- simulator-001d: Expected 400, got undefined
- simulator-001e: Expected 400, got undefined
- macros-001b: Expected 400, got undefined
- macros-002b: Expected 400, got undefined

**Fix Required**:
- Check if API should return 400 for these cases or if test expectations are wrong
- May need to add validation to backend endpoints

### Category 7: Workflow Test Failures (6 tests)
**Root Cause**: Complex multi-step tests hit multiple issues

Tests affected:
- workflow-002: Profile duplicate → rename → activate
- workflow-003: Profile validation → fix → activate
- workflow-004: Device rename → layout change → disable
- workflow-005: Config update → add mappings → verify layers
- workflow-006: Macro record → simulate → playback
- workflow-007: Simulator event → mapping → output

**Fix Required**:
- Fix underlying API issues first
- Review each workflow step
- May need test data/setup adjustments

## Next Steps

### Immediate Priorities (High Impact)

1. **Fix Socket Connection Requirements** (10 tests)
   - Investigate how to properly set up active profile with socket
   - May need daemon configuration or test mode

2. **Fix Device Block Setup** (6 tests)
   - Create helper to set up valid config with device blocks
   - Update config test setup functions

3. **Fix WebSocket Subscriptions** (4 tests)
   - Debug WebSocket subscription acknowledgment
   - May be backend bug in subscription handling

### Medium Priority

4. **Review HTTP 400 Errors** (15 tests)
   - Systematic review of each failing test
   - Align request payloads with API expectations

5. **Fix Status Mismatch Tests** (4 tests)
   - Determine if API or test expectations are correct
   - Add proper validation if needed

### Lower Priority

6. **Fix HTTP 404 Tests** (4 tests)
   - Review test setup/cleanup
   - Ensure proper resource creation

7. **Fix Workflow Tests** (6 tests)
   - Fix after resolving underlying issues
   - Integration test nature means multiple failure points

## Verification Checklist

From `.spec-workflow/specs/rest-api-comprehensive-e2e/tasks.md`:

- [x] Run `npm install` - succeeds without errors
- [ ] Run `npx tsx scripts/automated-e2e-test.ts` - all 65+ tests pass (currently 27/83)
- [ ] Run tests 10 consecutive times - 0 flaky failures (not tested yet)
- [ ] Check execution time - < 3 minutes (current: ~30s)
- [ ] Verify all 40+ endpoints covered - generate coverage report
- [ ] Check CI workflow - passes on GitHub Actions
- [ ] Review HTML report - all tests documented
- [ ] Verify file sizes - all < 500 lines (needs checking)
- [ ] Check documentation - README, DEV_GUIDE, TROUBLESHOOTING complete
- [ ] Run `make verify` - all quality gates pass

## Test Execution Command

```bash
npx tsx scripts/automated-e2e-test.ts \
  --daemon-path target/release/keyrx_daemon \
  --port 9867 \
  --report-json test-results.json
```

## Analysis Tools

View failure breakdown:
```bash
cat test-results.json | jq '{total: .summary.total, passed: .summary.passed, failed: .summary.failed, passRate: .summary.passRate}'
```

Categorize failures:
```bash
cat test-results.json | jq -r '.results[] | select(.status == "fail") | .error' | grep -oE "(HTTP [0-9]+|SOCKET_NOT_CONNECTED|GENERATOR_ERROR|timeout)" | sort | uniq -c | sort -rn
```

Find specific test:
```bash
cat test-results.json | jq '.results[] | select(.id == "test-id-here")'
```
