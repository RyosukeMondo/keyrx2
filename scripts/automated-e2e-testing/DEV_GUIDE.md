# Developer Guide: Extending the E2E Test System

This guide shows you how to add new test cases, update expected results, and create custom fix strategies.

## Adding Test Cases

### Test Case Structure

Each test case implements the `TestCase` interface:

```typescript
interface TestCase {
  id: string;                    // Unique identifier (e.g., "test-001")
  name: string;                  // Descriptive name
  endpoint: string;              // API endpoint (e.g., "/api/status")
  scenario: string;              // Scenario name (e.g., "healthy")
  category: string;              // Category (e.g., "status", "profiles")
  priority: number;              // Priority 1-3 (1=high, 3=low)
  setup?: () => Promise<void>;   // Optional setup
  execute: (client: ApiClient) => Promise<unknown>;
  assert: (response: unknown, expected: unknown) => ValidationResult;
  cleanup?: () => Promise<void>; // Optional cleanup
}
```

### Test File Organization

Tests are organized by category in separate files:

| File | Purpose | When to Add Tests Here |
|------|---------|------------------------|
| `api-tests.ts` | Core endpoints | Basic daemon status, device listing, profile CRUD |
| `health-metrics.tests.ts` | Health & metrics | Health checks, version info, latency, event logs |
| `device-management.tests.ts` | Device operations | Rename, layout, enable/disable, forget devices |
| `profile-management.tests.ts` | Profile operations | Duplicate, rename, validate profiles |
| `config-layers.tests.ts` | Config management | Config updates, key mappings, layer management |
| `layouts.tests.ts` | Keyboard layouts | Layout listing, layout details |
| `macros.tests.ts` | Macro recording | Start/stop recording, get/clear events |
| `simulator.tests.ts` | Event simulation | Simulate events, reset simulator |
| `websocket.tests.ts` | WebSocket events | Connection, subscription, real-time events |
| `workflows.tests.ts` | Multi-step flows | Complex feature workflows combining multiple endpoints |

**Choosing the right file:**
- Single endpoint tests → Category-specific file
- Multi-endpoint workflows → `workflows.tests.ts`
- New feature category → Create new `feature-name.tests.ts`

### Step 1: Add Test to Test Suite

Edit the appropriate test file (e.g., `scripts/test-cases/profile-management.tests.ts`) and add your test to the array:

```typescript
export function getAllTestCases(): TestCase[] {
  return [
    // ... existing tests ...

    // Your new test
    {
      id: 'test-042',
      name: 'POST /api/profiles - create profile with custom config',
      endpoint: '/api/profiles',
      scenario: 'create_custom',
      category: 'profiles',
      priority: 2,

      // Setup: ensure clean state
      setup: async () => {
        // Delete test profile if it exists
        const client = new ApiClient({ baseUrl: 'http://localhost:9867' });
        try {
          await client.deleteProfile('test-profile');
        } catch {
          // Profile doesn't exist, that's fine
        }
      },

      // Execute: make the API call
      execute: async (client: ApiClient) => {
        return await client.createProfile('test-profile', {
          description: 'Test profile',
          mappings: []
        });
      },

      // Assert: validate response
      assert: (response, expected) => {
        const comparator = new ResponseComparator();
        return comparator.compare(response, expected, {
          ignoreFields: ['createdAt', 'id']
        });
      },

      // Cleanup: remove test data
      cleanup: async () => {
        const client = new ApiClient({ baseUrl: 'http://localhost:9867' });
        await client.deleteProfile('test-profile');
      }
    }
  ];
}
```

### Step 2: Add Expected Result

Edit `scripts/fixtures/expected-results.json`:

```json
{
  "version": "1.0",
  "apiVersion": "0.1.0",
  "endpoints": {
    "/api/profiles": {
      "scenarios": {
        "create_custom": {
          "status": 201,
          "body": {
            "name": "test-profile",
            "description": "Test profile",
            "mappings": [],
            "active": false
          }
        }
      }
    }
  }
}
```

### Step 3: Register New Test Files

