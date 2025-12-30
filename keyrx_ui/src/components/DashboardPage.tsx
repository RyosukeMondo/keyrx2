/**
 * DashboardPage - Real-time daemon monitoring dashboard
 *
 * Main dashboard view displaying live daemon state, performance metrics,
 * and key event timeline. Connects to daemon via WebSocket for real-time updates.
 *
 * Dashboard sections:
 * - Connection status banner (connected/disconnected)
 * - State indicator panel (active modifiers, locks, current layer)
 * - Metrics chart (latency over 60-second rolling window)
 * - Event timeline (last 100 key events)
 *
 * @example
 * ```tsx
 * // Component manages WebSocket connection automatically
 * <DashboardPage />
 * ```
 */

import { useDaemonWebSocket } from '../hooks/useDaemonWebSocket';
import { useDashboardStore } from '../store/dashboardStore';
import { StateIndicatorPanel } from './StateIndicatorPanel';
import { MetricsChart } from './MetricsChart';
import { DashboardEventTimeline } from './DashboardEventTimeline';
import './DashboardPage.css';

/**
 * DashboardPage component - Main monitoring interface
 *
 * Establishes WebSocket connection to daemon on mount and displays real-time
 * updates across multiple visualization components. Automatically reconnects
 * on connection loss.
 *
 * Layout:
 * - Connection status banner (green: connected, red: disconnected)
 * - State indicator panel (modifiers, locks, layer badges)
 * - Metrics chart (latency over time)
 * - Event timeline (last 100 events)
 *
 * @returns Rendered dashboard page
 */
export function DashboardPage() {
  // WebSocket connection
  const { isConnected, isConnecting, isDisconnected } = useDaemonWebSocket();

  // Dashboard state
  const connectionStatus = useDashboardStore((state) => state.connectionStatus);

  return (
    <div className="dashboard-page">
      {/* Connection status banner */}
      <div className={`connection-banner connection-${connectionStatus}`}>
        <div className="connection-status">
          {isConnecting && (
            <>
              <div className="status-indicator connecting" />
              <span>Connecting to daemon...</span>
            </>
          )}
          {isConnected && (
            <>
              <div className="status-indicator connected" />
              <span>Connected to daemon</span>
            </>
          )}
          {isDisconnected && (
            <>
              <div className="status-indicator disconnected" />
              <span>Disconnected - attempting to reconnect...</span>
            </>
          )}
        </div>
      </div>

      {/* Main dashboard content */}
      <div className="dashboard-content">
        <div className="dashboard-grid">
          {/* State indicator panel - top */}
          <div className="panel state-panel">
            <h2>Daemon State</h2>
            <div className="panel-content">
              <StateIndicatorPanel />
            </div>
          </div>

          {/* Metrics chart - middle left */}
          <div className="panel metrics-panel">
            <h2>Latency Metrics</h2>
            <div className="panel-content">
              <MetricsChart />
            </div>
          </div>

          {/* Event timeline - middle right */}
          <div className="panel events-panel">
            <DashboardEventTimeline height={500} />
          </div>
        </div>
      </div>
    </div>
  );
}
