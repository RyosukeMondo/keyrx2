# WebSocket E2E Test Fixes

## Current Status

**Test Results**: 28/83 passing (33.7%)

## Root Cause Analysis

### WebSocket Protocol Mismatch

The WebSocket client (`scripts/api-client/websocket-client.ts`) and tests were implemented with an assumed protocol that doesn't match the actual daemon implementation.

#### What the Client Sends (Current - WRONG):
```json
{
  "type": "subscribe",
  "channel": "devices"
}
```

#### What the Daemon Expects (RPC Format - CORRECT):
```json
{
  "type": "subscribe",
  "content": {
    "id": "req-123",
    "channel": "devices"
  }
}
```

#### What the Daemon Responds (RPC Format):
```json
{
  "type": "response",
  "content": {
    "id": "req-123",
    "result": { "success": true },
    "error": null
  }
}
```

#### Event Broadcasting:
```json
{
  "type": "event",
  "content": {
    "channel": "devices",
    "data": { ... }
  }
}
```

### Daemon WebSocket Endpoints

1. **`/ws`** - Legacy broadcast-only endpoint (no subscriptions)
   - Location: `keyrx_daemon/src/web/ws.rs`
   - Behavior: Broadcasts ALL events to ALL clients
   - Message format: `{type: "latency|state|event", payload: {...}}`
   - Does NOT support subscribe/unsubscribe

2. **`/ws_rpc`** - RPC endpoint with subscription support
   - Location: `keyrx_daemon/src/web/ws_rpc.rs`
   - Behavior: Supports channel subscriptions
   - Message types: Query, Command, Subscribe, Unsubscribe
   - Protocol: JSON-RPC 2.0 inspired with `type` + `content` wrapper

### Message Type Definitions

See `keyrx_daemon/src/web/rpc_types.rs` for:
- `ClientMessage` enum (Query, Command, Subscribe, Unsubscribe)
- `ServerMessage` enum (Response, Event, Connected)
- `RpcError` struct

## Required Fixes

### 1. Update WebSocket Client (`api-client/websocket-client.ts`)

**Change message format to match RPC protocol:**

```typescript
// OLD (current):
{
  type: 'subscribe',
  channel: 'devices'
}

// NEW (required):
{
  type: 'subscribe',
  content: {
    id: 'uuid-here',
    channel: 'devices'
  }
}
```

**Update response parsing:**

```typescript
// OLD (expected):
{
  type: 'subscription_ack',
  channel: 'devices',
  success: true
}

// NEW (actual from daemon):
{
  type: 'response',
  content: {
    id: 'uuid-here',
    result: { success: true },
    error: null
  }
}
```

**Update event handling:**

```typescript
// OLD (expected):
{
  type: 'event',
  channel: 'devices',
  event: 'device_updated',
  data: {...}
}

// NEW (actual from daemon):
{
  type: 'event',
  content: {
    channel: 'devices',
    data: {...}
  }
}
```

### 2. Update WebSocket Tests (`test-cases/websocket.tests.ts`)

- Change WebSocket URL from `/ws` to `/ws_rpc`
- Update expectations to match RPC protocol
- Handle `Connected` handshake message on connection
- Update subscription assertions
- Update event assertions

### 3. Fix Other Test Issues

#### Device Configuration Requirements
Many tests fail with "Generator error: Device block not found". These tests need:
- A valid profile with device configuration
- Or mock device setup in test fixtures

#### Schema Validation Mismatches
- Error response codes don't match expectations
- Missing fields in response schemas
- Incorrect status codes (expecting 400, getting undefined)

#### Error Handling
- Tests expect specific error codes (400, 404, 409)
- Daemon returns 400 for many error cases
- Need to validate actual error messages, not just status codes

## Implementation Priority

1. **High Priority - WebSocket Protocol**
   - Update `websocket-client.ts` message format
   - Update `websocket.tests.ts` expectations
   - Test basic connection and subscription

2. **Medium Priority - Device Configuration**
   - Add device setup to test fixtures
   - Create test profiles with valid device blocks
   - Fix "Generator error: Device block not found" failures

3. **Medium Priority - Schema Validation**
   - Review response schemas in all tests
   - Update assertions to match actual responses
   - Fix error code expectations

4. **Low Priority - Error Handling**
   - Standardize error response assertions
   - Document actual error codes returned by daemon
   - Update test expectations accordingly

## Verification Steps

After fixes:
1. Run WebSocket tests: All 5 WebSocket tests should pass
2. Run device tests: Device management tests should pass
3. Run full suite: Aim for 80%+ pass rate
4. Document remaining failures
5. Create follow-up tasks for unresolved issues

## References

- Daemon RPC types: `keyrx_daemon/src/web/rpc_types.rs`
- Daemon RPC handler: `keyrx_daemon/src/web/ws_rpc.rs`
- Daemon legacy WS: `keyrx_daemon/src/web/ws.rs`
- WebSocket contract tests: `keyrx_daemon/tests/websocket_contract_test.rs`
- Client implementation: `scripts/api-client/websocket-client.ts`
- WebSocket tests: `scripts/test-cases/websocket.tests.ts`
