//! WASM validation module using Rhai engine for deterministic results.
//!
//! This module provides browser-based configuration validation using the
//! embedded Rhai parser. While not using keyrx_compiler directly (to avoid
//! circular dependencies), it uses the same Rhai engine for deterministic validation.

#![cfg(feature = "wasm")]

extern crate std;

use serde::Serialize;

/// Validation error structure returned to JavaScript.
///
/// This format matches the TypeScript ValidationError interface in the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    /// Line number where the error occurred (1-indexed)
    pub line: usize,
    /// Column number where the error occurred (1-indexed)
    pub column: usize,
    /// Length of the error span in characters
    pub length: usize,
    /// Human-readable error message
    pub message: std::string::String,
}

/// Validate a Rhai configuration source using the embedded parser.
///
/// This function uses the same Rhai engine as parse_rhai_config to ensure
/// validation results are deterministic.
///
/// # Arguments
/// * `rhai_source` - Rhai DSL source code as a string
///
/// # Returns
/// * `Ok(Vec<ValidationError>)` - Vector of validation errors (empty if valid)
/// * `Err(String)` - Internal error during validation
///
/// # Example
/// ```rust
/// use keyrx_core::wasm::validation::validate_rhai_config;
///
/// let source = r#"
///   device("*") {
///     map("A", "B");
///   }
/// "#;
///
/// let errors = validate_rhai_config(source).expect("Validation should not fail");
/// assert!(errors.is_empty()); // Valid configuration
/// ```
pub fn validate_rhai_config(
    rhai_source: &str,
) -> Result<std::vec::Vec<ValidationError>, std::string::String> {
    // Validate input size (1MB limit)
    const MAX_CONFIG_SIZE: usize = 1024 * 1024;
    if rhai_source.len() > MAX_CONFIG_SIZE {
        return Ok(std::vec![ValidationError {
            line: 1,
            column: 1,
            length: 0,
            message: std::format!(
                "Configuration too large: {} bytes (max {})",
                rhai_source.len(),
                MAX_CONFIG_SIZE
            ),
        }]);
    }

    // Try to parse the configuration using the same parser as load_config
    match super::parse_rhai_config(rhai_source) {
        Ok(_) => {
            // Valid configuration - return empty error array
            Ok(std::vec::Vec::new())
        }
        Err(error_msg) => {
            // Parse error occurred - extract line/column from Rhai error
            let (line, column) = extract_line_column_from_error(&error_msg);

            let error = ValidationError {
                line,
                column,
                length: 1,
                message: error_msg,
            };

            Ok(std::vec![error])
        }
    }
}

/// Extract line and column numbers from error message string.
///
/// Error messages from Rhai typically contain position information like:
/// "Parse error: ... (line 5, column 10)"
fn extract_line_column_from_error(error_msg: &str) -> (usize, usize) {
    // Look for pattern "(line X, column Y)" or "(line X, position Y)"
    if let Some(start) = error_msg.find("(line ") {
        let after_line = &error_msg[start + 6..];
        if let Some(comma) = after_line.find(',') {
            let line_str = &after_line[..comma];
            if let Ok(line) = line_str.trim().parse::<usize>() {
                let after_comma = &after_line[comma + 1..];

                // Try "column" or "position"
                let col_prefix = if after_comma.contains("column") {
                    "column "
                } else if after_comma.contains("position") {
                    "position "
                } else {
                    return (line.max(1), 1);
                };

                if let Some(col_start) = after_comma.find(col_prefix) {
                    let after_col = &after_comma[col_start + col_prefix.len()..];
                    if let Some(end) = after_col.find(')') {
                        let col_str = &after_col[..end];
                        if let Ok(col) = col_str.trim().parse::<usize>() {
                            return (line.max(1), col.max(1));
                        }
                    }
                }

                return (line.max(1), 1);
            }
        }
    }

    // Default to line 1, column 1 if parsing fails
    (1, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let source = r#"
            device("*") {
                map("A", "B");
            }
        "#;

        let errors = validate_rhai_config(source).expect("Validation should not fail");
        assert!(errors.is_empty(), "Valid config should return no errors");
    }

    #[test]
    fn test_invalid_syntax() {
        let source = r#"
            device("*") {
                map("A", "B")  // Missing semicolon or closing brace
        "#;

        let errors = validate_rhai_config(source).expect("Validation should not fail");
        assert!(!errors.is_empty(), "Invalid syntax should return errors");
        assert!(errors[0].line > 0, "Error should have valid line number");
        assert!(
            errors[0].column > 0,
            "Error should have valid column number"
        );
    }

    #[test]
    fn test_config_too_large() {
        let source = "x".repeat(2 * 1024 * 1024); // 2MB

        let errors = validate_rhai_config(&source).expect("Validation should not fail");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("too large"));
    }

    #[test]
    fn test_error_line_column_extraction() {
        let error_msg = "Parse error: Unexpected token (line 5, column 10)";
        let (line, column) = extract_line_column_from_error(error_msg);
        assert_eq!(line, 5);
        assert_eq!(column, 10);
    }

    #[test]
    fn test_error_extraction_with_position() {
        let error_msg = "Syntax error (line 3, position 15)";
        let (line, column) = extract_line_column_from_error(error_msg);
        assert_eq!(line, 3);
        assert_eq!(column, 15);
    }

    #[test]
    fn test_error_extraction_fallback() {
        let error_msg = "Generic error with no position";
        let (line, column) = extract_line_column_from_error(error_msg);
        assert_eq!(line, 1);
        assert_eq!(column, 1);
    }

    #[test]
    fn test_deterministic_validation() {
        let source = r#"
            device("*") {
                map("A", "B");
                map("C", "D");
            }
        "#;

        // Run validation multiple times
        let errors1 = validate_rhai_config(source).expect("Validation should not fail");
        let errors2 = validate_rhai_config(source).expect("Validation should not fail");
        let errors3 = validate_rhai_config(source).expect("Validation should not fail");

        // Results should be identical
        assert_eq!(errors1.len(), errors2.len());
        assert_eq!(errors2.len(), errors3.len());
    }
}
