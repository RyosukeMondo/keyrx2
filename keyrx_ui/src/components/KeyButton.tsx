import React, { useMemo } from 'react';
import { Tooltip } from './Tooltip';
import { cn } from '../utils/cn';
import type { KeyMapping } from '@/types';

interface KeyButtonProps {
  keyCode: string;
  label: string;
  mapping?: KeyMapping;
  onClick: () => void;
  isPressed?: boolean;
  className?: string;
}

export const KeyButton = React.memo<KeyButtonProps>(
  ({ keyCode, label, mapping, onClick, isPressed = false, className = '' }) => {
    const hasMapping = !!mapping;

    const tooltipContent = useMemo(() => {
      if (!mapping) return `${keyCode} (Default)`;

      switch (mapping.type) {
        case 'simple':
          return `${keyCode} → ${mapping.tapAction}`;
        case 'tap_hold':
          return `${keyCode} → Tap: ${mapping.tapAction}, Hold: ${mapping.holdAction} (${mapping.threshold}ms)`;
        case 'macro':
          return `${keyCode} → Macro (${mapping.macroSteps?.length || 0} steps)`;
        case 'layer_switch':
          return `${keyCode} → Layer: ${mapping.targetLayer}`;
        default:
          return `${keyCode} (Default)`;
      }
    }, [keyCode, mapping]);

    // Get remap display text
    const remapText = useMemo(() => {
      if (!mapping) return '';

      switch (mapping.type) {
        case 'simple':
          return mapping.tapAction || '';
        case 'tap_hold':
          return `T:${mapping.tapAction} H:${mapping.holdAction}`;
        case 'macro':
          return 'MACRO';
        case 'layer_switch':
          return mapping.targetLayer || '';
        default:
          return '';
      }
    }, [mapping]);

    // Determine border and background color based on mapping type
    const getKeyStyle = () => {
      if (!mapping) return {
        border: 'border-slate-600',
        bg: 'bg-slate-700',
      };

      switch (mapping.type) {
        case 'simple':
          return {
            border: 'border-green-500',
            bg: 'bg-slate-700',
          };
        case 'tap_hold':
          return {
            border: 'border-red-500',
            bg: 'bg-red-900/15',
          };
        case 'macro':
          return {
            border: 'border-purple-500',
            bg: 'bg-purple-900/15',
          };
        case 'layer_switch':
          return {
            border: 'border-yellow-500',
            bg: 'bg-yellow-900/15',
          };
        default:
          return {
            border: 'border-slate-600',
            bg: 'bg-slate-700',
          };
      }
    };

    const style = getKeyStyle();

    return (
      <Tooltip content={tooltipContent}>
        <button
          onClick={onClick}
          aria-label={`Key ${keyCode}. Current mapping: ${tooltipContent}. Click to configure.`}
          className={cn(
            'relative flex flex-col items-center justify-center',
            'rounded border transition-all duration-150',
            'hover:brightness-110 hover:-translate-y-0.5',
            'focus:outline focus:outline-2 focus:outline-primary-500',
            'min-h-[50px]',
            style.border,
            style.bg,
            isPressed && 'bg-green-500 border-green-400',
            className
          )}
          style={{
            aspectRatio: '1',
          }}
        >
          {/* Original key label (small, gray) */}
          <span className="text-[10px] text-slate-400 font-mono">
            {label}
          </span>

          {/* Remap label (bold, yellow) */}
          {hasMapping && (
            <span className="text-[12px] text-yellow-300 font-bold font-mono mt-0.5">
              {remapText}
            </span>
          )}
        </button>
      </Tooltip>
    );
  }
);

KeyButton.displayName = 'KeyButton';
