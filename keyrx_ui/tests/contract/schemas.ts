/**
 * Zod schemas for WebSocket message contract validation.
 *
 * These schemas ensure runtime type safety for WebSocket messages exchanged
 * between the frontend and daemon. They validate message structure, required
 * fields, and data types at runtime, catching protocol violations before they
 * cause errors.
 *
 * Contract testing strategy:
 * 1. Define strict schemas for all message types
 * 2. Validate incoming messages against schemas
 * 3. Catch protocol changes early in development
 * 4. Provide clear error messages for invalid data
 */

import { z } from 'zod';

// ============================================================================
// RPC Error Schema
// ============================================================================

/**
 * JSON-RPC 2.0 error structure.
 */
export const RpcErrorSchema = z.object({
  /** Numeric error code (JSON-RPC 2.0 standard codes or custom) */
  code: z.number(),
  /** Human-readable error message */
  message: z.string(),
  /** Optional additional error data */
  data: z.unknown().optional(),
});

export type RpcError = z.infer<typeof RpcErrorSchema>;

// ============================================================================
// Client Message Schemas
// ============================================================================

/**
 * Query message - read-only operation request.
 */
export const QueryMessageSchema = z.object({
  type: z.literal('query'),
  id: z.string(),
  method: z.string(),
  params: z.unknown().optional(),
});

/**
 * Command message - state-modifying operation request.
 */
export const CommandMessageSchema = z.object({
  type: z.literal('command'),
  id: z.string(),
  method: z.string(),
  params: z.unknown().optional(),
});

/**
 * Subscribe message - subscribe to real-time channel.
 */
export const SubscribeMessageSchema = z.object({
  type: z.literal('subscribe'),
  id: z.string(),
  channel: z.enum(['daemon-state', 'events', 'latency']),
});

/**
 * Unsubscribe message - unsubscribe from real-time channel.
 */
export const UnsubscribeMessageSchema = z.object({
  type: z.literal('unsubscribe'),
  id: z.string(),
  channel: z.enum(['daemon-state', 'events', 'latency']),
});

/**
 * Union of all client message types.
 */
export const ClientMessageSchema = z.discriminatedUnion('type', [
  QueryMessageSchema,
  CommandMessageSchema,
  SubscribeMessageSchema,
  UnsubscribeMessageSchema,
]);

export type ClientMessage = z.infer<typeof ClientMessageSchema>;

// ============================================================================
// Server Message Schemas
// ============================================================================

/**
 * Response message - result of query or command.
 */
export const ResponseMessageSchema = z.object({
  type: z.literal('response'),
  id: z.string(),
  result: z.unknown().optional(),
  error: RpcErrorSchema.optional(),
});

/**
 * Event message - real-time data from subscription channel.
 */
export const EventMessageSchema = z.object({
  type: z.literal('event'),
  channel: z.enum(['daemon-state', 'events', 'latency']),
  data: z.unknown(),
});

/**
 * Connected message - initial handshake from server.
 */
export const ConnectedMessageSchema = z.object({
  type: z.literal('connected'),
  version: z.string(),
  timestamp: z.number(),
});

/**
 * Union of all server message types.
 */
export const ServerMessageSchema = z.discriminatedUnion('type', [
  ResponseMessageSchema,
  EventMessageSchema,
  ConnectedMessageSchema,
]);

export type ServerMessage = z.infer<typeof ServerMessageSchema>;

// ============================================================================
// Event Data Schemas (for specific channels)
// ============================================================================

/**
 * Daemon state snapshot - broadcast on "daemon-state" channel.
 */
export const DaemonStateSchema = z.object({
  /** Active modifier IDs (e.g., ["MD_00", "MD_01"]) */
  modifiers: z.array(z.string()),
  /** Active lock IDs (e.g., ["LK_00"]) */
  locks: z.array(z.string()),
  /** Current active layer name */
  layer: z.string(),
});

export type DaemonState = z.infer<typeof DaemonStateSchema>;

/**
 * Device connected event - when a new device is detected.
 */
export const DeviceConnectedEventSchema = z.object({
  /** Unique device identifier (serial number or pattern) */
  serial: z.string(),
  /** Device vendor name */
  vendor: z.string(),
  /** Device product name */
  product: z.string(),
  /** Timestamp when device was connected (microseconds since UNIX epoch) */
  timestamp: z.number(),
});

export type DeviceConnectedEvent = z.infer<typeof DeviceConnectedEventSchema>;

/**
 * Profile activated event - when active profile changes.
 */
export const ProfileActivatedEventSchema = z.object({
  /** Name of the activated profile */
  name: z.string(),
  /** Timestamp when profile was activated (microseconds since UNIX epoch) */
  timestamp: z.number(),
});

export type ProfileActivatedEvent = z.infer<typeof ProfileActivatedEventSchema>;

/**
 * Individual key event - broadcast on "events" channel.
 */
export const KeyEventSchema = z.object({
  /** Timestamp in microseconds since UNIX epoch */
  timestamp: z.number(),
  /** Key code (e.g., "KEY_A") */
  keyCode: z.string(),
  /** Event type */
  eventType: z.enum(['press', 'release']),
  /** Input key (before mapping) */
  input: z.string(),
  /** Output key (after mapping) */
  output: z.string(),
  /** Processing latency in microseconds */
  latency: z.number(),
});

export type KeyEvent = z.infer<typeof KeyEventSchema>;

/**
 * Latency metrics - broadcast on "latency" channel.
 */
export const LatencyMetricsSchema = z.object({
  /** Minimum latency in microseconds */
  min: z.number(),
  /** Average latency in microseconds */
  avg: z.number(),
  /** Maximum latency in microseconds */
  max: z.number(),
  /** 95th percentile latency in microseconds */
  p95: z.number(),
  /** 99th percentile latency in microseconds */
  p99: z.number(),
  /** Timestamp of this stats snapshot (microseconds since UNIX epoch) */
  timestamp: z.number(),
});

export type LatencyMetrics = z.infer<typeof LatencyMetricsSchema>;

// ============================================================================
// Event Message Type Guards with Schema Validation
// ============================================================================

/**
 * Validate and extract daemon state event data.
 */
export function parseDaemonStateEvent(event: unknown): DaemonState {
  return DaemonStateSchema.parse(event);
}

/**
 * Validate and extract device connected event data.
 */
export function parseDeviceConnectedEvent(event: unknown): DeviceConnectedEvent {
  return DeviceConnectedEventSchema.parse(event);
}

/**
 * Validate and extract profile activated event data.
 */
export function parseProfileActivatedEvent(event: unknown): ProfileActivatedEvent {
  return ProfileActivatedEventSchema.parse(event);
}

/**
 * Validate and extract key event data.
 */
export function parseKeyEvent(event: unknown): KeyEvent {
  return KeyEventSchema.parse(event);
}

/**
 * Validate and extract latency metrics data.
 */
export function parseLatencyMetrics(event: unknown): LatencyMetrics {
  return LatencyMetricsSchema.parse(event);
}

// ============================================================================
// Safe Parsing Helpers (return results instead of throwing)
// ============================================================================

/**
 * Safely parse a server message without throwing.
 * Returns { success: true, data } on success or { success: false, error } on failure.
 */
export function safeParseServerMessage(message: unknown) {
  return ServerMessageSchema.safeParse(message);
}

/**
 * Safely parse a client message without throwing.
 * Returns { success: true, data } on success or { success: false, error } on failure.
 */
export function safeParseClientMessage(message: unknown) {
  return ClientMessageSchema.safeParse(message);
}