If you created a new test file (e.g., `new-feature.tests.ts`), register it in `scripts/automated-e2e-test.ts`:

```typescript
// Import your new test file
import { getAllTestCases as getNewFeatureTests } from './test-cases/new-feature.tests.js';

// Add to test collection
const allTests = [
  ...getApiTests(),
  ...getHealthMetricsTests(),
  ...getDeviceManagementTests(),
  ...getProfileManagementTests(),
  ...getConfigLayersTests(),
  ...getLayoutsTests(),
  ...getMacrosTests(),
  ...getSimulatorTests(),
  ...getWebSocketTests(),
  ...getWorkflowTests(),
  ...getNewFeatureTests(),  // Add your tests here
];
```

### Step 4: Run Your Test

```bash
# Run all tests
npm run test:e2e:auto

# Run with auto-fix enabled
npm run test:e2e:auto -- --fix

# Or run the test runner directly
npx tsx scripts/automated-e2e-test.ts --daemon-path target/release/keyrx_daemon
```

### Creating a New Test File

When adding a new feature category, create a new test file following this template:

**File:** `scripts/test-cases/new-feature.tests.ts`

```typescript
/**
 * Test suite for new feature endpoints
 *
 * Covers:
 * - GET /api/feature
 * - POST /api/feature
 * - DELETE /api/feature/:id
 */

import { TestCase } from '../test-executor/types.js';
import { ApiClient } from '../api-client/client.ts';
import { ResponseComparator } from '../comparator/response-comparator.js';

export function getAllTestCases(): TestCase[] {
  return [
    // Test 1: List features
    {
      id: 'feature-001',
      name: 'GET /api/feature - list all features',
      endpoint: '/api/feature',
      scenario: 'list',
      category: 'feature',
      priority: 1,

      execute: async (client: ApiClient) => {
        return await client.getFeatures();
      },

      assert: (response, expected) => {
        const comparator = new ResponseComparator();
        return comparator.compare(response, expected);
      },
    },

    // Test 2: Create feature
    {
      id: 'feature-002',
      name: 'POST /api/feature - create new feature',
      endpoint: '/api/feature',
      scenario: 'create',
      category: 'feature',
      priority: 2,

      setup: async () => {
        // Clean up any existing test feature
        const client = new ApiClient({ baseUrl: 'http://localhost:9867' });
        try {
          await client.deleteFeature('test-feature');
        } catch {
          // Doesn't exist, that's fine
        }
      },

      execute: async (client: ApiClient) => {
        return await client.createFeature({
          name: 'test-feature',
          enabled: true,
        });
      },

      assert: (response, expected) => {
        const comparator = new ResponseComparator();
        return comparator.compare(response, expected, {
          ignoreFields: ['id', 'created_at'],
        });
      },

      cleanup: async () => {
        const client = new ApiClient({ baseUrl: 'http://localhost:9867' });
        await client.deleteFeature('test-feature');
      },
    },

    // Add more tests...
  ];
}
```

**File:** `scripts/fixtures/expected-results.json`

Add expected results for your new endpoints:

```json
{
  "endpoints": {
    "/api/feature": {
      "scenarios": {
        "list": {
          "status": 200,
          "body": {
            "features": []
          }
        },
        "create": {
          "status": 201,
          "body": {
            "name": "test-feature",
            "enabled": true
          }
        }
      }
    }
  }
}
```

## Updating Expected Results

### When to Update

Update expected results when:
- ✅ API contract changes (new fields, different structure)
- ✅ Bug fixes that change correct behavior
- ✅ Feature additions that modify responses
- ❌ Random test failures (investigate instead)
- ❌ Auto-fix suggests changes (review first)

### How to Update

#### Method 1: Manual Update (Recommended)

1. **Run the test** to see actual vs expected:
   ```bash
   npm run test:e2e:auto
   ```

2. **Review the diff** in the test output:
   ```
   Expected: { "status": "running", "version": "0.1.0" }
   Actual:   { "status": "running", "version": "0.2.0", "uptime": 1234 }
   ```

3. **Verify the actual response is correct** by checking:
   - Recent code changes
   - API documentation
   - Manual API testing

