//! Profile management endpoints.

use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use super::error::ApiError;
use crate::config::profile_manager::{ProfileError, ProfileManager, ProfileTemplate};
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/profiles", get(list_profiles).post(create_profile))
        .route("/profiles/active", get(get_active_profile))
        .route("/profiles/:name/activate", post(activate_profile))
        .route(
            "/profiles/:name/config",
            get(get_profile_config).put(set_profile_config),
        )
        .route("/profiles/:name", delete(delete_profile))
        .route("/profiles/:name/duplicate", post(duplicate_profile))
        .route("/profiles/:name/rename", put(rename_profile))
}

#[derive(Serialize)]
struct ProfileResponse {
    name: String,
    rhai_path: String,
    krx_path: String,
    modified_at: u64,
    layer_count: usize,
    is_active: bool,
}

#[derive(Serialize)]
struct ProfilesListResponse {
    profiles: Vec<ProfileResponse>,
}

/// GET /api/profiles - List all profiles
async fn list_profiles() -> Result<Json<ProfilesListResponse>, ApiError> {
    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    pm.scan_profiles()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let active_profile = query_active_profile();

    let profiles: Vec<ProfileResponse> = pm
        .list()
        .iter()
        .map(|meta| ProfileResponse {
            name: meta.name.clone(),
            rhai_path: meta.rhai_path.display().to_string(),
            krx_path: meta.krx_path.display().to_string(),
            modified_at: meta
                .modified_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            layer_count: meta.layer_count,
            is_active: active_profile.as_ref() == Some(&meta.name),
        })
        .collect();

    Ok(Json(ProfilesListResponse { profiles }))
}

/// POST /api/profiles - Create new profile
#[derive(Deserialize)]
struct CreateProfileRequest {
    name: String,
    template: String, // "blank" or "qmk-layers"
}

async fn create_profile(
    Json(payload): Json<CreateProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    let template = match payload.template.as_str() {
        "blank" => ProfileTemplate::Blank,
        "qmk-layers" => ProfileTemplate::QmkLayers,
        _ => return Err(ApiError::BadRequest("Invalid template".to_string())),
    };

    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let metadata = pm
        .create(&payload.name, template)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": metadata.name,
            "rhai_path": metadata.rhai_path.display().to_string(),
            "krx_path": metadata.krx_path.display().to_string(),
        }
    })))
}

/// POST /api/profiles/:name/activate - Activate profile
async fn activate_profile(Path(name): Path<String>) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let result = pm
        .activate(&name)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    if !result.success {
        return Err(ApiError::InternalError(
            result.error.unwrap_or_else(|| "Unknown error".to_string()),
        ));
    }

    Ok(Json(json!({
        "success": true,
        "compile_time_ms": result.compile_time_ms,
        "reload_time_ms": result.reload_time_ms,
    })))
}

/// DELETE /api/profiles/:name - Delete profile
async fn delete_profile(Path(name): Path<String>) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    pm.delete(&name)
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

/// POST /api/profiles/:name/duplicate - Duplicate profile
#[derive(Deserialize)]
struct DuplicateProfileRequest {
    new_name: String,
}

async fn duplicate_profile(
    Path(name): Path<String>,
    Json(payload): Json<DuplicateProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let metadata = pm
        .duplicate(&name, &payload.new_name)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": metadata.name,
            "rhai_path": metadata.rhai_path.display().to_string(),
        }
    })))
}

/// PUT /api/profiles/:name/rename - Rename profile
#[derive(Deserialize)]
struct RenameProfileRequest {
    new_name: String,
}

async fn rename_profile(
    Path(name): Path<String>,
    Json(payload): Json<RenameProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Scan profiles to ensure the profile list is up to date
    pm.scan_profiles()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let metadata = pm.rename(&name, &payload.new_name).map_err(|e| match e {
        ProfileError::NotFound(_) => ApiError::NotFound(format!("Profile '{}' not found", name)),
        ProfileError::AlreadyExists(_) => {
            ApiError::BadRequest(format!("Profile '{}' already exists", payload.new_name))
        }
        ProfileError::InvalidName(_) => ApiError::BadRequest(e.to_string()),
        _ => ApiError::InternalError(e.to_string()),
    })?;

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": metadata.name,
            "rhai_path": metadata.rhai_path.display().to_string(),
            "krx_path": metadata.krx_path.display().to_string(),
        }
    })))
}

/// GET /api/profiles/active - Get active profile
async fn get_active_profile(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let active_profile = state.profile_service.get_active_profile().await;

    Ok(Json(json!({
        "active_profile": active_profile,
    })))
}

/// GET /api/profiles/:name/config - Get profile configuration
async fn get_profile_config(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let config = state
        .profile_service
        .get_profile_config(&name)
        .await
        .map_err(|e| match e {
            ProfileError::NotFound(_) => {
                ApiError::NotFound(format!("Profile '{}' not found", name))
            }
            _ => ApiError::InternalError(e.to_string()),
        })?;

    Ok(Json(json!({
        "name": name,
        "config": config,
    })))
}

/// PUT /api/profiles/:name/config - Set profile configuration
#[derive(Deserialize)]
struct SetProfileConfigRequest {
    config: String,
}

async fn set_profile_config(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<SetProfileConfigRequest>,
) -> Result<Json<Value>, ApiError> {
    state
        .profile_service
        .set_profile_config(&name, &payload.config)
        .await
        .map_err(|e| match e {
            ProfileError::NotFound(_) => {
                ApiError::NotFound(format!("Profile '{}' not found", name))
            }
            _ => ApiError::InternalError(e.to_string()),
        })?;

    Ok(Json(json!({
        "success": true,
    })))
}

/// Get config directory path
fn get_config_dir() -> Result<std::path::PathBuf, ApiError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| ApiError::InternalError("Cannot determine home directory".to_string()))?;

    Ok(std::path::PathBuf::from(home).join(".config/keyrx"))
}

/// Query active profile name
fn query_active_profile() -> Option<String> {
    use crate::ipc::{DaemonIpc, IpcRequest, IpcResponse, DEFAULT_SOCKET_PATH};

    let socket_path = std::path::PathBuf::from(DEFAULT_SOCKET_PATH);
    let mut ipc = crate::ipc::unix_socket::UnixSocketIpc::new(socket_path);

    let response = ipc.send_request(&IpcRequest::GetStatus).ok()?;

    match response {
        IpcResponse::Status {
            running: _,
            uptime_secs: _,
            active_profile,
            device_count: _,
        } => active_profile,
        _ => None,
    }
}
