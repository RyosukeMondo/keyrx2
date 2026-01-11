/**
 * Device management API client
 */

import { apiClient } from './client';
import { validateApiResponse, DeviceListResponseSchema, DeviceEntrySchema } from './schemas';
import type { DeviceEntry, DeviceScope } from '../types';

interface RenameDeviceRequest {
  name: string;
}

interface SetScopeRequest {
  scope: DeviceScope;
}

interface DeviceResponse {
  success: boolean;
}

interface DevicesListResponse {
  devices: DeviceEntry[];
}

/**
 * Fetch all connected devices
 */
export async function fetchDevices(): Promise<DeviceEntry[]> {
  const response = await apiClient.get<DevicesListResponse>('/api/devices');
  const validated = validateApiResponse(DeviceListResponseSchema, response, 'GET /api/devices');

  // Map validated response to DeviceEntry format
  return validated.devices.map((device) => ({
    id: device.id,
    name: device.name,
    path: '', // REST API doesn't provide path, use empty string
    serial: device.serial || null,
    active: true, // Devices returned by REST API are assumed active
    scope: device.scope === 'DeviceSpecific' ? 'device-specific' : 'global',
    layout: device.layout || null,
  }));
}

/**
 * Rename a device
 */
export async function renameDevice(
  id: string,
  name: string
): Promise<DeviceResponse> {
  const request: RenameDeviceRequest = { name };
  const response = await apiClient.put<DeviceEntry>(`/api/devices/${id}/name`, request);
  // Validate the returned device entry
  validateApiResponse(DeviceEntrySchema, response, `PUT /api/devices/${id}/name`);
  return { success: true };
}

/**
 * Set device scope (global or local)
 */
export async function setDeviceScope(
  id: string,
  scope: DeviceScope
): Promise<DeviceResponse> {
  const request: SetScopeRequest = { scope };
  const response = await apiClient.put<DeviceEntry>(`/api/devices/${id}/scope`, request);
  // Validate the returned device entry
  validateApiResponse(DeviceEntrySchema, response, `PUT /api/devices/${id}/scope`);
  return { success: true };
}

/**
 * Forget a device (remove from device list)
 */
export async function forgetDevice(id: string): Promise<DeviceResponse> {
  const response = await apiClient.delete<DeviceEntry>(`/api/devices/${id}`);
  // Validate the returned device entry
  validateApiResponse(DeviceEntrySchema, response, `DELETE /api/devices/${id}`);
  return { success: true };
}
