/**
 * Visual Regression Testing - Key Pages
 *
 * Tests visual appearance of critical pages to catch unintended UI changes.
 * Uses Playwright's screenshot comparison with a tolerance threshold.
 *
 * Important: Screenshots are platform-specific. Generate baseline screenshots
 * on the same platform where CI runs (Linux). Use --update-snapshots to update.
 */

import { test, expect } from '@playwright/test';
import { setupFreshTestEnvironment, waitForPageReady } from '../helpers';

test.describe('Visual Regression - Key Pages', () => {
  test.beforeEach(async ({ page }) => {
    // Fresh environment for consistent visual testing
    await setupFreshTestEnvironment(page);
  });

  /**
   * Home/Dashboard Page
   * Most important page - shows overall system status
   */
  test('Dashboard page visual snapshot', async ({ page }) => {
    await page.goto('/');
    await waitForPageReady(page);

    // Wait for any loading spinners to disappear
    await page.waitForSelector('[data-loading="true"]', {
      state: 'hidden',
      timeout: 5000,
    }).catch(() => {
      // No loading spinner found, that's OK
    });

    // Hide dynamic content that changes between runs
    await page.evaluate(() => {
      // Hide timestamps if present
      document.querySelectorAll('[data-timestamp]').forEach((el) => {
        (el as HTMLElement).style.visibility = 'hidden';
      });

      // Hide any elements with current time
      document.querySelectorAll('.timestamp, .time').forEach((el) => {
        (el as HTMLElement).style.visibility = 'hidden';
      });
    });

    // Take screenshot and compare
    await expect(page).toHaveScreenshot('dashboard.png', {
      // Allow 0.2% pixel difference (font rendering, anti-aliasing)
      maxDiffPixelRatio: 0.002,
      // Full page screenshot
      fullPage: true,
      // Animations to end state
      animations: 'disabled',
    });
  });

  /**
   * Devices Page
   * Shows connected keyboards and device management
   */
  test('Devices page visual snapshot', async ({ page }) => {
    await page.goto('/devices');
    await waitForPageReady(page);

    // Wait for device list to load
    await page.waitForSelector('[data-testid="device-list"], .device-card', {
      timeout: 5000,
    }).catch(() => {
      // No devices, that's OK - will show empty state
    });

    // Hide dynamic content
    await page.evaluate(() => {
      document.querySelectorAll('[data-timestamp], .timestamp').forEach((el) => {
        (el as HTMLElement).style.visibility = 'hidden';
      });
    });

    await expect(page).toHaveScreenshot('devices.png', {
      maxDiffPixelRatio: 0.002,
      fullPage: true,
      animations: 'disabled',
    });
  });

  /**
   * Profiles Page
   * Shows keyboard profiles and profile management
   */
  test('Profiles page visual snapshot', async ({ page }) => {
    await page.goto('/profiles');
    await waitForPageReady(page);

    // Wait for profiles to load
    await page.waitForSelector('[data-testid="profile-list"], .profile-card', {
      timeout: 5000,
    }).catch(() => {
      // No profiles, that's OK
    });

    // Hide dynamic content
    await page.evaluate(() => {
      document.querySelectorAll('[data-timestamp], .timestamp').forEach((el) => {
        (el as HTMLElement).style.visibility = 'hidden';
      });
    });

    await expect(page).toHaveScreenshot('profiles.png', {
      maxDiffPixelRatio: 0.002,
      fullPage: true,
      animations: 'disabled',
    });
  });

  /**
   * Configuration Page
   * Shows Monaco editor with Rhai configuration
   */
  test('Configuration page visual snapshot', async ({ page }) => {
    await page.goto('/config');
    await waitForPageReady(page);

    // Wait for Monaco editor to load
    await page.waitForSelector('.monaco-editor, [class*="editor"]', {
      timeout: 10000,
    }).catch(() => {
      // Editor not found, that's OK
    });

    // Wait a bit more for Monaco to fully render
    await page.waitForTimeout(1000);

    // Hide cursor and dynamic elements in editor
    await page.evaluate(() => {
      // Hide cursor
      document.querySelectorAll('.cursor').forEach((el) => {
        (el as HTMLElement).style.display = 'none';
      });

      // Hide timestamps
      document.querySelectorAll('[data-timestamp], .timestamp').forEach((el) => {
        (el as HTMLElement).style.visibility = 'hidden';
      });
    });

    await expect(page).toHaveScreenshot('config.png', {
      maxDiffPixelRatio: 0.002,
      fullPage: true,
      animations: 'disabled',
    });
  });

  /**
   * Simulator Page
   * Shows keyboard simulator for testing key remapping
   */
  test('Simulator page visual snapshot', async ({ page }) => {
    await page.goto('/simulator');
    await waitForPageReady(page);

    // Wait for simulator to load
    await page.waitForSelector('[data-testid="simulator"], .simulator', {
      timeout: 5000,
    }).catch(() => {
      // Simulator not found, that's OK
    });

    // Hide dynamic content
    await page.evaluate(() => {
      document.querySelectorAll('[data-timestamp], .timestamp').forEach((el) => {
        (el as HTMLElement).style.visibility = 'hidden';
      });
    });

    await expect(page).toHaveScreenshot('simulator.png', {
      maxDiffPixelRatio: 0.002,
      fullPage: true,
      animations: 'disabled',
    });
  });
});

