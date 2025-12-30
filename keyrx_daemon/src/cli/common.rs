//! Common CLI output utilities.
//!
//! This module provides centralized output formatting for CLI commands,
//! supporting both JSON and human-readable text formats.

use serde::{Deserialize, Serialize};

/// JSON output for errors.
///
/// This structure is used to serialize error messages to JSON format
/// for machine-readable output.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::cli::common::ErrorOutput;
/// use serde_json;
///
/// let error = ErrorOutput {
///     success: false,
///     error: "File not found".to_string(),
///     code: 404,
/// };
/// let json = serde_json::to_string(&error).unwrap();
/// assert!(json.contains("\"success\":false"));
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ErrorOutput {
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message
    pub error: String,
    /// Error code
    pub code: u32,
}

/// Output a successful result.
///
/// When `json` is true, outputs the data as JSON to stdout.
/// When `json` is false, the caller is responsible for formatting
/// and outputting human-readable text.
///
/// # Arguments
///
/// * `data` - The data to output (must implement Serialize)
/// * `json` - Whether to output as JSON
///
/// # Examples
///
/// ```
/// use keyrx_daemon::cli::common::output_success;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Result {
///     status: String,
/// }
///
/// let result = Result { status: "ok".to_string() };
/// output_success(&result, true);  // Outputs JSON to stdout
/// ```
///
/// # Panics
///
/// Panics if serialization fails, which should only happen if the data
/// structure contains non-serializable types.
pub fn output_success<T: Serialize>(data: &T, json: bool) {
    if json {
        // Serialize to JSON and output to stdout
        match serde_json::to_string(data) {
            Ok(json_str) => println!("{}", json_str),
            Err(e) => {
                // This should never happen with proper Serialize implementations
                eprintln!("Internal error: Failed to serialize output: {}", e);
                std::process::exit(1);
            }
        }
    }
    // For non-JSON mode, caller handles formatting and output
}

/// Output an error message.
///
/// When `json` is true, outputs an ErrorOutput structure as JSON to stdout.
/// When `json` is false, outputs the error message to stderr with "Error: " prefix.
///
/// # Arguments
///
/// * `message` - The error message to display
/// * `code` - Error code (used in JSON output)
/// * `json` - Whether to output as JSON
///
/// # Examples
///
/// ```
/// use keyrx_daemon::cli::common::output_error;
///
/// output_error("Configuration file not found", 404, false);
/// // Outputs to stderr: Error: Configuration file not found
///
/// output_error("Configuration file not found", 404, true);
/// // Outputs to stdout: {"success":false,"error":"Configuration file not found","code":404}
/// ```
pub fn output_error(message: &str, code: u32, json: bool) {
    if json {
        let output = ErrorOutput {
            success: false,
            error: message.to_string(),
            code,
        };
        // Output to stdout for consistency with success outputs
        match serde_json::to_string(&output) {
            Ok(json_str) => println!("{}", json_str),
            Err(_) => {
                // Fallback to plain text if JSON serialization fails
                eprintln!("Error: {}", message);
            }
        }
    } else {
        eprintln!("Error: {}", message);
    }
}

