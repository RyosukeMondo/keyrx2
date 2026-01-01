//! Config RPC method handlers.
//!
//! This module implements all configuration-related RPC methods for WebSocket communication.
//! Each method accepts parameters as serde_json::Value, validates them, and delegates
//! to the ConfigService for business logic execution.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::rhai_generator::KeyAction;
use crate::services::ConfigService;
use crate::web::rpc_types::{RpcError, INTERNAL_ERROR, INVALID_PARAMS};

/// Parameters for get_config query
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GetConfigParams {
    // No parameters needed
}

/// Parameters for update_config command
#[derive(Debug, Deserialize)]
struct UpdateConfigParams {
    code: String,
}

/// Parameters for set_key_mapping command
#[derive(Debug, Deserialize)]
struct SetKeyMappingParams {
    layer: String,
    key: String,
    action_type: String, // "simple", "tap_hold", "macro"
    // For simple remap
    output: Option<String>,
    // For tap-hold
    tap: Option<String>,
    hold: Option<String>,
    threshold_ms: Option<u16>,
    // For macros
    #[allow(dead_code)]
    macro_sequence: Option<String>,
}

/// Parameters for delete_key_mapping command
#[derive(Debug, Deserialize)]
struct DeleteKeyMappingParams {
    layer: String,
    key: String,
}

/// Parameters for get_layers query
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GetLayersParams {
    // No parameters needed
}

/// Config information returned by get_config
#[derive(Debug, Serialize)]
struct ConfigRpcInfo {
    code: String,
    hash: String,
    profile: String,
}

/// Layer information returned by get_layers
#[derive(Debug, Serialize)]
struct LayerRpcInfo {
    id: String,
    mapping_count: usize,
}

/// Get current configuration
pub async fn get_config(config_service: &ConfigService, _params: Value) -> Result<Value, RpcError> {
    let config = config_service
        .get_config()
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))?;

    let info = ConfigRpcInfo {
        code: config.code,
        hash: config.hash,
        profile: config.profile,
    };

    serde_json::to_value(&info).map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))
}

/// Update configuration
pub async fn update_config(
    config_service: &ConfigService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: UpdateConfigParams = serde_json::from_value(params)
        .map_err(|e| RpcError::new(INVALID_PARAMS, format!("Invalid parameters: {}", e)))?;

    // Enforce 1MB size limit
    const MAX_CONFIG_SIZE: usize = 1024 * 1024; // 1MB
    if params.code.len() > MAX_CONFIG_SIZE {
        return Err(RpcError::new(
            INVALID_PARAMS,
            format!(
                "Configuration too large: {} bytes (max {} bytes)",
                params.code.len(),
                MAX_CONFIG_SIZE
            ),
        ));
    }

    config_service
        .update_config(params.code)
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))?;

    Ok(serde_json::json!({ "success": true }))
}

/// Set a key mapping
pub async fn set_key_mapping(
    config_service: &ConfigService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: SetKeyMappingParams = serde_json::from_value(params)
        .map_err(|e| RpcError::new(INVALID_PARAMS, format!("Invalid parameters: {}", e)))?;

    // Parse action type
    let action = match params.action_type.as_str() {
        "simple" => {
            let output = params.output.ok_or_else(|| {
                RpcError::new(INVALID_PARAMS, "Missing 'output' field for simple remap")
            })?;
            KeyAction::SimpleRemap { output }
        }
        "tap_hold" => {
            let tap = params
                .tap
                .ok_or_else(|| RpcError::new(INVALID_PARAMS, "Missing 'tap' field for tap_hold"))?;
            let hold = params.hold.ok_or_else(|| {
                RpcError::new(INVALID_PARAMS, "Missing 'hold' field for tap_hold")
            })?;
            let threshold_ms = params.threshold_ms.unwrap_or(200);
            KeyAction::TapHold {
                tap,
                hold,
                threshold_ms,
            }
        }
        _ => {
            return Err(RpcError::new(
                INVALID_PARAMS,
                format!(
                    "Unsupported action type: {}. Use 'simple' or 'tap_hold'",
                    params.action_type
                ),
            ))
        }
    };

    config_service
        .set_key_mapping(params.layer, params.key, action)
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))?;

    Ok(serde_json::json!({ "success": true }))
}

/// Delete a key mapping
pub async fn delete_key_mapping(
    config_service: &ConfigService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: DeleteKeyMappingParams = serde_json::from_value(params)
        .map_err(|e| RpcError::new(INVALID_PARAMS, format!("Invalid parameters: {}", e)))?;

    config_service
        .delete_key_mapping(params.layer, params.key)
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))?;

    Ok(serde_json::json!({ "success": true }))
}

/// Get all layers
pub async fn get_layers(config_service: &ConfigService, _params: Value) -> Result<Value, RpcError> {
    let layers = config_service
        .get_layers()
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))?;

    let layers: Vec<LayerRpcInfo> = layers
        .into_iter()
        .map(|l| LayerRpcInfo {
            id: l.id,
            mapping_count: l.mapping_count,
        })
        .collect();

    serde_json::to_value(&layers).map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_get_config_params() {
        let params = json!({});
        let result: Result<GetConfigParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_update_config_params() {
        let params = json!({
            "code": "let x = 42;"
        });
        let result: Result<UpdateConfigParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().code, "let x = 42;");
    }

    #[test]
    fn test_deserialize_set_key_mapping_params_simple() {
        let params = json!({
            "layer": "base",
            "key": "A",
            "action_type": "simple",
            "output": "B"
        });
        let result: Result<SetKeyMappingParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.layer, "base");
        assert_eq!(params.key, "A");
        assert_eq!(params.action_type, "simple");
        assert_eq!(params.output.unwrap(), "B");
    }

    #[test]
    fn test_deserialize_set_key_mapping_params_tap_hold() {
        let params = json!({
            "layer": "base",
            "key": "Space",
            "action_type": "tap_hold",
            "tap": "Space",
            "hold": "Ctrl",
            "threshold_ms": 150
        });
        let result: Result<SetKeyMappingParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.action_type, "tap_hold");
        assert_eq!(params.tap.unwrap(), "Space");
        assert_eq!(params.hold.unwrap(), "Ctrl");
        assert_eq!(params.threshold_ms.unwrap(), 150);
    }

    #[test]
    fn test_deserialize_delete_key_mapping_params() {
        let params = json!({
            "layer": "base",
            "key": "A"
        });
        let result: Result<DeleteKeyMappingParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.layer, "base");
        assert_eq!(params.key, "A");
    }

    #[test]
    fn test_deserialize_get_layers_params() {
        let params = json!({});
        let result: Result<GetLayersParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_rpc_info_serialization() {
        let info = ConfigRpcInfo {
            code: "let x = 1;".to_string(),
            hash: "abc123".to_string(),
            profile: "default".to_string(),
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["code"], "let x = 1;");
        assert_eq!(json["hash"], "abc123");
        assert_eq!(json["profile"], "default");
    }

    #[test]
    fn test_layer_rpc_info_serialization() {
        let info = LayerRpcInfo {
            id: "base".to_string(),
            mapping_count: 42,
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["id"], "base");
        assert_eq!(json["mapping_count"], 42);
    }
}
