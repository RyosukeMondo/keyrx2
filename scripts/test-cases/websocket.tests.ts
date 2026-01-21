/**
 * WebSocket API Test Cases
 *
 * Tests for WebSocket real-time event communication:
 * - WebSocket connection lifecycle (connect, disconnect)
 * - Channel subscription (devices, profiles, metrics, state)
 * - Event notifications (device changes, profile changes)
 * - Connection resilience (reconnection)
 */

import { ApiClient } from '../api-client/client.js';
import {
  WebSocketClient,
  ConnectionState,
  ConnectionError,
  TimeoutError,
  SubscriptionError,
} from '../api-client/websocket-client.js';
import type { TestCase, TestResult } from './api-tests.js';
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
 * WebSocket URL for tests (derived from API base URL)
 */
const getWebSocketUrl = (apiBaseUrl: string): string => {
  // Convert http://host:port/api to ws://host:port/ws
  const url = new URL(apiBaseUrl);
  const protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${url.host}/ws`;
};

/**
 * WebSocket test cases
 */
export const websocketTestCases: TestCase[] = [
  // =================================================================
  // Connection Tests
  // =================================================================
  {
    id: 'websocket-001',
    name: 'WebSocket - Connect and disconnect lifecycle',
    endpoint: '/ws',
    scenario: 'connection',
    category: 'websocket',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const wsUrl = getWebSocketUrl(client.baseUrl);
      const wsClient = new WebSocketClient({ url: wsUrl });

      try {
        // Connect
        await wsClient.connect(5000);
        const connectedState = wsClient.getState();
        const isConnected = wsClient.isConnected();

        // Disconnect
        wsClient.disconnect();
        const disconnectedState = wsClient.getState();

        return {
          status: 200,
          data: {
            success: true,
            connectedState,
            isConnected,
            disconnectedState,
          },
        };
      } catch (error) {
        return {
          status: 500,
          data: {
            success: false,
            error: error instanceof Error ? error.message : String(error),
          },
        };
      } finally {
        wsClient.disconnect();
      }
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        success: boolean;
        connectedState?: string;
        isConnected?: boolean;
        disconnectedState?: string;
        error?: string;
      };

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Connection failed: ${actualData.error}`,
        };
      }

      // Verify connected state
      if (actualData.connectedState !== ConnectionState.CONNECTED) {
        return {
          passed: false,
          actual,
          expected: { connectedState: ConnectionState.CONNECTED },
          error: `Expected state 'connected', got '${actualData.connectedState}'`,
        };
      }

      // Verify isConnected flag
      if (!actualData.isConnected) {
        return {
          passed: false,
          actual,
          expected: { isConnected: true },
          error: 'Expected isConnected to be true',
        };
      }

      // Verify disconnected state
      if (actualData.disconnectedState !== ConnectionState.CLOSED) {
        return {
          passed: false,
          actual,
          expected: { disconnectedState: ConnectionState.CLOSED },
          error: `Expected state 'closed', got '${actualData.disconnectedState}'`,
        };
      }

      return { passed: true, actual, expected: expected.body };
    },
    cleanup: noOpCleanup,
    expectedResponse: {
      status: 200,
      body: {
        success: true,
        connectedState: ConnectionState.CONNECTED,
        isConnected: true,
        disconnectedState: ConnectionState.CLOSED,
      },
    },
  },

  // =================================================================
  // Subscription Tests
  // =================================================================
  {
    id: 'websocket-002',
    name: 'WebSocket - Subscribe to channel and receive acknowledgment',
    endpoint: '/ws',
    scenario: 'subscription',
    category: 'websocket',
    priority: 1,
    setup: noOpSetup,
    execute: async (client) => {
      const wsUrl = getWebSocketUrl(client.baseUrl);
      const wsClient = new WebSocketClient({ url: wsUrl });

      try {
        // Connect
        await wsClient.connect(5000);

        // Subscribe to 'devices' channel
        await wsClient.subscribe('devices', 5000);

        // Verify subscription
        const subscriptions = wsClient.getSubscriptions();

        // Disconnect
        wsClient.disconnect();

        return {
          status: 200,
          data: {
            success: true,
            subscribed: subscriptions.has('devices'),
            subscriptions: Array.from(subscriptions),
          },
        };
      } catch (error) {
        return {
          status: 500,
          data: {
            success: false,
            error: error instanceof Error ? error.message : String(error),
            errorType: error instanceof Error ? error.constructor.name : 'Unknown',
          },
        };
      } finally {
        wsClient.disconnect();
      }
    },
    assert: (actual, expected) => {
      const actualData = actual as {
        success: boolean;
        subscribed?: boolean;
        subscriptions?: string[];
        error?: string;
        errorType?: string;
      };

      if (!actualData.success) {
        return {
          passed: false,
          actual,
          expected: expected.body,
          error: `Subscription failed: ${actualData.error} (${actualData.errorType})`,
        };
      }

      // Verify subscription successful
      if (!actualData.subscribed) {
        return {
          passed: false,
          actual,
          expected: { subscribed: true },
          error: 'Expected to be subscribed to devices channel',
        };
      }

      // Verify subscriptions list
      if (!actualData.subscriptions?.includes('devices')) {
        return {
          passed: false,
          actual,
          expected: { subscriptions: ['devices'] },
          error: `Expected subscriptions to include 'devices', got: ${actualData.subscriptions?.join(', ')}`,
        };
      }

      return { passed: true, actual, expected: expected.body };
    },
    cleanup: noOpCleanup,
    expectedResponse: {
      status: 200,
      body: {
        success: true,
        subscribed: true,
        subscriptions: ['devices'],
      },
    },
  },
];
