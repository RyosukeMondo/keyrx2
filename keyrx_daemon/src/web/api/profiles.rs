//! Profile management endpoints.

use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
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
#[serde(rename_all = "camelCase")]
struct ProfileResponse {
    name: String,
    rhai_path: String,
    krx_path: String,
    #[serde(serialize_with = "serialize_systemtime_as_rfc3339")]
    modified_at: std::time::SystemTime,
    #[serde(serialize_with = "serialize_systemtime_as_rfc3339")]
    created_at: std::time::SystemTime,
    layer_count: usize,
    #[serde(rename = "deviceCount")]
    device_count: usize,
    #[serde(rename = "keyCount")]
    key_count: usize,
    is_active: bool,
}

/// Serialize SystemTime as RFC 3339 / ISO 8601 string
fn serialize_systemtime_as_rfc3339<S>(
    time: &std::time::SystemTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    let datetime: DateTime<Utc> = (*time).into();
    datetime.to_rfc3339().serialize(serializer)
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
            modified_at: meta.modified_at,
            created_at: meta.modified_at, // Use modified_at as created_at for now
            layer_count: meta.layer_count,
            device_count: 0, // TODO: Track device count per profile
            key_count: 0,    // TODO: Parse Rhai config to count key mappings
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn test_serialize_systemtime_as_rfc3339() {
        // Create a known timestamp: 2024-01-01T00:00:00Z
        let timestamp = UNIX_EPOCH + Duration::from_secs(1704067200);

        // Serialize to JSON
        let json_value = serde_json::to_value(&ProfileResponse {
            name: "test".to_string(),
            rhai_path: "/test.rhai".to_string(),
            krx_path: "/test.krx".to_string(),
            modified_at: timestamp,
            created_at: timestamp,
            layer_count: 1,
            device_count: 0,
            key_count: 0,
            is_active: false,
        })
        .unwrap();

        // Check that modifiedAt is a string in ISO 8601 / RFC 3339 format
        let modified_at_str = json_value["modifiedAt"].as_str().unwrap();

        // Should be in format: YYYY-MM-DDTHH:MM:SS.sssZ or similar RFC 3339
        assert!(
            modified_at_str.contains('T'),
            "Timestamp should contain 'T' separator: {}",
            modified_at_str
        );
        assert!(
            modified_at_str.ends_with('Z')
                || modified_at_str.contains('+')
                || modified_at_str.contains('-'),
            "Timestamp should have timezone (Z or offset): {}",
            modified_at_str
        );

        // Verify it can be parsed back by JavaScript Date constructor
        // RFC 3339 format is guaranteed to be parseable by new Date()
        assert!(
            modified_at_str.len() >= 20, // Minimum length for ISO 8601
            "Timestamp too short: {}",
            modified_at_str
        );
    }

    #[test]
    fn test_profile_response_camel_case_fields() {
        let timestamp = UNIX_EPOCH + Duration::from_secs(1704067200);

        let response = ProfileResponse {
            name: "gaming".to_string(),
            rhai_path: "/profiles/gaming.rhai".to_string(),
            krx_path: "/profiles/gaming.krx".to_string(),
            modified_at: timestamp,
            created_at: timestamp,
            layer_count: 3,
            device_count: 2,
            key_count: 127,
            is_active: true,
        };

        let json_value = serde_json::to_value(&response).unwrap();

        // Verify camelCase field names
        assert!(
            json_value["modifiedAt"].is_string(),
            "modifiedAt should be a string"
        );
        assert!(
            json_value["createdAt"].is_string(),
            "createdAt should be a string"
        );
        assert!(
            json_value["layerCount"].is_number(),
            "layerCount should be a number"
        );
        assert!(
            json_value["deviceCount"].is_number(),
            "deviceCount should be a number"
        );
        assert!(
            json_value["keyCount"].is_number(),
            "keyCount should be a number"
        );
        assert!(
            json_value["isActive"].is_boolean(),
            "isActive should be a boolean"
        );

        // Verify snake_case fields are NOT present
        assert!(
            json_value.get("modified_at").is_none(),
            "Should not have snake_case modified_at"
        );
        assert!(
            json_value.get("created_at").is_none(),
            "Should not have snake_case created_at"
        );
        assert!(
            json_value.get("layer_count").is_none(),
            "Should not have snake_case layer_count"
        );
        assert!(
            json_value.get("is_active").is_none(),
            "Should not have snake_case is_active"
        );
    }
}
