/**
 * Config & Layers API Test Cases
 *
 * Tests for configuration and layer management endpoints:
 * - GET /api/config - Get current configuration
 * - PUT /api/config - Update configuration (save raw Rhai content)
 * - POST /api/config/key-mappings - Add key mapping
 * - DELETE /api/config/key-mappings/:id - Delete key mapping
 * - GET /api/layers - List layers
 */

import { ApiClient } from '../api-client/client.js';
import type { TestCase, TestResult } from './api-tests.js';
import { extractData } from './api-tests.js';
import type { ScenarioDefinition } from './types.js';
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
 * Helper to ensure we have an active profile with valid device blocks
 */
async function ensureActiveProfile(client: ApiClient, profileName: string): Promise<void> {
  const profiles = await client.getProfiles();

  if (profiles.data.profiles.length === 0) {
    // Create new profile
    await client.createProfile(profileName);
    // Activate it first
    await client.activateProfile(profileName);
    // Then set valid config with device blocks using PUT /api/config
    await client.customRequest('PUT', '/api/config', z.any(), {
      content: `// Auto-generated config\ndevice_start("*");\ndevice_end();`,
    });
  } else {
    // Check if any profile is active
    const activeProfile = await client.getActiveProfile();
    if (!activeProfile.data.profile) {
      // Activate the first profile
      const firstProfile = profiles.data.profiles[0].name;
      await client.activateProfile(firstProfile);
      // Update its config to have device blocks
      await client.customRequest('PUT', '/api/config', z.any(), {
        content: `// Auto-generated config\ndevice_start("*");\ndevice_end();`,
      });
    } else {
      // Profile is already active, ensure it has device blocks
      try {
        await client.customRequest('GET', '/api/config', z.any());
      } catch {
        // If GET fails, update the config
        await client.customRequest('PUT', '/api/config', z.any(), {
          content: `// Auto-generated config\ndevice_start("*");\ndevice_end();`,
        });
      }
    }
  }
}

/**
 * Config response schema
 */
const ConfigSchema = z.object({
  profile: z.string(),
  base_mappings: z.array(z.string()),
  layers: z.array(
    z.object({
      id: z.string(),
      mapping_count: z.number(),
    })
  ),
});

type Config = z.infer<typeof ConfigSchema>;

/**
 * Layer info schema
 */
const LayerInfoSchema = z.object({
  id: z.string(),
  mapping_count: z.number(),
  mappings: z.array(z.string()),
});

const LayersResponseSchema = z.object({
  layers: z.array(LayerInfoSchema),
});

type LayersResponse = z.infer<typeof LayersResponseSchema>;

/**
 * Success response schema
 */
const SuccessResponseSchema = z.object({
  success: z.boolean(),
});

/**
 * Update config response schema
 */
const UpdateConfigResponseSchema = z.object({
  success: z.boolean(),
  message: z.string(),
  profile: z.string(),
  validation_error: z.string().optional(),
});

type UpdateConfigResponse = z.infer<typeof UpdateConfigResponseSchema>;

/**
 * Config & Layers test cases
 */
