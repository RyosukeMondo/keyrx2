# Design Document

## 1. System Architecture Overview

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Browser (Client)                      │
├─────────────────────────────────────────────────────────────┤
│  React Application (keyrx_ui_v2)                            │
│  ├── Pages (Router)                                         │
│  │   ├── ConfigPage (Visual + Code Editor Tabs)            │
│  │   ├── DashboardPage (Real-time Monitoring)              │
│  │   ├── ProfilesPage                                       │
│  │   └── DevicesPage                                        │
│  ├── Components                                             │
│  │   ├── KeyboardVisualizer                                │
│  │   ├── MonacoEditor (lazy-loaded)                        │
│  │   ├── DashboardEventTimeline                            │
│  │   └── MetricsChart                                       │
│  ├── Hooks                                                  │
│  │   ├── useUnifiedApi (WebSocket RPC client)              │
│  │   └── useWasm (WASM validation)                         │
│  └── WASM Module (keyrx_core compiled to wasm32)           │
│      ├── validate_config()                                 │
│      └── simulate()                                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ├─ HTTP (initial page load)
                              └─ WebSocket (ws://localhost:9867/api)
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Daemon (keyrx_daemon)                      │
├─────────────────────────────────────────────────────────────┤
│  Axum Web Server                                            │
│  ├── Static File Handler (embedded UI)                     │
│  ├── WebSocket RPC Handler                                 │
│  │   ├── Message Router (method dispatch)                  │
│  │   ├── Request/Response Correlation (UUID tracking)      │
│  │   └── Subscription Manager (channels)                   │
│  └── Daemon State Bridge                                    │
│      ├── Profile Manager                                    │
│      ├── Device Manager                                     │
│      ├── Config Manager                                     │
│      └── Event Stream                                       │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Architectural Principles

**Single Source of Truth (SSOT)**:
- Configuration: `.krx` binary files (rkyv-serialized)
- State: Daemon holds authoritative state, UI subscribes to changes
- No client-side caching of server state (React Query handles staleness)

**Real-Time by Default**:
- All state changes broadcast via WebSocket events
- UI subscribes to relevant channels and updates reactively
- No polling required

**Type Safety End-to-End**:
- Rust types (keyrx_daemon) → JSON schema → TypeScript types (UI)
- Code generation for RPC method types
- Compile-time verification of message structure

**Deterministic Behavior**:
- WASM validation matches daemon validation byte-for-byte
- Same configuration always produces same output
- Enables automated testing and AI verification

## 2. Component Design

### 2.1 Unified WebSocket RPC API

#### 2.1.1 Message Protocol

**Client → Server Messages:**

```typescript
// Query (read-only, no side effects)
{
  type: "query",
  id: "uuid-v4",
  method: "getProfiles" | "getDevices" | "getConfig" | "getLayers" | "getLatency" | "getEvents",
  params?: any
}

// Command (write operation, has side effects)
{
  type: "command",
  id: "uuid-v4",
  method: "createProfile" | "activateProfile" | "deleteProfile" | "duplicateProfile" | "renameProfile" |
          "renameDevice" | "setScopeDevice" | "forgetDevice" |
          "updateConfig" | "setKeyMapping" | "deleteKeyMapping" |
          "clearEvents" | "simulate" | "resetSimulator",
  params: any
}

// Subscribe (start receiving events)
{
  type: "subscribe",
  id: "uuid-v4",
  channel: "daemon-state" | "events" | "latency",
  params?: any
}

// Unsubscribe (stop receiving events)
{
  type: "unsubscribe",
  id: "uuid-v4",
  channel: string
}
```

**Server → Client Messages:**

```typescript
// Response (reply to query/command)
{
  type: "response",
  id: "uuid-v4",  // Matches request ID
  result?: any,
  error?: {
    code: number,
    message: string,
    data?: any
  }
}

// Event (subscription broadcast)
{
  type: "event",
  channel: "daemon-state" | "events" | "latency",
  data: any
}

// Connected (handshake confirmation)
{
  type: "connected",
  version: "1.0.0",
  timestamp: number
}
```

#### 2.1.2 RPC Methods

**Profile Operations:**
- `getProfiles()` → `Profile[]`
- `createProfile(name: string, basedOn?: string)` → `Profile`
- `activateProfile(name: string)` → `void`
- `deleteProfile(name: string)` → `void`
- `duplicateProfile(name: string, newName: string)` → `Profile`
- `renameProfile(oldName: string, newName: string)` → `Profile`

**Device Operations:**
- `getDevices()` → `Device[]`
- `renameDevice(serialNumber: string, name: string)` → `Device`
- `setScopeDevice(serialNumber: string)` → `void`
- `forgetDevice(serialNumber: string)` → `void`

**Config Operations:**
- `getConfig(profileName: string)` → `{ code: string, hash: string }`
- `updateConfig(profileName: string, code: string)` → `{ hash: string }`
- `setKeyMapping(profileName: string, keyCode: number, mapping: KeyMapping)` → `void`
- `deleteKeyMapping(profileName: string, keyCode: number)` → `void`
- `getLayers(profileName: string)` → `Layer[]`

**Metrics Operations:**
- `getLatency()` → `LatencyMetrics`
- `getEvents(limit?: number, offset?: number)` → `KeyEvent[]`
- `clearEvents()` → `void`

**Simulator Operations:**
- `simulate(code: string, input: KeyEvent[])` → `SimulationResult`
- `resetSimulator()` → `void`

#### 2.1.3 Subscription Channels

**daemon-state:**
```typescript
{
  modifiers: number[],  // Active modifier IDs
  locks: number[],      // Active lock IDs
  layer: number,        // Current layer ID
  timestamp: number     // Microseconds since epoch
}
```

**events:**
```typescript
{
  timestamp: number,    // Microseconds
  keyCode: number,
  eventType: "press" | "release" | "tap" | "hold",
  latency: number,      // Microseconds
  layer: number
}
```

**latency:**
```typescript
{
  min: number,          // Microseconds
  avg: number,
  max: number,
  p50: number,
  p95: number,
  p99: number,
  count: number,
  timestamp: number
}
```

#### 2.1.4 Rust Implementation

**File Structure:**
```
keyrx_daemon/src/web/
├── mod.rs              // Axum server setup
├── rpc_types.rs        // Message type definitions
├── ws_rpc.rs           // WebSocket RPC handler
├── handlers/           // RPC method implementations
│   ├── mod.rs
│   ├── profile.rs      // Profile RPC methods
│   ├── device.rs       // Device RPC methods
│   ├── config.rs       // Config RPC methods
│   └── metrics.rs      // Metrics RPC methods
└── subscriptions.rs    // Subscription channel manager
```

**Core Types:**

```rust
// keyrx_daemon/src/web/rpc_types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClientMessage {
    Query {
        id: String,
        method: String,
        #[serde(default)]
        params: serde_json::Value,
    },
    Command {
        id: String,
        method: String,
        params: serde_json::Value,
    },
    Subscribe {
        id: String,
        channel: String,
        #[serde(default)]
        params: serde_json::Value,
    },
    Unsubscribe {
        id: String,
        channel: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerMessage {
    Response {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<RpcError>,
    },
    Event {
        channel: String,
        data: serde_json::Value,
    },
    Connected {
        version: String,
        timestamp: u64,
    },
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// Standard JSON-RPC error codes
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;
```

**WebSocket Handler:**

```rust
// keyrx_daemon/src/web/ws_rpc.rs
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use tokio::sync::broadcast;
use std::sync::Arc;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut subscriptions = Vec::new();

    // Send handshake
    let handshake = ServerMessage::Connected {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64,
    };
    if sender.send(Message::Text(serde_json::to_string(&handshake).unwrap())).await.is_err() {
        return;
    }

    // Message loop
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                let error = ServerMessage::Response {
                    id: "unknown".to_string(),
                    result: None,
                    error: Some(RpcError {
                        code: PARSE_ERROR,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let _ = sender.send(Message::Text(serde_json::to_string(&error).unwrap())).await;
                continue;
            }
        };

        let response = match client_msg {
            ClientMessage::Query { id, method, params } => {
                handle_query(&state, &id, &method, params).await
            }
            ClientMessage::Command { id, method, params } => {
                handle_command(&state, &id, &method, params).await
            }
            ClientMessage::Subscribe { id, channel, params } => {
                handle_subscribe(&state, &mut subscriptions, &id, &channel, params).await
            }
            ClientMessage::Unsubscribe { id, channel } => {
                handle_unsubscribe(&mut subscriptions, &id, &channel).await
            }
        };

        if let Some(resp) = response {
            let _ = sender.send(Message::Text(serde_json::to_string(&resp).unwrap())).await;
        }
    }
}

async fn handle_query(
    state: &AppState,
    id: &str,
    method: &str,
    params: serde_json::Value,
) -> Option<ServerMessage> {
    let result = match method {
        "getProfiles" => handlers::profile::get_profiles(state).await,
        "getDevices" => handlers::device::get_devices(state).await,
        "getConfig" => handlers::config::get_config(state, params).await,
        "getLayers" => handlers::config::get_layers(state, params).await,
        "getLatency" => handlers::metrics::get_latency(state).await,
        "getEvents" => handlers::metrics::get_events(state, params).await,
        _ => Err(RpcError {
            code: METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method),
            data: None,
        }),
    };

    Some(ServerMessage::Response {
        id: id.to_string(),
        result: result.ok(),
        error: result.err(),
    })
}

async fn handle_command(
    state: &AppState,
    id: &str,
    method: &str,
    params: serde_json::Value,
) -> Option<ServerMessage> {
    let result = match method {
        "createProfile" => handlers::profile::create_profile(state, params).await,
        "activateProfile" => handlers::profile::activate_profile(state, params).await,
        "deleteProfile" => handlers::profile::delete_profile(state, params).await,
        "updateConfig" => handlers::config::update_config(state, params).await,
        "setKeyMapping" => handlers::config::set_key_mapping(state, params).await,
        // ... other methods
        _ => Err(RpcError {
            code: METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method),
            data: None,
        }),
    };

    Some(ServerMessage::Response {
        id: id.to_string(),
        result: result.ok(),
        error: result.err(),
    })
}
```

#### 2.1.5 TypeScript Implementation

**File Structure:**
```
keyrx_ui_v2/src/
├── api/
│   ├── rpc.ts              // Type-safe RPC client
│   └── types.ts            // RPC method types
├── hooks/
│   └── useUnifiedApi.ts    // WebSocket RPC hook
└── types/
    └── rpc.ts              // Message type definitions
```

**RPC Types:**

```typescript
// keyrx_ui_v2/src/types/rpc.ts
export type RpcMethod =
  // Queries
  | "getProfiles"
  | "getDevices"
  | "getConfig"
  | "getLayers"
  | "getLatency"
  | "getEvents"
  // Commands
  | "createProfile"
  | "activateProfile"
  | "deleteProfile"
  | "duplicateProfile"
  | "renameProfile"
  | "renameDevice"
  | "setScopeDevice"
  | "forgetDevice"
  | "updateConfig"
  | "setKeyMapping"
  | "deleteKeyMapping"
  | "clearEvents"
  | "simulate"
  | "resetSimulator";

export type SubscriptionChannel = "daemon-state" | "events" | "latency";

export interface ClientMessage {
  type: "query" | "command" | "subscribe" | "unsubscribe";
  id: string;
  method?: RpcMethod;
  channel?: SubscriptionChannel;
  params?: any;
}

export interface ServerMessage {
  type: "response" | "event" | "connected";
  id?: string;
  channel?: string;
  result?: any;
  error?: RpcError;
  data?: any;
  version?: string;
  timestamp?: number;
}

export interface RpcError {
  code: number;
  message: string;
  data?: any;
}

export interface DaemonState {
  modifiers: number[];
  locks: number[];
  layer: number;
  timestamp: number;
}

export interface KeyEvent {
  timestamp: number;
  keyCode: number;
  eventType: "press" | "release" | "tap" | "hold";
  latency: number;
  layer: number;
}

export interface LatencyMetrics {
  min: number;
  avg: number;
  max: number;
  p50: number;
  p95: number;
  p99: number;
  count: number;
  timestamp: number;
}
```

**useUnifiedApi Hook:**

```typescript
// keyrx_ui_v2/src/hooks/useUnifiedApi.ts
import { useCallback, useEffect, useRef, useState } from 'react';
import useWebSocket, { ReadyState } from 'react-use-websocket';
import { v4 as uuidv4 } from 'uuid';
import type {
  ClientMessage,
  ServerMessage,
  RpcMethod,
  SubscriptionChannel,
} from '@/types/rpc';

interface PendingRequest {
  resolve: (value: any) => void;
  reject: (error: Error) => void;
  timeout: NodeJS.Timeout;
}

export function useUnifiedApi(url: string = 'ws://localhost:9867/api') {
  const [isConnected, setIsConnected] = useState(false);
  const pendingRequests = useRef<Map<string, PendingRequest>>(new Map());
  const subscriptions = useRef<Map<string, (data: any) => void>>(new Map());

  const { sendJsonMessage, lastJsonMessage, readyState } = useWebSocket<ServerMessage>(
    url,
    {
      shouldReconnect: () => true,
      reconnectInterval: 3000,
      reconnectAttempts: 10,
    }
  );

  // Handle incoming messages
  useEffect(() => {
    if (!lastJsonMessage) return;

    const msg = lastJsonMessage;

    if (msg.type === 'connected') {
      setIsConnected(true);
      console.info('[RPC] Connected to daemon', msg);
      return;
    }

    if (msg.type === 'response' && msg.id) {
      const pending = pendingRequests.current.get(msg.id);
      if (pending) {
        clearTimeout(pending.timeout);
        pendingRequests.current.delete(msg.id);

        if (msg.error) {
          pending.reject(new Error(`${msg.error.message} (code: ${msg.error.code})`));
        } else {
          pending.resolve(msg.result);
        }
      }
      return;
    }

    if (msg.type === 'event' && msg.channel) {
      const handler = subscriptions.current.get(msg.channel);
      if (handler) {
        handler(msg.data);
      }
    }
  }, [lastJsonMessage]);

  // Update connection status
  useEffect(() => {
    setIsConnected(readyState === ReadyState.OPEN);
  }, [readyState]);

  // Query method (read-only)
  const query = useCallback(
    <T = any>(method: RpcMethod, params?: any): Promise<T> => {
      return new Promise((resolve, reject) => {
        const id = uuidv4();
        const timeout = setTimeout(() => {
          pendingRequests.current.delete(id);
          reject(new Error(`Request timeout: ${method}`));
        }, 30000);

        pendingRequests.current.set(id, { resolve, reject, timeout });

        const message: ClientMessage = {
          type: 'query',
          id,
          method,
          params,
        };

        sendJsonMessage(message);
      });
    },
    [sendJsonMessage]
  );

  // Command method (write operation)
  const command = useCallback(
    <T = any>(method: RpcMethod, params: any): Promise<T> => {
      return new Promise((resolve, reject) => {
        const id = uuidv4();
        const timeout = setTimeout(() => {
          pendingRequests.current.delete(id);
          reject(new Error(`Request timeout: ${method}`));
        }, 30000);

        pendingRequests.current.set(id, { resolve, reject, timeout });

        const message: ClientMessage = {
          type: 'command',
          id,
          method,
          params,
        };

        sendJsonMessage(message);
      });
    },
    [sendJsonMessage]
  );

  // Subscribe to channel
  const subscribe = useCallback(
    (channel: SubscriptionChannel, handler: (data: any) => void): void => {
      subscriptions.current.set(channel, handler);

      const message: ClientMessage = {
        type: 'subscribe',
        id: uuidv4(),
        channel,
      };

      sendJsonMessage(message);
    },
    [sendJsonMessage]
  );

  // Unsubscribe from channel
  const unsubscribe = useCallback(
    (channel: SubscriptionChannel): void => {
      subscriptions.current.delete(channel);

      const message: ClientMessage = {
        type: 'unsubscribe',
        id: uuidv4(),
        channel,
      };

      sendJsonMessage(message);
    },
    [sendJsonMessage]
  );

  return {
    query,
    command,
    subscribe,
    unsubscribe,
    isConnected,
    readyState,
  };
}
```

**Type-Safe RPC Client:**

```typescript
// keyrx_ui_v2/src/api/rpc.ts
import type { Profile, Device, Layer, KeyMapping } from './types';
import type { DaemonState, KeyEvent, LatencyMetrics } from '@/types/rpc';

export class RpcClient {
  constructor(private api: ReturnType<typeof useUnifiedApi>) {}

  // Profile methods
  async getProfiles(): Promise<Profile[]> {
    return this.api.query('getProfiles');
  }

  async createProfile(name: string, basedOn?: string): Promise<Profile> {
    return this.api.command('createProfile', { name, basedOn });
  }

  async activateProfile(name: string): Promise<void> {
    return this.api.command('activateProfile', { name });
  }

  async deleteProfile(name: string): Promise<void> {
    return this.api.command('deleteProfile', { name });
  }

  async duplicateProfile(name: string, newName: string): Promise<Profile> {
    return this.api.command('duplicateProfile', { name, newName });
  }

  async renameProfile(oldName: string, newName: string): Promise<Profile> {
    return this.api.command('renameProfile', { oldName, newName });
  }

  // Device methods
  async getDevices(): Promise<Device[]> {
    return this.api.query('getDevices');
  }

  async renameDevice(serialNumber: string, name: string): Promise<Device> {
    return this.api.command('renameDevice', { serialNumber, name });
  }

  // Config methods
  async getConfig(profileName: string): Promise<{ code: string; hash: string }> {
    return this.api.query('getConfig', { profileName });
  }

  async updateConfig(profileName: string, code: string): Promise<{ hash: string }> {
    return this.api.command('updateConfig', { profileName, code });
  }

  async getLayers(profileName: string): Promise<Layer[]> {
    return this.api.query('getLayers', { profileName });
  }

  // Metrics methods
  async getLatency(): Promise<LatencyMetrics> {
    return this.api.query('getLatency');
  }

  async getEvents(limit?: number, offset?: number): Promise<KeyEvent[]> {
    return this.api.query('getEvents', { limit, offset });
  }

  // Subscriptions
  onDaemonState(handler: (state: DaemonState) => void): void {
    this.api.subscribe('daemon-state', handler);
  }

  onEvents(handler: (event: KeyEvent) => void): void {
    this.api.subscribe('events', handler);
  }

  onLatency(handler: (metrics: LatencyMetrics) => void): void {
    this.api.subscribe('latency', handler);
  }
}
```

### 2.2 Monaco Code Editor Integration

#### 2.2.1 Component Architecture

```typescript
// keyrx_ui_v2/src/components/MonacoEditor.tsx
import React, { useRef, useEffect, useState } from 'react';
import Editor, { Monaco } from '@monaco-editor/react';
import type * as monacoEditor from 'monaco-editor';
import { useWasm } from '@/hooks/useWasm';

interface MonacoEditorProps {
  value: string;
  onChange: (value: string) => void;
  onValidate?: (errors: ValidationError[]) => void;
  readOnly?: boolean;
}

export const MonacoEditor: React.FC<MonacoEditorProps> = ({
  value,
  onChange,
  onValidate,
  readOnly = false,
}) => {
  const editorRef = useRef<monacoEditor.editor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<Monaco | null>(null);
  const { validateConfig, isWasmReady } = useWasm();
  const [markers, setMarkers] = useState<monacoEditor.editor.IMarkerData[]>([]);

  // Register Rhai language
  const handleEditorWillMount = (monaco: Monaco) => {
    monacoRef.current = monaco;

    // Register Rhai language
    monaco.languages.register({ id: 'rhai' });

    // Define syntax highlighting
    monaco.languages.setMonarchTokensProvider('rhai', {
      keywords: [
        'let', 'const', 'if', 'else', 'while', 'for', 'loop', 'break', 'continue',
        'return', 'fn', 'private', 'true', 'false', 'import', 'export', 'as',
      ],
      operators: [
        '=', '>', '<', '!', '~', '?', ':',
        '==', '<=', '>=', '!=', '&&', '||', '++', '--',
        '+', '-', '*', '/', '&', '|', '^', '%', '<<',
        '>>', '>>>', '+=', '-=', '*=', '/=', '&=', '|=',
        '^=', '%=', '<<=', '>>=', '>>>=',
      ],
      tokenizer: {
        root: [
          [/[a-z_$][\w$]*/, {
            cases: {
              '@keywords': 'keyword',
              '@default': 'identifier',
            },
          }],
          [/[A-Z][\w$]*/, 'type.identifier'],
          [/"([^"\\]|\\.)*$/, 'string.invalid'],
          [/"/, 'string', '@string'],
          [/\d+/, 'number'],
          [/\/\/.*$/, 'comment'],
        ],
        string: [
          [/[^\\"]+/, 'string'],
          [/"/, 'string', '@pop'],
        ],
      },
    });

    // Configure theme
    monaco.editor.defineTheme('rhai-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'keyword', foreground: 'C586C0' },
        { token: 'identifier', foreground: '9CDCFE' },
        { token: 'type.identifier', foreground: '4EC9B0' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'comment', foreground: '6A9955' },
      ],
      colors: {
        'editor.background': '#1E1E1E',
      },
    });
  };

  // Handle editor mount
  const handleEditorDidMount = (
    editor: monacoEditor.editor.IStandaloneCodeEditor,
    monaco: Monaco
  ) => {
    editorRef.current = editor;

    // Configure editor options
    editor.updateOptions({
      minimap: { enabled: false },
      fontSize: 14,
      lineHeight: 21,
      tabSize: 2,
      insertSpaces: true,
      automaticLayout: true,
      scrollBeyondLastLine: false,
      renderWhitespace: 'selection',
      rulers: [80, 120],
    });

    // F8: Go to next error
    editor.addCommand(monaco.KeyCode.F8, () => {
      const model = editor.getModel();
      if (!model) return;

      const markers = monaco.editor.getModelMarkers({ resource: model.uri });
      if (markers.length === 0) return;

      const position = editor.getPosition();
      if (!position) return;

      // Find next marker after current position
      const nextMarker = markers.find(
        (m) =>
          m.startLineNumber > position.lineNumber ||
          (m.startLineNumber === position.lineNumber &&
            m.startColumn > position.column)
      ) || markers[0];

      editor.setPosition({
        lineNumber: nextMarker.startLineNumber,
        column: nextMarker.startColumn,
      });
      editor.revealLineInCenter(nextMarker.startLineNumber);
    });
  };

  // Debounced validation
  useEffect(() => {
    if (!isWasmReady) return;

    const timeoutId = setTimeout(async () => {
      const errors = await validateConfig(value);

      const newMarkers: monacoEditor.editor.IMarkerData[] = errors.map((err) => ({
        severity: monacoRef.current!.MarkerSeverity.Error,
        startLineNumber: err.line,
        startColumn: err.column,
        endLineNumber: err.line,
        endColumn: err.column + (err.length || 1),
        message: err.message,
      }));

      setMarkers(newMarkers);
      onValidate?.(errors);

      if (editorRef.current && monacoRef.current) {
        const model = editorRef.current.getModel();
        if (model) {
          monacoRef.current.editor.setModelMarkers(model, 'rhai', newMarkers);
        }
      }
    }, 500);

    return () => clearTimeout(timeoutId);
  }, [value, isWasmReady, validateConfig, onValidate]);

  return (
    <Editor
      height="600px"
      language="rhai"
      theme="rhai-dark"
      value={value}
      onChange={(val) => onChange(val || '')}
      beforeMount={handleEditorWillMount}
      onMount={handleEditorDidMount}
      options={{
        readOnly,
      }}
    />
  );
};
```

#### 2.2.2 WASM Integration

```typescript
// keyrx_ui_v2/src/hooks/useWasm.ts
import { useState, useEffect, useCallback } from 'react';
import init, { validate_config, simulate } from '@/wasm/keyrx_core';

export interface ValidationError {
  line: number;
  column: number;
  length: number;
  message: string;
}

export function useWasm() {
  const [isWasmReady, setIsWasmReady] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    init()
      .then(() => {
        setIsWasmReady(true);
        console.info('[WASM] Initialized successfully');
      })
      .catch((err) => {
        setError(err);
        console.error('[WASM] Initialization failed:', err);
      });
  }, []);

  const validateConfig = useCallback(
    async (code: string): Promise<ValidationError[]> => {
      if (!isWasmReady) {
        console.warn('[WASM] Not ready, skipping validation');
        return [];
      }

      try {
        const result = validate_config(code);
        return JSON.parse(result);
      } catch (err) {
        console.error('[WASM] Validation error:', err);
        return [];
      }
    },
    [isWasmReady]
  );

  const runSimulation = useCallback(
    async (code: string, input: any[]): Promise<any> => {
      if (!isWasmReady) {
        throw new Error('WASM not ready');
      }

      try {
        const result = simulate(code, JSON.stringify(input));
        return JSON.parse(result);
      } catch (err) {
        console.error('[WASM] Simulation error:', err);
        throw err;
      }
    },
    [isWasmReady]
  );

  return {
    isWasmReady,
    error,
    validateConfig,
    runSimulation,
  };
}
```

### 2.3 Real-Time Dashboard

#### 2.3.1 Dashboard Page Architecture

```typescript
// keyrx_ui_v2/src/pages/DashboardPage.tsx
import React, { useState, useEffect } from 'react';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';
import { RpcClient } from '@/api/rpc';
import { StateIndicatorPanel } from '@/components/StateIndicatorPanel';
import { MetricsChart } from '@/components/MetricsChart';
import { DashboardEventTimeline } from '@/components/DashboardEventTimeline';
import type { DaemonState, KeyEvent, LatencyMetrics } from '@/types/rpc';

export const DashboardPage: React.FC = () => {
  const api = useUnifiedApi();
  const rpc = new RpcClient(api);

  const [daemonState, setDaemonState] = useState<DaemonState | null>(null);
  const [events, setEvents] = useState<KeyEvent[]>([]);
  const [latencyHistory, setLatencyHistory] = useState<LatencyMetrics[]>([]);
  const [isPaused, setIsPaused] = useState(false);

  // Subscribe to real-time updates
  useEffect(() => {
    if (!api.isConnected) return;

    rpc.onDaemonState((state) => {
      setDaemonState(state);
    });

    rpc.onEvents((event) => {
      if (!isPaused) {
        setEvents((prev) => {
          const updated = [event, ...prev];
          return updated.slice(0, 100); // Keep max 100 events (FIFO)
        });
      }
    });

    rpc.onLatency((metrics) => {
      setLatencyHistory((prev) => {
        const updated = [...prev, metrics];
        // Keep 60 seconds of data (assuming 1s interval)
        return updated.slice(-60);
      });
    });

    return () => {
      api.unsubscribe('daemon-state');
      api.unsubscribe('events');
      api.unsubscribe('latency');
    };
  }, [api.isConnected, isPaused]);

  return (
    <div className="flex flex-col gap-4 p-4 md:p-6">
      {/* Connection Banner */}
      <div
        className={`px-4 py-3 rounded-md font-medium ${
          api.isConnected
            ? 'bg-green-900/20 text-green-400 border border-green-700'
            : 'bg-red-900/20 text-red-400 border border-red-700'
        }`}
      >
        {api.isConnected ? '🟢 Connected to daemon' : '🔴 Disconnected (reconnecting...)'}
      </div>

      {/* State Indicators */}
      <StateIndicatorPanel state={daemonState} />

      {/* Metrics Chart */}
      <MetricsChart data={latencyHistory} />

      {/* Event Timeline */}
      <DashboardEventTimeline
        events={events}
        isPaused={isPaused}
        onTogglePause={() => setIsPaused(!isPaused)}
        onClear={() => setEvents([])}
      />
    </div>
  );
};
```

#### 2.3.2 State Indicator Component

```typescript
// keyrx_ui_v2/src/components/StateIndicatorPanel.tsx
import React from 'react';
import type { DaemonState } from '@/types/rpc';

interface StateIndicatorPanelProps {
  state: DaemonState | null;
}

export const StateIndicatorPanel: React.FC<StateIndicatorPanelProps> = ({ state }) => {
  if (!state) {
    return (
      <div className="bg-slate-800 rounded-lg p-4">
        <p className="text-slate-400">Waiting for daemon state...</p>
      </div>
    );
  }

  return (
    <div className="bg-slate-800 rounded-lg p-4">
      <h2 className="text-lg font-medium text-slate-100 mb-4">Daemon State</h2>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Modifiers */}
        <div>
          <h3 className="text-sm font-medium text-slate-300 mb-2">Modifiers</h3>
          <div className="flex flex-wrap gap-2">
            {state.modifiers.length === 0 ? (
              <span className="text-slate-500 text-sm">None</span>
            ) : (
              state.modifiers.map((id) => (
                <span
                  key={id}
                  className="px-2 py-1 bg-blue-600 text-white text-xs rounded"
                >
                  MOD_{id}
                </span>
              ))
            )}
          </div>
        </div>

        {/* Locks */}
        <div>
          <h3 className="text-sm font-medium text-slate-300 mb-2">Locks</h3>
          <div className="flex flex-wrap gap-2">
            {state.locks.length === 0 ? (
              <span className="text-slate-500 text-sm">None</span>
            ) : (
              state.locks.map((id) => (
                <span
                  key={id}
                  className="px-2 py-1 bg-orange-600 text-white text-xs rounded"
                >
                  LOCK_{id}
                </span>
              ))
            )}
          </div>
        </div>

        {/* Layer */}
        <div>
          <h3 className="text-sm font-medium text-slate-300 mb-2">Active Layer</h3>
          <span className="px-3 py-2 bg-green-600 text-white text-sm rounded font-medium">
            Layer {state.layer}
          </span>
        </div>
      </div>
    </div>
  );
};
```

#### 2.3.3 Metrics Chart Component

```typescript
// keyrx_ui_v2/src/components/MetricsChart.tsx
import React from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  ReferenceLine,
} from 'recharts';
import type { LatencyMetrics } from '@/types/rpc';

interface MetricsChartProps {
  data: LatencyMetrics[];
}

export const MetricsChart: React.FC<MetricsChartProps> = ({ data }) => {
  const chartData = data.map((m, idx) => ({
    index: idx,
    avg: m.avg / 1000, // Convert to milliseconds
    p95: m.p95 / 1000,
    p99: m.p99 / 1000,
  }));

  return (
    <div className="bg-slate-800 rounded-lg p-4">
      <h2 className="text-lg font-medium text-slate-100 mb-4">Latency Metrics</h2>

      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
          <XAxis dataKey="index" stroke="#9CA3AF" />
          <YAxis stroke="#9CA3AF" label={{ value: 'ms', angle: -90, position: 'insideLeft' }} />
          <Tooltip
            contentStyle={{ backgroundColor: '#1F2937', border: 'none', borderRadius: '8px' }}
            labelStyle={{ color: '#F3F4F6' }}
          />
          <Legend />
          <ReferenceLine y={5} stroke="#EF4444" strokeDasharray="3 3" label="Target (5ms)" />
          <Line type="monotone" dataKey="avg" stroke="#3B82F6" name="Average" />
          <Line type="monotone" dataKey="p95" stroke="#F59E0B" name="P95" />
          <Line type="monotone" dataKey="p99" stroke="#EF4444" name="P99" />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};
```

### 2.4 Tab-Based Configuration Editor

```typescript
// keyrx_ui_v2/src/pages/ConfigPage.tsx (updated)
import React, { useState, useCallback } from 'react';
import { Card } from '@/components/Card';
import { KeyboardVisualizer } from '@/components/KeyboardVisualizer';
import { MonacoEditor } from '@/components/MonacoEditor';
import { RpcClient } from '@/api/rpc';
import { useUnifiedApi } from '@/hooks/useUnifiedApi';

type EditorTab = 'visual' | 'code';

export const ConfigPage: React.FC<{ profileName: string }> = ({ profileName }) => {
  const api = useUnifiedApi();
  const rpc = new RpcClient(api);

  const [activeTab, setActiveTab] = useState<EditorTab>('visual');
  const [configCode, setConfigCode] = useState('');
  const [validationErrors, setValidationErrors] = useState<ValidationError[]>([]);
  const [isSaving, setIsSaving] = useState(false);

  // Load config on mount
  useEffect(() => {
    if (api.isConnected) {
      rpc.getConfig(profileName).then((config) => {
        setConfigCode(config.code);
      });
    }
  }, [profileName, api.isConnected]);

  // Save configuration
  const handleSave = useCallback(async () => {
    if (validationErrors.length > 0) {
      alert('Cannot save: Configuration has validation errors');
      return;
    }

    setIsSaving(true);
    try {
      await rpc.updateConfig(profileName, configCode);
      console.info('Configuration saved successfully');
    } catch (err) {
      console.error('Failed to save configuration:', err);
      alert(`Failed to save: ${err.message}`);
    } finally {
      setIsSaving(false);
    }
  }, [profileName, configCode, validationErrors]);

  // Keyboard shortcut: Ctrl+S / Cmd+S
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        handleSave();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleSave]);

  return (
    <div className="flex flex-col gap-4 p-4 md:p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-slate-100">
          Configuration Editor - {profileName}
        </h1>
        <button
          onClick={handleSave}
          disabled={isSaving || validationErrors.length > 0}
          className="px-4 py-2 bg-primary-500 text-white rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isSaving ? 'Saving...' : 'Save (Ctrl+S)'}
        </button>
      </div>

      {/* Validation Status */}
      {validationErrors.length > 0 && (
        <div className="px-4 py-3 bg-red-900/20 text-red-400 border border-red-700 rounded-md">
          ⚠️ {validationErrors.length} validation error{validationErrors.length > 1 ? 's' : ''} found
        </div>
      )}

      {/* Tab Selector */}
      <div className="flex gap-2 border-b border-slate-700">
        <button
          onClick={() => setActiveTab('visual')}
          className={`px-4 py-2 font-medium transition-colors ${
            activeTab === 'visual'
              ? 'bg-primary-500 text-white border-b-2 border-primary-500'
              : 'text-slate-400 hover:text-slate-300'
          }`}
        >
          Visual Editor
        </button>
        <button
          onClick={() => setActiveTab('code')}
          className={`px-4 py-2 font-medium transition-colors ${
            activeTab === 'code'
              ? 'bg-primary-500 text-white border-b-2 border-primary-500'
              : 'text-slate-400 hover:text-slate-300'
          }`}
        >
          Code Editor
        </button>
      </div>

      {/* Editor Content */}
      <Card variant="default" padding="lg">
        {activeTab === 'visual' ? (
          <KeyboardVisualizer
            layout="ANSI_104"
            keyMappings={new Map()}
            onKeyClick={(keyCode) => console.log('Key clicked:', keyCode)}
          />
        ) : (
          <MonacoEditor
            value={configCode}
            onChange={setConfigCode}
            onValidate={setValidationErrors}
          />
        )}
      </Card>
    </div>
  );
};
```

### 2.5 Build Process Integration

#### 2.5.1 WASM Build Configuration

```toml
# keyrx_core/Cargo.toml
[package]
name = "keyrx_core"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
rkyv = { version = "0.7", default-features = false }
serde = { version = "1.0", default-features = false, features = ["derive"] }
wasm-bindgen = { version = "0.2", optional = true }
serde-wasm-bindgen = { version = "0.6", optional = true }

[features]
default = ["std"]
std = []
wasm = ["wasm-bindgen", "serde-wasm-bindgen"]

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Single codegen unit for better optimization
strip = true        # Strip debug symbols
```

```javascript
// keyrx_ui_v2/vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import path from 'path';

export default defineConfig({
  plugins: [
    react(),
    wasm(),
    topLevelAwait(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    target: 'esnext',
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor': ['react', 'react-dom', 'react-router-dom'],
          'monaco': ['@monaco-editor/react', 'monaco-editor'],
          'charts': ['recharts'],
        },
      },
    },
  },
  optimizeDeps: {
    exclude: ['@/wasm/keyrx_core'],
  },
});
```

#### 2.5.2 Build Script

```bash
#!/usr/bin/env bash
# scripts/build_ui.sh

set -e

echo "[1/4] Building WASM module..."
cd keyrx_core
wasm-pack build --target web --out-dir ../keyrx_ui_v2/src/wasm/pkg --release
cd ..

echo "[2/4] Installing UI dependencies..."
cd keyrx_ui_v2
npm install

echo "[3/4] Building UI..."
npm run build

echo "[4/4] Verifying build..."
if [ ! -f "dist/index.html" ]; then
  echo "ERROR: UI build failed - dist/index.html not found"
  exit 1
fi

echo "✓ UI build complete: keyrx_ui_v2/dist/"
```

#### 2.5.3 Daemon Embedding

```rust
// keyrx_daemon/build.rs
use std::path::Path;

fn main() {
    // Verify UI dist directory exists
    let ui_dist = Path::new("../keyrx_ui_v2/dist");
    if !ui_dist.exists() {
        panic!(
            "UI directory not found: {}\nRun 'scripts/build_ui.sh' first",
            ui_dist.display()
        );
    }

    // Trigger rebuild if UI files change
    println!("cargo:rerun-if-changed=../keyrx_ui_v2/dist");
}
```

```rust
// keyrx_daemon/src/web/static_files.rs
use axum::{
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use include_dir::{include_dir, Dir};

static UI_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../keyrx_ui_v2/dist");

pub async fn serve_static(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Default to index.html for SPA routing
    let path = if path.is_empty() || !path.contains('.') {
        "index.html"
    } else {
        path
    };

    match UI_DIR.get_file(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let body = file.contents();

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(body.into())
                .unwrap()
        }
        None if path != "index.html" => {
            // Fallback to index.html for client-side routing
            serve_static("/".parse().unwrap()).await.into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_embedded() {
        assert!(UI_DIR.get_file("index.html").is_some(), "index.html not found in embedded UI");
    }
}
```

## 3. Data Flow

### 3.1 Configuration Edit Flow

```
User Types in Monaco Editor
         │
         ├─ (500ms debounce)
         │
         ▼
WASM validate_config(code)
         │
         ├─ Success → Clear markers
         └─ Errors → Set error markers
         │
         ▼
User Presses Ctrl+S
         │
         ├─ IF errors > 0 → Block save
         └─ IF errors = 0 → Send RPC command
         │
         ▼
command("updateConfig", { profileName, code })
         │
         ▼
Daemon validates & saves .krx file
         │
         ├─ Success → Return { hash }
         └─ Error → Return RpcError
         │
         ▼
UI displays success/error message
```

### 3.2 Real-Time Event Flow

```
Hardware Key Press
         │
         ▼
Platform Layer (evdev/Raw Input)
         │
         ▼
Daemon Processes Event (keyrx_core)
         │
         ├─ Update internal state
         ├─ Record latency metric
         └─ Broadcast events
         │
         ├─ Channel: "daemon-state" → { modifiers, locks, layer }
         ├─ Channel: "events" → { keyCode, eventType, latency }
         └─ Channel: "latency" → { min, avg, max, p95, p99 }
         │
         ▼
All Subscribed WebSocket Clients Receive Events
         │
         ▼
UI React State Updates
         │
         ├─ StateIndicatorPanel re-renders
         ├─ MetricsChart appends data point
         └─ DashboardEventTimeline prepends event
```

### 3.3 Profile Activation Flow

```
User Clicks "Activate" on ProfilesPage
         │
         ▼
command("activateProfile", { name })
         │
         ▼
Daemon Loads .krx Binary
         │
         ├─ Read file from ~/.config/keyrx/profiles/{name}.krx
         ├─ Deserialize with rkyv (zero-copy)
         └─ Swap active configuration atomically
         │
         ▼
Daemon Broadcasts "daemon-state" Event
         │
         ▼
All Subscribed Clients Update UI
         │
         ├─ ProfilesPage shows new active profile
         └─ DashboardPage updates state indicators
```

## 4. Security Design

### 4.1 WebSocket Security

**Origin Validation:**
```rust
// keyrx_daemon/src/web/mod.rs
use axum::extract::ws::WebSocketUpgrade;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    // Only allow localhost connections
    let origin = headers.get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !origin.starts_with("http://localhost") && !origin.starts_with("http://127.0.0.1") {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(ws.on_upgrade(|socket| handle_socket(socket)))
}
```

**Content Security Policy:**
```html
<!-- keyrx_ui_v2/index.html -->
<meta http-equiv="Content-Security-Policy" content="
  default-src 'self';
  script-src 'self' 'wasm-unsafe-eval';
  style-src 'self' 'unsafe-inline';
  connect-src 'self' ws://localhost:9867;
  img-src 'self' data:;
">
```

### 4.2 Input Validation

```rust
// keyrx_daemon/src/web/handlers/config.rs
use serde::Deserialize;

#[derive(Deserialize)]
struct UpdateConfigParams {
    #[serde(rename = "profileName")]
    profile_name: String,
    code: String,
}

pub async fn update_config(
    state: &AppState,
    params: serde_json::Value,
) -> Result<serde_json::Value, RpcError> {
    let params: UpdateConfigParams = serde_json::from_value(params).map_err(|e| RpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid parameters: {}", e),
        data: None,
    })?;

    // Validate profile name (no path traversal)
    if params.profile_name.contains("..") || params.profile_name.contains('/') {
        return Err(RpcError {
            code: INVALID_PARAMS,
            message: "Invalid profile name".to_string(),
            data: None,
        });
    }

    // Validate code length (max 1MB)
    if params.code.len() > 1_000_000 {
        return Err(RpcError {
            code: INVALID_PARAMS,
            message: "Configuration too large (max 1MB)".to_string(),
            data: None,
        });
    }

    // Validate with keyrx_core compiler
    let config = keyrx_compiler::parse(&params.code).map_err(|e| RpcError {
        code: INVALID_PARAMS,
        message: format!("Configuration validation failed: {}", e),
        data: None,
    })?;

    // Save configuration
    state.profile_manager.save_config(&params.profile_name, &config).await?;

    Ok(serde_json::json!({ "hash": config.hash() }))
}
```

## 5. Performance Optimizations

### 5.1 Code Splitting

```javascript
// keyrx_ui_v2/src/App.tsx
import { lazy, Suspense } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';

const ProfilesPage = lazy(() => import('./pages/ProfilesPage'));
const ConfigPage = lazy(() => import('./pages/ConfigPage'));
const DashboardPage = lazy(() => import('./pages/DashboardPage'));
const DevicesPage = lazy(() => import('./pages/DevicesPage'));

function App() {
  return (
    <BrowserRouter>
      <Suspense fallback={<LoadingSkeleton />}>
        <Routes>
          <Route path="/" element={<ProfilesPage />} />
          <Route path="/config/:profile" element={<ConfigPage />} />
          <Route path="/dashboard" element={<DashboardPage />} />
          <Route path="/devices" element={<DevicesPage />} />
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}
```

### 5.2 Virtual Scrolling

```typescript
// keyrx_ui_v2/src/components/DashboardEventTimeline.tsx
import { FixedSizeList as List } from 'react-window';

export const DashboardEventTimeline: React.FC<{ events: KeyEvent[] }> = ({ events }) => {
  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const event = events[index];
    return (
      <div style={style} className="border-b border-slate-700 px-4 py-2">
        <div className="flex items-center justify-between">
          <span className="text-sm text-slate-300">{formatKeyCode(event.keyCode)}</span>
          <span className="text-xs text-slate-400">{formatTimestamp(event.timestamp)}</span>
        </div>
      </div>
    );
  };

  return (
    <List
      height={400}
      itemCount={events.length}
      itemSize={50}
      width="100%"
    >
      {Row}
    </List>
  );
};
```

### 5.3 Debounced Validation

```typescript
// Already implemented in MonacoEditor component (see 2.2.1)
// 500ms debounce prevents excessive WASM calls while typing
```

## 6. Testing Strategy

### 6.1 Unit Tests

**Frontend (Vitest):**
```typescript
// keyrx_ui_v2/src/hooks/useUnifiedApi.test.ts
import { renderHook, waitFor } from '@testing-library/react';
import { useUnifiedApi } from './useUnifiedApi';

describe('useUnifiedApi', () => {
  it('should connect to WebSocket', async () => {
    const { result } = renderHook(() => useUnifiedApi('ws://localhost:9867/api'));

    await waitFor(() => {
      expect(result.current.isConnected).toBe(true);
    });
  });

  it('should send query and receive response', async () => {
    const { result } = renderHook(() => useUnifiedApi('ws://localhost:9867/api'));

    await waitFor(() => expect(result.current.isConnected).toBe(true));

    const profiles = await result.current.query('getProfiles');
    expect(Array.isArray(profiles)).toBe(true);
  });
});
```

**Backend (Rust):**
```rust
// keyrx_daemon/src/web/ws_rpc_test.rs
#[tokio::test]
async fn test_rpc_query_get_profiles() {
    let state = Arc::new(AppState::default());
    let params = serde_json::json!({});

    let response = handle_query(&state, "test-id", "getProfiles", params).await;

    assert!(response.is_some());
    let msg = response.unwrap();
    assert!(matches!(msg, ServerMessage::Response { .. }));
}
```

### 6.2 Integration Tests

```typescript
// keyrx_ui_v2/tests/integration/config-editor.test.tsx
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ConfigPage } from '@/pages/ConfigPage';

describe('ConfigPage Integration', () => {
  it('should switch between visual and code editor tabs', async () => {
    render(<ConfigPage profileName="Default" />);

    // Initially visual tab is active
    expect(screen.getByText('Visual Editor')).toHaveClass('bg-primary-500');

    // Click code editor tab
    await userEvent.click(screen.getByText('Code Editor'));

    // Monaco editor should render
    await waitFor(() => {
      expect(screen.getByRole('textbox')).toBeInTheDocument();
    });
  });

  it('should prevent save when validation errors exist', async () => {
    render(<ConfigPage profileName="Default" />);

    // Switch to code editor
    await userEvent.click(screen.getByText('Code Editor'));

    // Type invalid code
    const editor = screen.getByRole('textbox');
    await userEvent.type(editor, 'invalid syntax here');

    // Wait for validation
    await waitFor(() => {
      expect(screen.getByText(/validation error/i)).toBeInTheDocument();
    });

    // Save button should be disabled
    expect(screen.getByText(/save/i)).toBeDisabled();
  });
});
```

### 6.3 E2E Tests (Playwright)

```typescript
// keyrx_ui_v2/e2e/dashboard.spec.ts
import { test, expect } from '@playwright/test';

test('dashboard displays real-time updates', async ({ page }) => {
  await page.goto('http://localhost:9867/dashboard');

  // Wait for connection
  await expect(page.locator('text=Connected to daemon')).toBeVisible();

  // State indicators should be present
  await expect(page.locator('text=Daemon State')).toBeVisible();

  // Metrics chart should render
  await expect(page.locator('text=Latency Metrics')).toBeVisible();

  // Event timeline should be empty initially
  const timeline = page.locator('[data-testid="event-timeline"]');
  await expect(timeline).toBeVisible();
});
```

### 6.4 Accessibility Tests

```typescript
// keyrx_ui_v2/tests/a11y/config-page.test.tsx
import { render } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { ConfigPage } from '@/pages/ConfigPage';

expect.extend(toHaveNoViolations);

test('ConfigPage has no accessibility violations', async () => {
  const { container } = render(<ConfigPage profileName="Default" />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});
```

### 6.5 Visual Regression Tests

```typescript
// keyrx_ui_v2/e2e/visual/dashboard.spec.ts
import { test, expect } from '@playwright/test';

test('dashboard matches screenshot', async ({ page }) => {
  await page.goto('http://localhost:9867/dashboard');
  await page.waitForSelector('text=Connected to daemon');

  await expect(page).toHaveScreenshot('dashboard-connected.png', {
    fullPage: true,
    animations: 'disabled',
  });
});
```

## 7. Deployment and Build Pipeline

### 7.1 CI/CD Workflow

```yaml
# .github/workflows/ui-tests.yml
name: UI Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: cargo install wasm-pack

      - name: Build WASM
        run: |
          cd keyrx_core
          wasm-pack build --target web --out-dir ../keyrx_ui_v2/src/wasm/pkg --release

      - name: Install dependencies
        run: |
          cd keyrx_ui_v2
          npm ci

      - name: Run tests
        run: |
          cd keyrx_ui_v2
          npm test -- --coverage

      - name: Run E2E tests
        run: |
          cd keyrx_ui_v2
          npx playwright install
          npm run test:e2e

      - name: Build UI
        run: |
          cd keyrx_ui_v2
          npm run build

      - name: Verify bundle size
        run: |
          cd keyrx_ui_v2/dist
          SIZE=$(du -sb . | cut -f1)
          MAX_SIZE=512000  # 500KB
          if [ $SIZE -gt $MAX_SIZE ]; then
            echo "Bundle size $SIZE exceeds limit $MAX_SIZE"
            exit 1
          fi
```

## 8. Migration Strategy

### 8.1 Phased Rollout

**Phase 0: Backend RPC Implementation (Week 1)**
- Implement Rust RPC types and handlers
- Add WebSocket endpoint with RPC router
- Test with curl/wscat
- No UI changes yet

**Phase 1: Frontend RPC Client (Week 2)**
- Implement TypeScript RPC types
- Create useUnifiedApi hook
- Test with mock WebSocket server
- Parallel to backend work

**Phase 2: Migrate Existing Pages (Week 3-4)**
- Convert ProfilesPage to use RPC
- Convert DevicesPage to use RPC
- Remove old REST API calls
- Keep existing UI components

**Phase 3: Monaco + Dashboard (Week 5)**
- Integrate Monaco editor with tabs
- Build real-time dashboard
- WASM validation integration

**Phase 4: Polish + Testing (Week 6)**
- Accessibility audit
- Performance optimization
- Visual regression tests
- Documentation

### 8.2 Rollback Plan

**If RPC API fails:**
- Keep old REST endpoints alongside RPC
- Use feature flag to switch between APIs
- Monitor error rates in production
- Gradual rollout to subset of users first

**Compatibility layer:**
```typescript
// Adapter that supports both REST and RPC
export function createApiClient(useRpc: boolean) {
  if (useRpc) {
    return new RpcClient(useUnifiedApi());
  } else {
    return new RestClient();  // Old implementation
  }
}
```

## 9. Success Metrics

### 9.1 Technical Metrics

- ✅ All tests pass (unit, integration, E2E, visual, accessibility)
- ✅ Code coverage ≥ 80% overall
- ✅ Bundle size < 500KB initial load (gzipped)
- ✅ Monaco editor chunk < 2MB
- ✅ WASM module < 1MB
- ✅ Lighthouse score ≥ 90

### 9.2 Performance Metrics

- ✅ WebSocket connection < 500ms
- ✅ Dashboard update latency < 100ms
- ✅ WASM validation < 200ms for typical config
- ✅ Monaco initialization < 1 second
- ✅ Page load (FCP) < 2 seconds

### 9.3 User Experience Metrics

- ✅ Configuration change time < 5 seconds (edit → save → deploy)
- ✅ WASM simulation matches daemon behavior byte-for-byte
- ✅ Validation errors caught before deployment
- ✅ All features accessible via keyboard navigation
- ✅ Mobile-responsive on 3 breakpoints (375px, 768px, 1024px)
