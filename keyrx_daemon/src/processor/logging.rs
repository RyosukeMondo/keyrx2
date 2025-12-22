//! Structured JSON logging for event processing.
//!
//! Provides type-safe, structured logging in JSON format for observability.
//! All logs follow the schema:
//! `{"timestamp":"...", "level":"...", "service":"keyrx_daemon", "event_type":"...", "context":{...}}`

use std::time::{SystemTime, UNIX_EPOCH};

use keyrx_core::config::KeyCode;
use log::{debug, error, info};

/// Returns the current Unix timestamp in ISO 8601 format.
///
/// Note: This uses a simplified calendar calculation and may be slightly
/// inaccurate for edge cases (leap years, month boundaries). For production
/// use, consider using the `chrono` crate for precise timestamps.
fn current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            let nanos = d.subsec_nanos();
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                1970 + secs / 31_557_600,
                ((secs % 31_557_600) / 2_629_800) + 1,
                ((secs % 2_629_800) / 86400) + 1,
                (secs % 86400) / 3600,
                (secs % 3600) / 60,
                secs % 60,
                nanos / 1_000_000
            )
        })
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00.000Z"))
}

/// Logs when configuration is loaded and processor is initialized.
pub fn log_config_loaded(mapping_count: usize) {
    info!(
        r#"{{"timestamp":"{}","level":"INFO","service":"keyrx_daemon","event_type":"config_loaded","context":{{"mapping_count":{}}}}}"#,
        current_timestamp(),
        mapping_count
    );
}

/// Logs when a key event has been processed.
pub fn log_key_processed(input_key: KeyCode, output_keys: &[KeyCode], latency_us: u64) {
    let output_keys_json: Vec<String> = output_keys
        .iter()
        .map(|k| format!(r#""{:?}""#, k))
        .collect();

    debug!(
        r#"{{"timestamp":"{}","level":"DEBUG","service":"keyrx_daemon","event_type":"key_processed","context":{{"input_key":"{:?}","output_keys":[{}],"latency_us":{}}}}}"#,
        current_timestamp(),
        input_key,
        output_keys_json.join(","),
        latency_us
    );
}

/// Logs when modifier/lock state transitions occur.
pub fn log_state_transition(transition_context: &str) {
    debug!(
        r#"{{"timestamp":"{}","level":"DEBUG","service":"keyrx_daemon","event_type":"state_transition","context":{}}}"#,
        current_timestamp(),
        transition_context
    );
}

/// Logs platform errors (input/output device failures).
pub fn log_platform_error(error: &str, device: &str) {
    error!(
        r#"{{"timestamp":"{}","level":"ERROR","service":"keyrx_daemon","event_type":"platform_error","context":{{"error":"{}","device":"{}"}}}}"#,
        current_timestamp(),
        error,
        device
    );
}

/// Formats a state transition context for modifier activation.
pub fn format_modifier_activated(modifier_id: u8) -> String {
    format!(
        r#"{{"transition_type":"modifier_activated","modifier_id":{}}}"#,
        modifier_id
    )
}

/// Formats a state transition context for modifier deactivation.
pub fn format_modifier_deactivated(modifier_id: u8) -> String {
    format!(
        r#"{{"transition_type":"modifier_deactivated","modifier_id":{}}}"#,
        modifier_id
    )
}

/// Formats a state transition context for lock toggle.
pub fn format_lock_toggled(lock_id: u8) -> String {
    format!(
        r#"{{"transition_type":"lock_toggled","lock_id":{}}}"#,
        lock_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_format() {
        let ts = current_timestamp();
        // Should match ISO 8601 format: YYYY-MM-DDTHH:MM:SS.sssZ
        assert!(ts.contains('T'));
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 24);
    }

    #[test]
    fn test_format_modifier_activated() {
        let json = format_modifier_activated(5);
        assert!(json.contains(r#""transition_type":"modifier_activated""#));
        assert!(json.contains(r#""modifier_id":5"#));
    }

    #[test]
    fn test_format_modifier_deactivated() {
        let json = format_modifier_deactivated(10);
        assert!(json.contains(r#""transition_type":"modifier_deactivated""#));
        assert!(json.contains(r#""modifier_id":10"#));
    }

    #[test]
    fn test_format_lock_toggled() {
        let json = format_lock_toggled(3);
        assert!(json.contains(r#""transition_type":"lock_toggled""#));
        assert!(json.contains(r#""lock_id":3"#));
    }
}
