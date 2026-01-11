import React, { useState } from 'react';

/**
 * Layer Switcher - Beautiful layer selection styled like user_layout.html
 * Supports MD_00 to MD_FF (0-255 layers)
 */

interface LayerSwitcherProps {
  activeLayer: string;
  availableLayers: string[];
  onLayerChange: (layer: string) => void;
}

export function LayerSwitcher({
  activeLayer,
  availableLayers,
  onLayerChange,
}: LayerSwitcherProps) {
  const [customLayerInput, setCustomLayerInput] = useState('');

  // Quick access layers (base + first 9 modifiers)
  const quickLayers = ['base', ...Array.from({ length: 10 }, (_, i) => `md-${i.toString().padStart(2, '0')}`)];

  const handleCustomLayerSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const layerNum = parseInt(customLayerInput, 16);
    if (!isNaN(layerNum) && layerNum >= 0 && layerNum <= 255) {
      const layerName = `md-${layerNum.toString(16).padStart(2, '0')}`;
      onLayerChange(layerName);
      setCustomLayerInput('');
    }
  };

  return (
    <div className="p-4 bg-gradient-to-r from-red-900/20 to-red-800/20 rounded-lg border border-red-500/30">
      <div className="flex items-center gap-4 flex-wrap">
        <span className="text-red-400 font-bold text-sm">LAYERS:</span>

        {/* Quick access layer buttons */}
        {quickLayers.map((layer) => (
          <button
            key={layer}
            onClick={() => onLayerChange(layer)}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-all transform hover:scale-105 ${
              activeLayer === layer
                ? 'bg-red-500 text-white shadow-lg shadow-red-500/50'
                : 'bg-slate-800 text-slate-300 border border-red-500/50 hover:bg-slate-700 hover:border-red-400'
            }`}
          >
            {layer === 'base' ? 'Base Layer' : layer.toUpperCase()}
          </button>
        ))}

        {/* Custom layer input (MD_00 to MD_FF) */}
        <form onSubmit={handleCustomLayerSubmit} className="flex items-center gap-2 ml-4">
          <label htmlFor="custom-layer" className="text-red-400 text-sm">
            MD_
          </label>
          <input
            id="custom-layer"
            type="text"
            value={customLayerInput}
            onChange={(e) => setCustomLayerInput(e.target.value.toUpperCase())}
            placeholder="00-FF"
            maxLength={2}
            className="w-16 px-2 py-1 bg-slate-800 border border-red-500/50 rounded text-white text-sm font-mono text-center focus:outline-none focus:ring-2 focus:ring-red-500"
          />
          <button
            type="submit"
            className="px-3 py-1 bg-red-600 text-white text-sm rounded hover:bg-red-500 transition-colors"
          >
            Go
          </button>
        </form>

        {/* Current layer display (if custom) */}
        {activeLayer !== 'base' && !quickLayers.includes(activeLayer) && (
          <div className="ml-2 px-3 py-1 bg-red-500/20 border border-red-500 rounded text-red-300 text-sm">
            Current: {activeLayer.toUpperCase()}
          </div>
        )}
      </div>
    </div>
  );
}
