/**
 * DeviceList - Real-time display of connected input devices
 *
 * Fetches device list from REST API and maintains WebSocket connection
 * for real-time updates. Displays device status and handles reconnection
 * automatically on connection loss.
 *
 * @example
 * ```tsx
 * // Default URLs
 * <DeviceList />
 *
 * // Custom endpoints
 * <DeviceList
 *   wsUrl="ws://localhost:9867/ws"
 *   apiBaseUrl="http://localhost:9867/api"
 * />
 * ```
 */

import { useEffect, useState, useCallback, useRef } from 'react'
import './DeviceList.css'

/**
 * Device information from daemon
 */
interface Device {
  /** Unique device identifier */
  id: string
  /** Human-readable device name */
  name: string
  /** System device path (e.g., /dev/input/event0) */
  path: string
  /** Device serial number if available */
  serial: string | null
  /** Whether the device is currently being monitored */
  active: boolean
}

/**
 * API response structure for device list
 */
interface DevicesResponse {
  /** Array of connected devices */
  devices: Device[]
}

/**
 * Props for DeviceList component
 */
interface DeviceListProps {
  /** WebSocket URL for real-time device updates (default: ws://localhost:9867/ws) */
  wsUrl?: string
  /** REST API base URL for fetching device list (default: http://localhost:9867/api) */
  apiBaseUrl?: string
}

/**
 * Loading state for async operations
 */
type LoadingState = 'loading' | 'success' | 'error'

/**
 * DeviceList component displays connected input devices with real-time updates
 *
 * Features:
 * - Fetches initial device list from REST API
 * - WebSocket connection for live device updates
 * - Automatic reconnection on WebSocket disconnect (5 second interval)
 * - Visual indication of active devices
 * - Error handling with user-friendly messages
 * - Loading states during API calls
 *
 * @param props - Component props
 * @returns Rendered device list component
 */
export function DeviceList({
  wsUrl = 'ws://localhost:9867/ws',
  apiBaseUrl = 'http://localhost:9867/api'
}: DeviceListProps) {
  const [devices, setDevices] = useState<Device[]>([])
  const [loadingState, setLoadingState] = useState<LoadingState>('loading')
  const [error, setError] = useState<string | null>(null)
  const [activeDeviceId, setActiveDeviceId] = useState<string | null>(null)
  const activeTimeoutRef = useRef<number | null>(null)

  const fetchDevices = useCallback(async () => {
    try {
      setLoadingState('loading')
      setError(null)
      const response = await fetch(`${apiBaseUrl}/devices`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }
      const data: DevicesResponse = await response.json()
      setDevices(data.devices)
      setLoadingState('success')
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Unknown error'
      setError(`Failed to fetch devices: ${message}`)
      setLoadingState('error')
    }
  }, [apiBaseUrl])

  useEffect(() => {
    fetchDevices()
  }, [fetchDevices])

  useEffect(() => {
    let ws: WebSocket | null = null
    let reconnectTimeout: number | null = null

    const connect = () => {
      try {
        ws = new WebSocket(wsUrl)

        ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data)
            if (data.device_id) {
              setActiveDeviceId(data.device_id)

              if (activeTimeoutRef.current !== null) {
                window.clearTimeout(activeTimeoutRef.current)
              }
              activeTimeoutRef.current = window.setTimeout(() => {
                setActiveDeviceId(null)
                activeTimeoutRef.current = null
              }, 500)
            }
          } catch (error) {
            // Log non-JSON messages for debugging
            if (import.meta.env.DEV) {
              console.debug('Received non-JSON WebSocket message:', {
                message: event.data,
                error: error instanceof Error ? error.message : String(error)
              })
            }
          }
        }

        ws.onclose = () => {
          reconnectTimeout = window.setTimeout(connect, 5000)
        }

        ws.onerror = () => {
          ws?.close()
        }
      } catch (error) {
        // Log WebSocket connection errors for debugging
        if (import.meta.env.DEV) {
          console.debug('WebSocket connection failed, scheduling reconnection:', {
            wsUrl,
            reconnectDelay: 5000,
            error: error instanceof Error ? error.message : String(error)
          })
        }
        reconnectTimeout = window.setTimeout(connect, 5000)
      }
    }

    connect()

    return () => {
      if (activeTimeoutRef.current !== null) {
        window.clearTimeout(activeTimeoutRef.current)
      }
      if (reconnectTimeout !== null) {
        window.clearTimeout(reconnectTimeout)
      }
      ws?.close()
    }
  }, [wsUrl])

  if (loadingState === 'loading') {
    return (
      <div className="device-list">
        <h2>Connected Devices</h2>
        <div className="loading">Loading devices...</div>
      </div>
    )
  }

  if (loadingState === 'error') {
    return (
      <div className="device-list">
        <h2>Connected Devices</h2>
        <div className="error">
          <p>{error}</p>
          <button onClick={fetchDevices}>Retry</button>
        </div>
      </div>
    )
  }

  if (devices.length === 0) {
    return (
      <div className="device-list">
        <h2>Connected Devices</h2>
        <div className="empty">
          <p>No keyboard devices found.</p>
          <p className="hint">Make sure the daemon has permission to access input devices.</p>
        </div>
      </div>
    )
  }

  return (
    <div className="device-list">
      <h2>Connected Devices</h2>
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Serial</th>
            <th>Path</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {devices.map((device) => (
            <tr
              key={device.id}
              className={activeDeviceId === device.id ? 'active' : ''}
            >
              <td className="device-name">{device.name}</td>
              <td className="device-serial">
                {device.serial ?? <span className="no-serial">N/A</span>}
              </td>
              <td className="device-path">{device.path}</td>
              <td className="device-status">
                <span className={`status-indicator ${device.active ? 'connected' : 'disconnected'}`}>
                  {device.active ? 'Connected' : 'Disconnected'}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

export default DeviceList
