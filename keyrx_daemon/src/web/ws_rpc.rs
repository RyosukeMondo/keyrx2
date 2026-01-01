//! WebSocket RPC handler for bidirectional client-server communication.
//!
//! This module implements the WebSocket RPC protocol for handling queries, commands,
//! and subscriptions from web clients. It uses the message types defined in rpc_types.rs
//! and provides request/response correlation via UUID tracking.

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::web::rpc_types::{ClientMessage, RpcError, ServerMessage};

pub fn create_router() -> Router {
    Router::new().route("/", get(websocket_handler))
}

/// WebSocket upgrade handler
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_websocket)
}

/// Handle WebSocket RPC connection
async fn handle_websocket(mut socket: WebSocket) {
    log::info!("WebSocket RPC client connected");

    // Send Connected handshake immediately
    let connected = ServerMessage::Connected {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64,
    };

    let connected_json = match serde_json::to_string(&connected) {
        Ok(json) => json,
        Err(e) => {
            log::error!("Failed to serialize Connected message: {}", e);
            return;
        }
    };

    if socket.send(Message::Text(connected_json)).await.is_err() {
        log::warn!("Failed to send Connected handshake");
        return;
    }

    // Queue for outgoing messages
    let outgoing_queue = Arc::new(Mutex::new(VecDeque::<ServerMessage>::new()));

    // Main message processing loop
    loop {
        // Check if there are outgoing messages to send
        let next_message = {
            let mut queue = outgoing_queue.lock().await;
            queue.pop_front()
        };

        if let Some(response) = next_message {
            let response_json = match serde_json::to_string(&response) {
                Ok(json) => json,
                Err(e) => {
                    log::error!("Failed to serialize response: {}", e);
                    continue;
                }
            };

            if socket.send(Message::Text(response_json)).await.is_err() {
                log::info!("WebSocket RPC client disconnected (send failed)");
                break;
            }
        }

        // Process incoming messages
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => {
                log::debug!("Received WebSocket RPC message: {}", text);

                // Parse message
                let client_msg: ClientMessage = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::warn!("Failed to parse message: {}", e);
                        // Send parse error response (can't correlate to request ID)
                        let error_response = ServerMessage::Response {
                            id: String::new(),
                            result: None,
                            error: Some(RpcError::parse_error(format!("Invalid JSON: {}", e))),
                        };
                        let mut queue = outgoing_queue.lock().await;
                        queue.push_back(error_response);
                        continue;
                    }
                };

                // Process message and queue response
                process_client_message(client_msg, Arc::clone(&outgoing_queue)).await;
            }
            Some(Ok(Message::Close(_))) => {
                log::info!("WebSocket RPC client closed connection");
                break;
            }
            Some(Ok(_)) => {
                // Ignore binary/ping/pong messages
            }
            Some(Err(e)) => {
                log::warn!("WebSocket error: {}", e);
                break;
            }
            None => {
                log::info!("WebSocket RPC client disconnected");
                break;
            }
        }
    }

    log::info!("WebSocket RPC connection closed");
}

/// Process a client message and queue the appropriate response
async fn process_client_message(
    msg: ClientMessage,
    outgoing_queue: Arc<Mutex<VecDeque<ServerMessage>>>,
) {
    let response = match msg {
        ClientMessage::Query { id, method, params } => handle_query(id, method, params).await,
        ClientMessage::Command { id, method, params } => handle_command(id, method, params).await,
        ClientMessage::Subscribe { id, channel } => handle_subscribe(id, channel).await,
        ClientMessage::Unsubscribe { id, channel } => handle_unsubscribe(id, channel).await,
    };

    // Queue the response
    let mut queue = outgoing_queue.lock().await;
    queue.push_back(response);
}

/// Handle query request (read-only operations)
async fn handle_query(id: String, method: String, _params: Value) -> ServerMessage {
    // For now, return METHOD_NOT_FOUND for all queries
    // Actual query handlers will be implemented in later tasks (Phase 1, Tasks 3-6)
    ServerMessage::Response {
        id,
        result: None,
        error: Some(RpcError::method_not_found(method)),
    }
}

/// Handle command request (state-modifying operations)
async fn handle_command(id: String, method: String, _params: Value) -> ServerMessage {
    // For now, return METHOD_NOT_FOUND for all commands
    // Actual command handlers will be implemented in later tasks (Phase 1, Tasks 3-6)
    ServerMessage::Response {
        id,
        result: None,
        error: Some(RpcError::method_not_found(method)),
    }
}

/// Handle subscription request
async fn handle_subscribe(id: String, channel: String) -> ServerMessage {
    // For now, return success but don't actually subscribe
    // Actual subscription logic will be implemented in Task 7 (Subscription Channel Manager)
    log::debug!("Subscribe request for channel: {}", channel);

    ServerMessage::Response {
        id,
        result: Some(serde_json::json!({
            "subscribed": true,
            "channel": channel
        })),
        error: None,
    }
}

/// Handle unsubscribe request
async fn handle_unsubscribe(id: String, channel: String) -> ServerMessage {
    // For now, return success
    // Actual unsubscribe logic will be implemented in Task 7 (Subscription Channel Manager)
    log::debug!("Unsubscribe request for channel: {}", channel);

    ServerMessage::Response {
        id,
        result: Some(serde_json::json!({
            "unsubscribed": true,
            "channel": channel
        })),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_router() {
        let router = create_router();
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
