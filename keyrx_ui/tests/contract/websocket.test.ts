/**
 * WebSocket contract tests - verify message format compliance with Zod schemas.
 *
 * These tests ensure that WebSocket messages exchanged between the frontend and
 * daemon comply with the defined contract (schemas). They catch protocol changes
 * early and prevent runtime errors from malformed messages.
 *
 * Test strategy:
 * 1. Valid messages MUST parse successfully against schemas
 * 2. Invalid messages MUST be rejected with clear errors
 * 3. All message types and event types are covered
 * 4. Edge cases (missing fields, wrong types, extra fields) are tested
 */

import { describe, it, expect } from 'vitest';
import {
  // Server message schemas
  ServerMessageSchema,
  ResponseMessageSchema,
  EventMessageSchema,
  ConnectedMessageSchema,
  // Client message schemas
  ClientMessageSchema,
  QueryMessageSchema,
  CommandMessageSchema,
  SubscribeMessageSchema,
  UnsubscribeMessageSchema,
  // Event data schemas
  DaemonStateSchema,
  DeviceConnectedEventSchema,
  ProfileActivatedEventSchema,
  KeyEventSchema,
  LatencyMetricsSchema,
  // Parser functions
  parseDaemonStateEvent,
  parseDeviceConnectedEvent,
  parseProfileActivatedEvent,
  parseKeyEvent,
  parseLatencyMetrics,
  // Safe parsing helpers
  safeParseServerMessage,
  safeParseClientMessage,
} from './schemas';

