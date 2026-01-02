/**
 * Color Contrast Accessibility Tests
 *
 * Verifies WCAG 1.4.3 color contrast compliance across all pages.
 * - Normal text: ≥4.5:1 contrast ratio
 * - Large text (≥18pt or ≥14pt bold): ≥3:1 contrast ratio
 *
 * Requirements: Requirement 4.3, Task 18
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { renderWithProviders } from './testUtils';
import { runColorContrastAudit } from './AccessibilityTestHelper';

// Page imports
import { DashboardPage } from '../src/pages/DashboardPage';
import { DevicesPage } from '../src/pages/DevicesPage';
import { ProfilesPage } from '../src/pages/ProfilesPage';
import { ConfigPage } from '../src/pages/ConfigPage';
import { MetricsPage } from '../src/pages/MetricsPage';
import { SimulatorPage } from '../src/pages/SimulatorPage';

describe('Color Contrast Compliance - WCAG 1.4.3', () => {
  describe('DashboardPage', () => {
    it('should meet color contrast requirements', async () => {
      const { container } = renderWithProviders(<DashboardPage />, {
        wrapWithRouter: true,
      });

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('DevicesPage', () => {
    it('should meet color contrast requirements', async () => {
      const { container } = renderWithProviders(<DevicesPage />, {
        wrapWithRouter: true,
      });

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });

    it('should have sufficient contrast in device list items', async () => {
      const { container } = renderWithProviders(<DevicesPage />, {
        wrapWithRouter: true,
      });

      // Wait for potential device list rendering
      await new Promise((resolve) => setTimeout(resolve, 100));

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('ProfilesPage', () => {
    it('should meet color contrast requirements', async () => {
      const { container } = renderWithProviders(<ProfilesPage />, {
        wrapWithRouter: true,
      });

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });

    it('should have sufficient contrast in profile cards', async () => {
      const { container } = renderWithProviders(<ProfilesPage />, {
        wrapWithRouter: true,
      });

      // Wait for potential profile list rendering
      await new Promise((resolve) => setTimeout(resolve, 100));

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('ConfigPage', () => {
    it('should meet color contrast requirements', async () => {
      const { container } = renderWithProviders(<ConfigPage />, {
        wrapWithRouter: true,
        wrapWithWasm: true,
      });

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });

    it('should have sufficient contrast in editor UI', async () => {
      const { container } = renderWithProviders(<ConfigPage />, {
        wrapWithRouter: true,
        wrapWithWasm: true,
      });

      // Wait for editor to potentially render
      await new Promise((resolve) => setTimeout(resolve, 100));

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('MetricsPage', () => {
    it('should meet color contrast requirements', async () => {
      const { container } = renderWithProviders(<MetricsPage />, {
        wrapWithRouter: true,
      });

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });

    it('should have sufficient contrast in charts and graphs', async () => {
      const { container } = renderWithProviders(<MetricsPage />, {
        wrapWithRouter: true,
      });

      // Wait for potential chart rendering
      await new Promise((resolve) => setTimeout(resolve, 100));

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('SimulatorPage', () => {
    it('should meet color contrast requirements', async () => {
      const { container } = renderWithProviders(<SimulatorPage />, {
        wrapWithRouter: true,
        wrapWithWasm: true,
      });

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });

    it('should have sufficient contrast in simulator controls', async () => {
      const { container } = renderWithProviders(<SimulatorPage />, {
        wrapWithRouter: true,
        wrapWithWasm: true,
      });

      // Wait for simulator UI to potentially render
      await new Promise((resolve) => setTimeout(resolve, 100));

      const results = await runColorContrastAudit(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('Common UI Elements', () => {
    it('should have sufficient contrast in navigation elements', async () => {
      const { container } = renderWithProviders(<DashboardPage />, {
        wrapWithRouter: true,
      });

      // Focus on navigation/header elements
      const nav = container.querySelector('nav, header, [role="navigation"]');
      if (nav) {
        const results = await runColorContrastAudit(nav);
        expect(results).toHaveNoViolations();
      }
    });

    it('should have sufficient contrast in buttons', async () => {
      const { container } = renderWithProviders(<ProfilesPage />, {
        wrapWithRouter: true,
      });

      // Test button contrast specifically
      const buttons = container.querySelectorAll('button');
      if (buttons.length > 0) {
        const results = await runColorContrastAudit(container);
        expect(results).toHaveNoViolations();
      }
    });

    it('should have sufficient contrast in form inputs', async () => {
      const { container } = renderWithProviders(<ConfigPage />, {
        wrapWithRouter: true,
        wrapWithWasm: true,
      });

      // Test input field contrast
      const inputs = container.querySelectorAll('input, textarea, select');
      if (inputs.length > 0) {
        const results = await runColorContrastAudit(container);
        expect(results).toHaveNoViolations();
      }
    });

    it('should have sufficient contrast in links', async () => {
      const { container } = renderWithProviders(<DashboardPage />, {
        wrapWithRouter: true,
      });

      // Test link contrast
      const links = container.querySelectorAll('a');
      if (links.length > 0) {
        const results = await runColorContrastAudit(container);
        expect(results).toHaveNoViolations();
      }
    });
  });

  describe('State-specific Contrast', () => {
    it('should have sufficient contrast in disabled states', async () => {
      const { container } = renderWithProviders(<ProfilesPage />, {
        wrapWithRouter: true,
      });

      // Check disabled buttons if they exist
      const disabledElements = container.querySelectorAll('[disabled], [aria-disabled="true"]');
      if (disabledElements.length > 0) {
        const results = await runColorContrastAudit(container);
        expect(results).toHaveNoViolations();
      }
    });

    it('should have sufficient contrast in focus states', async () => {
      const { container, getByRole } = renderWithProviders(<ProfilesPage />, {
        wrapWithRouter: true,
      });

      // Try to find and focus a button
      const buttons = container.querySelectorAll('button');
      if (buttons.length > 0) {
        (buttons[0] as HTMLElement).focus();

        const results = await runColorContrastAudit(container);
        expect(results).toHaveNoViolations();
      }
    });

    it('should have sufficient contrast in error states', async () => {
      const { container } = renderWithProviders(<ConfigPage />, {
        wrapWithRouter: true,
        wrapWithWasm: true,
      });

      // Check error messages if they exist
      const errorElements = container.querySelectorAll('[role="alert"], .error, .text-red-500, .text-red-600');
      if (errorElements.length > 0) {
        const results = await runColorContrastAudit(container);
        expect(results).toHaveNoViolations();
      }
    });

    it('should have sufficient contrast in success states', async () => {
      const { container } = renderWithProviders(<ProfilesPage />, {
        wrapWithRouter: true,
      });

      // Check success messages if they exist
      const successElements = container.querySelectorAll('[role="status"], .success, .text-green-500, .text-green-600');
      if (successElements.length > 0) {
        const results = await runColorContrastAudit(container);
        expect(results).toHaveNoViolations();
      }
    });
  });
});
