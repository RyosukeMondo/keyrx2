/**
 * ProfileCard - Display a keyboard configuration profile with action buttons
 *
 * Renders a card UI showing profile metadata (name, layer count, modified time)
 * and status indicator. Provides action buttons for managing the profile.
 *
 * @example
 * ```tsx
 * <ProfileCard
 *   profile={myProfile}
 *   onActivate={() => activateProfile(myProfile.name)}
 *   onDelete={() => deleteProfile(myProfile.name)}
 *   onDuplicate={() => duplicateProfile(myProfile.name)}
 *   onExport={() => exportProfile(myProfile.name)}
 *   onRename={() => renameProfile(myProfile.name)}
 * />
 * ```
 */

import { Profile } from './ProfilesPage';
import { formatTimestampRelative } from '../utils/timeFormatting';
import './ProfileCard.css';

/**
 * Props for ProfileCard component
 */
interface ProfileCardProps {
  /** Profile object containing name, layer count, modification time, and active status */
  profile: Profile;
  /** Callback when user clicks the Activate button (only shown if profile is inactive) */
  onActivate: () => void;
  /** Callback when user clicks the Delete button (disabled if profile is active) */
  onDelete: () => void;
  /** Callback when user clicks the Duplicate button */
  onDuplicate: () => void;
  /** Callback when user clicks the Export button */
  onExport: () => void;
  /** Callback when user clicks the Rename button */
  onRename: () => void;
}

/**
 * ProfileCard component displays a single profile with metadata and action buttons
 *
 * Features:
 * - Visual status indicator (active/inactive)
 * - Profile metadata (name, layer count, last modified time)
 * - Primary action: Activate button (hidden for active profile)
 * - Secondary actions: Rename, Duplicate, Export, Delete
 * - Delete button is disabled for the active profile
 *
 * @param props - Component props
 * @returns Rendered profile card component
 */
export function ProfileCard({
  profile,
  onActivate,
  onDelete,
  onDuplicate,
  onExport,
  onRename,
}: ProfileCardProps) {

  return (
    <div className={`profile-card ${profile.is_active ? 'active' : ''}`}>
      <div className="profile-header">
        <div className="profile-status">
          {profile.is_active ? (
            <span className="status-indicator active">●</span>
          ) : (
            <span className="status-indicator">○</span>
          )}
        </div>
        <div className="profile-info">
          <h3 className="profile-name">{profile.name}</h3>
          <div className="profile-meta">
            <span className="profile-layers">{profile.layer_count} layers</span>
            <span className="profile-separator">•</span>
            <span className="profile-modified">
              Modified {formatTimestampRelative(profile.modified_at)}
            </span>
          </div>
        </div>
      </div>

      <div className="profile-actions">
        {!profile.is_active && (
          <button onClick={onActivate} className="action-button activate-button">
            Activate
          </button>
        )}
        {profile.is_active && <span className="active-label">Active Profile</span>}
      </div>

      <div className="profile-secondary-actions">
        <button onClick={onRename} className="secondary-action" title="Rename">
          Rename
        </button>
        <button onClick={onDuplicate} className="secondary-action" title="Duplicate">
          Duplicate
        </button>
        <button onClick={onExport} className="secondary-action" title="Export">
          Export
        </button>
        <button
          onClick={onDelete}
          className="secondary-action delete-action"
          title="Delete"
          disabled={profile.is_active}
        >
          Delete
        </button>
      </div>
    </div>
  );
}
