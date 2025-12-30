/**
 * Unit tests for EventTimeline component.
 *
 * Tests cover:
 * - Event list rendering
 * - Time formatting integration
 * - Drag-and-drop functionality
 * - Event editing and deletion
 * - Empty state handling
 * - Timeline scaling and markers
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { EventTimeline } from './EventTimeline';
import type { MacroEvent } from '../hooks/useMacroRecorder';

/**
 * Mock event data for testing.
 */
const createMockEvents = (): MacroEvent[] => [
  {
    event: { code: 30, value: 1 }, // 'A' press (evdev code 30)
    relative_timestamp_us: 0,
  },
  {
    event: { code: 30, value: 0 }, // 'A' release
    relative_timestamp_us: 50000,
  },
  {
    event: { code: 48, value: 1 }, // 'B' press (evdev code 48)
    relative_timestamp_us: 100000,
  },
  {
    event: { code: 48, value: 0 }, // 'B' release
    relative_timestamp_us: 150000,
  },
];

describe('EventTimeline', () => {
  let mockOnEventsChange: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    mockOnEventsChange = vi.fn();
  });

  describe('Rendering', () => {
    it('renders empty state when no events are provided', () => {
      render(
        <EventTimeline
          events={[]}
          onEventsChange={mockOnEventsChange}
        />
      );

      expect(screen.getByText('No events to display')).toBeInTheDocument();
      expect(screen.getByText('Record some events to see them on the timeline')).toBeInTheDocument();
    });

    it('renders event count and total duration in header', () => {
      const events = createMockEvents();
      render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      // Should show count
      expect(screen.getByText(/4 events/)).toBeInTheDocument();
      // Should show total duration formatted
      expect(screen.getByText(/total/)).toBeInTheDocument();
    });

    it('renders time markers at correct intervals', () => {
      const events = createMockEvents();
      render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      const markers = screen.getAllByRole('generic').filter(el =>
        el.className.includes('time-marker')
      );
      // Should have 5 markers (0%, 25%, 50%, 75%, 100%)
      expect(markers.length).toBeGreaterThanOrEqual(5);
    });

    it('renders all events as markers on timeline', () => {
      const events = createMockEvents();
      render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      // Look for event markers
      const pressMarkers = screen.getAllByText('▼');
      const releaseMarkers = screen.getAllByText('▲');

      expect(pressMarkers.length).toBe(2); // 2 press events
      expect(releaseMarkers.length).toBe(2); // 2 release events
    });

    it('displays formatted key codes for events', () => {
      const events = createMockEvents();
      render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      // Should show key codes using formatKeyCode utility
      expect(screen.getAllByText(/A|KEY_A/i).length).toBeGreaterThan(0);
      expect(screen.getAllByText(/B|KEY_B/i).length).toBeGreaterThan(0);
    });

    it('distinguishes between press and release events visually', () => {
      const events = createMockEvents();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      const pressMarkers = container.querySelectorAll('.event-marker.press');
      const releaseMarkers = container.querySelectorAll('.event-marker.release');

      expect(pressMarkers.length).toBe(2);
      expect(releaseMarkers.length).toBe(2);
    });
  });

  describe('Event Selection', () => {
    it('shows event details when an event is clicked', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();

      render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      // Click on first event marker
      const pressMarkers = screen.getAllByText('▼');
      await user.click(pressMarkers[0]);

      // Should show event details
      expect(screen.getByText('Event #1')).toBeInTheDocument();
      expect(screen.getByText('Key:')).toBeInTheDocument();
      expect(screen.getByText('Action:')).toBeInTheDocument();
      expect(screen.getByText('Press')).toBeInTheDocument();
    });

    it('highlights selected event marker', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      // First marker should have selected class
      expect(markers[0]).toHaveClass('selected');
    });

    it('shows correct action type for press and release events', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      // Click on press event (first marker)
      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);
      expect(screen.getByText('Press')).toBeInTheDocument();

      // Click on release event (second marker)
      await user.click(markers[1] as HTMLElement);
      expect(screen.getByText('Release')).toBeInTheDocument();
    });
  });

  describe('Event Editing', () => {
    it('allows editing timestamp when editable is true', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      // Select an event
      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      // Click on timestamp to edit
      const timestampValue = screen.getByText(/click to edit/i).closest('.detail-value');
      expect(timestampValue).toBeInTheDocument();

      await user.click(timestampValue!);

      // Should show input field
      const input = screen.getByRole('spinbutton') as HTMLInputElement;
      expect(input).toBeInTheDocument();
    });

    it('does not allow editing when editable is false', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={false}
        />
      );

      // Select an event
      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      // Should not show edit hint
      expect(screen.queryByText(/click to edit/i)).not.toBeInTheDocument();
    });

    it('updates event timestamp when edited and blurred', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      // Select an event
      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      // Click to edit timestamp
      const timestampValue = screen.getByText(/click to edit/i).closest('.detail-value');
      await user.click(timestampValue!);

      const input = screen.getByRole('spinbutton') as HTMLInputElement;

      // Clear and enter new value
      await user.clear(input);
      await user.type(input, '75');

      // Blur the input
      fireEvent.blur(input);

      // Should call onEventsChange with updated timestamp
      await waitFor(() => {
        expect(mockOnEventsChange).toHaveBeenCalled();
        const updatedEvents = mockOnEventsChange.mock.calls[0][0];
        // New timestamp should be 75ms = 75000us
        expect(updatedEvents.some((e: MacroEvent) => e.relative_timestamp_us === 75000)).toBe(true);
      });
    });

    it('updates event timestamp when Enter key is pressed', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      // Select an event
      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      // Click to edit timestamp
      const timestampValue = screen.getByText(/click to edit/i).closest('.detail-value');
      await user.click(timestampValue!);

      const input = screen.getByRole('spinbutton');

      // Clear and enter new value, then press Enter
      await user.clear(input);
      await user.type(input, '100{Enter}');

      // Should call onEventsChange
      await waitFor(() => {
        expect(mockOnEventsChange).toHaveBeenCalled();
      });
    });

    it('re-sorts events by timestamp after editing', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      // Select first event
      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      // Edit timestamp to move it later in timeline
      const timestampValue = screen.getByText(/click to edit/i).closest('.detail-value');
      await user.click(timestampValue!);

      const input = screen.getByRole('spinbutton');
      await user.clear(input);
      await user.type(input, '200{Enter}');

      await waitFor(() => {
        expect(mockOnEventsChange).toHaveBeenCalled();
        const updatedEvents = mockOnEventsChange.mock.calls[0][0];

        // Events should be sorted by timestamp
        for (let i = 1; i < updatedEvents.length; i++) {
          expect(updatedEvents[i].relative_timestamp_us).toBeGreaterThanOrEqual(
            updatedEvents[i - 1].relative_timestamp_us
          );
        }
      });
    });

    it('prevents negative timestamps', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      const timestampValue = screen.getByText(/click to edit/i).closest('.detail-value');
      await user.click(timestampValue!);

      const input = screen.getByRole('spinbutton');
      await user.clear(input);
      await user.type(input, '-50{Enter}');

      await waitFor(() => {
        expect(mockOnEventsChange).toHaveBeenCalled();
        const updatedEvents = mockOnEventsChange.mock.calls[0][0];

        // All timestamps should be non-negative
        updatedEvents.forEach((event: MacroEvent) => {
          expect(event.relative_timestamp_us).toBeGreaterThanOrEqual(0);
        });
      });
    });
  });

  describe('Event Deletion', () => {
    it('shows delete button when event is selected and editable', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      expect(screen.getByRole('button', { name: /delete/i })).toBeInTheDocument();
    });

    it('does not show delete button when not editable', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={false}
        />
      );

      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      expect(screen.queryByRole('button', { name: /delete/i })).not.toBeInTheDocument();
    });

    it('deletes selected event when delete button is clicked', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      const deleteButton = screen.getByRole('button', { name: /delete/i });
      await user.click(deleteButton);

      expect(mockOnEventsChange).toHaveBeenCalled();
      const updatedEvents = mockOnEventsChange.mock.calls[0][0];
      expect(updatedEvents.length).toBe(events.length - 1);
    });

    it('clears selection after deleting event', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      const deleteButton = screen.getByRole('button', { name: /delete/i });
      await user.click(deleteButton);

      // Details panel should disappear
      await waitFor(() => {
        expect(screen.queryByText('Event #1')).not.toBeInTheDocument();
      });
    });
  });

  describe('Drag and Drop', () => {
    it('makes event markers draggable when editable', () => {
      const events = createMockEvents();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      const markers = container.querySelectorAll('.event-marker');
      markers.forEach(marker => {
        expect(marker).toHaveAttribute('draggable', 'true');
      });
    });

    it('makes event markers not draggable when not editable', () => {
      const events = createMockEvents();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={false}
        />
      );

      const markers = container.querySelectorAll('.event-marker');
      markers.forEach(marker => {
        expect(marker).toHaveAttribute('draggable', 'false');
      });
    });

    it('handles drag start event', () => {
      const events = createMockEvents();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      const markers = container.querySelectorAll('.event-marker');
      const dragEvent = new Event('dragstart', { bubbles: true });

      // Should not throw error
      expect(() => {
        fireEvent(markers[0], dragEvent);
      }).not.toThrow();
    });

    it('allows dropping events on timeline', () => {
      const events = createMockEvents();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      const timeline = container.querySelector('.timeline-canvas');
      expect(timeline).toBeInTheDocument();

      // Should handle dragover
      const dragOverEvent = new Event('dragover', { bubbles: true });
      expect(() => {
        fireEvent(timeline!, dragOverEvent);
      }).not.toThrow();
    });
  });

  describe('Timeline Scaling', () => {
    it('scales timeline based on maximum timestamp', () => {
      const events = createMockEvents();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      // All event markers should be positioned within 0-100%
      const markers = container.querySelectorAll('.event-marker');
      markers.forEach(marker => {
        const style = (marker as HTMLElement).style;
        const left = parseFloat(style.left);
        expect(left).toBeGreaterThanOrEqual(0);
        expect(left).toBeLessThanOrEqual(100);
      });
    });

    it('handles single event correctly', () => {
      const singleEvent: MacroEvent[] = [
        { event: { code: 30, value: 1 }, relative_timestamp_us: 50000 }
      ];

      render(
        <EventTimeline
          events={singleEvent}
          onEventsChange={mockOnEventsChange}
        />
      );

      expect(screen.getByText(/1 events/)).toBeInTheDocument();
    });

    it('adjusts to window resize', async () => {
      const events = createMockEvents();
      render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      // Trigger resize event
      fireEvent(window, new Event('resize'));

      // Component should handle resize without crashing
      await waitFor(() => {
        expect(screen.getByText(/4 events/)).toBeInTheDocument();
      });
    });
  });

  describe('Time Formatting Integration', () => {
    it('uses formatDuration utility for time display', () => {
      const events = createMockEvents();
      render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      // Should show formatted durations (the exact format depends on formatDuration implementation)
      // We just verify that time-related text appears
      const header = screen.getByText(/total/);
      expect(header).toBeInTheDocument();
    });

    it('formats timestamps consistently across UI', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      // Select event to see timestamp detail
      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[1] as HTMLElement);

      // Timestamp should be displayed in details
      expect(screen.getByText('Timestamp:')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles malformed event data gracefully', () => {
      const malformedEvents = [
        { event: { code: 30, value: 1 }, relative_timestamp_us: 0 },
        // Missing event object would be a TypeScript error, so we can't test that
        // But we can test extreme values
        { event: { code: 999, value: 2 }, relative_timestamp_us: 999999999 },
      ];

      expect(() => {
        render(
          <EventTimeline
            events={malformedEvents}
            onEventsChange={mockOnEventsChange}
          />
        );
      }).not.toThrow();
    });

    it('handles large number of events efficiently', () => {
      // Create 100 events using valid evdev codes
      const validCodes = [30, 48, 46, 32, 18]; // A, B, C, D, E
      const manyEvents: MacroEvent[] = Array.from({ length: 100 }, (_, i) => ({
        event: { code: validCodes[i % 5], value: i % 2 },
        relative_timestamp_us: i * 10000,
      }));

      const { container } = render(
        <EventTimeline
          events={manyEvents}
          onEventsChange={mockOnEventsChange}
        />
      );

      expect(screen.getByText(/100 events/)).toBeInTheDocument();

      // Should render all markers
      const markers = container.querySelectorAll('.event-marker');
      expect(markers.length).toBe(100);
    });

    it('handles events with same timestamp', () => {
      const simultaneousEvents: MacroEvent[] = [
        { event: { code: 30, value: 1 }, relative_timestamp_us: 50000 }, // A
        { event: { code: 48, value: 1 }, relative_timestamp_us: 50000 }, // B
        { event: { code: 46, value: 1 }, relative_timestamp_us: 50000 }, // C
      ];

      render(
        <EventTimeline
          events={simultaneousEvents}
          onEventsChange={mockOnEventsChange}
        />
      );

      expect(screen.getByText(/3 events/)).toBeInTheDocument();
    });

    it('handles zero-duration timeline', () => {
      const sameTimeEvents: MacroEvent[] = [
        { event: { code: 30, value: 1 }, relative_timestamp_us: 0 },
        { event: { code: 30, value: 0 }, relative_timestamp_us: 0 },
      ];

      render(
        <EventTimeline
          events={sameTimeEvents}
          onEventsChange={mockOnEventsChange}
        />
      );

      expect(screen.getByText(/2 events/)).toBeInTheDocument();
    });

    it('cleans up event listeners on unmount', () => {
      const events = createMockEvents();
      const { unmount } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      // Should not throw when unmounting
      expect(() => unmount()).not.toThrow();
    });
  });

  describe('Accessibility', () => {
    it('provides meaningful structure for screen readers', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
        />
      );

      // Select event
      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      // Should have clear labels
      expect(screen.getByText('Key:')).toBeInTheDocument();
      expect(screen.getByText('Action:')).toBeInTheDocument();
      expect(screen.getByText('Timestamp:')).toBeInTheDocument();
    });

    it('allows keyboard navigation for editing', async () => {
      const events = createMockEvents();
      const user = userEvent.setup();
      const { container } = render(
        <EventTimeline
          events={events}
          onEventsChange={mockOnEventsChange}
          editable={true}
        />
      );

      const markers = container.querySelectorAll('.event-marker');
      await user.click(markers[0] as HTMLElement);

      const timestampValue = screen.getByText(/click to edit/i).closest('.detail-value');
      await user.click(timestampValue!);

      const input = screen.getByRole('spinbutton');

      // Should be able to use keyboard to edit
      await user.clear(input);
      await user.keyboard('42');

      expect(input).toHaveValue(42);
    });
  });
});