4. **Update `expected-results.json`**:
   ```json
   {
     "endpoints": {
       "/api/status": {
         "scenarios": {
           "healthy": {
             "status": 200,
             "body": {
               "status": "running",
               "version": "0.2.0",
               "uptime": 1234
             }
           }
         }
       }
     }
   }
   ```

5. **Re-run the test** to confirm it passes

#### Method 2: Auto-Fix Assisted

1. **Enable auto-fix**:
   ```bash
   npm run test:e2e:auto -- --fix
   ```

2. **Review suggested changes** in the output

3. **Manually verify** the changes are correct

4. **Commit** the updated `expected-results.json`

## Writing Fix Strategies

Fix strategies automatically remediate common test failures.

### Strategy Interface

```typescript
interface FixStrategy {
  name: string;
  canFix: (issue: Issue) => boolean;
  apply: (issue: Issue, context: FixContext) => Promise<FixResult>;
}

interface FixContext {
  daemonFixture: DaemonFixture;
  apiClient: ApiClient;
  expectedResultsPath: string;
}

interface FixResult {
  success: boolean;
  message: string;
  retry: boolean;  // Should the test be retried?
}
```

### Example: Custom Timeout Strategy

Create a new file `scripts/auto-fix/custom-strategies.ts`:

```typescript
import { FixStrategy, Issue, FixContext, FixResult } from './types.js';

export class CustomTimeoutStrategy implements FixStrategy {
  name = 'custom-timeout';

  canFix(issue: Issue): boolean {
    // Only fix timeout errors
    return issue.type === 'network' &&
           issue.description.includes('timeout');
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    try {
      // Wait longer before retrying
      await new Promise(resolve => setTimeout(resolve, 5000));

      return {
        success: true,
        message: 'Waited 5 seconds for service to stabilize',
        retry: true
      };
    } catch (error) {
      return {
        success: false,
        message: `Failed to apply timeout fix: ${error.message}`,
        retry: false
      };
    }
  }
}
```

### Register Your Strategy

Edit `scripts/auto-fix/fix-strategies.ts`:

```typescript
import { CustomTimeoutStrategy } from './custom-strategies.js';

export function getAllFixStrategies(): FixStrategy[] {
  return [
    new RestartDaemonStrategy(),
    new UpdateExpectedResultStrategy(),
    new RetryTestStrategy(),
    new CustomTimeoutStrategy(),  // Add your strategy
  ];
}
```

### Testing Your Strategy

1. **Create a failing test** that triggers the issue
2. **Run with auto-fix**: `npm run test:e2e:auto -- --fix`
3. **Verify the strategy is applied** in the output
4. **Check the test passes** after fix

## Running Tests Locally

### Quick Commands

```bash
# Run all tests with auto-fix
npm run test:e2e:auto

# Run without auto-fix
npx tsx scripts/automated-e2e-test.ts --daemon-path target/release/keyrx_daemon --no-fix

# Use custom port
npx tsx scripts/automated-e2e-test.ts --port 9868

# Limit fix iterations
npx tsx scripts/automated-e2e-test.ts --fix --max-iterations 1

# Save report
npx tsx scripts/automated-e2e-test.ts --report-json my-results.json
```

### Debugging Failed Tests

#### 1. Run a Single Test

Temporarily modify `scripts/test-cases/api-tests.ts`:

```typescript
export function getAllTestCases(): TestCase[] {
  const allTests = [
    // ... all test definitions ...
  ];

  // Uncomment to run only one test
  // return allTests.filter(t => t.id === 'test-042');

  return allTests;
}
```

#### 2. Enable Verbose Logging

Edit `scripts/automated-e2e-test.ts`:

```typescript
const executor = new TestExecutor({
  testTimeout: 30000,
  verbose: true,  // Enable verbose logging
  expectedResults,
});
```

#### 3. Inspect Daemon Logs

Daemon output is captured automatically. Check the test output for:

```
📋 Daemon logs:
[timestamp] INFO Starting KeyRx daemon
[timestamp] ERROR Failed to load config
```

