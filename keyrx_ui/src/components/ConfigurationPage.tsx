/**
 * ConfigurationPage - Main page for editing Rhai configuration files.
 *
 * This page integrates the ConfigEditor and ValidationStatusPanel components
 * to provide a comprehensive configuration editing experience.
 */

import { useState, useCallback } from 'react';
import { ConfigEditor } from './ConfigEditor';
import { ValidationStatusPanel } from './ValidationStatusPanel';
import type { ValidationResult, ValidationError, ValidationWarning } from '@/types/validation';
import type { ConfigStorage } from '@/services/ConfigStorage';
import { LocalStorageImpl } from '@/services/LocalStorageImpl';
import { useApi } from '../contexts/ApiContext';
import './ConfigurationPage.css';

/**
 * Props for ConfigurationPage component
 */
export interface ConfigurationPageProps {
  /** Initial configuration content to load */
  initialConfig?: string;
  /** Storage implementation for saving configuration (defaults to LocalStorageImpl for backward compatibility) */
  storage?: ConfigStorage;
}

/**
 * ConfigurationPage component.
 *
 * Features:
 * - Monaco-based code editor with syntax highlighting
 * - Real-time validation with error/warning display
 * - Validation status panel with jump-to-error functionality
 * - Save configuration with validation checks
 */
export function ConfigurationPage({
  initialConfig = '',
  storage = new LocalStorageImpl(),
}: ConfigurationPageProps) {
  const { apiBaseUrl } = useApi();
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [isValidating] = useState(false);
  const [saveStatus, setSaveStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [saveErrorMessage, setSaveErrorMessage] = useState<string>('');

  /**
   * Handle save configuration.
   * Saves to backend API via PUT /api/config.
   * Falls back to ConfigStorage for local persistence if API is unavailable.
   */
  const handleSaveConfig = useCallback(async (content: string) => {
    try {
      console.log('Saving configuration:', content);

      // Save to backend via API
      const response = await fetch(`${apiBaseUrl}/api/config`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ content }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: { message: 'Failed to save configuration' } }));
        throw new Error(errorData.error?.message || `HTTP error! status: ${response.status}`);
      }

      const result = await response.json();

      // Also save to local storage as backup
      await storage.save('keyrx_config', content);

      setSaveStatus('success');
      setSaveErrorMessage('');

      // Show validation warning if present
      if (result.validation_error) {
        setSaveErrorMessage(`Saved with validation warning: ${result.validation_error}`);
      }

      setTimeout(() => setSaveStatus('idle'), 3000);
    } catch (error) {
      console.error('Failed to save configuration:', error);
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      setSaveStatus('error');
      setSaveErrorMessage(errorMessage);

      // Try to save to local storage as fallback
      try {
        await storage.save('keyrx_config', content);
        setSaveErrorMessage(`${errorMessage} (saved to local storage as backup)`);
      } catch (storageError) {
        console.error('Failed to save to local storage:', storageError);
      }

      setTimeout(() => {
        setSaveStatus('idle');
        setSaveErrorMessage('');
      }, 5000);
      // Don't rethrow - error is handled via state
    }
  }, [apiBaseUrl, storage]);

  /**
   * Handle validation result changes from the editor.
   */
  const handleValidationChange = useCallback((result: ValidationResult | null) => {
    setValidationResult(result);
  }, []);

  /**
   * Handle click on error in validation panel - jump to error in editor.
   */
  const handleErrorClick = useCallback((error: ValidationError) => {
    // The editor's F8 functionality handles jumping to errors
    // This could be enhanced to directly control the editor position
    console.log('Jump to error:', error);
  }, []);

  /**
   * Handle click on warning in validation panel.
   */
  const handleWarningClick = useCallback((warning: ValidationWarning) => {
    console.log('Jump to warning:', warning);
  }, []);

  return (
    <div className="configuration-page">
      {/* Page Header */}
      <div className="page-header">
        <div className="header-content">
          <h1>Configuration Editor</h1>
          <p className="header-subtitle">
            Edit your Rhai keyboard configuration with real-time validation
          </p>
        </div>
        {saveStatus === 'success' && (
          <div className="save-notification success">
            ✓ Configuration saved successfully
          </div>
        )}
        {saveStatus === 'error' && (
          <div className="save-notification error">
            ✗ Failed to save configuration
            {saveErrorMessage && <div className="error-details">{saveErrorMessage}</div>}
          </div>
        )}
      </div>

      {/* Main Content Area */}
      <div className="page-content">
        {/* Left Panel: Editor */}
        <div className="editor-panel">
          <ConfigEditor
            initialValue={initialConfig}
            onSave={handleSaveConfig}
            onValidationChange={handleValidationChange}
          />
        </div>

        {/* Right Panel: Validation Status */}
        <div className="validation-panel">
          <ValidationStatusPanel
            validationResult={validationResult}
            isValidating={isValidating}
            onErrorClick={handleErrorClick}
            onWarningClick={handleWarningClick}
          />
        </div>
      </div>

      {/* Help Text */}
      <div className="page-footer">
        <div className="help-text">
          <strong>Keyboard Shortcuts:</strong> F8 - Jump to next error | Ctrl+S - Save (if valid)
        </div>
      </div>
    </div>
  );
}

export default ConfigurationPage;