/**
 * Visual Regression - Component States
 * Test visual appearance of key UI components in different states
 */
test.describe('Visual Regression - Component States', () => {
  test.beforeEach(async ({ page }) => {
    await setupFreshTestEnvironment(page);
  });

  /**
   * Test modal dialogs
   */
  test('Modal dialog visual snapshot', async ({ page }) => {
    await page.goto('/profiles');
    await waitForPageReady(page);

    // Try to open a modal (e.g., Add Profile)
    const addButton = page.getByRole('button', { name: /add|new.*profile/i });

    if (await addButton.isVisible()) {
      await addButton.click();

      // Wait for modal to appear
      await page.waitForSelector('[role="dialog"]', { timeout: 2000 });

      // Hide dynamic content
      await page.evaluate(() => {
        document.querySelectorAll('[data-timestamp], .timestamp').forEach((el) => {
          (el as HTMLElement).style.visibility = 'hidden';
        });
      });

      // Screenshot just the modal
      const modal = page.locator('[role="dialog"]');
      await expect(modal).toHaveScreenshot('modal-add-profile.png', {
        maxDiffPixelRatio: 0.002,
        animations: 'disabled',
      });
    } else {
      test.skip();
    }
  });

  /**
   * Test error states
   */
  test('Error message visual snapshot', async ({ page }) => {
    await page.goto('/devices');
    await waitForPageReady(page);

    // Look for any error messages
    const errorElement = page.locator('[role="alert"], .error-message, .alert-error');

    const count = await errorElement.count();

    if (count > 0) {
      // Found an error, take screenshot
      await expect(errorElement.first()).toHaveScreenshot('error-state.png', {
        maxDiffPixelRatio: 0.002,
        animations: 'disabled',
      });
    } else {
      // No errors found - skip test
      test.skip();
    }
  });
});

/**
 * Visual Regression - Responsive Design
 * Test visual appearance at different viewport sizes
 */
test.describe('Visual Regression - Responsive Design', () => {
  test.beforeEach(async ({ page }) => {
    await setupFreshTestEnvironment(page);
  });

  /**
   * Mobile viewport (375x667 - iPhone SE)
   */
  test('Dashboard on mobile viewport', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    await page.goto('/');
    await waitForPageReady(page);

    // Hide dynamic content
    await page.evaluate(() => {
      document.querySelectorAll('[data-timestamp], .timestamp').forEach((el) => {
        (el as HTMLElement).style.visibility = 'hidden';
      });
    });

    await expect(page).toHaveScreenshot('dashboard-mobile.png', {
      maxDiffPixelRatio: 0.002,
      fullPage: true,
      animations: 'disabled',
    });
  });

  /**
   * Tablet viewport (768x1024 - iPad)
   */
  test('Dashboard on tablet viewport', async ({ page }) => {
    // Set tablet viewport
    await page.setViewportSize({ width: 768, height: 1024 });

    await page.goto('/');
    await waitForPageReady(page);

    // Hide dynamic content
    await page.evaluate(() => {
      document.querySelectorAll('[data-timestamp], .timestamp').forEach((el) => {
        (el as HTMLElement).style.visibility = 'hidden';
      });
    });

    await expect(page).toHaveScreenshot('dashboard-tablet.png', {
      maxDiffPixelRatio: 0.002,
      fullPage: true,
      animations: 'disabled',
    });
  });

  /**
   * Desktop viewport (1920x1080 - Full HD)
   */
  test('Dashboard on desktop viewport', async ({ page }) => {
    // Set desktop viewport
    await page.setViewportSize({ width: 1920, height: 1080 });

    await page.goto('/');
    await waitForPageReady(page);

    // Hide dynamic content
    await page.evaluate(() => {
      document.querySelectorAll('[data-timestamp], .timestamp').forEach((el) => {
        (el as HTMLElement).style.visibility = 'hidden';
      });
    });

    await expect(page).toHaveScreenshot('dashboard-desktop.png', {
      maxDiffPixelRatio: 0.002,
      fullPage: true,
      animations: 'disabled',
    });
  });
});
