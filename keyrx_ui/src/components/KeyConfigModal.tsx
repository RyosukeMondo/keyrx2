import React, { useState } from 'react';
import { Modal } from './Modal';
import type { KeyMapping } from '@/types';
import type { PaletteKey } from './KeyPalette';

/**
 * Advanced Key Configuration Modal
 * Supports: simple, tap_hold, modified, macro, layer_switch
 */

interface KeyConfigModalProps {
  isOpen: boolean;
  onClose: () => void;
  physicalKey: string;
  currentMapping?: KeyMapping;
  onSave: (mapping: KeyMapping) => void;
  availableKeys: PaletteKey[];
}

type MappingType = 'simple' | 'tap_hold' | 'macro' | 'layer_switch';

export function KeyConfigModal({
  isOpen,
  onClose,
  physicalKey,
  currentMapping,
  onSave,
  availableKeys,
}: KeyConfigModalProps) {
  const [mappingType, setMappingType] = useState<MappingType>(
    currentMapping?.type || 'simple'
  );
  const [tapAction, setTapAction] = useState(currentMapping?.tapAction || '');
  const [holdAction, setHoldAction] = useState(currentMapping?.holdAction || '');
  const [threshold, setThreshold] = useState(currentMapping?.threshold || 200);
  const [targetLayer, setTargetLayer] = useState(currentMapping?.targetLayer || '');

  const handleSave = () => {
    const mapping: KeyMapping = {
      type: mappingType,
      tapAction: mappingType !== 'layer_switch' ? tapAction : undefined,
      holdAction: mappingType === 'tap_hold' ? holdAction : undefined,
      threshold: mappingType === 'tap_hold' ? threshold : undefined,
      targetLayer: mappingType === 'layer_switch' ? targetLayer : undefined,
    };
    onSave(mapping);
    onClose();
  };

  return (
    <Modal open={isOpen} onClose={onClose} title={`Configure: ${physicalKey}`}>
      <div className="space-y-6">
        {/* Mapping Type Selector */}
        <div>
          <label className="block text-sm font-medium text-slate-300 mb-2">
            Mapping Type
          </label>
          <div className="grid grid-cols-2 gap-2">
            {(['simple', 'tap_hold', 'macro', 'layer_switch'] as MappingType[]).map((type) => (
              <button
                key={type}
                onClick={() => setMappingType(type)}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  mappingType === type
                    ? 'bg-primary-500 text-white'
                    : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                }`}
              >
                {type === 'tap_hold' ? 'Tap/Hold' : type.charAt(0).toUpperCase() + type.slice(1).replace('_', ' ')}
              </button>
            ))}
          </div>
        </div>

        {/* Simple Mapping */}
        {mappingType === 'simple' && (
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Target Key
            </label>
            <select
              value={tapAction}
              onChange={(e) => setTapAction(e.target.value)}
              className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="">Select a key...</option>
              {availableKeys.map((key) => (
                <option key={key.id} value={key.id}>
                  {key.label} ({key.id})
                </option>
              ))}
            </select>
          </div>
        )}

        {/* Tap/Hold Mapping */}
        {mappingType === 'tap_hold' && (
          <>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Tap Action (quick press)
              </label>
              <select
                value={tapAction}
                onChange={(e) => setTapAction(e.target.value)}
                className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
              >
                <option value="">Select a key...</option>
                {availableKeys.map((key) => (
                  <option key={key.id} value={key.id}>
                    {key.label} ({key.id})
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Hold Action (when held)
              </label>
              <select
                value={holdAction}
                onChange={(e) => setHoldAction(e.target.value)}
                className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
              >
                <option value="">Select a key...</option>
                {availableKeys.filter(k => k.category === 'modifier' || k.category === 'layer').map((key) => (
                  <option key={key.id} value={key.id}>
                    {key.label} ({key.id})
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Hold Threshold (ms): {threshold}
              </label>
              <input
                type="range"
                min="50"
                max="500"
                step="10"
                value={threshold}
                onChange={(e) => setThreshold(parseInt(e.target.value))}
                className="w-full"
              />
              <div className="flex justify-between text-xs text-slate-500 mt-1">
                <span>50ms (fast)</span>
                <span>500ms (slow)</span>
              </div>
            </div>
          </>
        )}

        {/* Layer Switch */}
        {mappingType === 'layer_switch' && (
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Target Layer
            </label>
            <select
              value={targetLayer}
              onChange={(e) => setTargetLayer(e.target.value)}
              className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="">Select a layer...</option>
              {Array.from({ length: 10 }, (_, i) => (
                <option key={i} value={`MD_${i.toString().padStart(2, '0')}`}>
                  Layer MD_{i.toString().padStart(2, '0')}
                </option>
              ))}
            </select>
          </div>
        )}

        {/* Macro (placeholder) */}
        {mappingType === 'macro' && (
          <div className="p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <p className="text-sm text-slate-400">
              Macro configuration coming soon. Use Code Editor tab for now.
            </p>
          </div>
        )}

        {/* Actions */}
        <div className="flex justify-end gap-3 pt-4 border-t border-slate-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-slate-300 hover:text-slate-100 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={
              (mappingType === 'simple' && !tapAction) ||
              (mappingType === 'tap_hold' && (!tapAction || !holdAction)) ||
              (mappingType === 'layer_switch' && !targetLayer)
            }
            className="px-6 py-2 bg-primary-500 text-white rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
          >
            Save Mapping
          </button>
        </div>
      </div>
    </Modal>
  );
}
