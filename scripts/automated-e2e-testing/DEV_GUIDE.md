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

### Step 1: Add Test to Test Suite

Edit `scripts/test-cases/api-tests.ts` and add your test to the array:

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

### Step 3: Run Your Test

```bash
# Run all tests
npm run test:e2e:auto

# Run with auto-fix enabled
npm run test:e2e:auto -- --fix

# Or run the test runner directly
npx tsx scripts/automated-e2e-test.ts --daemon-path target/release/keyrx_daemon
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

- Test cases: `scripts/test-cases/api-tests.ts`
- Expected results: `scripts/fixtures/expected-results.json`
- Fix strategies: `scripts/auto-fix/fix-strategies.ts`
- Test runner: `scripts/automated-e2e-test.ts`

### npm Scripts

- `npm run test:e2e:auto` - Run automated tests
- `npm run test:e2e:auto:report` - Generate HTML report
- `npm run metrics:report` - View metrics summary
- `npm run metrics:latest` - View latest test run

### Key Interfaces

- `TestCase` - Test definition
- `FixStrategy` - Auto-fix strategy
- `ValidationResult` - Assertion result
- `ApiClient` - Type-safe API wrapper

## Further Reading

- [README.md](./README.md) - System overview and architecture
- [Example Test](./examples/example-test.ts) - Complete working example
- [API Client](../api-client/client.ts) - Available API methods
- [Fix Strategies](../auto-fix/fix-strategies.ts) - Built-in strategies
