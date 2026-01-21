/**
 * Example: CRUD Endpoint Tests
 *
 * This example demonstrates how to test Create, Read, Update, Delete operations
 * with proper setup and cleanup to ensure test isolation.
 *
 * Use this pattern for:
 * - POST endpoints (create resources)
 * - PUT/PATCH endpoints (update resources)
 * - DELETE endpoints (remove resources)
 * - Any test that modifies system state
 */

import { TestCase } from '../test-executor/types.js';
import { ApiClient } from '../api-client/client.js';
import { ResponseComparator } from '../comparator/response-comparator.js';

/**
 * Example 1: CREATE (POST) Test
 *
 * Tests creating a new resource with cleanup to remove test data.
 */
export function getCreateExample(): TestCase {
  return {
    id: 'example-create-001',
    name: 'POST /api/profiles - create new profile',
    endpoint: '/api/profiles',
    scenario: 'create',
    category: 'profiles',
    priority: 2,

    // Setup: ensure clean state before test
    setup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      // Delete test profile if it already exists
      try {
        await client.deleteProfile('example-test-profile');
      } catch (error) {
        // Profile doesn't exist - that's fine
        // Only re-throw if it's not a 404 error
        if (error.status !== 404) {
          throw error;
        }
      }
    },

    // Execute: create the resource
    execute: async (client: ApiClient) => {
      return await client.createProfile('example-test-profile', {
        template: 'basic',
      });
    },

    // Assert: validate the response
    assert: (response, expected) => {
      const comparator = new ResponseComparator();

      // Ignore dynamic fields that vary between runs
      return comparator.compare(response, expected, {
        ignoreFields: ['id', 'created_at', 'updated_at'],
      });
    },

    // Cleanup: remove test data
    cleanup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      try {
        await client.deleteProfile('example-test-profile');
      } catch (error) {
        // If cleanup fails, log but don't fail the test
        console.warn('Cleanup warning:', error.message);
      }
    },
  };
}

/**
 * Example 2: UPDATE (PUT) Test
 *
 * Tests updating an existing resource. Requires creating the resource first.
 */
export function getUpdateExample(): TestCase {
  return {
    id: 'example-update-001',
    name: 'PUT /api/profiles/:name - update profile configuration',
    endpoint: '/api/profiles/:name',
    scenario: 'update',
    category: 'profiles',
    priority: 2,

    // Setup: create the resource to update
    setup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      // Ensure profile doesn't exist
      try {
        await client.deleteProfile('example-update-profile');
      } catch {
        // Doesn't exist, good
      }

      // Create the profile
      await client.createProfile('example-update-profile', {
        template: 'basic',
      });
    },

    // Execute: update the resource
    execute: async (client: ApiClient) => {
      const updatedConfig = `
        // Updated configuration
        map_key("a", tap("b"));
      `;

      return await client.updateProfile('example-update-profile', {
        source: updatedConfig,
      });
    },

    // Assert: validate the update succeeded
    assert: (response, expected) => {
      const comparator = new ResponseComparator();
      return comparator.compare(response, expected);
    },

    // Cleanup: remove test data
    cleanup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      try {
        await client.deleteProfile('example-update-profile');
      } catch (error) {
        console.warn('Cleanup warning:', error.message);
      }
    },
  };
}

/**
 * Example 3: DELETE Test
 *
 * Tests deleting a resource. Creates it first, then deletes.
 */
export function getDeleteExample(): TestCase {
  return {
    id: 'example-delete-001',
    name: 'DELETE /api/profiles/:name - delete profile',
    endpoint: '/api/profiles/:name',
    scenario: 'delete',
    category: 'profiles',
    priority: 2,

    // Setup: create the resource to delete
    setup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      // Create profile
      await client.createProfile('example-delete-profile', {
        template: 'basic',
      });
    },

    // Execute: delete the resource
    execute: async (client: ApiClient) => {
      return await client.deleteProfile('example-delete-profile');
    },

    // Assert: validate deletion succeeded
    assert: (response, expected) => {
      const comparator = new ResponseComparator();
      return comparator.compare(response, expected);
    },

    // Cleanup: verify resource is gone
    cleanup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      // Try to get the profile - should return 404
      try {
        await client.getProfile('example-delete-profile');
        // If we get here, profile still exists - that's a problem!
        throw new Error('Profile was not deleted');
      } catch (error) {
        // 404 is expected - profile is deleted
        if (error.status === 404) {
          // Success!
          return;
        }
        // Other errors are unexpected
        throw error;
      }
    },
  };
}

/**
 * Example 4: Error Case Test
 *
 * Tests that API returns proper errors for invalid requests.
 */
export function getErrorExample(): TestCase {
  return {
    id: 'example-error-001',
    name: 'POST /api/profiles - duplicate name returns 409',
    endpoint: '/api/profiles',
    scenario: 'duplicate_error',
    category: 'profiles',
    priority: 2,

    // Setup: create a profile so we can test duplicate error
    setup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      try {
        await client.deleteProfile('example-duplicate-profile');
      } catch {
        // Doesn't exist, good
      }

      // Create the first profile
      await client.createProfile('example-duplicate-profile', {
        template: 'basic',
      });
    },

    // Execute: try to create duplicate (should fail)
    execute: async (client: ApiClient) => {
      try {
        await client.createProfile('example-duplicate-profile', {
          template: 'basic',
        });
        // If we get here, API didn't return an error - test fails
        return {
          error: false,
          message: 'Expected 409 error but got success',
        };
      } catch (error) {
        // Error is expected - return error details
        return {
          error: true,
          status: error.status,
          code: error.code,
          message: error.message,
        };
      }
    },

    // Assert: validate we got the right error
    assert: (response, expected) => {
      // Check error status and code
      return (
        response.error === true &&
        response.status === 409 &&
        response.code === 'PROFILE_EXISTS'
      );
    },

    // Cleanup: remove test profile
    cleanup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      try {
        await client.deleteProfile('example-duplicate-profile');
      } catch (error) {
        console.warn('Cleanup warning:', error.message);
      }
    },
  };
}

/**
 * Expected results configuration (add to expected-results.json):
 *
 * {
 *   "endpoints": {
 *     "/api/profiles": {
 *       "scenarios": {
 *         "create": {
 *           "status": 201,
 *           "body": {
 *             "success": true,
 *             "profile": {
 *               "name": "example-test-profile"
 *             }
 *           }
 *         }
 *       }
 *     },
 *     "/api/profiles/:name": {
 *       "scenarios": {
 *         "update": {
 *           "status": 200,
 *           "body": {
 *             "success": true
 *           }
 *         },
 *         "delete": {
 *           "status": 200,
 *           "body": {
 *             "success": true
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
 * 1. Always clean up test data (in setup and cleanup)
 * 2. Make cleanup idempotent (handle 404 gracefully)
 * 3. Use unique test identifiers to avoid conflicts
 * 4. Ignore dynamic fields (id, timestamps) in assertions
 * 5. Test error cases separately with proper error handling
 * 6. Setup creates preconditions, cleanup removes artifacts
 * 7. Tests should be independent and not rely on execution order
 */

/**
 * Best Practices:
 *
 * - Use descriptive names for test resources (example-test-profile)
 * - Catch and handle specific errors in cleanup (don't fail silently)
 * - Verify resources are actually deleted in cleanup
 * - Test both success and error cases
 * - Use try-catch in execute when testing error responses
 */