/// Output a list of items.
///
/// When `json` is true, outputs the items as a JSON array to stdout.
/// When `json` is false, the caller is responsible for formatting
/// and outputting human-readable text.
///
/// # Arguments
///
/// * `items` - Vector of items to output (must implement Serialize)
/// * `json` - Whether to output as JSON
///
/// # Examples
///
/// ```
/// use keyrx_daemon::cli::common::output_list;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Item {
///     name: String,
///     value: i32,
/// }
///
/// let items = vec![
///     Item { name: "first".to_string(), value: 1 },
///     Item { name: "second".to_string(), value: 2 },
/// ];
/// output_list(&items, true);  // Outputs JSON array to stdout
/// ```
///
/// # Panics
///
/// Panics if serialization fails, which should only happen if the item
/// structure contains non-serializable types.
pub fn output_list<T: Serialize>(items: &[T], json: bool) {
    if json {
        // Serialize the entire list as a JSON array
        match serde_json::to_string(items) {
            Ok(json_str) => println!("{}", json_str),
            Err(e) => {
                // This should never happen with proper Serialize implementations
                eprintln!("Internal error: Failed to serialize output: {}", e);
                std::process::exit(1);
            }
        }
    }
    // For non-JSON mode, caller handles formatting and output
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize, Debug, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[derive(Serialize, Debug, PartialEq)]
    struct ComplexData {
        id: u64,
        tags: Vec<String>,
        metadata: Option<String>,
    }

    #[test]
    fn test_error_output_struct() {
        let error = ErrorOutput {
            success: false,
            error: "Test error".to_string(),
            code: 42,
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"Test error\""));
        assert!(json.contains("\"code\":42"));
    }

    #[test]
    fn test_error_output_struct_serialization() {
        let error = ErrorOutput {
            success: false,
            error: "File not found".to_string(),
            code: 404,
        };

        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ErrorOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn test_output_success_json_simple() {
        let data = TestData {
            name: "test".to_string(),
            value: 123,
        };

        // In JSON mode, output_success should produce valid JSON
        // We can't easily capture stdout in tests, but we can verify
        // the function doesn't panic
        output_success(&data, true);
    }

    #[test]
    fn test_output_success_json_complex() {
        let data = ComplexData {
            id: 999,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            metadata: Some("test metadata".to_string()),
        };

        output_success(&data, true);
    }

    #[test]
    fn test_output_success_json_with_special_characters() {
        #[derive(Serialize)]
        struct SpecialChars {
            text: String,
        }

        let data = SpecialChars {
            text: "Special: \"quotes\", \\backslash, \nnewline".to_string(),
        };

        output_success(&data, true);
    }

    #[test]
    fn test_output_success_non_json() {
        let data = TestData {
            name: "test".to_string(),
            value: 456,
        };

        // In non-JSON mode, the function should do nothing
        // (caller handles output)
        output_success(&data, false);
    }

    #[test]
    fn test_output_error_json() {
        // JSON mode should output ErrorOutput structure
        output_error("Test error message", 100, true);
    }

    #[test]
    fn test_output_error_non_json() {
        // Non-JSON mode should output to stderr
        output_error("Test error message", 200, false);
    }

    #[test]
    fn test_output_error_with_special_characters() {
        output_error("Error: \"file\" not found\nLine 2", 404, true);
        output_error("Error: \"file\" not found\nLine 2", 404, false);
    }

    #[test]
    fn test_output_error_empty_message() {
        output_error("", 0, true);
        output_error("", 0, false);
    }

    #[test]
    fn test_output_error_long_message() {
        let long_message = "a".repeat(10000);
        output_error(&long_message, 500, true);
        output_error(&long_message, 500, false);
    }

    #[test]
    fn test_output_list_json_empty() {
        let items: Vec<TestData> = vec![];
        output_list(&items, true);
    }

    #[test]
    fn test_output_list_json_single_item() {
        let items = vec![TestData {
            name: "single".to_string(),
            value: 1,
        }];
        output_list(&items, true);
    }

    #[test]
    fn test_output_list_json_multiple_items() {
        let items = vec![
            TestData {
                name: "first".to_string(),
                value: 1,
            },
            TestData {
                name: "second".to_string(),
                value: 2,
            },
            TestData {
                name: "third".to_string(),
                value: 3,
            },
        ];
        output_list(&items, true);
    }

    #[test]
    fn test_output_list_json_complex_items() {
        let items = vec![
            ComplexData {
                id: 1,
                tags: vec!["rust".to_string(), "cli".to_string()],
                metadata: Some("first item".to_string()),
            },
            ComplexData {
                id: 2,
                tags: vec![],
                metadata: None,
            },
        ];
        output_list(&items, true);
    }

    #[test]
    fn test_output_list_non_json() {
        let items = vec![TestData {
            name: "test".to_string(),
            value: 789,
        }];
        // Non-JSON mode should do nothing (caller handles output)
        output_list(&items, false);
    }

    #[test]
    fn test_output_list_large_dataset() {
        let items: Vec<TestData> = (0..1000)
            .map(|i| TestData {
                name: format!("item_{}", i),
                value: i,
            })
            .collect();
        output_list(&items, true);
    }

    #[test]
    fn test_error_output_clone() {
        let error = ErrorOutput {
            success: false,
            error: "Cloneable error".to_string(),
            code: 123,
        };
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_error_output_debug() {
        let error = ErrorOutput {
            success: false,
            error: "Debug test".to_string(),
            code: 999,
        };
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Debug test"));
        assert!(debug_str.contains("999"));
    }
}
