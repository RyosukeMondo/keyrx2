# E2E Test Status Analysis and Categorization

**Last Updated:** 2026-01-22
**Total Tests:** 83
**Categories:** 6 failure patterns + 1 passing category

## Test Status by Category

### ✅ Category A: Fully Passing (27 tests)

These tests work correctly and validate important functionality:

#### Health & Status (2/3 tests)
- ✅ `health-001` - GET /api/health - Daemon healthy
- ✅ `version-001` - GET /api/version - Get daemon version

#### Profiles (8/20 tests)
- ✅ GET /api/profiles - List profiles
- ✅ GET /api/profiles/active - No active profile
- ✅ POST /api/profiles - Create new profile
- ✅ POST /api/profiles - Reject duplicate profile
- ✅ POST /api/profiles/:name/activate - Activate existing profile
- ✅ POST /api/profiles/:name/activate - Reject nonexistent profile
- ✅ DELETE /api/profiles/:name - Delete existing profile
- ✅ DELETE /api/profiles/:name - Reject nonexistent profile

#### Devices (1/15 tests)
- ✅ GET /api/devices - List all devices

#### Layouts (1/3 tests)
- ✅ GET /api/layouts - Get available keyboard layouts

#### Macros (6/8 tests)
- ✅ POST /api/macros/start-recording - Start recording successfully
- ✅ POST /api/macros/stop-recording - Stop recording successfully
- ✅ GET /api/macros/recorded-events - Get events while not recording
- ✅ GET /api/macros/recorded-events - Get events while recording
- ✅ POST /api/macros/clear - Clear events successfully
- ✅ POST /api/macros/clear - Verify events are cleared

#### Simulator (5/7 tests)
- ✅ POST /api/simulator/events - Simulate single key press/release
- ✅ POST /api/simulator/events - Simulate key sequence
- ✅ POST /api/simulator/events - Use built-in scenario
- ✅ POST /api/simulator/reset - Reset simulator state
- ✅ POST /api/simulator/reset - Verify idempotency

#### Config (2/11 tests)
- ✅ PUT /api/config - Update configuration with valid Rhai
- ✅ PUT /api/config - Update configuration with invalid syntax

#### Metrics (1/4 tests)
- ✅ DELETE /api/metrics/events - Clear event log (not implemented)

#### WebSocket (1/5 tests)
- ✅ `websocket-001` - Connect and disconnect lifecycle

---

## ⚠️ Category B: Socket Dependency (8 tests) - EXPECTED FAILURES

These tests require a physical keyboard device socket connection. They correctly fail with 503 "Socket not connected" in test environment.

**Status:** ✅ Working as designed - Test environment limitation
**Action:** Mark as integration tests or add mock device support

### Affected Tests:
1. `health-007` - GET /api/daemon/state
   - Error: HTTP 503 Socket not connected
   - Reason: Requires raw device state from connected keyboard

2. `metrics-001` - GET /api/metrics/latency
   - Error: HTTP 503 Socket not connected
   - Reason: Requires active input processing for latency calculation

3. `metrics-002` - GET /api/metrics/events (default limit)
   - Error: HTTP 503 Socket not connected
   - Reason: Event log requires active input stream

4. `metrics-002b` - GET /api/metrics/events?count=10
   - Error: HTTP 503 Socket not connected
   - Reason: Event log requires active input stream

5. `config-001` - GET /api/config
   - Error: HTTP 400 Generator error: Device block not found
   - Reason: Requires active profile with device configuration

6. `config-003` - POST /api/config/key-mappings (simple)
   - Error: HTTP 400 Generator error: Device block not found
   - Reason: Requires active device context

7. `config-003b` - POST /api/config/key-mappings (tap-hold)
   - Error: HTTP 400 Generator error: Device block not found
   - Reason: Requires active device context

8. `config-004` - DELETE /api/config/key-mappings/:id
   - Error: HTTP 400 Generator error: Device block not found
   - Reason: Requires active device context

9. `config-005` - GET /api/layers
   - Error: HTTP 400 Generator error: Device block not found
   - Reason: Requires active profile with layer configuration

