/**
 * Example: Simple GET Endpoint Test
 *
 * This example demonstrates how to test a basic GET endpoint
 * that returns data without requiring setup or cleanup.
 *
 * Use this pattern for:
 * - Read-only endpoints
 * - Health checks
 * - Status endpoints
 * - List endpoints that don't require specific data
 */

import { TestCase } from '../test-executor/types.js';
import { ApiClient } from '../api-client/client.js';
import { ResponseComparator } from '../comparator/response-comparator.js';

export function getSimpleEndpointExample(): TestCase {
  return {
    // Unique identifier - use descriptive naming
    id: 'example-simple-001',

    // Human-readable test name
    name: 'GET /api/health - check daemon health status',

    // Endpoint being tested (for grouping and reporting)
    endpoint: '/api/health',

    // Scenario name (maps to expected results)
    scenario: 'healthy',

    // Category for organization (health, devices, profiles, etc.)
    category: 'health',

    // Priority: 1=critical, 2=important, 3=nice-to-have
    priority: 1,

    // Execute function: makes the API call
    execute: async (client: ApiClient) => {
      // Call the API method
      const response = await client.getHealth();

      // Return the response (will be compared with expected)
      return response;
    },

    // Assert function: validates the response
    assert: (response, expected) => {
      // Use ResponseComparator for deep equality checks
      const comparator = new ResponseComparator();

      // Compare actual response with expected results
      return comparator.compare(response, expected);
    },

    // Optional: setup function runs before execute
    // Not needed for simple read-only tests
    // setup: async () => {
    //   // Initialize test preconditions
    // },

    // Optional: cleanup function runs after execute
    // Not needed for simple read-only tests
    // cleanup: async () => {
    //   // Clean up test artifacts
    // },
  };
}

/**
 * Expected result configuration (add to expected-results.json):
 *
 * {
 *   "endpoints": {
 *     "/api/health": {
 *       "scenarios": {
 *         "healthy": {
 *           "status": 200,
 *           "body": {
 *             "status": "ok",
 *             "version": "0.1.0"
 *           }
 *         }
 *       }
 *     }
 *   }
 * }
 */

/**
 * Key Points:
 *
 * 1. Simple GET tests don't need setup/cleanup
 * 2. Use ResponseComparator for consistent validation
 * 3. Expected results must match actual API response structure
 * 4. Test should be idempotent (can run multiple times)
 * 5. No side effects on system state
 */

/**
 * Running this test:
 *
 * 1. Add to test suite in api-tests.ts:
 *    import { getSimpleEndpointExample } from './examples/simple-endpoint.example.js';
 *    const tests = [getSimpleEndpointExample()];
 *
 * 2. Add expected result to expected-results.json (see above)
 *
 * 3. Run tests:
 *    npm run test:e2e:auto
 */
