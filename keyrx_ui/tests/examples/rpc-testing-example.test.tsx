/**
 * Example: Testing Components with RPC Communication
 *
 * This file demonstrates how to test components that use WebSocket RPC
 * communication with the daemon. It shows best practices for:
 * - Setting up WebSocket mocks
 * - Handling RPC requests and responses
 * - Testing error scenarios
 * - Proper cleanup
 *
 * Use these patterns when writing tests for components that use:
 * - useUnifiedApi hook
 * - RpcClient
 * - Any component making WebSocket RPC calls
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';
import {
  setupMockWebSocket,
  cleanupMockWebSocket,
  getMockWebSocket,
  simulateConnected,
  sendRpcResponse,
  sendRpcError,
  waitForRpcRequest,
  WS_URL,
} from '../testUtils';

// Mock uuid for deterministic test IDs
let uuidCounter = 0;
vi.mock('uuid', () => ({
  v4: () => `test-uuid-${uuidCounter++}`,
}));

describe('RPC Testing Examples', () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    uuidCounter = 0;
    await setupMockWebSocket();
  });

  afterEach(() => {
    cleanupMockWebSocket();
  });

  describe('Query/Command Pattern', () => {
    it('should handle successful query response', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Wait for WebSocket connection
      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();

      // Wait for connection
      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      // Make a query in the background
      const queryPromise = result.current.query('get_profiles');

      // Wait for the query request and respond
      const request = await waitForRpcRequest('get_profiles');
      expect(request.method).toBe('get_profiles');

      // Send successful response using the new helper
      const mockProfiles = [
        { name: 'default', active: true, path: '/path/to/default.rhai' },
        { name: 'gaming', active: false, path: '/path/to/gaming.rhai' },
      ];
      sendRpcResponse(request.id, mockProfiles);

      // Verify the query resolved with the correct data
      const profiles = await queryPromise;
      expect(profiles).toEqual(mockProfiles);
    });

    it('should handle RPC error response', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Wait for connection
      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();
      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      // Make a command that will fail
      const commandPromise = result.current.command('activate_profile', { name: 'nonexistent' });

      // Wait for the command request
      const request = await waitForRpcRequest('activate_profile');
      expect(request.params).toEqual({ name: 'nonexistent' });

      // Send error response using the new helper
      sendRpcError(request.id, -32602, 'Profile not found', { profile: 'nonexistent' });

      // Verify the command rejected with the error
      await expect(commandPromise).rejects.toThrow('Profile not found');
    });

    it('should handle command with parameters', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      // Wait for connection
      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();
      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      // Make a command with parameters
      const commandPromise = result.current.command('rename_profile', {
        old_name: 'default',
        new_name: 'custom',
      });

      // Wait for the request and verify parameters
      const request = await waitForRpcRequest('rename_profile');
      expect(request.params).toEqual({
        old_name: 'default',
        new_name: 'custom',
      });

      // Send success response (void for commands)
      sendRpcResponse(request.id, null);

      // Verify the command resolved
      await expect(commandPromise).resolves.toBeNull();
    });
  });

  describe('Error Scenarios', () => {
    it('should handle validation errors', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();
      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      // Make a query
      const queryPromise = result.current.query('get_profile_config', { name: 'invalid-name' });

      const request = await waitForRpcRequest('get_profile_config');

      // Send validation error
      sendRpcError(
        request.id,
        -32602, // Invalid params
        'Invalid profile name',
        {
          field: 'name',
          constraint: 'must not contain special characters',
        }
      );

      // Verify error is propagated
      await expect(queryPromise).rejects.toThrow('Invalid profile name');
    });

    it('should handle internal server errors', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();
      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      const commandPromise = result.current.command('set_profile_config', {
        name: 'default',
        source: 'invalid rhai code',
      });

      const request = await waitForRpcRequest('set_profile_config');

      // Send internal error
      sendRpcError(
        request.id,
        -32603, // Internal error
        'Failed to compile configuration',
        {
          line: 5,
          column: 10,
          message: 'Unexpected token',
        }
      );

      await expect(commandPromise).rejects.toThrow('Failed to compile configuration');
    });
  });

  describe('Multiple Requests', () => {
    it('should handle concurrent requests correctly', async () => {
      const { result } = renderHook(() => useUnifiedApi(WS_URL));

      const server = getMockWebSocket();
      await server.connected;
      await simulateConnected();
      await waitFor(() => {
        expect(result.current.isConnected).toBe(true);
      });

      // Make multiple concurrent requests
      const profilesPromise = result.current.query('get_profiles');
      const devicesPromise = result.current.query('get_devices');
      const configPromise = result.current.query('get_config');

      // Handle first request (profiles)
      const req1 = await waitForRpcRequest();
      sendRpcResponse(req1.id, [{ name: 'default' }]);

      // Handle second request (devices)
      const req2 = await waitForRpcRequest();
      sendRpcResponse(req2.id, [{ id: 'device-1', name: 'Keyboard' }]);

      // Handle third request (config)
      const req3 = await waitForRpcRequest();
      sendRpcResponse(req3.id, { code: '// config', hash: 'abc123' });

      // All requests should resolve correctly
      const [profiles, devices, config] = await Promise.all([
        profilesPromise,
        devicesPromise,
        configPromise,
      ]);

      expect(profiles).toHaveLength(1);
      expect(devices).toHaveLength(1);
      expect(config.code).toBe('// config');
    });
  });
});

/**
 * Best Practices Summary:
 *
 * 1. Always use setupMockWebSocket() in beforeEach
 * 2. Always use cleanupMockWebSocket() in afterEach
 * 3. Wait for server.connected before sending messages
 * 4. Use simulateConnected() to complete handshake
 * 5. Use waitForRpcRequest() to capture requests
 * 6. Use sendRpcResponse() for success responses
 * 7. Use sendRpcError() for error responses
 * 8. Always await promises to catch errors
 * 9. Use waitFor() for async state updates
 * 10. Test both success and error paths
 *
 * Common Pitfalls to Avoid:
 *
 * ❌ Sending responses before waiting for requests
 * ❌ Forgetting to call simulateConnected()
 * ❌ Not waiting for connection state
 * ❌ Hardcoding request IDs (use waitForRpcRequest)
 * ❌ Not cleaning up WebSocket mocks
 * ❌ Sending invalid JSON structures
 * ❌ Not testing error scenarios
 */
