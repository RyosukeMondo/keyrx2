//! Macro recorder endpoints.

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

use super::error::ApiError;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/macros/start-recording", post(start_macro_recording))
        .route("/macros/stop-recording", post(stop_macro_recording))
        .route("/macros/recorded-events", get(get_recorded_events))
        .route("/macros/clear", post(clear_recorded_events))
}

/// POST /api/macros/start-recording - Start recording macro
async fn start_macro_recording(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    state
        .macro_recorder
        .start_recording()
        .map_err(ApiError::BadRequest)?;

    Ok(Json(json!({
        "success": true,
        "message": "Recording started"
    })))
}

/// POST /api/macros/stop-recording - Stop recording macro
async fn stop_macro_recording(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    state
        .macro_recorder
        .stop_recording()
        .map_err(ApiError::BadRequest)?;

    let event_count = state.macro_recorder.event_count();

    Ok(Json(json!({
        "success": true,
        "message": "Recording stopped",
        "event_count": event_count
    })))
}

/// GET /api/macros/recorded-events - Get recorded events
async fn get_recorded_events(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let events = state
        .macro_recorder
        .get_recorded_events()
        .map_err(ApiError::InternalError)?;

    let recording = state.macro_recorder.is_recording();

    Ok(Json(json!({
        "success": true,
        "recording": recording,
        "event_count": events.len(),
        "events": events
    })))
}

/// POST /api/macros/clear - Clear recorded events
async fn clear_recorded_events(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    state
        .macro_recorder
        .clear_events()
        .map_err(ApiError::InternalError)?;

    Ok(Json(json!({
        "success": true,
        "message": "Events cleared"
    })))
}
