/**
 * StateIndicatorPanel - Real-time daemon state visualization
 *
 * Displays active modifiers, lock keys, and current layer as color-coded badges.
 * Badges appear/disappear with animations as state changes in real-time.
 *
 * Badge colors and meanings:
 * - Blue badges: Active modifiers (Shift, Ctrl, Alt, etc.)
 * - Orange badges: Active lock keys (Caps Lock, Num Lock, Scroll Lock)
 * - Green badge: Current layer (base layer if none active)
 *
 * Features:
 * - Real-time updates from WebSocket
 * - Fade-in animations for new badges
 * - Grouped by type (modifiers, locks, layer)
 * - Empty state messages when no modifiers/locks active
 *
 * @example
 * ```tsx
 * // Component reads from dashboardStore automatically
 * <StateIndicatorPanel />
 * ```
 */

import { useDashboardStore } from '../store/dashboardStore';
import './StateIndicatorPanel.css';

/**
 * StateIndicatorPanel component displaying daemon state as badges
 *
 * Subscribes to dashboard state store and renders color-coded badges
 * for active modifiers, locks, and current layer. Badges animate in
 * when state changes.
 *
 * State mapping:
 * - modifiers: Array of modifier IDs (converted to names)
 * - locks: Array of lock IDs (converted to names)
 * - layer: Current layer number (0 = base layer)
 *
 * @returns Rendered state indicator panel
 */
export function StateIndicatorPanel() {
  const currentState = useDashboardStore((state) => state.currentState);

  return (
    <div className="state-indicator-panel">
      {/* Modifiers section */}
      <div className="state-section">
        <h3 className="state-section-title">Active Modifiers</h3>
        <div className="badge-container" role="group" aria-label="Active modifiers">
          {currentState.modifiers.length === 0 ? (
            <span className="empty-state" aria-live="polite">
              No active modifiers
            </span>
          ) : (
            currentState.modifiers.map((mod) => (
              <span
                key={mod}
                className="state-badge state-badge-modifier"
                role="status"
                aria-label={`Modifier ${mod} active`}
              >
                {mod}
              </span>
            ))
          )}
        </div>
      </div>

      {/* Locks section */}
      <div className="state-section">
        <h3 className="state-section-title">Active Locks</h3>
        <div className="badge-container" role="group" aria-label="Active locks">
          {currentState.locks.length === 0 ? (
            <span className="empty-state" aria-live="polite">
              No active locks
            </span>
          ) : (
            currentState.locks.map((lock) => (
              <span
                key={lock}
                className="state-badge state-badge-lock"
                role="status"
                aria-label={`Lock ${lock} active`}
              >
                {lock}
              </span>
            ))
          )}
        </div>
      </div>

      {/* Layer section */}
      <div className="state-section">
        <h3 className="state-section-title">Current Layer</h3>
        <div className="badge-container" role="group" aria-label="Current layer">
          <span
            className="state-badge state-badge-layer"
            role="status"
            aria-label={`Layer ${currentState.layer} active`}
          >
            {currentState.layer}
          </span>
        </div>
      </div>
    </div>
  );
}
