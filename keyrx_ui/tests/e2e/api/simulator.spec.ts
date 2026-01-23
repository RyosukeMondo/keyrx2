/**
 * E2E Simulation API Tests
 *
 * Tests for keyrx_daemon simulation REST API endpoints.
 * Uses Playwright's APIRequestContext to test simulation functionality.
 *
 * These tests verify:
 * - Loading profiles for simulation
 * - Built-in scenarios (tap-hold under/over threshold)
 * - DSL event simulation
 * - Deterministic results with seed
 * - Error handling (unknown scenario, no profile)
 * - Reset endpoint
 */

import { test, expect } from '@playwright/test';
import { createApiHelpers, ApiHelpers } from '../fixtures/api';

/**
 * Test fixtures with API helpers
 */
let api: ApiHelpers;

test.beforeAll(async ({ request }) => {
  api = createApiHelpers(request, 'http://localhost:9867');
  await api.waitForReady(30000);
});

test.describe('Simulation Profile Management', () => {
  const testProfileName = `E2E-Sim-Test-${Date.now()}`;

  test.afterAll(async () => {
    // Cleanup test profile
    try {
      await api.deleteProfile(testProfileName);
    } catch (err) {
      // Profile may not exist
    }
  });

  test('load-profile endpoint succeeds with valid profile', async () => {
    // Create a test profile
    const profile = await api.createProfile(testProfileName, 'blank');
    expect(profile.name).toBe(testProfileName);

    // Update with tap-hold configuration
    const config = `
device_start("*");
  map("VK_A", "VK_ESCAPE");
  tap_hold("VK_B", "VK_ESCAPE", "VK_LEFTCTRL", 200);
device_end();
`;
    await api.updateProfileConfig(testProfileName, config);

    // Activate the profile
    await api.activateProfile(testProfileName);

    // Load profile for simulation
    const result = await api.loadSimulatorProfile(testProfileName);

    expect(result.success).toBe(true);
    expect(result.message).toContain(testProfileName);
  });

  test('load-profile returns 404 for non-existent profile', async ({ request }) => {
    const response = await request.post('http://localhost:9867/api/simulator/load-profile', {
      data: { name: 'NonExistentProfile12345' },
    });

    expect(response.status()).toBe(404);
  });
});

test.describe('Built-in Scenarios', () => {
  const scenarioProfileName = `E2E-Scenario-Test-${Date.now()}`;

  test.beforeAll(async () => {
    // Create profile with tap-hold configuration
    await api.createProfile(scenarioProfileName, 'blank');

    const config = `
device_start("*");
  tap_hold("VK_A", "VK_ESCAPE", "VK_LEFTCTRL", 200);
device_end();
`;
    await api.updateProfileConfig(scenarioProfileName, config);
    await api.activateProfile(scenarioProfileName);
    await api.loadSimulatorProfile(scenarioProfileName);
  });

  test.afterAll(async () => {
    // Cleanup
    try {
      await api.deleteProfile(scenarioProfileName);
    } catch (err) {
      // Profile may not exist
    }
  });

  test('tap-hold-under-threshold scenario produces tap output', async () => {
    const result = await api.simulateScenario('tap-hold-under-threshold');

    expect(result.success).toBe(true);
    expect(result.outputs).toBeDefined();
    expect(Array.isArray(result.outputs)).toBe(true);

    // Should contain Escape key (tap action)
    const escapeEvents = result.outputs.filter(e => e.key === 'Escape');
    expect(escapeEvents.length).toBeGreaterThan(0);

    // Verify event structure
    if (result.outputs.length > 0) {
      expect(result.outputs[0]).toHaveProperty('key');
      expect(result.outputs[0]).toHaveProperty('event_type');
      expect(result.outputs[0]).toHaveProperty('timestamp_us');
      expect(['press', 'release']).toContain(result.outputs[0].event_type);
    }
  });

  test('tap-hold-over-threshold scenario produces hold output', async () => {
    const result = await api.simulateScenario('tap-hold-over-threshold');

    expect(result.success).toBe(true);
    expect(result.outputs).toBeDefined();
    expect(Array.isArray(result.outputs)).toBe(true);

    // Should contain Control key (hold action)
    const controlEvents = result.outputs.filter(e => e.key === 'LeftControl');
    expect(controlEvents.length).toBeGreaterThan(0);
  });

  test('unknown scenario returns error', async ({ request }) => {
    const response = await request.post('http://localhost:9867/api/simulator/events', {
      data: { scenario: 'unknown-scenario-name' },
    });

    // Should fail with appropriate error
    expect([400, 404, 500]).toContain(response.status());
  });

  test('run all scenarios endpoint returns results summary', async () => {
    const result = await api.runAllScenarios();

    expect(result.success).toBe(true);
    expect(result.scenarios).toBeDefined();
    expect(Array.isArray(result.scenarios)).toBe(true);
    expect(result.total).toBeGreaterThan(0);
    expect(result.passed).toBeGreaterThanOrEqual(0);
    expect(result.failed).toBeGreaterThanOrEqual(0);
    expect(result.passed + result.failed).toBe(result.total);

    // Verify scenario result structure
    if (result.scenarios.length > 0) {
      const scenario = result.scenarios[0];
      expect(scenario).toHaveProperty('scenario');
      expect(scenario).toHaveProperty('passed');
      expect(scenario).toHaveProperty('input');
      expect(scenario).toHaveProperty('output');
      expect(typeof scenario.passed).toBe('boolean');
    }
  });
});

