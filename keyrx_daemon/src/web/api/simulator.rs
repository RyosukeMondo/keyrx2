//! Simulator endpoints.

use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use super::error::ApiError;
use crate::config::simulation_engine::{BuiltinScenario, EventSequence, SimulatedEvent};
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/simulator/events", post(simulate_events))
        .route("/simulator/reset", post(reset_simulator))
}

#[derive(Deserialize)]
struct SimulateEventsRequest {
    /// Optional scenario name (e.g., "tap-hold-under-threshold")
    scenario: Option<String>,
    /// Optional custom event sequence
    events: Option<Vec<SimulatedEvent>>,
    /// Seed for deterministic behavior
    seed: Option<u64>,
}

#[derive(Serialize)]
struct SimulateEventsResponse {
    success: bool,
    /// Number of events processed
    event_count: usize,
    /// List of output events generated
    outputs: Vec<String>,
    /// Execution time in microseconds
    duration_us: u64,
}

/// POST /api/simulator/events - Simulate events
async fn simulate_events(
    Json(payload): Json<SimulateEventsRequest>,
) -> Result<Json<SimulateEventsResponse>, ApiError> {
    // Determine event sequence
    let sequence = if let Some(scenario_name) = payload.scenario {
        // Use built-in scenario
        let scenario = match scenario_name.as_str() {
            "tap-hold-under-threshold" => BuiltinScenario::TapHoldUnderThreshold,
            "tap-hold-over-threshold" => BuiltinScenario::TapHoldOverThreshold,
            "permissive-hold" => BuiltinScenario::PermissiveHold,
            "cross-device-modifiers" => BuiltinScenario::CrossDeviceModifiers,
            "macro-sequence" => BuiltinScenario::MacroSequence,
            _ => {
                return Err(ApiError::BadRequest(format!(
                    "Unknown scenario: {}. Available: tap-hold-under-threshold, tap-hold-over-threshold, permissive-hold, cross-device-modifiers, macro-sequence",
                    scenario_name
                )))
            }
        };
        scenario.generate_events()
    } else if let Some(events) = payload.events {
        // Use custom event sequence
        EventSequence {
            events,
            seed: payload.seed.unwrap_or(0),
        }
    } else {
        return Err(ApiError::BadRequest(
            "Must provide either 'scenario' or 'events'".to_string(),
        ));
    };

    // Note: Actual simulation would require running the events through
    // the keyrx_core processor. For now, we just return the input events
    // as a demonstration. Full implementation would need:
    // 1. Load the active profile's .krx file
    // 2. Create a processor instance
    // 3. Feed events through the processor
    // 4. Collect output events

    Ok(Json(SimulateEventsResponse {
        success: true,
        event_count: sequence.events.len(),
        outputs: sequence
            .events
            .iter()
            .map(|e| format!("{:?} {} at {}μs", e.event_type, e.key, e.timestamp_us))
            .collect(),
        duration_us: sequence.events.last().map(|e| e.timestamp_us).unwrap_or(0),
    }))
}

/// POST /api/simulator/reset - Reset simulator
async fn reset_simulator() -> Result<Json<Value>, ApiError> {
    // Simulator state is not persistent, so there's nothing to reset
    // This endpoint exists for API completeness
    Ok(Json(json!({
        "success": true,
        "message": "Simulator state is ephemeral (no persistent state to reset)"
    })))
}
