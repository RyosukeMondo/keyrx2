/**
 * Example: WebSocket Event Test
 *
 * This example demonstrates how to test WebSocket connections,
 * subscriptions, and real-time event notifications.
 *
 * Use this pattern for:
 * - WebSocket connection tests
 * - Channel subscription tests
 * - Real-time event notification tests
 * - Reconnection and resilience tests
 */

import { TestCase } from '../test-executor/types.js';
import { ApiClient } from '../api-client/client.js';
import { WebSocketClient } from '../api-client/websocket-client.js';

/**
 * Example 1: Basic WebSocket Connection
 *
 * Tests connecting and disconnecting from WebSocket endpoint.
 */
export function getWebSocketConnectionExample(): TestCase {
  return {
    id: 'example-websocket-001',
    name: 'WebSocket - connect and disconnect',
    endpoint: '/ws',
    scenario: 'connection',
    category: 'websocket',
    priority: 1,

    execute: async (client: ApiClient) => {
      const wsUrl = 'ws://localhost:9867/ws';
      const wsClient = new WebSocketClient(wsUrl);

      try {
        // Step 1: Connect
        await wsClient.connect();
        const connected = wsClient.isConnected();

        // Step 2: Disconnect
        await wsClient.disconnect();
        const disconnected = !wsClient.isConnected();

        return {
          connected,
          disconnected,
          success: connected && disconnected,
        };
      } catch (error) {
        return {
          success: false,
          error: error.message,
        };
      } finally {
        // Ensure cleanup even if test fails
        if (wsClient.isConnected()) {
          await wsClient.disconnect();
        }
      }
    },

    assert: (response, expected) => {
      return response.success === true;
    },
  };
}

/**
 * Example 2: Channel Subscription
 *
 * Tests subscribing to a WebSocket channel.
 */
export function getWebSocketSubscriptionExample(): TestCase {
  return {
    id: 'example-websocket-002',
    name: 'WebSocket - subscribe to channel',
    endpoint: '/ws',
    scenario: 'subscription',
    category: 'websocket',
    priority: 2,

    execute: async (client: ApiClient) => {
      const wsUrl = 'ws://localhost:9867/ws';
      const wsClient = new WebSocketClient(wsUrl);

      try {
        // Step 1: Connect
        await wsClient.connect();

        // Step 2: Subscribe to devices channel
        const subscription = await wsClient.subscribe('devices');

        // Step 3: Verify subscription
        const hasSubscription = wsClient.hasSubscription('devices');

        // Step 4: Unsubscribe
        await wsClient.unsubscribe('devices');
        const noSubscription = !wsClient.hasSubscription('devices');

        return {
          subscribed: subscription.success,
          hasSubscription,
          unsubscribed: noSubscription,
          success: subscription.success && hasSubscription && noSubscription,
        };
      } catch (error) {
        return {
          success: false,
          error: error.message,
        };
      } finally {
        await wsClient.disconnect();
      }
    },

    assert: (response, expected) => {
      return response.success === true;
    },
  };
}

/**
 * Example 3: Event Notification
 *
 * Tests receiving events via WebSocket when resources change.
 */
export function getWebSocketEventExample(): TestCase {
  return {
    id: 'example-websocket-003',
    name: 'WebSocket - receive device update event',
    endpoint: '/ws',
    scenario: 'device_event',
    category: 'websocket',
    priority: 2,

    execute: async (client: ApiClient) => {
      const wsUrl = 'ws://localhost:9867/ws';
      const wsClient = new WebSocketClient(wsUrl);

      try {
        // Step 1: Connect and subscribe
        await wsClient.connect();
        await wsClient.subscribe('devices');

        // Step 2: Get a device to update
        const devices = await client.getDevices();
        if (devices.length === 0) {
          return {
            success: false,
            error: 'No devices available',
          };
        }

        const deviceId = devices[0].id;
        const originalEnabled = devices[0].enabled;

        // Step 3: Set up event listener
        const eventPromise = wsClient.waitForEvent(
          (msg) => {
            return (
              msg.channel === 'devices' &&
              msg.event === 'device_updated' &&
              msg.data?.device_id === deviceId
            );
          },
          5000 // 5 second timeout
        );

        // Step 4: Trigger the event via REST API
        await client.patchDevice(deviceId, { enabled: !originalEnabled });

        // Step 5: Wait for event
        const event = await eventPromise;

        // Step 6: Restore original state
        await client.patchDevice(deviceId, { enabled: originalEnabled });

        return {
          eventReceived: !!event,
          eventChannel: event?.channel,
          eventType: event?.event,
          deviceId: event?.data?.device_id,
          success: !!event && event.channel === 'devices',
        };
      } catch (error) {
        return {
          success: false,
          error: error.message,
        };
      } finally {
        await wsClient.disconnect();
      }
    },

    assert: (response, expected) => {
      return response.success === true && response.eventReceived === true;
    },
  };
}

/**
 * Example 4: Multiple Events
 *
 * Tests receiving multiple events in sequence.
 */
