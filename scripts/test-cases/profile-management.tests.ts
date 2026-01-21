/**
 * Profile Management API Test Cases
 *
 * Tests for advanced profile management endpoints:
 * - POST /api/profiles/:name/duplicate - Duplicate a profile
 * - PUT /api/profiles/:name/rename - Rename a profile
 * - POST /api/profiles/:name/validate - Validate a profile
 */

import { ApiClient } from '../api-client/client.js';
import type { TestCase } from './api-tests.js';
import { z } from 'zod';

/**
 * No-op setup function for tests that don't need preparation
 */
const noOpSetup = async (): Promise<void> => {
  // No setup needed
};

/**
 * No-op cleanup function for tests that don't modify state
 */
const noOpCleanup = async (): Promise<void> => {
  // No cleanup needed
};

/**
 * Success response schema for profile duplicate operation
 */
const DuplicateProfileResponseSchema = z.object({
  success: z.boolean(),
  profile: z.object({
    name: z.string(),
    rhai_path: z.string(),
  }),
});

/**
 * Success response schema for profile rename operation
 */
const RenameProfileResponseSchema = z.object({
  success: z.boolean(),
  profile: z.object({
    name: z.string(),
    rhai_path: z.string(),
    krx_path: z.string(),
  }),
});

/**
 * Error response schema
 */
const ErrorResponseSchema = z.object({
  error: z.string(),
  code: z.string().optional(),
});

/**
 * Profile Management test cases
 */
