import { create } from 'zustand';
import type { KeyMapping } from '../types';

interface ConfigStore {
  // State
  currentProfile: string | null;
  activeLayer: string;
  keyMappings: Map<string, KeyMapping>;
  loading: boolean;
  error: string | null;

  // Actions
  fetchConfig: (profile: string) => Promise<void>;
  setKeyMapping: (key: string, mapping: KeyMapping) => Promise<void>;
  deleteKeyMapping: (key: string) => Promise<void>;
  switchLayer: (layerId: string) => void;
  clearError: () => void;
}

export const useConfigStore = create<ConfigStore>((set, get) => ({
  // Initial state
  currentProfile: null,
  activeLayer: 'base',
  keyMappings: new Map(),
  loading: false,
  error: null,

  // Fetch configuration for a profile
  fetchConfig: async (profile: string) => {
    set({ loading: true, error: null });
    try {
      const response = await fetch(`/api/config/${profile}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch config: ${response.statusText}`);
      }

      const data = await response.json();

      // Convert key mappings object to Map
      const keyMappings = new Map<string, KeyMapping>(
        Object.entries(data.keyMappings || {})
      );

      set({
        currentProfile: profile,
        keyMappings,
        activeLayer: data.activeLayer || 'base',
        loading: false,
      });
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : 'Unknown error';
      set({ error: errorMessage, loading: false });
    }
  },

  // Set or update a key mapping
  setKeyMapping: async (key: string, mapping: KeyMapping) => {
    const { currentProfile, keyMappings } = get();

    if (!currentProfile) {
      const errorMessage = 'No profile loaded';
      set({ error: errorMessage });
      throw new Error(errorMessage);
    }

    // Store old mapping for rollback
    const oldMapping = keyMappings.get(key);
    const oldMappings = new Map(keyMappings);

    // Optimistic update
    const updatedMappings = new Map(keyMappings);
    updatedMappings.set(key, mapping);
    set({ keyMappings: updatedMappings, error: null });

    try {
      const response = await fetch(`/api/config/${currentProfile}/key`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key, mapping }),
      });

      if (!response.ok) {
        throw new Error(`Failed to set key mapping: ${response.statusText}`);
      }
    } catch (error) {
      // Rollback on error
      set({ keyMappings: oldMappings });
      const errorMessage =
        error instanceof Error ? error.message : 'Unknown error';
      set({ error: errorMessage });
      throw error;
    }
  },

  // Delete a key mapping (restore to default)
  deleteKeyMapping: async (key: string) => {
    const { currentProfile, keyMappings } = get();

    if (!currentProfile) {
      const errorMessage = 'No profile loaded';
      set({ error: errorMessage });
      throw new Error(errorMessage);
    }

    // Store old mapping for rollback
    const oldMapping = keyMappings.get(key);
    const oldMappings = new Map(keyMappings);

    // Optimistic update
    const updatedMappings = new Map(keyMappings);
    updatedMappings.delete(key);
    set({ keyMappings: updatedMappings, error: null });

    try {
      const response = await fetch(`/api/config/${currentProfile}/key`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key }),
      });

      if (!response.ok) {
        throw new Error(
          `Failed to delete key mapping: ${response.statusText}`
        );
      }
    } catch (error) {
      // Rollback on error
      set({ keyMappings: oldMappings });
      const errorMessage =
        error instanceof Error ? error.message : 'Unknown error';
      set({ error: errorMessage });
      throw error;
    }
  },

  // Switch active layer (local state only, doesn't persist)
  switchLayer: (layerId: string) => {
    set({ activeLayer: layerId });
  },

  // Clear error state
  clearError: () => set({ error: null }),
}));