10. `workflow-002` - Profile duplicate → rename → activate
    - Error: HTTP 503 Socket not connected
    - Reason: Workflow requires profile activation with socket

---

## 🔧 Category C: Profile Name Length (6 tests) - EASY FIX

These tests fail because profile names exceed the 32 character limit.

**Status:** ❌ Test data issue
**Action:** Shorten profile names to ≤32 characters
**Estimated Fix Time:** 5 minutes

### Tests Needing Name Shortening:
1. `profiles-011` - POST /api/profiles/:name/duplicate
   - Current: `test-profile-duplicate-copy-success` (35 chars)
   - Fix: `prof-dup-test` (13 chars)

2. `profiles-011b` - Duplicate with name conflict
   - Current: `test-profile-duplicate-conflict` (33 chars)
   - Fix: `prof-dup-conflict` (17 chars)

3. `profiles-012` - PUT /api/profiles/:name/rename
   - Current: `test-profile-rename-new-name-success` (37 chars)
   - Fix: `prof-rename-new` (15 chars)

4. `profiles-012b` - Rename with invalid name
   - Current: `test-profile-rename-invalid-name` (32 chars) ✅ Borderline
   - Fix: `prof-rename-bad` (15 chars) - safer

5. `profiles-012c` - Rename with name conflict
   - Current: `test-profile-rename-conflict` (28 chars) ✅ OK
   - But new name: `test-profile-rename-conflict-target` (35 chars)
   - Fix: `prof-rename-target` (18 chars)

6. `profiles-013` - POST /api/profiles/:name/validate
   - Current: `test-profile-validate-valid` (28 chars) ✅ OK
   - But also uses: `test-profile-validate-nonexistent` (34 chars)
   - Fix: `prof-validate-404` (17 chars)

---

## 🔍 Category D: WebSocket Subscription (4 tests) - INVESTIGATION NEEDED

WebSocket tests timeout waiting for subscription acknowledgments.

**Status:** ⚠️ Implementation issue or protocol mismatch
**Action:** Investigate daemon WebSocket subscription handling
**Estimated Fix Time:** 1-2 hours

### Failing Tests:
1. `websocket-002` - Subscribe to channel and receive acknowledgment
   - Error: Subscription timeout for channel: devices (5000ms)
   - Expected: Subscription acknowledgment message

2. `websocket-003` - Receive device event notification
   - Error: Subscription timeout for channel: devices (5000ms)
   - Flow: Subscribe → Update device → Wait for event

3. `websocket-004` - Receive profile event notification
   - Error: Subscription timeout for channel: profiles (5000ms)
   - Flow: Subscribe → Activate profile → Wait for event

4. `websocket-005` - Reconnection and subscription restoration
   - Error: Subscription timeout for channel: devices (5000ms)
   - Flow: Connect → Subscribe → Disconnect → Reconnect → Verify subscription

### Debug Steps:
1. Check daemon WebSocket implementation:
   ```bash
   rg "subscription" keyrx_daemon/src/web/websocket.rs
   ```

2. Test manually with wscat:
   ```bash
   wscat -c ws://localhost:9867/ws
   > {"type":"subscribe","channel":"devices"}
   # Should receive acknowledgment
   ```

3. Check WebSocket message format in test vs daemon

---

## 🚫 Category E: Missing Endpoints (6 tests) - BACKEND TODO

These endpoints return 404, indicating they're not implemented.

**Status:** ⚠️ Missing backend implementation
**Action:** Implement missing device endpoints
**Estimated Fix Time:** 2-3 hours

### Missing Endpoints:
1. `PUT /api/devices/:id/name` - Rename device
   - Tests: `devices-004`, `devices-004b`, `devices-004c`, `devices-004d`
   - Expected: Rename device, return success
   - Actual: HTTP 404

2. `PUT /api/devices/:id/layout` - Set device layout
   - Tests: `devices-005`, `devices-005b`, `devices-005c`, `devices-005d`
   - Expected: Set layout, return success
   - Actual: HTTP 404

3. `GET /api/devices/:id/layout` - Get device layout
   - Tests: `devices-006`, `devices-006b`
   - Expected: Return layout name
   - Actual: HTTP 404