test.describe('DSL Event Simulation', () => {
  const dslProfileName = `E2E-DSL-Test-${Date.now()}`;

  test.beforeAll(async () => {
    // Create profile with simple mapping
    await api.createProfile(dslProfileName, 'blank');

    const config = `
device_start("*");
  map("VK_A", "VK_B");
  map("VK_C", "VK_D");
device_end();
`;
    await api.updateProfileConfig(dslProfileName, config);
    await api.activateProfile(dslProfileName);
    await api.loadSimulatorProfile(dslProfileName);
  });

  test.afterAll(async () => {
    // Cleanup
    try {
      await api.deleteProfile(dslProfileName);
    } catch (err) {
      // Profile may not exist
    }
  });

  test('DSL simulation produces output events', async () => {
    const result = await api.simulateEventsDsl('press:A,wait:50,release:A');

    expect(result.success).toBe(true);
    expect(result.outputs).toBeDefined();
    expect(Array.isArray(result.outputs)).toBe(true);
    expect(result.outputs.length).toBeGreaterThan(0);

    // Should have press and release events
    const pressEvents = result.outputs.filter(e => e.event_type === 'press');
    const releaseEvents = result.outputs.filter(e => e.event_type === 'release');

    expect(pressEvents.length).toBeGreaterThan(0);
    expect(releaseEvents.length).toBeGreaterThan(0);
  });

  test('DSL with seed produces deterministic results', async () => {
    const seed = 12345;
    const dsl = 'press:A,wait:100,release:A,press:C,wait:50,release:C';

    // Run simulation twice with same seed
    const result1 = await api.simulateEventsDsl(dsl, seed);
    const result2 = await api.simulateEventsDsl(dsl, seed);

    expect(result1.success).toBe(true);
    expect(result2.success).toBe(true);

    // Results should be identical
    expect(result1.outputs.length).toBe(result2.outputs.length);

    // Compare each event
    for (let i = 0; i < result1.outputs.length; i++) {
      expect(result1.outputs[i].key).toBe(result2.outputs[i].key);
      expect(result1.outputs[i].event_type).toBe(result2.outputs[i].event_type);
      // Timestamps should be deterministic with same seed
      expect(result1.outputs[i].timestamp_us).toBe(result2.outputs[i].timestamp_us);
    }
  });

  test('DSL with different seeds may produce different timing', async () => {
    const dsl = 'press:A,wait:100,release:A';

    // Run with different seeds
    const result1 = await api.simulateEventsDsl(dsl, 111);
    const result2 = await api.simulateEventsDsl(dsl, 222);

    expect(result1.success).toBe(true);
    expect(result2.success).toBe(true);

    // Both should have events, but timing might differ
    expect(result1.outputs.length).toBeGreaterThan(0);
    expect(result2.outputs.length).toBeGreaterThan(0);
  });

  test('complex DSL sequence produces correct event count', async () => {
    const dsl = 'press:A,release:A,press:C,release:C';

    const result = await api.simulateEventsDsl(dsl);

    expect(result.success).toBe(true);

    // Should have 4 output events (2 key presses mapped to 2 different keys)
    // A -> B, C -> D
    expect(result.outputs.length).toBe(4);
  });
});

test.describe('Error Handling', () => {
  test('simulate without loaded profile returns error', async () => {
    // Reset simulator to clear any loaded profile
    await api.resetSimulator();

    // Try to simulate without loading a profile first
    const response = await api.simulateEventsDsl('press:A,release:A').catch(err => {
      // Error is expected
      expect(err.message).toContain('failed');
      return null;
    });

    // If the API returns a response instead of throwing, check for error status
    if (response) {
      expect(response.success).toBe(false);
    }
  });

  test('invalid DSL syntax returns error', async ({ request }) => {
    const invalidDsl = 'invalid syntax {{{';

    const response = await request.post('http://localhost:9867/api/simulator/events', {
      data: { dsl: invalidDsl },
    });

    // Should return error status
    expect([400, 500]).toContain(response.status());
  });
});

test.describe('Simulator Reset', () => {
  const resetProfileName = `E2E-Reset-Test-${Date.now()}`;

  test.beforeAll(async () => {
    // Create and load a profile
    await api.createProfile(resetProfileName, 'blank');

    const config = `
device_start("*");
  map("VK_A", "VK_B");
device_end();
`;
    await api.updateProfileConfig(resetProfileName, config);
    await api.activateProfile(resetProfileName);
  });

  test.afterAll(async () => {
    // Cleanup
    try {
      await api.deleteProfile(resetProfileName);
    } catch (err) {
      // Profile may not exist
    }
  });

  test('reset endpoint clears simulator state', async () => {
    // Load profile
    await api.loadSimulatorProfile(resetProfileName);

    // Simulate some events
    await api.simulateEventsDsl('press:A,release:A');

    // Reset simulator
    const result = await api.resetSimulator();

    expect(result.success).toBe(true);

    // After reset, simulating without loading profile should fail
    const response = await api.simulateEventsDsl('press:A,release:A').catch(err => {
      // Error is expected
      expect(err.message).toContain('failed');
      return null;
    });

    // If the API returns a response instead of throwing, check for error status
    if (response) {
      expect(response.success).toBe(false);
    }
  });

  test('can reload profile after reset', async () => {
    // Reset
    await api.resetSimulator();

    // Load profile again
    const loadResult = await api.loadSimulatorProfile(resetProfileName);
    expect(loadResult.success).toBe(true);

    // Simulate should work now
    const simResult = await api.simulateEventsDsl('press:A,release:A');
    expect(simResult.success).toBe(true);
  });
});
