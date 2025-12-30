/**
 * ApiContext - Dependency Injection for API Endpoints
 *
 * Provides API and WebSocket base URLs to components for testability.
 * Supports environment variable overrides for different deployment environments.
 */

import { createContext, useContext, ReactNode } from 'react';

/**
 * API context value interface
 */
export interface ApiContextValue {
  /** Base URL for REST API endpoints (e.g., http://localhost:3030) */
  apiBaseUrl: string;
  /** Base URL for WebSocket connections (e.g., ws://localhost:9867) */
  wsBaseUrl: string;
}

/**
 * Default API base URL
 * Can be overridden by VITE_API_BASE_URL environment variable
 */
const DEFAULT_API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3030';

/**
 * Default WebSocket base URL
 * Can be overridden by VITE_WS_BASE_URL environment variable
 */
const DEFAULT_WS_BASE_URL = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:9867';

/**
 * API Context for dependency injection
 */
const ApiContext = createContext<ApiContextValue | undefined>(undefined);

/**
 * Props for ApiProvider component
 */
interface ApiProviderProps {
  /** Child components */
  children: ReactNode;
  /** Optional custom API base URL (overrides defaults and env vars) */
  apiBaseUrl?: string;
  /** Optional custom WebSocket base URL (overrides defaults and env vars) */
  wsBaseUrl?: string;
}

/**
 * ApiProvider Component
 *
 * Wraps the application to provide API endpoint configuration.
 * Enables testing by allowing mock URLs to be injected.
 *
 * @param props - Provider properties
 *
 * @example
 * ```tsx
 * // Production usage (uses defaults from env)
 * <ApiProvider>
 *   <App />
 * </ApiProvider>
 *
 * // Testing usage (inject mock URLs)
 * <ApiProvider
 *   apiBaseUrl="http://mock-api:3030"
 *   wsBaseUrl="ws://mock-ws:9867"
 * >
 *   <ComponentUnderTest />
 * </ApiProvider>
 * ```
 */
export function ApiProvider({ children, apiBaseUrl, wsBaseUrl }: ApiProviderProps) {
  const value: ApiContextValue = {
    apiBaseUrl: apiBaseUrl || DEFAULT_API_BASE_URL,
    wsBaseUrl: wsBaseUrl || DEFAULT_WS_BASE_URL,
  };

  return <ApiContext.Provider value={value}>{children}</ApiContext.Provider>;
}

/**
 * useApi Hook
 *
 * Accesses the API context to retrieve base URLs for API and WebSocket endpoints.
 * Must be used within an ApiProvider.
 *
 * @returns API context value with apiBaseUrl and wsBaseUrl
 * @throws Error if used outside ApiProvider
 *
 * @example
 * ```tsx
 * function ProfilesPage() {
 *   const { apiBaseUrl } = useApi();
 *
 *   useEffect(() => {
 *     fetch(`${apiBaseUrl}/api/profiles`)
 *       .then(res => res.json())
 *       .then(data => setProfiles(data));
 *   }, [apiBaseUrl]);
 *
 *   return <div>...</div>;
 * }
 * ```
 */
// eslint-disable-next-line react-refresh/only-export-components
export function useApi(): ApiContextValue {
  const context = useContext(ApiContext);

  if (!context) {
    throw new Error('useApi must be used within an ApiProvider');
  }

  return context;
}