#### 4. Test the API Manually

```bash
# Start daemon
./target/release/keyrx_daemon

# In another terminal, test the endpoint
curl http://localhost:9867/api/status

# Or use httpie
http :9867/api/status
```

## Best Practices

### Test Isolation

**Do:**
- Clean up test data in `cleanup` function
- Use unique identifiers for test resources
- Don't depend on other tests' state

**Don't:**
- Share state between tests
- Assume test execution order
- Leave test data in the system

**Example:**
```typescript
{
  id: 'test-profile-001',
  setup: async () => {
    // Ensure clean state
    await deleteTestProfile('my-test-profile');
  },
  execute: async (client) => {
    return await client.createProfile('my-test-profile', config);
  },
  cleanup: async () => {
    // Always clean up
    await deleteTestProfile('my-test-profile');
  }
}
```

### Determinism

**Do:**
- Use fixed timestamps in test data
- Use seeded random number generators
- Wait for async operations to complete

**Don't:**
- Use `Date.now()` or `Math.random()` in tests
- Assume timing (use polling instead)
- Ignore race conditions

**Example:**
```typescript
// Good: deterministic timestamp
const testData = {
  timestamp: '2026-01-21T00:00:00Z',
  value: 42
};

// Bad: non-deterministic
const testData = {
  timestamp: new Date().toISOString(),  // Changes every run!
  value: Math.random()                   // Non-deterministic!
};
```

### Performance

**Do:**
- Keep setup/cleanup fast
- Reuse API clients
- Use appropriate timeouts

**Don't:**
- Sleep unnecessarily
- Make redundant API calls
- Set timeouts too high "just in case"

**Example:**
```typescript
// Good: efficient polling
async function waitForReady(client: ApiClient) {
  for (let i = 0; i < 10; i++) {
    const status = await client.getStatus();
    if (status.ready) return;
    await sleep(100);  // Short poll interval
  }
  throw new Error('Not ready after 1 second');
}

// Bad: long sleep
async function waitForReady() {
  await sleep(10000);  // Always waits 10 seconds!
}
```

### Error Handling

**Do:**
- Provide descriptive error messages
- Include context in errors (test ID, timestamp)
- Fail fast on setup errors

**Don't:**
- Swallow errors silently
- Use generic error messages
- Continue after fatal errors

**Example:**
```typescript
// Good: clear error with context
try {
  const result = await client.createProfile(name, config);
  if (!result.success) {
    throw new Error(
      `Failed to create profile "${name}": ${result.error} ` +
      `(test: ${testId}, timestamp: ${timestamp})`
    );
  }
} catch (error) {
  throw new Error(`Setup failed for test ${testId}: ${error.message}`);
}

// Bad: swallowed error
try {
  await client.createProfile(name, config);
} catch (error) {
  // Silent failure!
}
```

## Writing WebSocket Tests

WebSocket tests verify real-time event notifications.

### Step 1: Use WebSocket Client

```typescript
import { WebSocketClient } from '../api-client/websocket-client.js';

{
  id: 'websocket-001',
  name: 'WebSocket - receive device update event',
  endpoint: '/ws',
  scenario: 'device_update',
  category: 'websocket',
  priority: 2,

  execute: async (client: ApiClient) => {
    const wsClient = new WebSocketClient('ws://localhost:9867/ws');

    try {
      // 1. Connect to WebSocket
      await wsClient.connect();

      // 2. Subscribe to channel
      await wsClient.subscribe('devices');

      // 3. Trigger an event via REST API
      await client.patchDevice('device-123', { enabled: false });

      // 4. Wait for WebSocket event
      const event = await wsClient.waitForEvent(
        (msg) => msg.channel === 'devices' && msg.event === 'device_updated',
        5000  // timeout in ms
      );

      return {
        success: true,
        event,
      };
    } finally {
      await wsClient.disconnect();
    }
  },

  assert: (response, expected) => {
    return response.success &&
           response.event?.data?.device_id === 'device-123';
  },
}
```

### Step 2: Common WebSocket Patterns

