import React from 'react';
import { Card } from './Card';

/**
 * Key Palette - Shows available keys/modifiers/layers for assignment
 * Based on 2025 UI/UX trends: categorized, searchable, drag-friendly
 */

export interface PaletteKey {
  id: string;
  label: string;
  category: 'virtual_key' | 'modifier' | 'lock' | 'layer';
  description?: string;
}

interface KeyPaletteProps {
  onKeySelect: (key: PaletteKey) => void;
  selectedKey?: PaletteKey | null;
}

// Comprehensive key lists based on keyrx specification
const VIRTUAL_KEYS: PaletteKey[] = [
  // Letters
  { id: 'A', label: 'A', category: 'virtual_key' },
  { id: 'B', label: 'B', category: 'virtual_key' },
  { id: 'C', label: 'C', category: 'virtual_key' },
  { id: 'D', label: 'D', category: 'virtual_key' },
  { id: 'E', label: 'E', category: 'virtual_key' },
  { id: 'F', label: 'F', category: 'virtual_key' },
  { id: 'G', label: 'G', category: 'virtual_key' },
  { id: 'H', label: 'H', category: 'virtual_key' },
  { id: 'I', label: 'I', category: 'virtual_key' },
  { id: 'J', label: 'J', category: 'virtual_key' },
  { id: 'K', label: 'K', category: 'virtual_key' },
  { id: 'L', label: 'L', category: 'virtual_key' },
  { id: 'M', label: 'M', category: 'virtual_key' },
  { id: 'N', label: 'N', category: 'virtual_key' },
  { id: 'O', label: 'O', category: 'virtual_key' },
  { id: 'P', label: 'P', category: 'virtual_key' },
  { id: 'Q', label: 'Q', category: 'virtual_key' },
  { id: 'R', label: 'R', category: 'virtual_key' },
  { id: 'S', label: 'S', category: 'virtual_key' },
  { id: 'T', label: 'T', category: 'virtual_key' },
  { id: 'U', label: 'U', category: 'virtual_key' },
  { id: 'V', label: 'V', category: 'virtual_key' },
  { id: 'W', label: 'W', category: 'virtual_key' },
  { id: 'X', label: 'X', category: 'virtual_key' },
  { id: 'Y', label: 'Y', category: 'virtual_key' },
  { id: 'Z', label: 'Z', category: 'virtual_key' },
  // Numbers
  { id: '0', label: '0', category: 'virtual_key' },
  { id: '1', label: '1', category: 'virtual_key' },
  { id: '2', label: '2', category: 'virtual_key' },
  { id: '3', label: '3', category: 'virtual_key' },
  { id: '4', label: '4', category: 'virtual_key' },
  { id: '5', label: '5', category: 'virtual_key' },
  { id: '6', label: '6', category: 'virtual_key' },
  { id: '7', label: '7', category: 'virtual_key' },
  { id: '8', label: '8', category: 'virtual_key' },
  { id: '9', label: '9', category: 'virtual_key' },
  // Function Keys
  { id: 'F1', label: 'F1', category: 'virtual_key' },
  { id: 'F2', label: 'F2', category: 'virtual_key' },
  { id: 'F3', label: 'F3', category: 'virtual_key' },
  { id: 'F4', label: 'F4', category: 'virtual_key' },
  { id: 'F5', label: 'F5', category: 'virtual_key' },
  { id: 'F6', label: 'F6', category: 'virtual_key' },
  { id: 'F7', label: 'F7', category: 'virtual_key' },
  { id: 'F8', label: 'F8', category: 'virtual_key' },
  { id: 'F9', label: 'F9', category: 'virtual_key' },
  { id: 'F10', label: 'F10', category: 'virtual_key' },
  { id: 'F11', label: 'F11', category: 'virtual_key' },
  { id: 'F12', label: 'F12', category: 'virtual_key' },
  // Special Keys
  { id: 'Escape', label: 'Esc', category: 'virtual_key' },
  { id: 'Enter', label: 'Enter', category: 'virtual_key' },
  { id: 'Space', label: 'Space', category: 'virtual_key' },
  { id: 'Backspace', label: 'BS', category: 'virtual_key' },
  { id: 'Tab', label: 'Tab', category: 'virtual_key' },
  { id: 'Delete', label: 'Del', category: 'virtual_key' },
  { id: 'Insert', label: 'Ins', category: 'virtual_key' },
  { id: 'Home', label: 'Home', category: 'virtual_key' },
  { id: 'End', label: 'End', category: 'virtual_key' },
  { id: 'PageUp', label: 'PgUp', category: 'virtual_key' },
  { id: 'PageDown', label: 'PgDn', category: 'virtual_key' },
  // Arrows
  { id: 'Up', label: '↑', category: 'virtual_key' },
  { id: 'Down', label: '↓', category: 'virtual_key' },
  { id: 'Left', label: '←', category: 'virtual_key' },
  { id: 'Right', label: '→', category: 'virtual_key' },
  // Symbols
  { id: 'Minus', label: '-', category: 'virtual_key' },
  { id: 'Equal', label: '=', category: 'virtual_key' },
  { id: 'LeftBracket', label: '[', category: 'virtual_key' },
  { id: 'RightBracket', label: ']', category: 'virtual_key' },
  { id: 'Backslash', label: '\\', category: 'virtual_key' },
  { id: 'Semicolon', label: ';', category: 'virtual_key' },
  { id: 'Quote', label: "'", category: 'virtual_key' },
  { id: 'Comma', label: ',', category: 'virtual_key' },
  { id: 'Period', label: '.', category: 'virtual_key' },
  { id: 'Slash', label: '/', category: 'virtual_key' },
];