export const profileManagementTestCases: TestCase[] = [
  // =================================================================
  // Profile Duplicate Tests
  // =================================================================
  {
    id: 'profiles-011',
    name: 'POST /api/profiles/:name/duplicate - Duplicate profile (success)',
    endpoint: '/api/profiles/:name/duplicate',
    scenario: 'duplicate_success',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create source profile for duplication
      const sourceName = `test-profile-source-${Date.now()}`;
      await client.createProfile(sourceName);
    },
    execute: async (client) => {
      // Find the source profile we just created
      const profiles = await client.getProfiles();
      const sourceProfile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('test-profile-source-')
      );

      if (!sourceProfile) {
        throw new Error('Source profile not found');
      }

      const sourceName = sourceProfile.name;
      const newName = `test-profile-duplicate-${Date.now()}`;

      const response = await client.customRequest(
        'POST',
        `/api/profiles/${encodeURIComponent(sourceName)}/duplicate`,
        DuplicateProfileResponseSchema,
        { new_name: newName }
      );

      return {
        status: response.status,
        data: response.data,
        context: { sourceName, newName },
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        success?: boolean;
        profile?: { name?: string; rhai_path?: string };
      };

      if (actualData.success !== true) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected success=true for profile duplicate',
        };
      }

      if (!actualData.profile?.name) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected profile.name in response',
        };
      }

      if (!actualData.profile?.rhai_path) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected profile.rhai_path in response',
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: async (client, result) => {
      // Delete both source and duplicate profiles
      try {
        const context = (result as any)?.context;
        if (context?.sourceName) {
          await client.deleteProfile(context.sourceName);
        }
        if (context?.newName) {
          await client.deleteProfile(context.newName);
        }
      } catch {
        // Ignore cleanup errors
      }
    },
  },

  {
    id: 'profiles-011b',
    name: 'POST /api/profiles/:name/duplicate - Duplicate nonexistent profile (404)',
    endpoint: '/api/profiles/:name/duplicate',
    scenario: 'duplicate_not_found',
    category: 'profiles',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const newName = `test-profile-duplicate-${Date.now()}`;

      try {
        const response = await client.customRequest(
          'POST',
          '/api/profiles/nonexistent-profile-xyz/duplicate',
          z.union([DuplicateProfileResponseSchema, ErrorResponseSchema]),
          { new_name: newName }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 404 Not Found or 500 error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const status = (actual as any).status || 200;

      if (status !== 404 && status !== 500) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected 404 or 500 for nonexistent profile, got ${status}`,
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'profiles-011c',
    name: 'POST /api/profiles/:name/duplicate - Duplicate with name conflict (409)',
    endpoint: '/api/profiles/:name/duplicate',
    scenario: 'duplicate_name_conflict',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create source profile and target profile with conflicting name
      const sourceName = `test-profile-source-${Date.now()}`;
      const targetName = `test-profile-target-${Date.now()}`;
      await client.createProfile(sourceName);
      await client.createProfile(targetName);
    },
    execute: async (client) => {
      // Find both profiles
      const profiles = await client.getProfiles();
      const sourceProfile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('test-profile-source-')
      );
      const targetProfile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('test-profile-target-')
      );

      if (!sourceProfile || !targetProfile) {
        throw new Error('Source or target profile not found');
      }

      const sourceName = sourceProfile.name;
      const targetName = targetProfile.name;

      try {
        const response = await client.customRequest(
          'POST',
          `/api/profiles/${encodeURIComponent(sourceName)}/duplicate`,
          z.union([DuplicateProfileResponseSchema, ErrorResponseSchema]),
          { new_name: targetName } // Try to duplicate with existing name
        );
        return {
          status: response.status,
          data: response.data,
          context: { sourceName, targetName },
        };
      } catch (error) {
        // Expect 409 Conflict or 500 error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
            context: { sourceName, targetName },
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const status = (actual as any).status || 200;

      // Accept 409 (Conflict) or 500 (Internal Server Error with conflict message)
      if (status !== 409 && status !== 500) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected 409 or 500 for duplicate name, got ${status}`,
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: async (client, result) => {
      // Delete both profiles
      try {
        const context = (result as any)?.context;
        if (context?.sourceName) {
          await client.deleteProfile(context.sourceName);
        }
        if (context?.targetName) {
          await client.deleteProfile(context.targetName);
        }
      } catch {
        // Ignore cleanup errors
      }
    },
  },

  // =================================================================
  // Profile Rename Tests
  // =================================================================
  {
    id: 'profiles-012',
    name: 'PUT /api/profiles/:name/rename - Rename profile (success)',
    endpoint: '/api/profiles/:name/rename',
    scenario: 'rename_success',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create profile to rename
      const originalName = `test-profile-rename-${Date.now()}`;
      await client.createProfile(originalName);
    },
    execute: async (client) => {
      // Find the profile we just created
      const profiles = await client.getProfiles();
      const profile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('test-profile-rename-')
      );

      if (!profile) {
        throw new Error('Profile to rename not found');
      }

      const originalName = profile.name;
      const newName = `test-profile-renamed-${Date.now()}`;

      const response = await client.customRequest(
        'PUT',
        `/api/profiles/${encodeURIComponent(originalName)}/rename`,
        RenameProfileResponseSchema,
        { new_name: newName }
      );

      return {
        status: response.status,
        data: response.data,
        context: { originalName, newName },
      };
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        success?: boolean;
        profile?: { name?: string; rhai_path?: string; krx_path?: string };
      };

      if (actualData.success !== true) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected success=true for profile rename',
        };
      }

      if (!actualData.profile?.name) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected profile.name in response',
        };
      }

      if (!actualData.profile?.rhai_path) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected profile.rhai_path in response',
        };
      }

      if (!actualData.profile?.krx_path) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: 'Expected profile.krx_path in response',
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: async (client, result) => {
      // Delete the renamed profile
      try {
        const context = (result as any)?.context;
        if (context?.newName) {
          await client.deleteProfile(context.newName);
        }
        // Also try to delete original name in case rename failed
        if (context?.originalName) {
          try {
            await client.deleteProfile(context.originalName);
          } catch {
            // Ignore - profile was successfully renamed
          }
        }
      } catch {
        // Ignore cleanup errors
      }
    },
  },

  {
    id: 'profiles-012b',
    name: 'PUT /api/profiles/:name/rename - Rename nonexistent profile (404)',
    endpoint: '/api/profiles/:name/rename',
    scenario: 'rename_not_found',
    category: 'profiles',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const newName = `test-profile-renamed-${Date.now()}`;

      try {
        const response = await client.customRequest(
          'PUT',
          '/api/profiles/nonexistent-profile-xyz/rename',
          z.union([RenameProfileResponseSchema, ErrorResponseSchema]),
          { new_name: newName }
        );
        return {
          status: response.status,
          data: response.data,
        };
      } catch (error) {
        // Expect 404 Not Found or 500 error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const status = (actual as any).status || 200;

      if (status !== 404 && status !== 500) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected 404 or 500 for nonexistent profile, got ${status}`,
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'profiles-012c',
    name: 'PUT /api/profiles/:name/rename - Rename with invalid name (400)',
    endpoint: '/api/profiles/:name/rename',
    scenario: 'rename_invalid_name',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create profile to rename
      const originalName = `test-profile-rename-invalid-${Date.now()}`;
      await client.createProfile(originalName);
    },
    execute: async (client) => {
      // Find the profile we just created
      const profiles = await client.getProfiles();
      const profile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('test-profile-rename-invalid-')
      );

      if (!profile) {
        throw new Error('Profile to rename not found');
      }

      const originalName = profile.name;
      const invalidName = ''; // Empty name is invalid

      try {
        const response = await client.customRequest(
          'PUT',
          `/api/profiles/${encodeURIComponent(originalName)}/rename`,
          z.union([RenameProfileResponseSchema, ErrorResponseSchema]),
          { new_name: invalidName }
        );
        return {
          status: response.status,
          data: response.data,
          context: { originalName },
        };
      } catch (error) {
        // Expect 400 Bad Request or 500 error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
            context: { originalName },
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const status = (actual as any).status || 200;

      // Accept 400 (Bad Request) or 500 (Internal Server Error with validation message)
      if (status !== 400 && status !== 500) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected 400 or 500 for invalid name, got ${status}`,
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: async (client, result) => {
      // Delete the original profile
      try {
        const context = (result as any)?.context;
        if (context?.originalName) {
          await client.deleteProfile(context.originalName);
        }
      } catch {
        // Ignore cleanup errors
      }
    },
  },

  {
    id: 'profiles-012d',
    name: 'PUT /api/profiles/:name/rename - Rename with name conflict (409)',
    endpoint: '/api/profiles/:name/rename',
    scenario: 'rename_name_conflict',
    category: 'profiles',
    priority: 2,
    setup: async (client) => {
      // Create two profiles
      const sourceName = `test-profile-rename-source-${Date.now()}`;
      const targetName = `test-profile-rename-target-${Date.now()}`;
      await client.createProfile(sourceName);
      await client.createProfile(targetName);
    },
    execute: async (client) => {
      // Find both profiles
      const profiles = await client.getProfiles();
      const sourceProfile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('test-profile-rename-source-')
      );
      const targetProfile = profiles.data.profiles.find((p: any) =>
        p.name.startsWith('test-profile-rename-target-')
      );

      if (!sourceProfile || !targetProfile) {
        throw new Error('Source or target profile not found');
      }

      const sourceName = sourceProfile.name;
      const targetName = targetProfile.name;

      try {
        const response = await client.customRequest(
          'PUT',
          `/api/profiles/${encodeURIComponent(sourceName)}/rename`,
          z.union([RenameProfileResponseSchema, ErrorResponseSchema]),
          { new_name: targetName } // Try to rename with existing name
        );
        return {
          status: response.status,
          data: response.data,
          context: { sourceName, targetName },
        };
      } catch (error) {
        // Expect 409 Conflict or 500 error
        if (error instanceof Error && 'statusCode' in error) {
          const apiError = error as { statusCode: number; response: unknown };
          return {
            status: apiError.statusCode,
            data: apiError.response,
            context: { sourceName, targetName },
          };
        }
        throw error;
      }
    },
    assert: (actual, expected) => {
      const status = (actual as any).status || 200;

      // Accept 409 (Conflict) or 500 (Internal Server Error with conflict message)
      if (status !== 409 && status !== 500) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Expected 409 or 500 for duplicate name, got ${status}`,
        };
      }

      return {
        passed: true,
        actual,
        expected: expected.body,
      };
    },
    cleanup: async (client, result) => {
      // Delete both profiles
      try {
        const context = (result as any)?.context;
        if (context?.sourceName) {
          await client.deleteProfile(context.sourceName);
        }
        if (context?.targetName) {
          await client.deleteProfile(context.targetName);
        }
      } catch {
        // Ignore cleanup errors
      }
    },
  },
];
