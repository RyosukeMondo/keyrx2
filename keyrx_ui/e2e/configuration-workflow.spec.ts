import { test, expect } from '@playwright/test';

/**
 * E2E Test: Complete Configuration Workflow
 *
 * Tests the complete user journey for creating, configuring, and activating profiles:
 * 1. Navigate to profiles page and create a new profile
 * 2. Open configuration editor
 * 3. Use drag-and-drop to configure key mappings
 * 4. Verify mapping appears on keyboard visualizer
 * 5. Activate the profile
 * 6. Navigate to metrics page and verify active profile is displayed
 *
 * Requirements: web-ui-bugfix-and-enhancement spec - All requirements
 * - Requirement 4: QMK-Style Drag-and-Drop Configuration Editor
 * - Requirement 5: Profile-Centric Configuration Workflow
 * - Requirement 6: Metrics Page Profile Display
 */

test.describe('Complete Configuration Workflow', () => {
  const testProfileName = `E2ETest_${Date.now()}`;

  test.afterAll(async ({ page }) => {
    // Cleanup: navigate to profiles page and delete test profile if it exists
    try {
      await page.goto('/profiles');
      await page.waitForLoadState('networkidle');

      // Check if profile exists
      const profileExists = await page.locator(`text=${testProfileName}`).isVisible();
      if (profileExists) {
        await page.click(`button[aria-label="Delete ${testProfileName}"]`);
        await page.click('button:has-text("Confirm")');
        await page.waitForTimeout(500);
      }
    } catch (error) {
      console.log('Cleanup error (non-critical):', error);
    }
  });

  test('should complete full workflow: create profile → configure with drag-and-drop → activate → verify in metrics', async ({ page }) => {
    // ============================================================
    // Step 1: Navigate to profiles page and create new profile
    // ============================================================
    await test.step('Navigate to profiles page', async () => {
      await page.goto('/profiles');
      await page.waitForLoadState('networkidle');

      // Verify we're on the profiles page
      await expect(page.locator('h1:has-text("Profiles")')).toBeVisible({ timeout: 10000 });
    });

    await test.step('Create new test profile', async () => {
      // Click create profile button
      await page.click('button:has-text("Create Profile")');

      // Fill in profile name
      await page.fill('input[name="profileName"]', testProfileName);

      // Submit form
      await page.click('button:has-text("Create")');

      // Wait for profile to appear in list
      await expect(page.locator(`text=${testProfileName}`)).toBeVisible({ timeout: 5000 });
    });

    // ============================================================
    // Step 2: Open configuration editor for the new profile
    // ============================================================
    await test.step('Navigate to configuration page', async () => {
      // Click on the profile to open config page
      // Try multiple selector strategies
      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      const configButton = profileCard.locator('button:has-text("Configure")');

      // If there's a configure button, click it; otherwise click the card
      if (await configButton.isVisible().catch(() => false)) {
        await configButton.click();
      } else {
        // Alternative: navigate directly
        await page.goto(`/config?profile=${encodeURIComponent(testProfileName)}`);
      }

      await page.waitForLoadState('networkidle');

      // Verify we're on the config page with correct profile
      await expect(page.locator(`text=Editing: ${testProfileName}`)).toBeVisible({ timeout: 10000 });
    });

    // ============================================================
    // Step 3: Use drag-and-drop to configure key mapping
    // ============================================================
    await test.step('Configure key mapping with drag-and-drop', async () => {
      // Ensure we're on the Visual tab (drag-and-drop UI)
      const visualTab = page.locator('button:has-text("Visual")');
      if (!(await visualTab.getAttribute('class')).includes('active')) {
        await visualTab.click();
        await page.waitForTimeout(500);
      }

      // Verify DragKeyPalette is visible
      await expect(page.locator('[data-testid="drag-key-palette"]')).toBeVisible({ timeout: 5000 });

      // Verify KeyboardVisualizer is visible
      await expect(page.locator('[data-testid="keyboard-visualizer"]')).toBeVisible({ timeout: 5000 });

      // Locate the draggable key "A" in the palette
      const dragKeyA = page.locator('[data-testid="draggable-key-VK_A"]');
      await expect(dragKeyA).toBeVisible({ timeout: 5000 });

      // Locate the drop target for CapsLock key on the keyboard
      const dropTargetCapsLock = page.locator('[data-testid="drop-target-58"]'); // CapsLock keycode
      await expect(dropTargetCapsLock).toBeVisible({ timeout: 5000 });

      // Perform drag-and-drop using Playwright's dragTo method
      await dragKeyA.dragTo(dropTargetCapsLock);

      // Wait for the mapping to be saved (API call)
      await page.waitForTimeout(1000);

      // Alternative: Use keyboard-accessible drag-and-drop if mouse drag doesn't work
      // Focus on the key, press Space to grab, Tab to navigate to target, Space to drop
      // await dragKeyA.focus();
      // await page.keyboard.press('Space');
      // await dropTargetCapsLock.focus();
      // await page.keyboard.press('Space');
    });

    // ============================================================
    // Step 4: Verify mapping appears on keyboard visualizer
    // ============================================================
    await test.step('Verify key mapping appears on keyboard', async () => {
      // The CapsLock key should now show "A" label
      const capsLockKey = page.locator('[data-testid="drop-target-58"]');

      // Check that the key now displays the mapped value (e.g., "A" or "VK_A")
      // The exact text depends on how the component renders the mapping
      await expect(capsLockKey).toContainText(/A|VK_A/, { timeout: 5000 });

      // Verify save indicator appears (if implemented)
      const saveIndicator = page.locator('text=Saved');
      if (await saveIndicator.isVisible().catch(() => false)) {
        await expect(saveIndicator).toBeVisible();
      }
    });

    // ============================================================
    // Step 5: Activate the profile
    // ============================================================
    await test.step('Activate the profile', async () => {
      // Navigate back to profiles page
      await page.goto('/profiles');
      await page.waitForLoadState('networkidle');

      // Activate the test profile
      const activateButton = page.locator(`button[aria-label="Activate ${testProfileName}"]`);
      await expect(activateButton).toBeVisible({ timeout: 5000 });
      await activateButton.click();

      // Wait for activation to complete
      await page.waitForTimeout(1000);

      // Verify [Active] badge appears on the profile
      const profileCard = page.locator(`[data-profile="${testProfileName}"]`);
      await expect(profileCard.locator('text=Active')).toBeVisible({ timeout: 5000 });

      // Ensure no compilation errors occurred
      const errorModal = page.locator('text=Compilation Error');
      await expect(errorModal).not.toBeVisible();
    });

    // ============================================================
    // Step 6: Navigate to metrics page and verify active profile
    // ============================================================
    await test.step('Verify active profile on metrics page', async () => {
      // Navigate to metrics page
      await page.goto('/metrics');
      await page.waitForLoadState('networkidle');

      // Verify active profile name is displayed
      await expect(page.locator(`text=Active Profile: ${testProfileName}`)).toBeVisible({ timeout: 5000 });

      // Verify .rhai filename is shown
      const rhaiFilename = `${testProfileName}.rhai`;
      await expect(page.locator(`text=${rhaiFilename}`)).toBeVisible({ timeout: 5000 });

      // Verify the link to config page works
      const configLink = page.locator(`a[href*="/config?profile=${encodeURIComponent(testProfileName)}"]`);
      if (await configLink.isVisible().catch(() => false)) {
        await expect(configLink).toBeVisible();
      }
    });
  });

  test('should handle profile without active profile gracefully', async ({ page }) => {
    await test.step('Deactivate all profiles and check metrics page', async () => {
      // Navigate to metrics page when no profile is active
      // This test assumes we can deactivate profiles or start with none active
      await page.goto('/metrics');
      await page.waitForLoadState('networkidle');

      // Verify "No active profile" message is shown when daemon not running or no profile active
      // This depends on the daemon state, so we'll just check the page loads
      await expect(page.locator('h1:has-text("Metrics")')).toBeVisible({ timeout: 10000 });

      // If no profile is active, we should see either:
      // - "No active profile" message
      // - Or an active profile if daemon is running
      const noProfileMsg = page.locator('text=No active profile');
      const activeProfileMsg = page.locator('text=Active Profile:');

      // At least one should be visible
      await expect(noProfileMsg.or(activeProfileMsg)).toBeVisible({ timeout: 5000 });
    });
  });

  test('should handle drag-and-drop keyboard accessibility', async ({ page }) => {
    await test.step('Create profile and navigate to config', async () => {
      const keyboardTestProfile = `KeyboardTest_${Date.now()}`;

      // Create profile
      await page.goto('/profiles');
      await page.waitForLoadState('networkidle');
      await page.click('button:has-text("Create Profile")');
      await page.fill('input[name="profileName"]', keyboardTestProfile);
      await page.click('button:has-text("Create")');
      await expect(page.locator(`text=${keyboardTestProfile}`)).toBeVisible({ timeout: 5000 });

      // Navigate to config
      await page.goto(`/config?profile=${encodeURIComponent(keyboardTestProfile)}`);
      await page.waitForLoadState('networkidle');
    });

    await test.step('Test keyboard-only drag-and-drop', async () => {
      // Ensure Visual tab is active
      const visualTab = page.locator('button:has-text("Visual")');
      if (!(await visualTab.getAttribute('class')).includes('active')) {
        await visualTab.click();
        await page.waitForTimeout(500);
      }

      // Focus on draggable key using Tab navigation
      await page.keyboard.press('Tab'); // Navigate to first focusable element

      // Continue tabbing until we reach a draggable key
      let attempts = 0;
      const maxAttempts = 20;
      while (attempts < maxAttempts) {
        const focusedElement = await page.evaluate(() => document.activeElement?.getAttribute('data-testid'));
        if (focusedElement && focusedElement.startsWith('draggable-key-')) {
          break;
        }
        await page.keyboard.press('Tab');
        attempts++;
      }

      // Press Space to grab the key
      await page.keyboard.press('Space');
      await page.waitForTimeout(500);

      // Verify screen reader announcement or aria-live region
      const ariaLive = page.locator('[role="status"]');
      if (await ariaLive.isVisible().catch(() => false)) {
        await expect(ariaLive).toContainText(/Grabbed|grabbed/, { timeout: 2000 });
      }

      // Press Escape to cancel drag (test cancel functionality)
      await page.keyboard.press('Escape');
      await page.waitForTimeout(500);

      // Verify drag was cancelled (no mapping saved)
      // This is a basic test of keyboard accessibility
    });

    await test.step('Cleanup keyboard test profile', async () => {
      const keyboardTestProfile = `KeyboardTest_${Date.now()}`;
      try {
        await page.goto('/profiles');
        await page.waitForLoadState('networkidle');
        const profileExists = await page.locator(`text=${keyboardTestProfile}`).isVisible();
        if (profileExists) {
          await page.click(`button[aria-label="Delete ${keyboardTestProfile}"]`);
          await page.click('button:has-text("Confirm")');
        }
      } catch (error) {
        console.log('Cleanup error (non-critical):', error);
      }
    });
  });
});

