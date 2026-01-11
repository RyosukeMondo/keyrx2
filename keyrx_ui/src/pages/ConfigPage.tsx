import React, { useState, useEffect } from 'react';
import { useSearchParams, useParams, useNavigate } from 'react-router-dom';
import { Card } from '@/components/Card';
import { SimpleCodeEditor } from '@/components/SimpleCodeEditor';
import { KeyPalette, type PaletteKey } from '@/components/KeyPalette';
import { DeviceSelector, type Device } from '@/components/DeviceSelector';
import { KeyConfigModal } from '@/components/KeyConfigModal';
import { LayerSwitcher } from '@/components/LayerSwitcher';
import { KeyboardVisualizer } from '@/components/KeyboardVisualizer';
import { useGetProfileConfig, useSetProfileConfig } from '@/hooks/useProfileConfig';
import { useProfiles, useCreateProfile } from '@/hooks/useProfiles';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';
import { useDevices } from '@/hooks/useDevices';
import type { KeyMapping } from '@/types';

interface ConfigPageProps {
  profileName?: string;
}

const ConfigPage: React.FC<ConfigPageProps> = ({
  profileName: propProfileName,
}) => {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { name: profileNameFromRoute } = useParams<{ name: string }>();
  const profileNameFromQuery = searchParams.get('profile');
  const defaultProfileName = propProfileName || profileNameFromRoute || profileNameFromQuery || 'Default';

  const [selectedProfileName, setSelectedProfileName] = useState<string>(defaultProfileName);

  const api = useUnifiedApi();
  const [activeTab, setActiveTab] = useState<'visual' | 'code'>('visual');
  const [configCode, setConfigCode] = useState<string>('// Loading...');

  // Fetch available profiles
  const { data: profiles, isLoading: isLoadingProfiles } = useProfiles();
  const { mutateAsync: createProfile } = useCreateProfile();

  // Visual editor state
  const [selectedPaletteKey, setSelectedPaletteKey] = useState<PaletteKey | null>(null);
  const [selectedPhysicalKey, setSelectedPhysicalKey] = useState<string | null>(null);
  const [keyMappings, setKeyMappings] = useState<Map<string, KeyMapping>>(new Map());
  const [scope, setScope] = useState<'global' | 'device-specific'>('global');
  const [selectedDevice, setSelectedDevice] = useState<string>('');
  const [activeLayer, setActiveLayer] = useState<string>('base');
  const [isModalOpen, setIsModalOpen] = useState(false);

  // Available layers
  const availableLayers = ['base', 'md-00', 'md-01', 'md-02', 'md-03', 'md-04', 'md-05'];

  // Query for profile config - doesn't block rendering
  const { data: profileConfig, isLoading, error } = useGetProfileConfig(selectedProfileName);
  const { mutateAsync: setProfileConfig } = useSetProfileConfig();

  // Fetch devices
  const { data: devicesData } = useDevices();

  // Auto-select first profile if "Default" doesn't exist and profiles are loaded
  useEffect(() => {
    if (profiles && profiles.length > 0 && !profiles.some(p => p.name === selectedProfileName)) {
      setSelectedProfileName(profiles[0].name);
    }
  }, [profiles, selectedProfileName]);

  // Check if selected profile exists
  const profileExists = profiles?.some(p => p.name === selectedProfileName);

  // Check if config file exists
  const configExists = !isLoading && !error && profileConfig?.source;
  const configMissing = !isLoading && !error && profileExists && !profileConfig?.source;

  // Update config code when data loads
  useEffect(() => {
    if (profileConfig?.source) {
      setConfigCode(profileConfig.source);
    } else if (configMissing) {
      // Default config template when config file doesn't exist
      setConfigCode(`// Configuration for profile: ${selectedProfileName}
// This is a new configuration file

// Example: Simple key remapping
// map("A", "B");  // Press A → outputs B

// Example: Tap/Hold behavior
// tap_hold("CapsLock", "Escape", "LCtrl", 200);

// Add your key mappings here...
`);
    }
  }, [profileConfig, configMissing, selectedProfileName]);

  // Handle profile selection change
  const handleProfileChange = (newProfileName: string) => {
    setSelectedProfileName(newProfileName);
    navigate(`/config?profile=${newProfileName}`);
  };

  // Handle profile creation
  const handleCreateProfile = async () => {
    try {
      await createProfile({ name: selectedProfileName, template: 'blank' });
    } catch (err) {
      console.error('Failed to create profile:', err);
    }
  };

  const handleSaveConfig = async () => {
    try {
      await setProfileConfig({ name: selectedProfileName, source: configCode });
    } catch (err) {
      console.error('Failed to save config:', err);
    }
  };

  // Handle key click: open modal for configuration
  const handlePhysicalKeyClick = (keyCode: string) => {
    setSelectedPhysicalKey(keyCode);
    setIsModalOpen(true);
  };

  // Handle save from modal
  const handleSaveMapping = (mapping: KeyMapping) => {
    if (selectedPhysicalKey) {
      const newMappings = new Map(keyMappings);
      newMappings.set(selectedPhysicalKey, mapping);
      setKeyMappings(newMappings);
      // TODO: Auto-save to backend
    }
  };

  // Transform devices for DeviceSelector
  const devices: Device[] = devicesData
    ? devicesData.map((d) => ({
        id: d.id,
        name: d.name,
        serial: d.serial || undefined,
      }))
    : [];

  // Get all available keys for modal (will be passed from KeyPalette component data)
  // For now, use a simplified list
  const getAllAvailableKeys = (): PaletteKey[] => {
    // This should include all VK_, MD_, LK_ keys
    // For now returning empty array, will be populated from KeyPalette
    return [];
  };

  return (
    <div className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8">
      {/* Header with Profile Selector */}
      <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
        <div className="flex-1">
          <h1 className="text-2xl md:text-3xl font-bold text-slate-100 mb-3">
            Configuration Editor
          </h1>

          {/* Profile Selector */}
          <div className="flex items-center gap-3 flex-wrap">
            <label htmlFor="profile-selector" className="text-sm font-medium text-slate-300">
              Profile:
            </label>
            <select
              id="profile-selector"
              value={selectedProfileName}
              onChange={(e) => handleProfileChange(e.target.value)}
              disabled={isLoadingProfiles || !api.isConnected}
              className="px-4 py-2 bg-slate-700 border border-slate-600 rounded-md text-slate-100 font-medium focus:outline-none focus:ring-2 focus:ring-primary-500 disabled:opacity-50"
            >
              {profiles?.map((profile) => (
                <option key={profile.name} value={profile.name}>
                  {profile.name}
                </option>
              ))}
            </select>

            {/* Status indicators */}
            {!api.isConnected && (
              <span className="text-xs px-2 py-1 bg-yellow-900/30 border border-yellow-500 text-yellow-400 rounded">
                ⚠️ Disconnected
              </span>
            )}
            {isLoading && api.isConnected && (
              <span className="text-xs px-2 py-1 bg-blue-900/30 border border-blue-500 text-blue-400 rounded">
                ⏳ Loading configuration...
              </span>
            )}
            {configExists && (
              <span className="text-xs px-2 py-1 bg-green-900/30 border border-green-500 text-green-400 rounded">
                ✅ Loaded
              </span>
            )}
            {configMissing && (
              <span className="text-xs px-2 py-1 bg-orange-900/30 border border-orange-500 text-orange-400 rounded">
                📝 New configuration
              </span>
            )}
            {error && (
              <span className="text-xs px-2 py-1 bg-red-900/30 border border-red-500 text-red-400 rounded">
                ❌ Error
              </span>
            )}
            {!profileExists && !isLoading && api.isConnected && (
              <span className="text-xs px-2 py-1 bg-orange-900/30 border border-orange-500 text-orange-400 rounded">
                ⚠️ Profile not found
              </span>
            )}
          </div>

          {/* Error message with action */}
          {!profileExists && !isLoading && api.isConnected && (
            <div className="mt-3 p-3 bg-orange-900/20 border border-orange-500 rounded-md">
              <p className="text-sm text-orange-300 mb-2">
                Profile "{selectedProfileName}" does not exist.
              </p>
              <button
                onClick={handleCreateProfile}
                className="px-4 py-1.5 bg-orange-600 hover:bg-orange-500 text-white text-sm font-medium rounded transition-colors"
              >
                Create Profile "{selectedProfileName}"
              </button>
            </div>
          )}

          {configMissing && (
            <div className="mt-3 p-3 bg-blue-900/20 border border-blue-500 rounded-md">
              <p className="text-sm text-blue-300">
                📝 No configuration file found for "{selectedProfileName}". A template has been loaded - click <strong>Save Configuration</strong> to create it.
              </p>
            </div>
          )}

          {error && (
            <div className="mt-3 p-3 bg-red-900/20 border border-red-500 rounded-md">
              <p className="text-sm text-red-300">
                {error instanceof Error ? error.message : 'Failed to load configuration'}
              </p>
            </div>
          )}
        </div>

        <button
          onClick={handleSaveConfig}
          disabled={!api.isConnected || !profileExists}
          className="px-6 py-3 bg-primary-500 text-white font-medium rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {configMissing ? 'Create Configuration' : 'Save Configuration'}
        </button>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 border-b border-slate-700">
        <button
          onClick={() => setActiveTab('visual')}
          className={`px-4 py-2 font-medium transition-colors ${
            activeTab === 'visual'
              ? 'text-primary-400 border-b-2 border-primary-400'
              : 'text-slate-400 hover:text-slate-300'
          }`}
        >
          Visual Editor
        </button>
        <button
          onClick={() => setActiveTab('code')}
          className={`px-4 py-2 font-medium transition-colors ${
            activeTab === 'code'
              ? 'text-primary-400 border-b-2 border-primary-400'
              : 'text-slate-400 hover:text-slate-300'
          }`}
        >
          Code Editor
        </button>
      </div>

      {/* Content */}
      {activeTab === 'visual' ? (
        <>
          {/* Device Selector (compact at top) */}
          <DeviceSelector
            devices={devices}
            scope={scope}
            selectedDevice={selectedDevice}
            onScopeChange={setScope}
            onDeviceChange={setSelectedDevice}
          />

          {/* KEYBOARD ON TOP - Beautiful Layout */}
          <Card className="bg-gradient-to-br from-slate-800 to-slate-900">
            <h3 className="text-xl font-bold text-primary-400 mb-4">Keyboard Layout</h3>
            <div className="flex justify-center p-4">
              <KeyboardVisualizer
                layout="ANSI_104"
                keyMappings={keyMappings}
                onKeyClick={handlePhysicalKeyClick}
                simulatorMode={false}
              />
            </div>
            <p className="text-center text-sm text-slate-400 mt-4">
              Click any key to configure its mapping
            </p>
          </Card>

          {/* Layer Switcher */}
          <LayerSwitcher
            activeLayer={activeLayer}
            availableLayers={availableLayers}
            onLayerChange={setActiveLayer}
          />

          {/* Legend - Color coding */}
          <div className="flex gap-4 flex-wrap text-xs text-slate-400 px-2">
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 rounded bg-green-500"></div>
              <span>Simple</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 rounded bg-primary-500"></div>
              <span>Modifier</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 rounded bg-purple-500"></div>
              <span>Lock</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 rounded bg-red-500"></div>
              <span>Tap/Hold</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 rounded bg-yellow-500"></div>
              <span>Layer Active</span>
            </div>
          </div>

          {/* Key Palette - Comprehensive list at bottom */}
          <Card>
            <KeyPalette
              onKeySelect={setSelectedPaletteKey}
              selectedKey={selectedPaletteKey}
            />
          </Card>

          {/* Configuration Modal */}
          {isModalOpen && selectedPhysicalKey && (
            <KeyConfigModal
              isOpen={isModalOpen}
              onClose={() => setIsModalOpen(false)}
              physicalKey={selectedPhysicalKey}
              currentMapping={keyMappings.get(selectedPhysicalKey)}
              onSave={handleSaveMapping}
              availableKeys={getAllAvailableKeys()}
            />
          )}
        </>
      ) : (
        <Card variant="default" padding="lg">
          <SimpleCodeEditor
            value={configCode}
            onChange={(value) => setConfigCode(value)}
            height="600px"
            language="javascript"
          />
        </Card>
      )}
    </div>
  );
};

export default ConfigPage;
