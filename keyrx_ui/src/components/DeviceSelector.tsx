import React from 'react';
import { Card } from './Card';

/**
 * Device Selector - Choose between global or device-specific configuration
 */

export interface Device {
  id: string;
  name: string;
  serial?: string;
}

interface DeviceSelectorProps {
  devices: Device[];
  scope: 'global' | 'device-specific';
  selectedDevice?: string;
  onScopeChange: (scope: 'global' | 'device-specific') => void;
  onDeviceChange: (deviceId: string) => void;
}

export function DeviceSelector({
  devices,
  scope,
  selectedDevice,
  onScopeChange,
  onDeviceChange,
}: DeviceSelectorProps) {
  return (
    <Card>
      <h3 className="text-sm font-semibold text-slate-100 mb-3">Configuration Scope</h3>

      {/* Scope Toggle */}
      <div className="flex gap-2 mb-4">
        <button
          onClick={() => onScopeChange('global')}
          className={`flex-1 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
            scope === 'global'
              ? 'bg-primary-500 text-white'
              : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
          }`}
        >
          Global
        </button>
        <button
          onClick={() => onScopeChange('device-specific')}
          className={`flex-1 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
            scope === 'device-specific'
              ? 'bg-primary-500 text-white'
              : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
          }`}
        >
          Device-Specific
        </button>
      </div>

      {/* Device Selector (only shown in device-specific mode) */}
      {scope === 'device-specific' && (
        <div>
          <label htmlFor="device-select" className="block text-xs text-slate-400 mb-2">
            Select Device:
          </label>
          <select
            id="device-select"
            value={selectedDevice || ''}
            onChange={(e) => onDeviceChange(e.target.value)}
            className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="">Choose a device...</option>
            {devices.map((device) => (
              <option key={device.id} value={device.id}>
                {device.name} {device.serial ? `(${device.serial})` : ''}
              </option>
            ))}
          </select>
        </div>
      )}

      {/* Info */}
      <p className="text-xs text-slate-500 mt-3">
        {scope === 'global'
          ? 'Changes apply to all devices'
          : 'Changes apply only to the selected device'}
      </p>
    </Card>
  );
}