const MODIFIERS: PaletteKey[] = [
  { id: 'MD_00', label: 'MD_00', category: 'modifier', description: 'Modifier/Layer 0' },
  { id: 'MD_01', label: 'MD_01', category: 'modifier', description: 'Modifier/Layer 1' },
  { id: 'MD_02', label: 'MD_02', category: 'modifier', description: 'Modifier/Layer 2' },
  { id: 'MD_03', label: 'MD_03', category: 'modifier', description: 'Modifier/Layer 3' },
  { id: 'MD_04', label: 'MD_04', category: 'modifier', description: 'Modifier/Layer 4' },
  { id: 'MD_05', label: 'MD_05', category: 'modifier', description: 'Modifier/Layer 5' },
  { id: 'MD_06', label: 'MD_06', category: 'modifier', description: 'Modifier/Layer 6' },
  { id: 'MD_07', label: 'MD_07', category: 'modifier', description: 'Modifier/Layer 7' },
  { id: 'MD_08', label: 'MD_08', category: 'modifier', description: 'Modifier/Layer 8' },
  { id: 'MD_09', label: 'MD_09', category: 'modifier', description: 'Modifier/Layer 9' },
  { id: 'LCtrl', label: 'LCtrl', category: 'modifier', description: 'Left Control' },
  { id: 'RCtrl', label: 'RCtrl', category: 'modifier', description: 'Right Control' },
  { id: 'LShift', label: 'LShift', category: 'modifier', description: 'Left Shift' },
  { id: 'RShift', label: 'RShift', category: 'modifier', description: 'Right Shift' },
  { id: 'LAlt', label: 'LAlt', category: 'modifier', description: 'Left Alt' },
  { id: 'RAlt', label: 'RAlt', category: 'modifier', description: 'Right Alt' },
  { id: 'LMeta', label: 'LWin', category: 'modifier', description: 'Left Windows/Super' },
  { id: 'RMeta', label: 'RWin', category: 'modifier', description: 'Right Windows/Super' },
];

const LOCKS: PaletteKey[] = [
  { id: 'LK_00', label: 'LK_00', category: 'lock', description: 'Lock 0 (CapsLock)' },
  { id: 'LK_01', label: 'LK_01', category: 'lock', description: 'Lock 1 (NumLock)' },
  { id: 'LK_02', label: 'LK_02', category: 'lock', description: 'Lock 2 (ScrollLock)' },
  { id: 'LK_03', label: 'LK_03', category: 'lock', description: 'Lock 3' },
  { id: 'LK_04', label: 'LK_04', category: 'lock', description: 'Lock 4' },
  { id: 'LK_05', label: 'LK_05', category: 'lock', description: 'Lock 5' },
  { id: 'LK_06', label: 'LK_06', category: 'lock', description: 'Lock 6' },
  { id: 'LK_07', label: 'LK_07', category: 'lock', description: 'Lock 7' },
  { id: 'LK_08', label: 'LK_08', category: 'lock', description: 'Lock 8' },
  { id: 'LK_09', label: 'LK_09', category: 'lock', description: 'Lock 9' },
];

