#!/usr/bin/env tsx
/**
 * Clean up test profiles from the daemon
 * Deletes all profiles starting with 'prf-' or 'test-'
 */

import axios from 'axios';

const BASE_URL = 'http://localhost:9867';

async function cleanupTestProfiles() {
  try {
    // Get all profiles
    const response = await axios.get(`${BASE_URL}/api/profiles`);
    const profiles = response.data.profiles || [];

    console.log(`Found ${profiles.length} total profiles`);

    // Filter test profiles
    const testProfiles = profiles.filter((p: any) =>
      p.name.startsWith('prf-') || p.name.startsWith('test-')
    );

    console.log(`Found ${testProfiles.length} test profiles to delete`);

    // Delete each test profile
    let deleted = 0;
    for (const profile of testProfiles) {
      try {
        await axios.delete(`${BASE_URL}/api/profiles/${encodeURIComponent(profile.name)}`);
        console.log(`  ✓ Deleted: ${profile.name}`);
        deleted++;
      } catch (error: any) {
        console.error(`  ✗ Failed to delete ${profile.name}: ${error.message}`);
      }
    }

    console.log(`\nDeleted ${deleted}/${testProfiles.length} test profiles`);

  } catch (error: any) {
    console.error('Error:', error.message);
    if (error.response) {
      console.error('Response:', error.response.data);
    }
  }
}

cleanupTestProfiles();
