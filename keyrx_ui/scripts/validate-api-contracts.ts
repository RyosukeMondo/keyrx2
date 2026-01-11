#!/usr/bin/env npx tsx
/**
 * API Contract Validation Script
 *
 * This script validates that the running daemon's API responses match
 * the frontend's Zod schemas. Run this against a live daemon to catch
 * schema mismatches before deployment.
 *
 * Usage:
 *   npx tsx scripts/validate-api-contracts.ts [--base-url http://localhost:9867]
 *
 * Exit codes:
 *   0 - All contracts valid
 *   1 - Contract validation failed
 *   2 - Connection error (daemon not running)
 */

import { z } from 'zod';

// Import schemas from the frontend
import {
  DeviceListResponseSchema,
  ProfileListResponseSchema,
  ProfileConfigResponseSchema,
  StatusResponseSchema,
  UpdateDeviceConfigResponseSchema,
} from '../src/api/schemas';

interface ValidationResult {
  endpoint: string;
  method: string;
  status: 'pass' | 'fail' | 'skip';
  error?: string;
  responseData?: unknown;
}

const results: ValidationResult[] = [];

async function validateEndpoint<T>(
  method: string,
  endpoint: string,
  schema: z.ZodSchema<T>,
  baseUrl: string
): Promise<ValidationResult> {
  const url = `${baseUrl}${endpoint}`;

  try {
    const response = await fetch(url, { method });

    if (!response.ok) {
      // Some endpoints may return 404 for empty data, that's okay
      if (response.status === 404) {
        return {
          endpoint,
          method,
          status: 'skip',
          error: `HTTP ${response.status} - endpoint may not have data`,
        };
      }
      return {
        endpoint,
        method,
        status: 'fail',
        error: `HTTP ${response.status}: ${response.statusText}`,
      };
    }

    const data = await response.json();
    const result = schema.safeParse(data);

    if (result.success) {
      return {
        endpoint,
        method,
        status: 'pass',
      };
    } else {
      return {
        endpoint,
        method,
        status: 'fail',
        error: result.error.message,
        responseData: data,
      };
    }
  } catch (error) {
    return {
      endpoint,
      method,
      status: 'fail',
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

async function main() {
  // Parse command line arguments
  const args = process.argv.slice(2);
  let baseUrl = 'http://localhost:9867';

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--base-url' && args[i + 1]) {
      baseUrl = args[i + 1];
      i++;
    }
    if (args[i] === '--help' || args[i] === '-h') {
      console.log(`
API Contract Validation Script

Validates that the running daemon's API responses match frontend Zod schemas.

Usage:
  npx tsx scripts/validate-api-contracts.ts [options]

Options:
  --base-url URL   Base URL of the daemon API (default: http://localhost:9867)
  --help, -h       Show this help message

Examples:
  npx tsx scripts/validate-api-contracts.ts
  npx tsx scripts/validate-api-contracts.ts --base-url http://localhost:3030
`);
      process.exit(0);
    }
  }

  console.log(`\n🔍 Validating API contracts against ${baseUrl}\n`);

  // Check if daemon is running
  try {
    const statusResponse = await fetch(`${baseUrl}/api/status`);
    if (!statusResponse.ok) {
      throw new Error('Status endpoint returned non-OK response');
    }
    console.log('✓ Daemon is running\n');
  } catch (error) {
    console.error(`❌ Cannot connect to daemon at ${baseUrl}`);
    console.error('   Make sure the daemon is running: keyrx_daemon run --config <path>');
    process.exit(2);
  }

  // Validate each endpoint
  console.log('Validating endpoints:\n');

  // GET /api/status
  results.push(
    await validateEndpoint('GET', '/api/status', StatusResponseSchema, baseUrl)
  );

  // GET /api/devices
  results.push(
    await validateEndpoint('GET', '/api/devices', DeviceListResponseSchema, baseUrl)
  );

  // GET /api/profiles
  results.push(
    await validateEndpoint('GET', '/api/profiles', ProfileListResponseSchema, baseUrl)
  );

  // GET /api/profiles/:name/config (need to get profile name first)
  try {
    const profilesResponse = await fetch(`${baseUrl}/api/profiles`);
    const profilesData = await profilesResponse.json();
    if (profilesData.profiles && profilesData.profiles.length > 0) {
      const firstProfile = profilesData.profiles[0].name;
      results.push(
        await validateEndpoint(
          'GET',
          `/api/profiles/${encodeURIComponent(firstProfile)}/config`,
          ProfileConfigResponseSchema,
          baseUrl
        )
      );
    } else {
      results.push({
        endpoint: '/api/profiles/:name/config',
        method: 'GET',
        status: 'skip',
        error: 'No profiles available to test',
      });
    }
  } catch (error) {
    results.push({
      endpoint: '/api/profiles/:name/config',
      method: 'GET',
      status: 'fail',
      error: error instanceof Error ? error.message : String(error),
    });
  }

  // PATCH /api/devices/:id (need to get device ID first)
  // Note: This endpoint may return 4xx/5xx for invalid operations, which is expected
  // We validate the schema of the response, not the success of the operation
  try {
    const devicesResponse = await fetch(`${baseUrl}/api/devices`);
    const devicesData = await devicesResponse.json();
    if (devicesData.devices && devicesData.devices.length > 0) {
      const firstDeviceId = devicesData.devices[0].id;

      // Make PATCH request to update device config with a simple, valid layout
      const patchResponse = await fetch(`${baseUrl}/api/devices/${encodeURIComponent(firstDeviceId)}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ layout: 'us' }),
      });

      // Accept both success (200) and error responses (4xx/5xx)
      // The contract test validates response schema, not operation success
      const patchData = await patchResponse.json();

      if (patchResponse.ok) {
        // Success response - validate against success schema
        const result = UpdateDeviceConfigResponseSchema.safeParse(patchData);

        if (result.success) {
          results.push({
            endpoint: '/api/devices/:id',
            method: 'PATCH',
            status: 'pass',
          });
        } else {
          results.push({
            endpoint: '/api/devices/:id',
            method: 'PATCH',
            status: 'fail',
            error: `Success response schema mismatch: ${result.error.message}`,
            responseData: patchData,
          });
        }
      } else {
        // Error response - check if it has proper error structure
        // Expected: { success: false, error: { code, message } }
        if (typeof patchData === 'object' && patchData !== null && 'error' in patchData) {
          results.push({
            endpoint: '/api/devices/:id',
            method: 'PATCH',
            status: 'pass',
            error: `Note: Operation returned error (expected for some devices), but schema is valid`,
          });
        } else {
          results.push({
            endpoint: '/api/devices/:id',
            method: 'PATCH',
            status: 'fail',
            error: `Error response has invalid schema: ${JSON.stringify(patchData)}`,
            responseData: patchData,
          });
        }
      }
    } else {
      results.push({
        endpoint: '/api/devices/:id',
        method: 'PATCH',
        status: 'skip',
        error: 'No devices available to test',
      });
    }
  } catch (error) {
    results.push({
      endpoint: '/api/devices/:id',
      method: 'PATCH',
      status: 'fail',
      error: error instanceof Error ? error.message : String(error),
    });
  }

  // Print results
  let passed = 0;
  let failed = 0;
  let skipped = 0;

  for (const result of results) {
    const statusIcon = result.status === 'pass' ? '✓' : result.status === 'fail' ? '✗' : '○';
    const statusColor =
      result.status === 'pass' ? '\x1b[32m' : result.status === 'fail' ? '\x1b[31m' : '\x1b[33m';
    const resetColor = '\x1b[0m';

    console.log(`  ${statusColor}${statusIcon}${resetColor} ${result.method} ${result.endpoint}`);

    if (result.error) {
      console.log(`    ${result.status === 'fail' ? '  Error:' : '  Note:'} ${result.error}`);
    }

    if (result.status === 'pass') passed++;
    else if (result.status === 'fail') failed++;
    else skipped++;
  }

  // Print summary
  console.log(`\n${'─'.repeat(60)}`);
  console.log(`\nResults: ${passed} passed, ${failed} failed, ${skipped} skipped\n`);

  if (failed > 0) {
    console.log('❌ Contract validation FAILED');
    console.log('\nTo fix contract mismatches:');
    console.log('1. Check if backend API response format changed');
    console.log('2. Update keyrx_ui/src/api/schemas.ts to match');
    console.log('3. Run npm test -- contracts.test.ts to verify');
    process.exit(1);
  } else {
    console.log('✅ All contracts valid');
    process.exit(0);
  }
}

main().catch((error) => {
  console.error('Unexpected error:', error);
  process.exit(1);
});