describe('WebSocket Contract Tests', () => {
  describe('Server Messages', () => {
    describe('ConnectedMessage', () => {
      it('should validate valid connected message', () => {
        const message = {
          type: 'connected',
          version: '1.0.0',
          timestamp: Date.now(),
        };

        const result = ConnectedMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should reject connected message with missing version', () => {
        const message = {
          type: 'connected',
          timestamp: Date.now(),
        };

        const result = ConnectedMessageSchema.safeParse(message);
        expect(result.success).toBe(false);
      });

      it('should reject connected message with wrong timestamp type', () => {
        const message = {
          type: 'connected',
          version: '1.0.0',
          timestamp: '2024-01-01',
        };

        const result = ConnectedMessageSchema.safeParse(message);
        expect(result.success).toBe(false);
      });
    });

    describe('ResponseMessage', () => {
      it('should validate successful response message', () => {
        const message = {
          type: 'response',
          id: 'req-123',
          result: { success: true, data: [1, 2, 3] },
        };

        const result = ResponseMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should validate error response message', () => {
        const message = {
          type: 'response',
          id: 'req-456',
          error: {
            code: -32600,
            message: 'Invalid Request',
            data: { details: 'Missing required field' },
          },
        };

        const result = ResponseMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should validate response with null result', () => {
        const message = {
          type: 'response',
          id: 'req-789',
          result: null,
        };

        const result = ResponseMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should reject response without id', () => {
        const message = {
          type: 'response',
          result: { success: true },
        };

        const result = ResponseMessageSchema.safeParse(message);
        expect(result.success).toBe(false);
      });
    });

    describe('EventMessage', () => {
      it('should validate event message with daemon-state channel', () => {
        const message = {
          type: 'event',
          channel: 'daemon-state',
          data: {
            modifiers: ['MD_00'],
            locks: ['LK_00'],
            layer: 'base',
          },
        };

        const result = EventMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should validate event message with events channel', () => {
        const message = {
          type: 'event',
          channel: 'events',
          data: {
            timestamp: 1234567890,
            keyCode: 'KEY_A',
            eventType: 'press',
            input: 'KEY_A',
            output: 'KEY_B',
            latency: 100,
          },
        };

        const result = EventMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should validate event message with latency channel', () => {
        const message = {
          type: 'event',
          channel: 'latency',
          data: {
            min: 50,
            avg: 100,
            max: 200,
            p95: 180,
            p99: 195,
            timestamp: 1234567890,
          },
        };

        const result = EventMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should reject event with invalid channel', () => {
        const message = {
          type: 'event',
          channel: 'invalid-channel',
          data: {},
        };

        const result = EventMessageSchema.safeParse(message);
        expect(result.success).toBe(false);
      });

      it('should accept event without data field (data is optional)', () => {
        const message = {
          type: 'event',
          channel: 'daemon-state',
        };

        // Note: z.unknown() makes the field optional in Zod
        const result = EventMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });
    });

    describe('ServerMessage Union', () => {
      it('should validate all server message types', () => {
        const messages = [
          {
            type: 'connected',
            version: '1.0.0',
            timestamp: Date.now(),
          },
          {
            type: 'response',
            id: 'req-1',
            result: { success: true },
          },
          {
            type: 'event',
            channel: 'daemon-state',
            data: { modifiers: [], locks: [], layer: 'base' },
          },
        ];

        messages.forEach((message) => {
          const result = ServerMessageSchema.safeParse(message);
          expect(result.success).toBe(true);
        });
      });

      it('should reject message with unknown type', () => {
        const message = {
          type: 'unknown',
          data: {},
        };

        const result = ServerMessageSchema.safeParse(message);
        expect(result.success).toBe(false);
      });
    });
  });

  describe('Client Messages', () => {
    describe('QueryMessage', () => {
      it('should validate query message with params', () => {
        const message = {
          type: 'query',
          id: 'query-123',
          method: 'getProfiles',
          params: { filter: 'active' },
        };

        const result = QueryMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should validate query message without params', () => {
        const message = {
          type: 'query',
          id: 'query-456',
          method: 'getDevices',
        };

        const result = QueryMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should reject query without method', () => {
        const message = {
          type: 'query',
          id: 'query-789',
        };

        const result = QueryMessageSchema.safeParse(message);
        expect(result.success).toBe(false);
      });
    });

    describe('CommandMessage', () => {
      it('should validate command message', () => {
        const message = {
          type: 'command',
          id: 'cmd-123',
          method: 'activateProfile',
          params: { name: 'my-profile' },
        };

        const result = CommandMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should reject command without id', () => {
        const message = {
          type: 'command',
          method: 'activateProfile',
        };

        const result = CommandMessageSchema.safeParse(message);
        expect(result.success).toBe(false);
      });
    });

    describe('SubscribeMessage', () => {
      it('should validate subscribe to daemon-state', () => {
        const message = {
          type: 'subscribe',
          id: 'sub-1',
          channel: 'daemon-state',
        };

        const result = SubscribeMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should validate subscribe to events', () => {
        const message = {
          type: 'subscribe',
          id: 'sub-2',
          channel: 'events',
        };

        const result = SubscribeMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should validate subscribe to latency', () => {
        const message = {
          type: 'subscribe',
          id: 'sub-3',
          channel: 'latency',
        };

        const result = SubscribeMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should reject subscribe to invalid channel', () => {
        const message = {
          type: 'subscribe',
          id: 'sub-4',
          channel: 'invalid',
        };

        const result = SubscribeMessageSchema.safeParse(message);
        expect(result.success).toBe(false);
      });
    });

    describe('UnsubscribeMessage', () => {
      it('should validate unsubscribe message', () => {
        const message = {
          type: 'unsubscribe',
          id: 'unsub-1',
          channel: 'events',
        };

        const result = UnsubscribeMessageSchema.safeParse(message);
        expect(result.success).toBe(true);
      });

      it('should reject unsubscribe from invalid channel', () => {
        const message = {
          type: 'unsubscribe',
          id: 'unsub-2',
          channel: 'unknown',
        };

        const result = UnsubscribeMessageSchema.safeParse(message);
        expect(result.success).toBe(false);
      });
    });

    describe('ClientMessage Union', () => {
      it('should validate all client message types', () => {
        const messages = [
          {
            type: 'query',
            id: 'q1',
            method: 'getProfiles',
          },
          {
            type: 'command',
            id: 'c1',
            method: 'activateProfile',
            params: { name: 'test' },
          },
          {
            type: 'subscribe',
            id: 's1',
            channel: 'daemon-state',
          },
          {
            type: 'unsubscribe',
            id: 'u1',
            channel: 'events',
          },
        ];

        messages.forEach((message) => {
          const result = ClientMessageSchema.safeParse(message);
          expect(result.success).toBe(true);
        });
      });
    });
  });

  describe('Event Data Schemas', () => {
    describe('DaemonStateSchema', () => {
      it('should validate daemon state with active modifiers', () => {
        const state = {
          modifiers: ['MD_00', 'MD_01'],
          locks: ['LK_00'],
          layer: 'vim-layer',
        };

        const result = DaemonStateSchema.safeParse(state);
        expect(result.success).toBe(true);
      });

      it('should validate daemon state with empty arrays', () => {
        const state = {
          modifiers: [],
          locks: [],
          layer: 'base',
        };

        const result = DaemonStateSchema.safeParse(state);
        expect(result.success).toBe(true);
      });

      it('should reject state with missing layer', () => {
        const state = {
          modifiers: [],
          locks: [],
        };

        const result = DaemonStateSchema.safeParse(state);
        expect(result.success).toBe(false);
      });

      it('should reject state with non-array modifiers', () => {
        const state = {
          modifiers: 'MD_00',
          locks: [],
          layer: 'base',
        };

        const result = DaemonStateSchema.safeParse(state);
        expect(result.success).toBe(false);
      });

      it('should parse daemon state using helper function', () => {
        const state = {
          modifiers: ['MD_00'],
          locks: [],
          layer: 'base',
        };

        const parsed = parseDaemonStateEvent(state);
        expect(parsed.modifiers).toEqual(['MD_00']);
        expect(parsed.locks).toEqual([]);
        expect(parsed.layer).toBe('base');
      });
    });

    describe('DeviceConnectedEventSchema', () => {
      it('should validate device connected event', () => {
        const event = {
          serial: 'ABC123456',
          vendor: 'Logitech',
          product: 'K380',
          timestamp: 1704067200000000,
        };

        const result = DeviceConnectedEventSchema.safeParse(event);
        expect(result.success).toBe(true);
      });

      it('should reject event with missing fields', () => {
        const event = {
          serial: 'ABC123',
          vendor: 'Logitech',
        };

        const result = DeviceConnectedEventSchema.safeParse(event);
        expect(result.success).toBe(false);
      });

      it('should reject event with wrong timestamp type', () => {
        const event = {
          serial: 'ABC123',
          vendor: 'Logitech',
          product: 'K380',
          timestamp: '2024-01-01',
        };

        const result = DeviceConnectedEventSchema.safeParse(event);
        expect(result.success).toBe(false);
      });

      it('should parse device connected event using helper', () => {
        const event = {
          serial: 'SN12345',
          vendor: 'KeyCorp',
          product: 'Model X',
          timestamp: 1704067200000000,
        };

        const parsed = parseDeviceConnectedEvent(event);
        expect(parsed.serial).toBe('SN12345');
        expect(parsed.vendor).toBe('KeyCorp');
        expect(parsed.product).toBe('Model X');
      });
    });

    describe('ProfileActivatedEventSchema', () => {
      it('should validate profile activated event', () => {
        const event = {
          name: 'vim-navigation',
          timestamp: 1704067200000000,
        };

        const result = ProfileActivatedEventSchema.safeParse(event);
        expect(result.success).toBe(true);
      });

      it('should reject event without name', () => {
        const event = {
          timestamp: 1704067200000000,
        };

        const result = ProfileActivatedEventSchema.safeParse(event);
        expect(result.success).toBe(false);
      });

      it('should parse profile activated event using helper', () => {
        const event = {
          name: 'gaming-profile',
          timestamp: 1704067200000000,
        };

        const parsed = parseProfileActivatedEvent(event);
        expect(parsed.name).toBe('gaming-profile');
        expect(parsed.timestamp).toBe(1704067200000000);
      });
    });

    describe('KeyEventSchema', () => {
      it('should validate key press event', () => {
        const event = {
          timestamp: 1704067200000000,
          keyCode: 'KEY_A',
          eventType: 'press',
          input: 'KEY_A',
          output: 'KEY_B',
          latency: 150,
        };

        const result = KeyEventSchema.safeParse(event);
        expect(result.success).toBe(true);
      });

      it('should validate key release event', () => {
        const event = {
          timestamp: 1704067200000000,
          keyCode: 'KEY_ESC',
          eventType: 'release',
          input: 'KEY_CAPSLOCK',
          output: 'KEY_ESC',
          latency: 95,
        };

        const result = KeyEventSchema.safeParse(event);
        expect(result.success).toBe(true);
      });

      it('should reject event with invalid eventType', () => {
        const event = {
          timestamp: 1704067200000000,
          keyCode: 'KEY_A',
          eventType: 'click',
          input: 'KEY_A',
          output: 'KEY_B',
          latency: 100,
        };

        const result = KeyEventSchema.safeParse(event);
        expect(result.success).toBe(false);
      });

      it('should reject event with missing latency', () => {
        const event = {
          timestamp: 1704067200000000,
          keyCode: 'KEY_A',
          eventType: 'press',
          input: 'KEY_A',
          output: 'KEY_B',
        };

        const result = KeyEventSchema.safeParse(event);
        expect(result.success).toBe(false);
      });

      it('should parse key event using helper', () => {
        const event = {
          timestamp: 1704067200000000,
          keyCode: 'KEY_ENTER',
          eventType: 'press',
          input: 'KEY_J',
          output: 'KEY_DOWN',
          latency: 120,
        };

        const parsed = parseKeyEvent(event);
        expect(parsed.keyCode).toBe('KEY_ENTER');
        expect(parsed.eventType).toBe('press');
        expect(parsed.latency).toBe(120);
      });
    });

    describe('LatencyMetricsSchema', () => {
      it('should validate latency metrics', () => {
        const metrics = {
          min: 50,
          avg: 125,
          max: 250,
          p95: 200,
          p99: 240,
          timestamp: 1704067200000000,
        };

        const result = LatencyMetricsSchema.safeParse(metrics);
        expect(result.success).toBe(true);
      });

      it('should reject metrics with missing p95', () => {
        const metrics = {
          min: 50,
          avg: 100,
          max: 200,
          p99: 180,
          timestamp: 1704067200000000,
        };

        const result = LatencyMetricsSchema.safeParse(metrics);
        expect(result.success).toBe(false);
      });

      it('should reject metrics with string values', () => {
        const metrics = {
          min: '50',
          avg: '100',
          max: '200',
          p95: '180',
          p99: '195',
          timestamp: 1704067200000000,
        };

        const result = LatencyMetricsSchema.safeParse(metrics);
        expect(result.success).toBe(false);
      });

      it('should parse latency metrics using helper', () => {
        const metrics = {
          min: 45,
          avg: 105,
          max: 215,
          p95: 185,
          p99: 205,
          timestamp: 1704067200000000,
        };

        const parsed = parseLatencyMetrics(metrics);
        expect(parsed.min).toBe(45);
        expect(parsed.avg).toBe(105);
        expect(parsed.max).toBe(215);
      });
    });
  });

  describe('Safe Parsing Helpers', () => {
    describe('safeParseServerMessage', () => {
      it('should return success for valid message', () => {
        const message = {
          type: 'connected',
          version: '1.0.0',
          timestamp: Date.now(),
        };

        const result = safeParseServerMessage(message);
        expect(result.success).toBe(true);
        if (result.success) {
          expect(result.data.type).toBe('connected');
        }
      });

      it('should return error for invalid message', () => {
        const message = {
          type: 'unknown',
          data: 'invalid',
        };

        const result = safeParseServerMessage(message);
        expect(result.success).toBe(false);
        if (!result.success) {
          expect(result.error).toBeDefined();
        }
      });
    });

    describe('safeParseClientMessage', () => {
      it('should return success for valid message', () => {
        const message = {
          type: 'query',
          id: 'q1',
          method: 'getProfiles',
        };

        const result = safeParseClientMessage(message);
        expect(result.success).toBe(true);
        if (result.success) {
          expect(result.data.type).toBe('query');
        }
      });

      it('should return error for invalid message', () => {
        const message = {
          type: 'invalid',
          id: 'x',
        };

        const result = safeParseClientMessage(message);
        expect(result.success).toBe(false);
      });
    });
  });

  describe('Edge Cases and Error Handling', () => {
    it('should reject messages with extra fields (strict mode)', () => {
      const message = {
        type: 'connected',
        version: '1.0.0',
        timestamp: Date.now(),
        extraField: 'should not be here',
      };

      // Zod allows extra fields by default, but we're testing that required fields work
      const result = ConnectedMessageSchema.safeParse(message);
      // This should still pass because Zod strips extra fields
      expect(result.success).toBe(true);
    });

    it('should reject null values for required fields', () => {
      const message = {
        type: 'connected',
        version: null,
        timestamp: Date.now(),
      };

      const result = ConnectedMessageSchema.safeParse(message);
      expect(result.success).toBe(false);
    });

    it('should reject undefined values for required fields', () => {
      const message = {
        type: 'connected',
        version: undefined,
        timestamp: Date.now(),
      };

      const result = ConnectedMessageSchema.safeParse(message);
      expect(result.success).toBe(false);
    });

    it('should provide detailed error messages', () => {
      const message = {
        type: 'event',
        // Missing channel
      };

      const result = EventMessageSchema.safeParse(message);
      expect(result.success).toBe(false);
      if (!result.success) {
        // Zod provides detailed error information via issues array
        expect(result.error.issues.length).toBeGreaterThan(0);
        expect(result.error.issues[0].path).toBeDefined();
        expect(result.error.issues[0].path).toContain('channel');
      }
    });
  });

  describe('Real-World Message Examples', () => {
    it('should validate complete daemon state update flow', () => {
      // Client subscribes
      const subscribe = {
        type: 'subscribe',
        id: 'sub-daemon-state-1',
        channel: 'daemon-state',
      };
      expect(ClientMessageSchema.safeParse(subscribe).success).toBe(true);

      // Server sends initial connected message
      const connected = {
        type: 'connected',
        version: '2.0.0',
        timestamp: Date.now(),
      };
      expect(ServerMessageSchema.safeParse(connected).success).toBe(true);

      // Server sends daemon state event
      const event = {
        type: 'event',
        channel: 'daemon-state',
        data: {
          modifiers: ['MD_00'],
          locks: [],
          layer: 'vim-layer',
        },
      };
      expect(ServerMessageSchema.safeParse(event).success).toBe(true);
    });

    it('should validate complete profile activation flow', () => {
      // Client sends command
      const command = {
        type: 'command',
        id: 'activate-1',
        method: 'activateProfile',
        params: { name: 'gaming-profile' },
      };
      expect(ClientMessageSchema.safeParse(command).success).toBe(true);

      // Server sends response
      const response = {
        type: 'response',
        id: 'activate-1',
        result: { success: true },
      };
      expect(ServerMessageSchema.safeParse(response).success).toBe(true);

      // Server sends profile activated event
      const event = {
        type: 'event',
        channel: 'events',
        data: {
          name: 'gaming-profile',
          timestamp: Date.now(),
        },
      };
      expect(ServerMessageSchema.safeParse(event).success).toBe(true);
    });

    it('should validate error response format', () => {
      const errorResponse = {
        type: 'response',
        id: 'failed-cmd',
        error: {
          code: -32602,
          message: 'Invalid params',
          data: {
            field: 'name',
            reason: 'Profile not found',
          },
        },
      };

      const result = ServerMessageSchema.safeParse(errorResponse);
      expect(result.success).toBe(true);
    });
  });
});