**Connection test:**
```typescript
execute: async (client) => {
  const wsClient = new WebSocketClient(wsUrl);
  await wsClient.connect();
  const connected = wsClient.isConnected();
  await wsClient.disconnect();
  return { connected };
}
```

**Subscription test:**
```typescript
execute: async (client) => {
  const wsClient = new WebSocketClient(wsUrl);
  await wsClient.connect();
  const ack = await wsClient.subscribe('profiles');
  await wsClient.disconnect();
  return { acknowledged: ack.success };
}
```

**Event notification test:**
```typescript
execute: async (client) => {
  const wsClient = new WebSocketClient(wsUrl);
  await wsClient.connect();
  await wsClient.subscribe('profiles');

  // Trigger event
  await client.activateProfile('test-profile');

  // Wait for notification
  const event = await wsClient.waitForEvent(
    (msg) => msg.event === 'profile_activated',
    5000
  );

  await wsClient.disconnect();
  return { received: !!event };
}
```

**Reconnection test:**
```typescript
execute: async (client) => {
  const wsClient = new WebSocketClient(wsUrl);
  await wsClient.connect();
  await wsClient.subscribe('devices');

  // Disconnect
  await wsClient.disconnect();

  // Reconnect
  await wsClient.connect();

  // Subscriptions should be restored automatically
  const isSubscribed = wsClient.hasSubscription('devices');

  await wsClient.disconnect();
  return { reconnected: isSubscribed };
}
```

## Writing Workflow Tests

Workflow tests validate multi-step feature scenarios.

### Step 1: Define the Workflow

Workflow tests go in `scripts/test-cases/workflows.tests.ts`:

```typescript
{
  id: 'workflow-008',
  name: 'Complete device configuration workflow',
  endpoint: 'multiple',  // Multiple endpoints
  scenario: 'device_config',
  category: 'workflow',
  priority: 2,

  execute: async (client: ApiClient) => {
    const results: any[] = [];

    // Step 1: List devices
    const devices = await client.getDevices();
    results.push({ step: 'list', count: devices.length });

    if (devices.length === 0) {
      throw new Error('No devices available for workflow test');
    }

    const deviceId = devices[0].id;

    // Step 2: Rename device
    await client.renameDevice(deviceId, 'workflow-test-device');
    results.push({ step: 'rename', success: true });

    // Step 3: Change layout
    await client.setDeviceLayout(deviceId, 'ansi-104');
    const layout = await client.getDeviceLayout(deviceId);
    results.push({ step: 'layout', layout });

    // Step 4: Disable device
    await client.patchDevice(deviceId, { enabled: false });
    const updated = await client.getDevices();
    const device = updated.find(d => d.id === deviceId);
    results.push({ step: 'disable', enabled: device?.enabled });

    // Step 5: Re-enable device
    await client.patchDevice(deviceId, { enabled: true });

    return { steps: results };
  },

  assert: (response, expected) => {
    const steps = response.steps;
    return steps.length === 4 &&
           steps[1].success === true &&
           steps[2].layout === 'ansi-104' &&
           steps[3].enabled === false;
  },

  cleanup: async () => {
    // Restore original device state if needed
  }
}
```

### Step 2: Workflow Best Practices

**Do:**
- Test real user scenarios (not just API coverage)
- Include error recovery (what if step 2 fails?)
- Return intermediate results for debugging
- Clean up all test artifacts

**Don't:**
- Make workflows too long (max 5-7 steps)
- Ignore intermediate failures
- Assume state from previous workflows

**Example workflow patterns:**

1. **Create → Configure → Use → Cleanup**
   ```typescript
   // Create resource
   const id = await client.createResource(data);

   // Configure it
   await client.updateResource(id, config);

   // Use it
   const result = await client.processWithResource(id);

   // Cleanup
   await client.deleteResource(id);
   ```

2. **Error → Fix → Retry**
   ```typescript
   // Trigger error
   let result = await client.validateProfile('invalid-profile');
   assert(result.valid === false);

   // Fix error
   await client.updateProfile('invalid-profile', fixedConfig);

   // Retry validation
   result = await client.validateProfile('invalid-profile');
   assert(result.valid === true);
   ```