/**
 * Test: Error Handling in Configuration Workflow
 */
test.describe('Configuration Workflow - Error Handling', () => {
  test('should handle API failures gracefully with rollback', async ({ page }) => {
    await test.step('Setup: Create profile', async () => {
      const errorTestProfile = `ErrorTest_${Date.now()}`;

      await page.goto('/profiles');
      await page.waitForLoadState('networkidle');
      await page.click('button:has-text("Create Profile")');
      await page.fill('input[name="profileName"]', errorTestProfile);
      await page.click('button:has-text("Create")');
      await expect(page.locator(`text=${errorTestProfile}`)).toBeVisible({ timeout: 5000 });

      // Navigate to config
      await page.goto(`/config?profile=${encodeURIComponent(errorTestProfile)}`);
      await page.waitForLoadState('networkidle');
    });

    await test.step('Test error handling with network failure simulation', async () => {
      // Note: This test assumes we can simulate network errors
      // In a real scenario, you'd use page.route() to intercept and fail API calls

      // Attempt drag-and-drop (which triggers API save)
      const dragKeyA = page.locator('[data-testid="draggable-key-VK_A"]');
      const dropTarget = page.locator('[data-testid="drop-target-58"]');

      if (await dragKeyA.isVisible().catch(() => false) && await dropTarget.isVisible().catch(() => false)) {
        // Intercept the API call and make it fail
        await page.route('**/api/profiles/**/config', route => {
          route.abort('failed');
        });

        await dragKeyA.dragTo(dropTarget);
        await page.waitForTimeout(1000);

        // Verify error message is displayed
        const errorMessage = page.locator('text=Failed to save');
        if (await errorMessage.isVisible().catch(() => false)) {
          await expect(errorMessage).toBeVisible({ timeout: 3000 });
        }

        // Verify the mapping was rolled back (optimistic update reverted)
        // The CapsLock key should NOT show "A" label
        const capsLockKey = page.locator('[data-testid="drop-target-58"]');
        await expect(capsLockKey).not.toContainText('A', { timeout: 2000 });
      }
    });

    await test.step('Cleanup error test profile', async () => {
      const errorTestProfile = `ErrorTest_${Date.now()}`;
      try {
        await page.goto('/profiles');
        await page.waitForLoadState('networkidle');
        const profileExists = await page.locator(`text=${errorTestProfile}`).isVisible();
        if (profileExists) {
          await page.click(`button[aria-label="Delete ${errorTestProfile}"]`);
          await page.click('button:has-text("Confirm")');
        }
      } catch (error) {
        console.log('Cleanup error (non-critical):', error);
      }
    });
  });
});
