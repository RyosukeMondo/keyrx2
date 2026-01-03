import React from 'react';
import { Check } from 'lucide-react';
import { Dropdown } from '../Dropdown';

/**
 * ProfileHeader component props
 */
export interface ProfileHeaderProps {
  /** Name of the currently edited profile */
  profileName: string;
  /** Whether this profile is currently active in the daemon */
  isActive?: boolean;
  /** Last modified timestamp */
  lastModified?: Date;
  /** Callback when user selects a different profile */
  onProfileChange?: (newProfileName: string) => void;
  /** List of available profiles for the dropdown selector */
  availableProfiles?: string[];
}

/**
 * ProfileHeader Component
 *
 * Displays profile context in the ConfigPage header, showing:
 * - Profile name with "Editing:" prefix
 * - Active badge (green checkmark + "Active") if profile is active
 * - Last modified timestamp
 * - Profile selector dropdown to switch between profiles
 *
 * This component provides visual confirmation of which profile is being edited
 * and allows quick switching between profiles without navigating away.
 *
 * Requirements: Requirement 5 - Profile-Centric Configuration Workflow
 *
 * @example
 * ```tsx
 * <ProfileHeader
 *   profileName="my-profile"
 *   isActive={true}
 *   lastModified={new Date()}
 *   onProfileChange={(name) => navigate(`/config?profile=${name}`)}
 *   availableProfiles={['default', 'my-profile', 'gaming']}
 * />
 * ```
 */
export const ProfileHeader: React.FC<ProfileHeaderProps> = ({
  profileName,
  isActive = false,
  lastModified,
  onProfileChange,
  availableProfiles = [],
}) => {
  // Format last modified date
  const formattedDate = lastModified
    ? new Date(lastModified).toLocaleString('en-CA', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        hour12: false,
      })
    : null;

  return (
    <div className="flex items-center justify-between flex-wrap gap-4 mb-6 pb-4 border-b border-slate-700">
      {/* Left side: Profile name, active badge, last modified */}
      <div className="flex items-center gap-4 flex-wrap">
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-bold text-slate-100">
            Editing: {profileName}
          </h1>
          {isActive && (
            <div
              className="flex items-center gap-1 bg-green-500 text-white px-3 py-1 rounded-md text-sm font-semibold"
              role="status"
              aria-label="Active profile"
            >
              <Check size={16} aria-hidden="true" />
              <span>Active</span>
            </div>
          )}
        </div>
        {formattedDate && (
          <span className="text-sm text-slate-400">
            Last modified: {formattedDate}
          </span>
        )}
      </div>

      {/* Right side: Profile selector dropdown */}
      {onProfileChange && availableProfiles.length > 0 && (
        <div className="w-64">
          <Dropdown
            options={availableProfiles.map((p) => ({ value: p, label: p }))}
            value={profileName}
            onChange={onProfileChange}
            aria-label="Select profile to edit"
            placeholder="Select a profile"
          />
        </div>
      )}
    </div>
  );
};
