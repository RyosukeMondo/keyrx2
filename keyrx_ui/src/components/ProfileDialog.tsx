/**
 * ProfileDialog - Modal dialog for creating or renaming keyboard profiles
 *
 * Provides a form with validation for profile names and template selection
 * for new profiles. Supports both create and rename modes.
 *
 * Validation rules:
 * - Name is required
 * - Maximum 32 characters
 * - Only letters, numbers, dashes, and underscores allowed
 *
 * @example
 * ```tsx
 * // Create mode
 * <ProfileDialog
 *   mode="create"
 *   onClose={handleClose}
 *   onSubmit={(name, template) => createProfile(name, template)}
 * />
 *
 * // Rename mode
 * <ProfileDialog
 *   mode="rename"
 *   initialName="old-name"
 *   onClose={handleClose}
 *   onSubmit={(name) => renameProfile('old-name', name)}
 * />
 * ```
 */

import { useState } from 'react';
import './ProfileDialog.css';

/**
 * Props for ProfileDialog component
 */
interface ProfileDialogProps {
  /** Dialog mode: 'create' shows template selector, 'rename' shows only name field */
  mode: 'create' | 'rename';
  /** Initial value for the name field (used in rename mode) */
  initialName?: string;
  /** Callback when dialog is closed without submitting */
  onClose: () => void;
  /** Callback when form is submitted successfully. In create mode, includes template selection */
  onSubmit: (name: string, template?: string) => void;
}

/**
 * ProfileDialog component for creating or renaming profiles
 *
 * Features:
 * - Real-time validation with error messages
 * - Template selection in create mode (blank or QMK-style layers)
 * - Escape key and overlay click to close
 * - Submit button disabled when validation fails
 * - Auto-focus on name input
 *
 * @param props - Component props
 * @returns Rendered dialog component
 */
export function ProfileDialog({ mode, initialName = '', onClose, onSubmit }: ProfileDialogProps) {
  const [name, setName] = useState(initialName);
  const [template, setTemplate] = useState('blank');
  const [error, setError] = useState<string | null>(null);

  /**
   * Validates profile name against constraints
   * Sets error state if validation fails
   * @param value - Profile name to validate
   * @returns true if valid, false otherwise
   */
  const validateName = (value: string): boolean => {
    if (!value.trim()) {
      setError('Profile name is required');
      return false;
    }
    if (value.length > 32) {
      setError('Profile name must be 32 characters or less');
      return false;
    }
    if (!/^[a-zA-Z0-9_-]+$/.test(value)) {
      setError('Profile name can only contain letters, numbers, dashes, and underscores');
      return false;
    }
    setError(null);
    return true;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!validateName(name)) {
      return;
    }
    if (mode === 'create') {
      onSubmit(name, template);
    } else {
      onSubmit(name);
    }
  };

  const handleNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setName(value);
    if (value) {
      validateName(value);
    } else {
      setError(null);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-content" onClick={(e) => e.stopPropagation()}>
        <h2>{mode === 'create' ? 'Create New Profile' : 'Rename Profile'}</h2>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="profile-name">Profile Name</label>
            <input
              id="profile-name"
              type="text"
              value={name}
              onChange={handleNameChange}
              placeholder="e.g., work, gaming, coding"
              autoFocus
              maxLength={32}
            />
            {error && <div className="error-message">{error}</div>}
          </div>

          {mode === 'create' && (
            <div className="form-group">
              <label htmlFor="profile-template">Template</label>
              <select
                id="profile-template"
                value={template}
                onChange={(e) => setTemplate(e.target.value)}
              >
                <option value="blank">Blank (empty configuration)</option>
                <option value="qmk-layers">QMK-style Layers</option>
              </select>
              <div className="template-description">
                {template === 'blank' && (
                  <p>Start with an empty configuration and build your own key mappings.</p>
                )}
                {template === 'qmk-layers' && (
                  <p>
                    Pre-configured with QMK-style layer system including base layer and lower
                    layer with example mappings.
                  </p>
                )}
              </div>
            </div>
          )}

          <div className="dialog-actions">
            <button type="button" onClick={onClose} className="cancel-button">
              Cancel
            </button>
            <button type="submit" className="submit-button" disabled={!!error || !name.trim()}>
              {mode === 'create' ? 'Create' : 'Rename'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
