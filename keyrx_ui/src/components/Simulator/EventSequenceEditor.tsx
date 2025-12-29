import React, { useState } from 'react';
import type { SimKeyEvent, EventSequence } from '../../wasm/core';
import './EventSequenceEditor.css';

interface EventSequenceEditorProps {
  onSubmit: (sequence: EventSequence) => void;
  disabled?: boolean;
}

interface EventForm {
  id: string;
  keyCode: string;
  eventType: 'press' | 'release';
  timestamp: number;
}

const COMMON_KEY_CODES = [
  { code: 'VK_A', name: 'A' },
  { code: 'VK_B', name: 'B' },
  { code: 'VK_C', name: 'C' },
  { code: 'VK_D', name: 'D' },
  { code: 'VK_E', name: 'E' },
  { code: 'VK_F', name: 'F' },
  { code: 'VK_G', name: 'G' },
  { code: 'VK_H', name: 'H' },
  { code: 'VK_I', name: 'I' },
  { code: 'VK_J', name: 'J' },
  { code: 'VK_K', name: 'K' },
  { code: 'VK_L', name: 'L' },
  { code: 'VK_M', name: 'M' },
  { code: 'VK_N', name: 'N' },
  { code: 'VK_O', name: 'O' },
  { code: 'VK_P', name: 'P' },
  { code: 'VK_Q', name: 'Q' },
  { code: 'VK_R', name: 'R' },
  { code: 'VK_S', name: 'S' },
  { code: 'VK_T', name: 'T' },
  { code: 'VK_U', name: 'U' },
  { code: 'VK_V', name: 'V' },
  { code: 'VK_W', name: 'W' },
  { code: 'VK_X', name: 'X' },
  { code: 'VK_Y', name: 'Y' },
  { code: 'VK_Z', name: 'Z' },
  { code: 'VK_LShift', name: 'Left Shift' },
  { code: 'VK_LCtrl', name: 'Left Ctrl' },
  { code: 'VK_LAlt', name: 'Left Alt' },
  { code: 'VK_Space', name: 'Space' },
  { code: 'VK_Enter', name: 'Enter' },
  { code: 'VK_Esc', name: 'Esc' },
];

