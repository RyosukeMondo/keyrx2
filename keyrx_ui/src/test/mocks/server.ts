/**
 * MSW server setup for Node.js environment (Vitest)
 *
 * Configures MSW server with both HTTP REST API handlers and WebSocket handlers.
 * HTTP handlers mock the daemon's REST API endpoints (devices, profiles, config, metrics).
 * WebSocket handlers mock the daemon's WebSocket RPC protocol for real-time updates.
 */

import { setupServer } from 'msw/node';
import { handlers } from './handlers';
import { createWebSocketHandlers } from './websocketHandlers';

/**
 * MSW server instance with HTTP and WebSocket handlers
 *
 * The server automatically intercepts:
 * - HTTP requests to /api/* (REST API endpoints)
 * - WebSocket connections to ws://localhost:3030/ws (RPC protocol)
 *
 * Both handler types coexist without conflicts, enabling comprehensive
 * integration testing of components that use both REST and WebSocket APIs.
 */
export const server = setupServer(
  ...handlers, // HTTP REST API handlers
  ...createWebSocketHandlers() // WebSocket RPC handlers
);
