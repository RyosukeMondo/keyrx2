import { Profile } from './ProfilesPage';
import { formatTimestampRelative } from '../utils/timeFormatting';
import './ProfileCard.css';

interface ProfileCardProps {
  profile: Profile;
  onActivate: () => void;
  onDelete: () => void;
  onDuplicate: () => void;
  onExport: () => void;
  onRename: () => void;
}

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
