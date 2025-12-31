//! Layout management endpoints.

use axum::{extract::Path, routing::get, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;

use super::error::ApiError;
use crate::config::layout_manager::LayoutManager;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/layouts", get(list_layouts))
        .route("/layouts/:name", get(get_layout))
}

/// GET /api/layouts - List keyboard layouts
async fn list_layouts() -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let lm = LayoutManager::new(config_dir.join("layouts"))
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let layouts: Vec<String> = lm.list().iter().map(|layout| layout.name.clone()).collect();

    Ok(Json(json!({
        "layouts": layouts
    })))
}

/// GET /api/layouts/:name - Get layout KLE JSON
async fn get_layout(Path(name): Path<String>) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let lm = LayoutManager::new(config_dir.join("layouts"))
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let layout = lm
        .get(&name)
        .ok_or_else(|| ApiError::NotFound(format!("Layout '{}' not found", name)))?;

    Ok(Json(layout.kle_json.clone()))
}

/// Get config directory path
fn get_config_dir() -> Result<std::path::PathBuf, ApiError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| ApiError::InternalError("Cannot determine home directory".to_string()))?;

    Ok(std::path::PathBuf::from(home).join(".config/keyrx"))
}
