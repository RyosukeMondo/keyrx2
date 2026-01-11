import React from 'react';
import { Card } from './Card';
import type { PaletteKey } from './KeyPalette';

/**
 * Key Configuration Panel - Right panel showing selected key details and configuration
 */

interface KeyConfigPanelProps {
  selectedPhysicalKey?: string | null;
  assignedKey?: PaletteKey | null;
  onClear?: () => void;
}

export function KeyConfigPanel({
  selectedPhysicalKey,
  assignedKey,
  onClear,
}: KeyConfigPanelProps) {
  if (!selectedPhysicalKey) {
    return (
      <Card className="h-full flex items-center justify-center">
        <div className="text-center text-slate-400">
          <p className="text-lg mb-2">No Key Selected</p>
          <p className="text-sm">Click a key on the keyboard to configure it</p>
        </div>
      </Card>
    );
  }

  return (
    <Card className="h-full">
      <h3 className="text-lg font-semibold text-slate-100 mb-4">Key Configuration</h3>

      {/* Selected Physical Key */}
      <div className="mb-6">
        <label className="block text-xs text-slate-400 mb-2">Physical Key</label>
        <div className="px-4 py-3 bg-slate-700/50 rounded-lg border border-slate-600">
          <div className="text-2xl font-bold text-primary-400 text-center">
            {selectedPhysicalKey}
          </div>
        </div>
      </div>

      {/* Current Assignment */}
      <div className="mb-6">
        <label className="block text-xs text-slate-400 mb-2">Current Assignment</label>
        {assignedKey ? (
          <div className="px-4 py-3 bg-slate-700/50 rounded-lg border border-slate-600">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-lg font-medium text-slate-100">{assignedKey.label}</div>
                <div className="text-xs text-slate-500 mt-1">{assignedKey.id}</div>
                {assignedKey.description && (
                  <div className="text-xs text-slate-400 mt-1">{assignedKey.description}</div>
                )}
              </div>
              <span className="px-2 py-1 text-xs font-medium bg-primary-500/20 text-primary-300 rounded">
                {assignedKey.category.replace('_', ' ')}
              </span>
            </div>
          </div>
        ) : (
          <div className="px-4 py-3 bg-slate-700/20 rounded-lg border border-slate-600 border-dashed text-center text-slate-500">
            Not assigned
          </div>
        )}
      </div>

      {/* Actions */}
      {assignedKey && onClear && (
        <button
          onClick={onClear}
          className="w-full px-4 py-2 bg-red-500/20 text-red-400 rounded-md hover:bg-red-500/30 transition-colors text-sm font-medium"
        >
          Clear Assignment
        </button>
      )}

      {/* Instructions */}
      <div className="mt-6 p-3 bg-slate-700/30 rounded-lg border border-slate-600">
        <h4 className="text-xs font-medium text-slate-300 mb-2">How to Assign:</h4>
        <ol className="text-xs text-slate-400 space-y-1 list-decimal list-inside">
          <li>Select a key from the palette (left panel)</li>
          <li>Click this physical key on the keyboard</li>
          <li>Assignment will be saved automatically</li>
        </ol>
      </div>
    </Card>
  );
}