3. **State Change → Verify → Restore**
   ```typescript
   // Save original state
   const original = await client.getState();

   // Change state
   await client.updateState(newState);

   // Verify change
   const updated = await client.getState();
   assert(updated !== original);

   // Restore
   await client.updateState(original);
   ```

## Common Patterns

### Testing POST Endpoints

```typescript
{
  id: 'test-post-001',
  name: 'POST /api/resource - create new resource',
  setup: async () => {
    // Ensure resource doesn't exist
    await deleteResource('test-resource');
  },
  execute: async (client) => {
    return await client.createResource({
      name: 'test-resource',
      data: { key: 'value' }
    });
  },
  assert: (response, expected) => {
    const comparator = new ResponseComparator();
    return comparator.compare(response, expected, {
      ignoreFields: ['id', 'createdAt']  // Ignore dynamic fields
    });
  },
  cleanup: async () => {
    await deleteResource('test-resource');
  }
}
```

### Testing Error Responses

```typescript
{
  id: 'test-error-001',
  name: 'GET /api/resource/:id - not found',
  execute: async (client) => {
    // Expect this to throw
    try {
      await client.getResource('nonexistent-id');
      throw new Error('Expected 404 error');
    } catch (error) {
      return {
        status: error.status,
        message: error.message
      };
    }
  },
  assert: (response, expected) => {
    return response.status === 404;
  }
}
```

### Testing with Multiple Steps

```typescript
{
  id: 'test-workflow-001',
  name: 'Complete profile workflow - create, activate, verify',
  execute: async (client) => {
    // Step 1: Create profile
    const created = await client.createProfile('workflow-test', {});

    // Step 2: Activate profile
    await client.activateProfile('workflow-test');

    // Step 3: Verify active
    const status = await client.getStatus();

    return {
      created,
      activeProfile: status.activeProfile
    };
  },
  assert: (response, expected) => {
    return response.activeProfile === 'workflow-test';
  },
  cleanup: async () => {
    await client.deleteProfile('workflow-test');
  }
}
```

## Quick Reference

### File Locations

**Test Cases:**
- Core endpoints: `scripts/test-cases/api-tests.ts`
- Health & metrics: `scripts/test-cases/health-metrics.tests.ts`
- Devices: `scripts/test-cases/device-management.tests.ts`
- Profiles: `scripts/test-cases/profile-management.tests.ts`
- Config & layers: `scripts/test-cases/config-layers.tests.ts`
- Layouts: `scripts/test-cases/layouts.tests.ts`
- Macros: `scripts/test-cases/macros.tests.ts`
- Simulator: `scripts/test-cases/simulator.tests.ts`
- WebSocket: `scripts/test-cases/websocket.tests.ts`
- Workflows: `scripts/test-cases/workflows.tests.ts`

**Infrastructure:**
- Expected results: `scripts/fixtures/expected-results.json`
- Fix strategies: `scripts/auto-fix/fix-strategies.ts`
- Test runner: `scripts/automated-e2e-test.ts`
- API client: `scripts/api-client/client.ts`
- WebSocket client: `scripts/api-client/websocket-client.ts`

### npm Scripts

- `npm run test:e2e:auto` - Run automated tests (100+ tests)
- `npm run test:e2e:auto:report` - Generate HTML report
- `npm run metrics:report` - View metrics summary
- `npm run metrics:latest` - View latest test run
- `npm run metrics:clear` - Clear metrics history

### Key Interfaces

- `TestCase` - Test definition with setup/execute/assert/cleanup
- `FixStrategy` - Auto-fix strategy for remediation
- `ValidationResult` - Assertion result with diff details
- `ApiClient` - Type-safe REST API wrapper (35+ methods)
- `WebSocketClient` - WebSocket connection manager with subscriptions

## Further Reading

- [README.md](./README.md) - System overview and architecture
- [Example Test](./examples/example-test.ts) - Complete working example
- [API Client](../api-client/client.ts) - Available API methods
- [Fix Strategies](../auto-fix/fix-strategies.ts) - Built-in strategies
