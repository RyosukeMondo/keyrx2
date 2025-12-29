import { create } from 'zustand';
import type { ProfileMetadata, Template, ActivationResult } from '../types';

interface ProfileStore {
  // State
  profiles: ProfileMetadata[];
  activeProfile: string | null;
  activating: boolean;
  activationProgress: number;
  loading: boolean;
  error: string | null;

  // Actions
  fetchProfiles: () => Promise<void>;
  createProfile: (name: string, template: Template) => Promise<void>;
  activateProfile: (name: string) => Promise<ActivationResult>;
  deleteProfile: (name: string) => Promise<void>;
  setActivationProgress: (progress: number) => void;
  clearError: () => void;
}

export const useProfileStore = create<ProfileStore>((set, get) => ({
  // Initial state
  profiles: [],
  activeProfile: null,
  activating: false,
  activationProgress: 0,
  loading: false,
  error: null,

  // Fetch all profiles
  fetchProfiles: async () => {
    set({ loading: true, error: null });
    try {
      const response = await fetch('/api/profiles');
      if (!response.ok) {
        throw new Error(`Failed to fetch profiles: ${response.statusText}`);
      }
      const profiles: ProfileMetadata[] = await response.json();

      // Find active profile
      const active = profiles.find((p) => p.isActive);
      set({
        profiles,
        activeProfile: active?.name || null,
        loading: false,
      });
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : 'Unknown error';
      set({ error: errorMessage, loading: false });
    }
  },

  // Create a new profile
  createProfile: async (name: string, template: Template) => {
    set({ loading: true, error: null });
    try {
      const response = await fetch('/api/profiles', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, template }),
      });

      if (!response.ok) {
        throw new Error(`Failed to create profile: ${response.statusText}`);
      }

      // Refresh profiles list
      await get().fetchProfiles();
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : 'Unknown error';
      set({ error: errorMessage, loading: false });
      throw error;
    }
  },

  // Activate a profile
  activateProfile: async (name: string) => {
    set({ activating: true, activationProgress: 0, error: null });
    try {
      const response = await fetch(`/api/profiles/${name}/activate`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`Failed to activate profile: ${response.statusText}`);
      }

      const result: ActivationResult = await response.json();

      if (result.success) {
        // Update profiles to reflect new active status
        const { profiles } = get();
        const updatedProfiles = profiles.map((p) => ({
          ...p,
          isActive: p.name === name,
        }));
        set({
          profiles: updatedProfiles,
          activeProfile: name,
          activating: false,
          activationProgress: 100,
        });
      } else {
        set({ activating: false, activationProgress: 0 });
        throw new Error(
          result.errors?.join(', ') || 'Activation failed'
        );
      }

      return result;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : 'Unknown error';
      set({ error: errorMessage, activating: false, activationProgress: 0 });
      throw error;
    }
  },

  // Delete a profile
  deleteProfile: async (name: string) => {
    const { profiles, activeProfile } = get();

    // Prevent deleting active profile
    if (name === activeProfile) {
      const errorMessage = 'Cannot delete the active profile';
      set({ error: errorMessage });
      throw new Error(errorMessage);
    }

    const oldProfiles = [...profiles];

    // Optimistic update
    const updatedProfiles = profiles.filter((p) => p.name !== name);
    set({ profiles: updatedProfiles, error: null });

    try {
      const response = await fetch(`/api/profiles/${name}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        throw new Error(`Failed to delete profile: ${response.statusText}`);
      }
    } catch (error) {
      // Rollback on error
      set({ profiles: oldProfiles });
      const errorMessage =
        error instanceof Error ? error.message : 'Unknown error';
      set({ error: errorMessage });
      throw error;
    }
  },

  // Set activation progress (for WebSocket updates)
  setActivationProgress: (progress: number) => {
    set({ activationProgress: progress });
  },

  // Clear error state
  clearError: () => set({ error: null }),
}));