export function getWebSocketMultipleEventsExample(): TestCase {
  return {
    id: 'example-websocket-004',
    name: 'WebSocket - receive multiple events',
    endpoint: '/ws',
    scenario: 'multiple_events',
    category: 'websocket',
    priority: 2,

    setup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      // Clean up test profiles
      try {
        await client.deleteProfile('ws-test-profile-1');
        await client.deleteProfile('ws-test-profile-2');
      } catch {
        // Profiles don't exist, that's fine
      }
    },

    execute: async (client: ApiClient) => {
      const wsUrl = 'ws://localhost:9867/ws';
      const wsClient = new WebSocketClient(wsUrl);
      const events = [];

      try {
        // Step 1: Connect and subscribe to profiles
        await wsClient.connect();
        await wsClient.subscribe('profiles');

        // Step 2: Set up event collector
        wsClient.on('message', (msg) => {
          if (msg.channel === 'profiles') {
            events.push({
              event: msg.event,
              timestamp: Date.now(),
            });
          }
        });

        // Step 3: Trigger multiple events
        await client.createProfile('ws-test-profile-1', { template: 'basic' });
        await new Promise((resolve) => setTimeout(resolve, 100)); // Small delay

        await client.createProfile('ws-test-profile-2', { template: 'basic' });
        await new Promise((resolve) => setTimeout(resolve, 100));

        await client.activateProfile('ws-test-profile-1');
        await new Promise((resolve) => setTimeout(resolve, 100));

        // Wait for events to arrive
        await new Promise((resolve) => setTimeout(resolve, 500));

        return {
          eventCount: events.length,
          events,
          success: events.length >= 3,
        };
      } catch (error) {
        return {
          success: false,
          error: error.message,
          eventCount: events.length,
        };
      } finally {
        await wsClient.disconnect();
      }
    },

    assert: (response, expected) => {
      // Should receive at least 3 events (2 creates, 1 activate)
      return response.success === true && response.eventCount >= 3;
    },

    cleanup: async () => {
      const client = new ApiClient({ baseUrl: 'http://localhost:9867' });

      try {
        await client.deleteProfile('ws-test-profile-1');
        await client.deleteProfile('ws-test-profile-2');
      } catch (error) {
        console.warn('Cleanup warning:', error.message);
      }
    },
  };
}

/**
 * Example 5: Reconnection Test
 *
 * Tests WebSocket reconnection and subscription restoration.
 */
export function getWebSocketReconnectionExample(): TestCase {
  return {
    id: 'example-websocket-005',
    name: 'WebSocket - reconnection and subscription restoration',
    endpoint: '/ws',
    scenario: 'reconnection',
    category: 'websocket',
    priority: 2,

    execute: async (client: ApiClient) => {
      const wsUrl = 'ws://localhost:9867/ws';
      const wsClient = new WebSocketClient(wsUrl, {
        autoReconnect: true,
        reconnectInterval: 1000,
      });

      try {
        // Step 1: Connect and subscribe
        await wsClient.connect();
        await wsClient.subscribe('devices');
        await wsClient.subscribe('profiles');

        const subscriptions1 = ['devices', 'profiles'].every((ch) =>
          wsClient.hasSubscription(ch)
        );

        // Step 2: Disconnect
        await wsClient.disconnect();
        const disconnected = !wsClient.isConnected();

        // Step 3: Reconnect
        await wsClient.connect();
        const reconnected = wsClient.isConnected();

        // Step 4: Check subscriptions restored
        // Note: Manual resubscription may be needed depending on implementation
        const subscriptions2 = ['devices', 'profiles'].every((ch) =>
          wsClient.hasSubscription(ch)
        );

        return {
          initialConnection: true,
          initialSubscriptions: subscriptions1,
          disconnected,
          reconnected,
          subscriptionsRestored: subscriptions2,
          success:
            subscriptions1 && disconnected && reconnected && subscriptions2,
        };
      } catch (error) {
        return {
          success: false,
          error: error.message,
        };
      } finally {
        await wsClient.disconnect();
      }
    },

    assert: (response, expected) => {
      return (
        response.success === true &&
        response.reconnected === true &&
        response.subscriptionsRestored === true
      );
    },
  };
}

/**
 * Key Points for WebSocket Tests:
 *
 * 1. Connection Management:
 *    - Always use try-finally to ensure disconnect
 *    - Check connection state before operations
 *    - Handle connection failures gracefully
 *
 * 2. Event Handling:
 *    - Use waitForEvent() with proper predicates
 *    - Set appropriate timeouts (5-10 seconds)
 *    - Collect events with on('message') for multiple events
 *
 * 3. Subscriptions:
 *    - Subscribe before triggering events
 *    - Verify subscription with hasSubscription()
 *    - Unsubscribe in cleanup if needed
 *
 * 4. Timing:
 *    - Add small delays between rapid operations
 *    - Use waitForEvent() instead of fixed sleeps
 *    - Account for network latency in timeouts
 *
 * 5. Error Handling:
 *    - Return error details in response
 *    - Don't let connection errors crash tests
 *    - Always disconnect in finally block
 */

/**
 * Common WebSocket Test Patterns:
 *
 * 1. Connection Test:
 *    Connect → Verify → Disconnect → Verify
 *
 * 2. Subscription Test:
 *    Connect → Subscribe → Verify → Unsubscribe → Verify
 *
 * 3. Event Test:
 *    Connect → Subscribe → Trigger → Wait → Verify
 *
 * 4. Multiple Events:
 *    Connect → Subscribe → Trigger N times → Collect → Verify count
 *
 * 5. Reconnection:
 *    Connect → Subscribe → Disconnect → Reconnect → Verify subscriptions
 */

/**
 * Available Channels:
 *
 * - 'devices': Device add/remove/update events
 * - 'profiles': Profile create/activate/delete events
 * - 'metrics': Metrics update events
 * - 'state': Daemon state change events
 */