### Implementation TODO:
```rust
// keyrx_daemon/src/web/api/devices.rs

// Add these endpoint handlers:
pub async fn rename_device(...) -> Result<Json<DeviceResponse>, ApiError>
pub async fn set_device_layout(...) -> Result<Json<DeviceResponse>, ApiError>
pub async fn get_device_layout(...) -> Result<Json<LayoutResponse>, ApiError>

// Register routes:
Router::new()
    .route("/devices/:id/name", put(rename_device))
    .route("/devices/:id/layout", put(set_device_layout))
    .route("/devices/:id/layout", get(get_device_layout))
```

---

## ❌ Category F: Missing Validation (15 tests) - BACKEND TODO

Tests expecting error responses (400, 404) but getting 200 OK.

**Status:** ⚠️ Missing backend validation
**Action:** Add validation logic to endpoints
**Estimated Fix Time:** 3-4 hours

### Validation Gaps:

#### 1. Device ID Existence (5 tests)
Tests expect 404 for nonexistent device IDs but get 200:
- `devices-003` - PATCH nonexistent device
- `devices-004b` - Rename nonexistent device
- `devices-005b` - Set layout on nonexistent device
- `devices-006b` - Get layout for nonexistent device
- `devices-007` - Delete nonexistent device

**Fix:** Add device existence check:
```rust
let device = state.registry.get_device(&device_id)
    .ok_or(ApiError::DeviceNotFound(device_id))?;
```

#### 2. Device Name Validation (3 tests)
Tests expect 400 for invalid names but get 200:
- `devices-002` - PATCH device with invalid name (special chars)
- `devices-004c` - Empty device name
- `devices-004d` - Name > 100 characters

**Fix:** Add name validation:
```rust
if name.is_empty() {
    return Err(ApiError::InvalidRequest("Device name cannot be empty"));
}
if name.len() > 100 {
    return Err(ApiError::InvalidRequest("Device name too long (max 100 chars)"));
}
if !name.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_') {
    return Err(ApiError::InvalidRequest("Device name contains invalid characters"));
}
```

#### 3. Layout Name Validation (3 tests)
Tests expect 400 for invalid layouts but get 200:
- `devices-005c` - Empty layout name
- `devices-005d` - Layout name > 50 characters

**Fix:** Add layout validation:
```rust
if layout.is_empty() {
    return Err(ApiError::InvalidRequest("Layout name cannot be empty"));
}
if layout.len() > 50 {
    return Err(ApiError::InvalidRequest("Layout name too long (max 50 chars)"));
}
```

#### 4. Profile Operations on Nonexistent Profiles (4 tests)
Tests expect 404/500 but get 200:
- `profiles-011b` - Duplicate nonexistent profile
- `profiles-012b` - Rename nonexistent profile
- `profiles-013b` - Validate nonexistent profile

**Fix:** Add profile existence check before operations

#### 5. Config API Validation (4 tests)
Tests expect validation errors but operations succeed:
- `config-003c` - Invalid action type
- `config-003d` - Missing required field
- `config-004b` - Invalid mapping ID format
- `config-004c` - Delete non-existent mapping

**Fix:** Add payload validation and ID format checks

#### 6. Macro/Simulator Validation (4 tests)
Tests expect 400 for invalid requests:
- `macros-001b` - Start recording when already recording
- `macros-002b` - Stop recording when not recording
- `simulator-001d` - No events or scenario provided
- `simulator-001e` - Unknown scenario name

**Fix:** Add state checks and parameter validation

#### 7. Layout Endpoint (2 tests)
- `layouts-002` - Expected KLE JSON array format
- `layouts-002b` - Expected 404 status for nonexistent layout

**Fix:** Return proper KLE format and add existence check

---

## 🔄 Category G: Workflow Tests (6 tests) - DOWNSTREAM FAILURES

Workflow tests fail due to issues in individual endpoints they depend on.

**Status:** ⬇️ Will resolve when dependencies are fixed
**Action:** No direct action - fix underlying endpoints first

### Failing Workflows:
1. `workflow-001` - Profile lifecycle
   - Depends on: profile operations
   - Blocked by: profile name validation

2. `workflow-002` - Profile duplicate → rename → activate
   - Depends on: duplicate, rename, activate endpoints
   - Blocked by: socket dependency, name length

