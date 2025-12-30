//! Helper functions for configuration CLI commands.
//!
//! This module contains utility functions for parsing macro sequences,
//! analyzing Rhai configuration files, and performing diffs.

use crate::config::rhai_generator::MacroStep;

/// Parse a macro sequence string into individual steps.
///
/// # Arguments
///
/// * `sequence` - Comma-separated sequence string (e.g., "press:VK_A,wait:50,release:VK_A")
///
/// # Returns
///
/// Vector of macro steps or error message
///
/// # Examples
///
/// ```
/// use keyrx_daemon::cli::config_helpers::parse_macro_sequence;
///
/// let steps = parse_macro_sequence("press:VK_A,wait:50,release:VK_A").unwrap();
/// assert_eq!(steps.len(), 3);
/// ```
pub fn parse_macro_sequence(sequence: &str) -> Result<Vec<MacroStep>, String> {
    let mut steps = Vec::new();
    for part in sequence.split(',') {
        let part = part.trim();
        if let Some(key) = part.strip_prefix("press:") {
            steps.push(MacroStep::Press(key.to_string()));
        } else if let Some(key) = part.strip_prefix("release:") {
            steps.push(MacroStep::Release(key.to_string()));
        } else if let Some(ms) = part.strip_prefix("wait:") {
            let ms = ms
                .parse::<u16>()
                .map_err(|_| format!("Invalid wait time: {}", ms))?;
            steps.push(MacroStep::Wait(ms));
        } else {
            return Err(format!("Invalid macro step: {}", part));
        }
    }
    Ok(steps)
}

/// Find a key mapping in Rhai configuration content.
///
/// Performs a line-based search to locate a key mapping in the specified layer.
///
/// # Arguments
///
/// * `content` - Rhai file content
/// * `key` - Key to search for
/// * `layer` - Layer name to search in
///
/// # Returns
///
/// The mapping line if found, None otherwise
pub fn find_key_mapping(content: &str, key: &str, layer: &str) -> Option<String> {
    let mut current_layer = "base";
    let mut in_when_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("when_start(") {
            in_when_block = true;
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    current_layer = &trimmed[start + 1..start + 1 + end];
                }
            }
        } else if trimmed.starts_with("when_end()") {
            in_when_block = false;
            current_layer = "base";
        } else if (current_layer == layer || (layer == "base" && !in_when_block))
            && (trimmed.starts_with("map(") || trimmed.starts_with("tap_hold("))
        {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    let first_key = &trimmed[start + 1..start + 1 + end];
                    if first_key == key {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Extract device ID from Rhai configuration content.
///
/// # Arguments
///
/// * `content` - Rhai file content
///
/// # Returns
///
/// Device ID string if found, None otherwise
pub fn extract_device_id(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("device_start(") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

/// Extract list of layer names from Rhai configuration content.
///
/// # Arguments
///
/// * `content` - Rhai file content
///
/// # Returns
///
/// Vector of layer names
pub fn extract_layer_list(content: &str) -> Vec<String> {
    let mut layers = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("when_start(") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    layers.push(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    layers
}

/// Count total number of key mappings in Rhai configuration content.
///
/// # Arguments
///
/// * `content` - Rhai file content
///
/// # Returns
///
/// Number of mappings (map() and tap_hold() calls)
pub fn count_mappings(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("map(") || trimmed.starts_with("tap_hold(")
        })
        .count()
}

/// Compute line-by-line differences between two configuration files.
///
/// This is a simple diff implementation, not a full unified diff algorithm.
///
/// # Arguments
///
/// * `content1` - First file content
/// * `content2` - Second file content
///
/// # Returns
///
/// Vector of difference descriptions
pub fn compute_diff(content1: &str, content2: &str) -> Vec<String> {
    let lines1: Vec<&str> = content1.lines().collect();
    let lines2: Vec<&str> = content2.lines().collect();

    let mut differences = Vec::new();

    // Simple line-by-line comparison (not a true diff algorithm)
    let max_len = lines1.len().max(lines2.len());
    for i in 0..max_len {
        let line1 = lines1.get(i).copied().unwrap_or("");
        let line2 = lines2.get(i).copied().unwrap_or("");

        if line1 != line2 {
            if !line1.is_empty() && !line2.is_empty() {
                differences.push(format!("Line {}: '{}' -> '{}'", i + 1, line1, line2));
            } else if line2.is_empty() {
                differences.push(format!("- Line {}: '{}'", i + 1, line1));
            } else {
                differences.push(format!("+ Line {}: '{}'", i + 1, line2));
            }
        }
    }

    differences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_macro_sequence_valid() {
        let result = parse_macro_sequence("press:VK_A,wait:50,release:VK_A");
        assert!(result.is_ok());
        let steps = result.unwrap();
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn test_parse_macro_sequence_invalid_step() {
        let result = parse_macro_sequence("invalid:VK_A");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_macro_sequence_invalid_wait_time() {
        let result = parse_macro_sequence("wait:abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_key_mapping_found() {
        let content = r#"
            map("VK_A", "VK_B")
            when_start("layer1")
            map("VK_C", "VK_D")
            when_end()
        "#;
        let result = find_key_mapping(content, "VK_A", "base");
        assert!(result.is_some());
        assert!(result.unwrap().contains("VK_A"));
    }

    #[test]
    fn test_find_key_mapping_not_found() {
        let content = r#"
            map("VK_A", "VK_B")
        "#;
        let result = find_key_mapping(content, "VK_Z", "base");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_device_id_found() {
        let content = r#"
            device_start("keyboard-123")
        "#;
        let result = extract_device_id(content);
        assert_eq!(result, Some("keyboard-123".to_string()));
    }

    #[test]
    fn test_extract_device_id_not_found() {
        let content = "map(\"VK_A\", \"VK_B\")";
        let result = extract_device_id(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_layer_list() {
        let content = r#"
            when_start("layer1")
            when_end()
            when_start("layer2")
            when_end()
        "#;
        let layers = extract_layer_list(content);
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0], "layer1");
        assert_eq!(layers[1], "layer2");
    }

    #[test]
    fn test_count_mappings() {
        let content = r#"
            map("VK_A", "VK_B")
            tap_hold("VK_Space", "VK_Space", "MD_00", 200)
            map("VK_C", "VK_D")
        "#;
        let count = count_mappings(content);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_compute_diff_no_differences() {
        let content1 = "line1\nline2\nline3";
        let content2 = "line1\nline2\nline3";
        let diff = compute_diff(content1, content2);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_compute_diff_with_differences() {
        let content1 = "line1\nline2\nline3";
        let content2 = "line1\nmodified\nline3";
        let diff = compute_diff(content1, content2);
        assert_eq!(diff.len(), 1);
        assert!(diff[0].contains("line2"));
        assert!(diff[0].contains("modified"));
    }

    #[test]
    fn test_compute_diff_added_lines() {
        let content1 = "line1\nline2";
        let content2 = "line1\nline2\nline3";
        let diff = compute_diff(content1, content2);
        assert_eq!(diff.len(), 1);
        assert!(diff[0].contains("+ Line 3"));
    }

    #[test]
    fn test_compute_diff_removed_lines() {
        let content1 = "line1\nline2\nline3";
        let content2 = "line1\nline2";
        let diff = compute_diff(content1, content2);
        assert_eq!(diff.len(), 1);
        assert!(diff[0].contains("- Line 3"));
    }
}
