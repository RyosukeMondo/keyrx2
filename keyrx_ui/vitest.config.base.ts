/// <reference types="vitest" />
import type { UserConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import path from 'path';
import os from 'os';

/**
 * Base Vitest configuration for shared test settings.
 * This config is imported by vitest.unit.config.ts and vitest.integration.config.ts
 * to avoid duplication of test settings.
 *
 * Note: Exported as plain object (not defineConfig) to avoid type conflicts
 * between vite and vitest's bundled vite version.
 */
export const baseConfig: UserConfig = {
  plugins: [
    wasm(),
    topLevelAwait(),
    react(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      // Fix wasm-pack 'env' import issue
      'env': path.resolve(__dirname, './src/wasm/env-shim.js'),
    },
  },
  optimizeDeps: {
    exclude: ['@/wasm/pkg/keyrx_core'],
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    css: true,
    // Parallel execution configuration
    pool: 'threads',
    poolOptions: {
      threads: {
        // Optimize thread count based on CPU cores
        // Use 75% of available cores to leave headroom for system
        maxThreads: Math.max(1, Math.floor((os.cpus().length || 4) * 0.75)),
        minThreads: 1,
      },
    },
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov'],
      exclude: [
        'node_modules/**',
        'dist/**',
        'src/test/**',
        '**/*.test.{ts,tsx}',
        '**/*.spec.{ts,tsx}',
        'src/wasm/pkg/**',
      ],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 80,
        statements: 80,
      },
    },
  },
};