export const configLayersTestCases: TestCase[] = [
  // =================================================================
  // Config Retrieval Tests
  // =================================================================
  {
    id: 'config-001',
    name: 'GET /api/config - Get current configuration',
    endpoint: '/api/config',
    scenario: 'default',
    category: 'config',
    priority: 1,
    setup: async (client) => {
      await ensureActiveProfile(client, 'test-config-get');
    },
    execute: async (client) => {
      const response = await client.customRequest('GET', '/api/config', ConfigSchema);
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as Config;

      // Validate structure
      const hasRequiredFields =
        typeof actualData.profile === 'string' &&
        Array.isArray(actualData.base_mappings) &&
        Array.isArray(actualData.layers);

      if (!hasRequiredFields) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Missing required config fields',
        };
      }

      // Validate layers structure
      const layersValid = actualData.layers.every(
        (layer) =>
          typeof layer.id === 'string' && typeof layer.mapping_count === 'number'
      );

      if (!layersValid) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Invalid layer structure',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Config Update Tests
  // =================================================================
  {
    id: 'config-002',
    name: 'PUT /api/config - Update configuration with valid Rhai',
    endpoint: '/api/config',
    scenario: 'update_valid',
    category: 'config',
    priority: 1,
    setup: async (client) => {
      // Ensure we have a profile to update
      const profiles = await client.getProfiles();
      if (profiles.data.profiles.length === 0) {
        await client.createProfile('test-config-update');
      }
    },
    execute: async (client) => {
      const validRhai = `// Valid Rhai configuration
device_start("*");
  map("VK_A", "VK_B");
device_end();
`;
      const response = await client.customRequest(
        'PUT',
        '/api/config',
        UpdateConfigResponseSchema,
        { content: validRhai }
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as UpdateConfigResponse;

      if (actualData.success !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Config update should succeed',
        };
      }

      if (typeof actualData.message !== 'string') {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Missing message field',
        };
      }

      if (typeof actualData.profile !== 'string') {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Missing profile field',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'config-002b',
    name: 'PUT /api/config - Update configuration with invalid syntax',
    endpoint: '/api/config',
    scenario: 'update_invalid',
    category: 'config',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      const invalidRhai = `// Invalid Rhai - missing closing paren
map_key("base", "A", remap("B";
`;
      const response = await client.customRequest(
        'PUT',
        '/api/config',
        UpdateConfigResponseSchema,
        { content: invalidRhai }
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as UpdateConfigResponse;

      // The file is written but validation error is included
      if (actualData.success !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Update should succeed even with validation errors',
        };
      }

      if (!actualData.validation_error) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Should include validation_error field',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Key Mapping Tests
  // =================================================================
  {
    id: 'config-003',
    name: 'POST /api/config/key-mappings - Add simple key mapping',
    endpoint: '/api/config/key-mappings',
    scenario: 'add_simple',
    category: 'config',
    priority: 1,
    setup: async (client) => {
      await ensureActiveProfile(client, 'test-mapping');
    },
    execute: async (client) => {
      const response = await client.customRequest(
        'POST',
        '/api/config/key-mappings',
        SuccessResponseSchema,
        {
          layer: 'base',
          key: 'VK_A',
          action_type: 'simple',
          output: 'VK_B',
        }
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as { success: boolean };

      if (actualData.success !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Key mapping addition should succeed',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      // Clean up the mapping we just added
      try {
        await client.customRequest('DELETE', '/api/config/key-mappings/base:A', z.any());
      } catch {
        // Ignore cleanup errors
      }
    },
  },

  {
    id: 'config-003b',
    name: 'POST /api/config/key-mappings - Add tap-hold mapping',
    endpoint: '/api/config/key-mappings',
    scenario: 'add_tap_hold',
    category: 'config',
    priority: 2,
    setup: async (client) => {
      await ensureActiveProfile(client, 'test-mapping-taphold');
    },
    execute: async (client) => {
      const response = await client.customRequest(
        'POST',
        '/api/config/key-mappings',
        SuccessResponseSchema,
        {
          layer: 'base',
          key: 'VK_SPACE',
          action_type: 'tap_hold',
          tap: 'VK_SPACE',
          hold: 'VK_LCTRL',
          threshold_ms: 200,
        }
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as { success: boolean };

      if (actualData.success !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Tap-hold mapping addition should succeed',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: async (client) => {
      // Clean up the mapping we just added
      try {
        await client.customRequest('DELETE', '/api/config/key-mappings/base:Space', z.any());
      } catch {
        // Ignore cleanup errors
      }
    },
  },

  {
    id: 'config-003c',
    name: 'POST /api/config/key-mappings - Invalid action type',
    endpoint: '/api/config/key-mappings',
    scenario: 'invalid_action',
    category: 'config',
    priority: 3,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'POST',
          '/api/config/key-mappings',
          z.any(),
          {
            layer: 'base',
            key: 'C',
            action_type: 'invalid_type',
            output: 'D',
          }
        );
        return response;
      } catch (error: unknown) {
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
      const result = actual as { status?: number; data?: unknown };
      const actualData = result.data as { error?: { code?: string; message?: string }; success?: boolean };

      if (!actualData || !actualData.error || actualData.success !== false) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected error response for invalid action type`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'config-003d',
    name: 'POST /api/config/key-mappings - Missing required field',
    endpoint: '/api/config/key-mappings',
    scenario: 'missing_field',
    category: 'config',
    priority: 3,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'POST',
          '/api/config/key-mappings',
          z.any(),
          {
            layer: 'base',
            key: 'E',
            action_type: 'simple',
            // Missing 'output' field
          }
        );
        return response;
      } catch (error: unknown) {
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
      const result = actual as { status?: number; data?: unknown };
      const actualData = result.data as { error?: { code?: string; message?: string }; success?: boolean };

      if (!actualData || !actualData.error || actualData.success !== false) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected error response for missing field`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Delete Key Mapping Tests
  // =================================================================
  {
    id: 'config-004',
    name: 'DELETE /api/config/key-mappings/:id - Delete existing mapping',
    endpoint: '/api/config/key-mappings/:id',
    scenario: 'delete_success',
    category: 'config',
    priority: 1,
    setup: async (client) => {
      await ensureActiveProfile(client, 'test-delete-mapping');

      // Add a mapping to delete
      await client.customRequest(
        'POST',
        '/api/config/key-mappings',
        z.any(),
        {
          layer: 'base',
          key: 'VK_X',
          action_type: 'simple',
          output: 'VK_Y',
        }
      );
    },
    execute: async (client) => {
      const response = await client.customRequest(
        'DELETE',
        '/api/config/key-mappings/base:VK_X',
        SuccessResponseSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as { success: boolean };

      if (actualData.success !== true) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Key mapping deletion should succeed',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'config-004b',
    name: 'DELETE /api/config/key-mappings/:id - Invalid ID format',
    endpoint: '/api/config/key-mappings/:id',
    scenario: 'invalid_format',
    category: 'config',
    priority: 3,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'DELETE',
          '/api/config/key-mappings/invalid_format_no_colon',
          z.any()
        );
        return response;
      } catch (error: unknown) {
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
      const result = actual as { status?: number; data?: unknown };
      const actualData = result.data as { error?: { code?: string; message?: string }; success?: boolean };

      if (!actualData || !actualData.error || actualData.success !== false) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected error response for invalid ID format`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  {
    id: 'config-004c',
    name: 'DELETE /api/config/key-mappings/:id - Non-existent mapping',
    endpoint: '/api/config/key-mappings/:id',
    scenario: 'not_found',
    category: 'config',
    priority: 2,
    setup: noOpSetup,
    execute: async (client) => {
      try {
        const response = await client.customRequest(
          'DELETE',
          '/api/config/key-mappings/base:NonExistentKey',
          z.any()
        );
        return response;
      } catch (error: unknown) {
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
      const result = actual as { status?: number; data?: unknown };
      const actualData = result.data as { error?: { code?: string; message?: string }; success?: boolean };

      // Should return an error for non-existent mapping
      if (!actualData || !actualData.error || actualData.success !== false) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: `Expected error response for non-existent mapping`,
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },

  // =================================================================
  // Layers Tests
  // =================================================================
  {
    id: 'config-005',
    name: 'GET /api/layers - List all layers',
    endpoint: '/api/layers',
    scenario: 'default',
    category: 'config',
    priority: 1,
    setup: async (client) => {
      await ensureActiveProfile(client, 'test-layers-get');
    },
    execute: async (client) => {
      const response = await client.customRequest(
        'GET',
        '/api/layers',
        LayersResponseSchema
      );
      return {
        status: response.status,
        data: response.data,
      };
    },
    assert: (actual, expected) => {
      const actualData = extractData(actual) as LayersResponse;

      if (!Array.isArray(actualData.layers)) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'layers should be an array',
        };
      }

      // Validate each layer has required fields
      const layersValid = actualData.layers.every(
        (layer) =>
          typeof layer.id === 'string' &&
          typeof layer.mapping_count === 'number' &&
          Array.isArray(layer.mappings)
      );

      if (!layersValid) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Invalid layer structure',
        };
      }

      // Should at least have 'base' layer
      const hasBaseLayer = actualData.layers.some((layer) => layer.id === 'base');
      if (!hasBaseLayer) {
        return {
          passed: false,
          actualData,
          expected: expected.body,
          error: 'Should include base layer',
        };
      }

      return {
        passed: true,
        actualData,
        expected: expected.body,
      };
    },
    cleanup: noOpCleanup,
  },
];