export const EventSequenceEditor: React.FC<EventSequenceEditorProps> = ({
  onSubmit,
  disabled = false,
}) => {
  const [events, setEvents] = useState<EventForm[]>([]);
  const [nextId, setNextId] = useState(1);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({});

  const createNewEvent = (): EventForm => ({
    id: `event-${nextId}`,
    keyCode: 'VK_A', // Default to 'A'
    eventType: 'press',
    timestamp: events.length > 0 ? events[events.length - 1].timestamp + 100 : 0,
  });

  const addEvent = () => {
    const newEvent = createNewEvent();
    setEvents([...events, newEvent]);
    setNextId(nextId + 1);
    setEditingId(newEvent.id);
  };

  const removeEvent = (id: string) => {
    setEvents(events.filter((e) => e.id !== id));
    setValidationErrors((prev) => {
      const newErrors = { ...prev };
      delete newErrors[id];
      return newErrors;
    });
  };

  const updateEvent = (id: string, field: keyof EventForm, value: string | number) => {
    setEvents(
      events.map((e) => (e.id === id ? { ...e, [field]: value } : e))
    );
    // Clear validation error for this field
    if (validationErrors[id]) {
      setValidationErrors((prev) => {
        const newErrors = { ...prev };
        delete newErrors[id];
        return newErrors;
      });
    }
  };

  const validateSequence = (): boolean => {
    const errors: Record<string, string> = {};
    let isValid = true;

    // Check each event
    events.forEach((event, index) => {
      // Validate timestamp is positive
      if (event.timestamp < 0) {
        errors[event.id] = 'Timestamp must be positive';
        isValid = false;
        return;
      }

      // Validate timestamps are in ascending order
      if (index > 0 && event.timestamp <= events[index - 1].timestamp) {
        errors[event.id] = 'Timestamp must be greater than previous event';
        isValid = false;
        return;
      }

      // Validate key code is not empty
      if (!event.keyCode || event.keyCode.trim() === '') {
        errors[event.id] = 'Key code is required';
        isValid = false;
        return;
      }
    });

    setValidationErrors(errors);
    return isValid;
  };

  const handleSubmit = () => {
    if (!validateSequence()) {
      return;
    }

    if (events.length === 0) {
      alert('Please add at least one event to the sequence');
      return;
    }

    // Convert EventForm[] to EventSequence
    const keyEvents: SimKeyEvent[] = events.map((e) => ({
      keycode: e.keyCode,
      event_type: e.eventType,
      timestamp_us: e.timestamp,
    }));

    const sequence: EventSequence = {
      events: keyEvents,
    };

    onSubmit(sequence);
  };

  return (
    <div className="event-sequence-editor">
      <div className="editor-header">
        <h3>Custom Event Sequence</h3>
        <button
          onClick={addEvent}
          disabled={disabled}
          className="btn-add"
          title="Add new event (Ctrl+Enter)"
        >
          + Add Event
        </button>
      </div>

      {events.length === 0 && (
        <div className="empty-state">
          <p>No events yet. Add events to create a custom sequence.</p>
          <p className="hint">
            Events must have ascending timestamps (in microseconds).
          </p>
        </div>
      )}

      <div className="events-list">
        {events.map((event, index) => (
          <div
            key={event.id}
            className={`event-item ${editingId === event.id ? 'editing' : ''} ${
              validationErrors[event.id] ? 'error' : ''
            }`}
          >
            <div className="event-number">{index + 1}</div>

            <div className="event-fields">
              <div className="field">
                <label htmlFor={`keycode-${event.id}`}>Key Code:</label>
                <select
                  id={`keycode-${event.id}`}
                  value={event.keyCode}
                  onChange={(e) =>
                    updateEvent(event.id, 'keyCode', e.target.value)
                  }
                  disabled={disabled}
                >
                  {COMMON_KEY_CODES.map((key) => (
                    <option key={key.code} value={key.code}>
                      {key.name} ({key.code})
                    </option>
                  ))}
                </select>
              </div>

              <div className="field">
                <label htmlFor={`type-${event.id}`}>Type:</label>
                <select
                  id={`type-${event.id}`}
                  value={event.eventType}
                  onChange={(e) =>
                    updateEvent(event.id, 'eventType', e.target.value as 'press' | 'release')
                  }
                  disabled={disabled}
                >
                  <option value="press">Press</option>
                  <option value="release">Release</option>
                </select>
              </div>

              <div className="field">
                <label htmlFor={`timestamp-${event.id}`}>
                  Timestamp (μs):
                </label>
                <input
                  id={`timestamp-${event.id}`}
                  type="number"
                  value={event.timestamp}
                  onChange={(e) =>
                    updateEvent(event.id, 'timestamp', parseInt(e.target.value, 10) || 0)
                  }
                  disabled={disabled}
                  min="0"
                  step="100"
                />
              </div>
            </div>

            <button
              onClick={() => removeEvent(event.id)}
              disabled={disabled}
              className="btn-remove"
              title="Remove event"
            >
              ×
            </button>

            {validationErrors[event.id] && (
              <div className="validation-error">{validationErrors[event.id]}</div>
            )}
          </div>
        ))}
      </div>

      {events.length > 0 && (
        <div className="editor-footer">
          <div className="event-summary">
            <span className="summary-item">
              <strong>{events.length}</strong> events
            </span>
            <span className="summary-item">
              <strong>{events[events.length - 1]?.timestamp || 0}</strong> μs duration
            </span>
          </div>
          <button
            onClick={handleSubmit}
            disabled={disabled || events.length === 0}
            className="btn-simulate"
          >
            Simulate Custom Sequence
          </button>
        </div>
      )}
    </div>
  );
};
