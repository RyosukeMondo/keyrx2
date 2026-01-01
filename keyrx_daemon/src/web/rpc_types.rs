//! RPC message types for WebSocket communication
//!
//! This module defines the type-safe message protocol for WebSocket RPC
//! communication between the daemon and frontend. It uses serde's tag-based
//! enum serialization for type discrimination and follows JSON-RPC 2.0
//! error code conventions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// JSON-RPC 2.0 standard error codes
/// Parse error - Invalid JSON was received by the server
pub const PARSE_ERROR: i32 = -32700;
/// Invalid request - The JSON sent is not a valid Request object
pub const INVALID_REQUEST: i32 = -32600;
/// Method not found - The method does not exist / is not available
pub const METHOD_NOT_FOUND: i32 = -32601;
/// Invalid params - Invalid method parameter(s)
pub const INVALID_PARAMS: i32 = -32602;
/// Internal error - Internal JSON-RPC error
pub const INTERNAL_ERROR: i32 = -32603;

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Query request - read-only operation that returns data
    #[serde(rename = "query")]
    Query {
        /// Unique identifier for request/response correlation
        id: String,
        /// Method name to invoke
        method: String,
        /// Optional parameters for the method
        #[serde(default)]
        params: Value,
    },
    /// Command request - operation that modifies state
    #[serde(rename = "command")]
    Command {
        /// Unique identifier for request/response correlation
        id: String,
        /// Method name to invoke
        method: String,
        /// Optional parameters for the method
        #[serde(default)]
        params: Value,
    },
    /// Subscribe to a channel for real-time updates
    #[serde(rename = "subscribe")]
    Subscribe {
        /// Unique identifier for request/response correlation
        id: String,
        /// Channel name to subscribe to
        channel: String,
    },
    /// Unsubscribe from a channel
    #[serde(rename = "unsubscribe")]
    Unsubscribe {
        /// Unique identifier for request/response correlation
        id: String,
        /// Channel name to unsubscribe from
        channel: String,
    },
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Response to a query or command
    #[serde(rename = "response")]
    Response {
        /// Request ID this response corresponds to
        id: String,
        /// Result data (success) or None if error
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<Value>,
        /// Error information (failure) or None if success
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<RpcError>,
    },
    /// Event broadcast to subscribed clients
    #[serde(rename = "event")]
    Event {
        /// Channel this event was published on
        channel: String,
        /// Event data
        data: Value,
    },
    /// Initial handshake message sent on connection
    #[serde(rename = "connected")]
    Connected {
        /// Protocol version
        version: String,
        /// Server timestamp in microseconds
        timestamp: u64,
    },
}

/// RPC error structure following JSON-RPC 2.0 conventions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcError {
    /// Numeric error code
    pub code: i32,
    /// Human-readable error message
    pub message: String,
    /// Optional additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    /// Creates a new RpcError with the given code and message
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Creates a new RpcError with additional data
    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Creates a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(PARSE_ERROR, message)
    }

    /// Creates an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(INVALID_REQUEST, message)
    }

    /// Creates a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::new(
            METHOD_NOT_FOUND,
            format!("Method not found: {}", method.into()),
        )
    }

    /// Creates an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(INVALID_PARAMS, message)
    }

    /// Creates an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(INTERNAL_ERROR, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_query_message_serialization() {
        let msg = ClientMessage::Query {
            id: "test-123".to_string(),
            method: "get_profiles".to_string(),
            params: json!({}),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg, deserialized);
        assert!(serialized.contains(r#""type":"query""#));
    }

    #[test]
    fn test_command_message_serialization() {
        let msg = ClientMessage::Command {
            id: "cmd-456".to_string(),
            method: "activate_profile".to_string(),
            params: json!({"name": "Default"}),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg, deserialized);
        assert!(serialized.contains(r#""type":"command""#));
    }

    #[test]
    fn test_subscribe_message_serialization() {
        let msg = ClientMessage::Subscribe {
            id: "sub-789".to_string(),
            channel: "daemon-state".to_string(),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg, deserialized);
        assert!(serialized.contains(r#""type":"subscribe""#));
    }

    #[test]
    fn test_unsubscribe_message_serialization() {
        let msg = ClientMessage::Unsubscribe {
            id: "unsub-101".to_string(),
            channel: "events".to_string(),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg, deserialized);
        assert!(serialized.contains(r#""type":"unsubscribe""#));
    }

    #[test]
    fn test_response_success_serialization() {
        let msg = ServerMessage::Response {
            id: "test-123".to_string(),
            result: Some(json!({"profiles": ["Default", "Gaming"]})),
            error: None,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ServerMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg, deserialized);
        assert!(serialized.contains(r#""type":"response""#));
        assert!(!serialized.contains("error"));
    }

    #[test]
    fn test_response_error_serialization() {
        let msg = ServerMessage::Response {
            id: "test-456".to_string(),
            result: None,
            error: Some(RpcError::method_not_found("unknown_method")),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ServerMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg, deserialized);
        assert!(serialized.contains(r#""type":"response""#));
        assert!(!serialized.contains("result"));
    }

    #[test]
    fn test_event_message_serialization() {
        let msg = ServerMessage::Event {
            channel: "daemon-state".to_string(),
            data: json!({"layer": 0, "modifiers": [], "locks": []}),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ServerMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg, deserialized);
        assert!(serialized.contains(r#""type":"event""#));
    }

    #[test]
    fn test_connected_message_serialization() {
        let msg = ServerMessage::Connected {
            version: "1.0.0".to_string(),
            timestamp: 1234567890,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ServerMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg, deserialized);
        assert!(serialized.contains(r#""type":"connected""#));
    }

    #[test]
    fn test_rpc_error_codes() {
        assert_eq!(PARSE_ERROR, -32700);
        assert_eq!(INVALID_REQUEST, -32600);
        assert_eq!(METHOD_NOT_FOUND, -32601);
        assert_eq!(INVALID_PARAMS, -32602);
        assert_eq!(INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_rpc_error_constructors() {
        let err = RpcError::parse_error("Invalid JSON");
        assert_eq!(err.code, PARSE_ERROR);
        assert_eq!(err.message, "Invalid JSON");
        assert_eq!(err.data, None);

        let err = RpcError::method_not_found("test_method");
        assert_eq!(err.code, METHOD_NOT_FOUND);
        assert!(err.message.contains("test_method"));

        let err = RpcError::with_data(
            INTERNAL_ERROR,
            "Test error",
            json!({"details": "More info"}),
        );
        assert_eq!(err.code, INTERNAL_ERROR);
        assert!(err.data.is_some());
    }

    #[test]
    fn test_round_trip_serialization() {
        // Test all message types for round-trip serialization
        let messages = vec![
            ClientMessage::Query {
                id: "1".to_string(),
                method: "test".to_string(),
                params: json!(null),
            },
            ClientMessage::Command {
                id: "2".to_string(),
                method: "test".to_string(),
                params: json!({"key": "value"}),
            },
            ClientMessage::Subscribe {
                id: "3".to_string(),
                channel: "test".to_string(),
            },
            ClientMessage::Unsubscribe {
                id: "4".to_string(),
                channel: "test".to_string(),
            },
        ];

        for msg in messages {
            let json = serde_json::to_string(&msg).unwrap();
            let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
            assert_eq!(msg, deserialized);
        }
    }
}