3. `workflow-003` - Profile validation → fix → activate
   - Depends on: validate, update, activate
   - Blocked by: missing `config` field in request

4. `workflow-004` - Device rename → layout change → disable
   - Depends on: device endpoints
   - Blocked by: missing device endpoints, null layout handling

5. `workflow-005` - Config update → add mappings → verify layers
   - Depends on: config/mapping/layer endpoints
   - Blocked by: device block not found error

6. `workflow-006` - Macro record → simulate → playback
   - Depends on: macro and simulator endpoints
   - Blocked by: event format mismatch

7. `workflow-007` - Simulator event → mapping → output
   - Depends on: simulator and config endpoints
   - Blocked by: invalid template error

---

## Summary by Priority

### 🟢 P0 - Quick Wins (11 tests, ~15 min)
1. **Profile name shortening** - 6 tests
   - Files to edit: `scripts/test-cases/profile-management.tests.ts`
   - Change: Shorten names to ≤32 chars

2. **Status test** - 1 test
   - `status-001` - Verify expected state format

### 🟡 P1 - Investigation Required (4 tests, ~2 hours)
1. **WebSocket subscription** - 4 tests
   - Debug daemon WebSocket implementation
   - Verify subscription protocol
   - Test manually with wscat

### 🟠 P2 - Backend Implementation (6 tests, ~3 hours)
1. **Device endpoints** - 6 tests
   - Implement PUT /api/devices/:id/name
   - Implement PUT /api/devices/:id/layout
   - Implement GET /api/devices/:id/layout

### 🔴 P3 - Backend Validation (15 tests, ~4 hours)
1. **Add validation logic** - 15 tests
   - Device ID existence checks
   - Name/layout validation
   - Profile operation validation
   - Config API validation
   - Macro/simulator validation

### ⚪ P4 - Test Environment (8 tests, ~ongoing)
1. **Socket dependency** - 8 tests
   - Mark as integration tests
   - Or add mock device support
   - Or update test expectations

### 🔵 P5 - Downstream (6 tests, ~auto-fix)
1. **Workflow tests** - 6 tests
   - Will pass once dependencies are fixed
   - No direct action needed

---

## Recommended Action Plan

### Phase 1: Quick Wins (Day 1, 1 hour)
1. ✅ Fix profile name lengths
2. ✅ Fix status test expectation
3. ✅ Verify execution completes successfully
4. ✅ Commit fixes

### Phase 2: Investigation (Day 1-2, 2-4 hours)
1. Debug WebSocket subscription handling
2. Test WebSocket manually
3. Fix WebSocket implementation
4. Verify WebSocket tests pass

### Phase 3: Backend Endpoints (Day 2-3, 3-4 hours)
1. Implement device name endpoint
2. Implement device layout endpoints
3. Add tests for new endpoints
4. Verify device tests pass

### Phase 4: Backend Validation (Day 3-5, 4-6 hours)
1. Add device validation
2. Add profile validation
3. Add config validation
4. Add macro/simulator validation
5. Verify validation tests pass

### Phase 5: Integration (Day 5, 1-2 hours)
1. Mark socket-dependent tests
2. Update documentation
3. Verify all tests categorized
4. Generate final report

### Total Estimated Time: 5-7 days

---

## Success Metrics

### Current State
- ✅ Test infrastructure: 100% complete
- ✅ Test execution: Stable, no crashes
- ✅ Performance: 29s (< 3 min target)
- ⚠️ Pass rate: 32.5% (27/83)

### Target State
- 🎯 Pass rate: 90%+ (75+/83)
- 🎯 Socket tests: Handled appropriately
- 🎯 Backend validation: Complete
- 🎯 WebSocket: Fully functional
- 🎯 Documentation: Updated

### Path to Success
- Fix 6 quick win tests → 40% pass rate
- Fix 4 WebSocket tests → 45% pass rate
- Fix 6 backend endpoint tests → 52% pass rate
- Fix 15 validation tests → 70% pass rate
- Fix 6 workflow tests (downstream) → 80% pass rate
- Handle 8 socket tests appropriately → 90% pass rate
