/**
 * MSW server setup for Node.js environment (Vitest)
 *
 * Provides HTTP REST API mocking for:
 * - Devices API (/api/devices)
 * - Profiles API (/api/profiles)
 * - Config API (/api/config)
 * - Metrics API (/api/metrics)
 *
 * WebSocket mocking is handled separately by jest-websocket-mock for better
 * compatibility with react-use-websocket. See tests/helpers/websocket.ts.
 */

import { setupServer } from 'msw/node';
import { handlers } from './handlers';

/**
 * MSW server instance with HTTP handlers only
 *
 * This hybrid approach uses:
 * - MSW for HTTP REST API mocking (proven to work well)
 * - jest-websocket-mock for WebSocket mocking (better react-use-websocket compatibility)
 *
 * Both tools coexist without conflicts, using the best tool for each job.
 */
export const server = setupServer(...handlers);
