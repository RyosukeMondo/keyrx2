/**
 * Integration tests for ConfigPage
 * Tests key configuration flow with API mocking via MSW
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ConfigPage } from './ConfigPage';
import { useConfigStore } from '../stores/configStore';

// Mock react-router-dom
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
  useParams: () => ({ profile: 'default' }),
  useSearchParams: () => [new URLSearchParams(), vi.fn()],
}));

describe('ConfigPage - Integration Tests', () => {
  beforeEach(() => {
    // Reset store state before each test
    const store = useConfigStore.getState();
    store.config = null;
    store.loading = false;
    store.error = null;
  });

  describe('Layer selector flow', () => {
    it('displays layer selector with available layers', async () => {
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      // Should show layer selector
      const layerSelector = screen.getByRole('combobox', {
        name: /Layer/i,
      });
      expect(layerSelector).toBeInTheDocument();
    });

    it('switches between layers', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        const layerSelector = screen.queryByRole('combobox', {
          name: /Layer/i,
        });
        if (layerSelector) {
          expect(layerSelector).toBeInTheDocument();
        }
      });

      // If multiple layers exist, test switching
      const layerSelector = screen.queryByRole('combobox', {
        name: /Layer/i,
      });

      if (layerSelector) {
        const options = layerSelector.querySelectorAll('option');
        if (options.length > 1) {
          await user.selectOptions(layerSelector, options[1].value);

          expect(layerSelector).toHaveValue(options[1].value);
        }
      }
    });
  });

  describe('Layout preset selector flow', () => {
    it('changes keyboard layout preset', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const layoutSelector = screen.getByRole('combobox', {
        name: /Keyboard Layout/i,
      });

      // Should have ANSI_104 as default
      expect(layoutSelector).toHaveValue('ANSI_104');

      // Change to ISO_105
      await user.selectOptions(layoutSelector, 'ISO_105');

      expect(layoutSelector).toHaveValue('ISO_105');
    });

    it('displays all available layout options', async () => {
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const layoutSelector = screen.getByRole('combobox', {
        name: /Keyboard Layout/i,
      });

      // Should have all layout options
      expect(layoutSelector).toContainHTML('ANSI_104');
      expect(layoutSelector).toContainHTML('ISO_105');
      expect(layoutSelector).toContainHTML('JIS_109');
      expect(layoutSelector).toContainHTML('HHKB');
      expect(layoutSelector).toContainHTML('NUMPAD');
    });
  });

  describe('Key click and configuration flow', () => {
    it('opens KeyConfigDialog when key is clicked', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      // Find a key button (this depends on KeyboardVisualizer implementation)
      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        // Dialog should open
        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });
      }
    });

    it('displays current key mapping in tooltip on hover', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        // Hover over key
        await user.hover(keyButtons[0]);

        // Tooltip should appear (with delay)
        await waitFor(
          () => {
            expect(screen.getByRole('tooltip')).toBeInTheDocument();
          },
          { timeout: 1000 }
        );
      }
    });
  });

  describe('Simple remap configuration', () => {
    it('configures simple key remap', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        // Select "Simple Remap" action type
        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'simple');

        // Select target key
        const targetKeySelector = screen.getByRole('combobox', {
          name: /Target Key/i,
        });
        await user.selectOptions(targetKeySelector, 'KEY_B');

        // Save configuration
        const saveButton = screen.getByRole('button', { name: /Save/i });
        await user.click(saveButton);

        // Dialog should close
        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });
  });

  describe('Tap-Hold configuration', () => {
    it('configures tap-hold action', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        // Select "Tap/Hold" action type
        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'tap_hold');

        // Configure tap action
        const tapKeySelector = screen.getByRole('combobox', {
          name: /Tap Action/i,
        });
        await user.selectOptions(tapKeySelector, 'KEY_A');

        // Configure hold action
        const holdKeySelector = screen.getByRole('combobox', {
          name: /Hold Action/i,
        });
        await user.selectOptions(holdKeySelector, 'KEY_LEFTCTRL');

        // Set threshold
        const thresholdInput = screen.getByRole('spinbutton', {
          name: /Threshold/i,
        });
        await user.clear(thresholdInput);
        await user.type(thresholdInput, '200');

        // Save
        const saveButton = screen.getByRole('button', { name: /Save/i });
        await user.click(saveButton);

        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });

    it('validates threshold value', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'tap_hold');

        const thresholdInput = screen.getByRole('spinbutton', {
          name: /Threshold/i,
        });

        // Try invalid value (negative)
        await user.clear(thresholdInput);
        await user.type(thresholdInput, '-100');

        // Should show validation error
        await waitFor(() => {
          expect(
            screen.getByText(/Threshold must be positive/i)
          ).toBeInTheDocument();
        });
      }
    });
  });

  describe('Layer switch configuration', () => {
    it('configures layer switch action', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        // Select "Layer Switch" action type
        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'layer_switch');

        // Select target layer
        const targetLayerSelector = screen.getByRole('combobox', {
          name: /Target Layer/i,
        });
        await user.selectOptions(targetLayerSelector, 'layer_1');

        // Save
        const saveButton = screen.getByRole('button', { name: /Save/i });
        await user.click(saveButton);

        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });
  });

  describe('Macro configuration', () => {
    it('configures macro with multiple steps', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        // Select "Macro" action type
        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'macro');

        // Add macro steps
        const addStepButton = screen.getByRole('button', {
          name: /Add Step/i,
        });

        // Add first step
        await user.click(addStepButton);
        const step1Type = screen.getAllByRole('combobox', {
          name: /Step Type/i,
        })[0];
        await user.selectOptions(step1Type, 'press');

        // Add second step
        await user.click(addStepButton);
        const step2Type = screen.getAllByRole('combobox', {
          name: /Step Type/i,
        })[1];
        await user.selectOptions(step2Type, 'delay');

        // Save
        const saveButton = screen.getByRole('button', { name: /Save/i });
        await user.click(saveButton);

        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });
  });

  describe('Configuration preview', () => {
    it('shows preview of key mapping in dialog', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        // Should show preview panel
        expect(screen.getByText(/Preview/i)).toBeInTheDocument();
      }
    });
  });

  describe('Cancel and close flows', () => {
    it('cancels configuration on Cancel button', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        // Make some changes
        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'simple');

        // Click Cancel
        const cancelButton = screen.getByRole('button', { name: /Cancel/i });
        await user.click(cancelButton);

        // Dialog should close without saving
        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });

    it('closes dialog on Escape key', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        // Press Escape
        await user.keyboard('{Escape}');

        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });
  });

  describe('Loading and error states', () => {
    it('shows loading state while fetching config', async () => {
      const store = useConfigStore.getState();
      store.loading = true;

      renderWithProviders(<ConfigPage />);

      expect(screen.getByRole('status', { name: /Loading/i })).toBeInTheDocument();
    });

    it('displays error message when fetch fails', async () => {
      const store = useConfigStore.getState();
      store.error = 'Failed to load configuration';

      renderWithProviders(<ConfigPage />);

      expect(
        screen.getByText(/Failed to load configuration/i)
      ).toBeInTheDocument();
    });
  });

  describe('Device Integration (Requirement 8)', () => {
    it('DeviceScopeToggle receives real devices from API', async () => {
      renderWithProviders(<ConfigPage />);

      // Wait for the page to load
      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      // Should show device scope toggle
      expect(screen.getByText(/Mapping Scope/i)).toBeInTheDocument();

      // Should have Global and Device-Specific options
      expect(screen.getByText('Global')).toBeInTheDocument();
      expect(screen.getByText('Device-Specific')).toBeInTheDocument();
    });

    it('displays real device list when device-specific mode selected', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      // Click Device-Specific button
      const deviceSpecificButton = screen.getByRole('button', {
        name: /Device-Specific/i,
      });
      await user.click(deviceSpecificButton);

      // Should show device selector
      await waitFor(() => {
        expect(screen.getByText(/Select Device/i)).toBeInTheDocument();
      });

      // Should show device dropdown (or "no devices" message if empty)
      const deviceInfo = screen.queryByText(/No devices available/i);
      if (deviceInfo) {
        // No devices connected - that's valid
        expect(deviceInfo).toBeInTheDocument();
      } else {
        // Devices exist - should show dropdown
        expect(screen.getByText(/Select a device/i)).toBeInTheDocument();
      }
    });

    it('switches between global and device-specific scope', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      // Should start in global mode
      const globalButton = screen.getByRole('button', { name: /^Global$/i });
      expect(globalButton).toHaveClass('bg-primary-500');

      // Switch to device-specific
      const deviceSpecificButton = screen.getByRole('button', {
        name: /Device-Specific/i,
      });
      await user.click(deviceSpecificButton);

      // Device-specific button should now be active
      await waitFor(() => {
        expect(deviceSpecificButton).toHaveClass('bg-primary-500');
      });

      // Switch back to global
      await user.click(globalButton);

      await waitFor(() => {
        expect(globalButton).toHaveClass('bg-primary-500');
      });
    });

    it('device selector only visible in device-specific mode', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      // In global mode, device selector should not be visible
      expect(screen.queryByText(/Select Device/i)).not.toBeInTheDocument();

      // Switch to device-specific mode
      const deviceSpecificButton = screen.getByRole('button', {
        name: /Device-Specific/i,
      });
      await user.click(deviceSpecificButton);

      // Device selector should now be visible
      await waitFor(() => {
        expect(screen.getByText(/Select Device/i)).toBeInTheDocument();
      });
    });

    it('shows appropriate help text for each scope', async () => {
      const user = userEvent.setup();
      renderWithProviders(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      // Global mode help text
      expect(
        screen.getByText(/Global mappings apply to all connected devices/i)
      ).toBeInTheDocument();

      // Switch to device-specific mode
      const deviceSpecificButton = screen.getByRole('button', {
        name: /Device-Specific/i,
      });
      await user.click(deviceSpecificButton);

      // Device-specific help text
      await waitFor(() => {
        expect(
          screen.getByText(/Device-specific mappings apply only to/i)
        ).toBeInTheDocument();
      });
    });
  });
});