const LAYERS: PaletteKey[] = [
  { id: 'MD_00', label: 'Base (MD_00)', category: 'layer', description: 'Base layer' },
  { id: 'MD_01', label: 'Layer MD_01', category: 'layer', description: 'Layer 1' },
  { id: 'MD_02', label: 'Layer MD_02', category: 'layer', description: 'Layer 2' },
  { id: 'MD_03', label: 'Layer MD_03', category: 'layer', description: 'Layer 3' },
  { id: 'MD_04', label: 'Layer MD_04', category: 'layer', description: 'Layer 4' },
  { id: 'MD_05', label: 'Layer MD_05', category: 'layer', description: 'Layer 5' },
  { id: 'MD_06', label: 'Layer MD_06', category: 'layer', description: 'Layer 6' },
  { id: 'MD_07', label: 'Layer MD_07', category: 'layer', description: 'Layer 7' },
  { id: 'MD_08', label: 'Layer MD_08', category: 'layer', description: 'Layer 8' },
  { id: 'MD_09', label: 'Layer MD_09', category: 'layer', description: 'Layer 9' },
];

export function KeyPalette({ onKeySelect, selectedKey }: KeyPaletteProps) {
  const [activeCategory, setActiveCategory] = React.useState<PaletteKey['category']>('virtual_key');

  const categories = [
    { id: 'virtual_key' as const, label: 'Keys', keys: VIRTUAL_KEYS },
    { id: 'modifier' as const, label: 'Modifiers', keys: MODIFIERS },
    { id: 'lock' as const, label: 'Locks', keys: LOCKS },
    { id: 'layer' as const, label: 'Layers', keys: LAYERS },
  ];

  const activeKeys = categories.find(c => c.id === activeCategory)?.keys || [];

  return (
    <Card className="h-full">
      <h3 className="text-lg font-semibold text-slate-100 mb-4">Key Palette</h3>

      {/* Category Tabs */}
      <div className="flex gap-1 mb-4 border-b border-slate-700">
        {categories.map(cat => (
          <button
            key={cat.id}
            onClick={() => setActiveCategory(cat.id)}
            className={`px-3 py-2 text-sm font-medium transition-colors ${
              activeCategory === cat.id
                ? 'text-primary-400 border-b-2 border-primary-400'
                : 'text-slate-400 hover:text-slate-300'
            }`}
          >
            {cat.label}
          </button>
        ))}
      </div>

      {/* Key Grid - Keyboard keycap style */}
      <div className="grid grid-cols-8 gap-2 overflow-y-auto max-h-[400px] p-4 bg-slate-800/50 rounded-lg">
        {activeKeys.map(key => (
          <button
            key={key.id}
            onClick={() => onKeySelect(key)}
            className={`
              relative flex flex-col items-center justify-center
              min-h-[50px] px-2 py-2
              rounded border transition-all
              hover:brightness-110 hover:-translate-y-0.5 hover:shadow-lg
              ${
                selectedKey?.id === key.id
                  ? 'border-primary-500 bg-primary-500/20 shadow-lg shadow-primary-500/50'
                  : 'border-slate-600 bg-slate-700 hover:border-slate-500'
              }
              ${key.category === 'modifier' ? 'border-cyan-500/50' : ''}
              ${key.category === 'lock' ? 'border-purple-500/50' : ''}
              ${key.category === 'layer' ? 'border-yellow-500/50' : ''}
            `}
            title={key.description || key.id}
          >
            {/* Key label (main) */}
            <div className="text-sm font-bold text-white font-mono">
              {key.label}
            </div>
            {/* Key ID (small, below) */}
            <div className="text-[9px] text-slate-400 mt-0.5 font-mono">
              {key.id}
            </div>
          </button>
        ))}
      </div>

      {/* Hint */}
      <p className="text-xs text-slate-500 mt-4">
        Click a key from palette, then click a keyboard key to assign
      </p>
    </Card>
  );
}
