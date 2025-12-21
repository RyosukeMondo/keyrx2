//! Rhai DSL parser with prefix validation
//!
//! This module provides parsing and validation for the KeyRx configuration DSL.
//! It enforces strict prefix rules:
//! - VK_ prefix: Virtual keys (output mappings)
//! - MD_ prefix: Custom modifiers (00-FE hex range)
//! - LK_ prefix: Custom locks (00-FE hex range)

// Allow dead_code for parser components that will be used by CLI in task 18
// Functions are called from Rhai closures which the compiler cannot detect
#![allow(dead_code)]

use crate::error::ParseError;
use keyrx_core::config::{
    BaseKeyMapping, Condition, ConditionItem, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode,
    KeyMapping, Metadata, Version,
};
use rhai::{Array, Engine, EvalAltResult, Scope};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// List of physical modifier names that cannot be used with MD_ prefix.
const PHYSICAL_MODIFIERS: &[&str] = &[
    "LShift", "RShift", "LCtrl", "RCtrl", "LAlt", "RAlt", "LMeta", "RMeta",
];

/// Parser state shared across Rhai custom functions
#[derive(Debug, Clone)]
struct ParserState {
    /// The configuration being built
    devices: Vec<DeviceConfig>,
    /// Current device being configured (None if no device() block active)
    current_device: Option<DeviceConfig>,
}

impl ParserState {
    fn new() -> Self {
        Self {
            devices: Vec::new(),
            current_device: None,
        }
    }
}

/// Main parser for Rhai DSL
pub struct Parser {
    engine: Engine,
    state: Arc<Mutex<ParserState>>,
}

impl Parser {
    /// Create a new parser with all custom functions registered
    pub fn new() -> Self {
        let mut engine = Engine::new();
        let state = Arc::new(Mutex::new(ParserState::new()));

        // Set resource limits
        engine.set_max_operations(10_000);
        engine.set_max_expr_depths(100, 100);
        engine.set_max_call_levels(100);

        // Register custom functions
        Self::register_map_function(&mut engine, Arc::clone(&state));
        Self::register_tap_hold_function(&mut engine, Arc::clone(&state));
        Self::register_when_functions(&mut engine, Arc::clone(&state));
        Self::register_modifier_functions(&mut engine, Arc::clone(&state));
        Self::register_device_function(&mut engine, Arc::clone(&state));

        Self { engine, state }
    }

    /// Parse a Rhai script file and return the resulting ConfigRoot
    pub fn parse_script(&mut self, path: &Path) -> Result<ConfigRoot, ParseError> {
        // Read the script file
        let script = std::fs::read_to_string(path).map_err(|_e| ParseError::ImportNotFound {
            path: path.to_path_buf(),
            searched_paths: vec![path.to_path_buf()],
        })?;

        // Set up timeout
        let start_time = SystemTime::now();
        let timeout = Duration::from_secs(10);

        // Execute the script
        let mut scope = Scope::new();
        self.engine
            .run_with_scope(&mut scope, &script)
            .map_err(|e| Self::convert_rhai_error(e, path))?;

        // Check timeout
        if SystemTime::now()
            .duration_since(start_time)
            .unwrap_or(Duration::ZERO)
            > timeout
        {
            return Err(ParseError::ResourceLimitExceeded {
                limit_type: "execution timeout (10 seconds)".to_string(),
            });
        }

        // Build final ConfigRoot
        let state = self.state.lock().unwrap();

        // If there's a current device, it wasn't properly finalized
        if state.current_device.is_some() {
            return Err(ParseError::SyntaxError {
                file: path.to_path_buf(),
                line: 0,
                column: 0,
                message: "Unclosed device() block".to_string(),
            });
        }

        let metadata = Metadata {
            compilation_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            source_hash: "TODO".to_string(), // Will be computed by serializer
        };

        Ok(ConfigRoot {
            version: Version::current(),
            devices: state.devices.clone(),
            metadata,
        })
    }

    /// Convert Rhai error to ParseError
    fn convert_rhai_error(err: Box<EvalAltResult>, path: &Path) -> ParseError {
        let position = err.position();
        ParseError::SyntaxError {
            file: path.to_path_buf(),
            line: position.line().unwrap_or(0),
            column: position.position().unwrap_or(0),
            message: err.to_string(),
        }
    }

    /// Register the map() function
    fn register_map_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
        let state_clone = Arc::clone(&state);
        engine.register_fn(
            "map",
            move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
                let mut state = state_clone.lock().unwrap();

                // Parse 'from' key (no prefix expected for physical keys)
                let from_key =
                    parse_key_name(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;

                // Parse 'to' with prefix validation
                let mapping = if to.starts_with("VK_") {
                    // Virtual key output
                    let to_key =
                        parse_virtual_key(to).map_err(|e| format!("Invalid 'to' key: {}", e))?;
                    BaseKeyMapping::Simple {
                        from: from_key,
                        to: to_key,
                    }
                } else if to.starts_with("MD_") {
                    // Custom modifier
                    let modifier_id =
                        parse_modifier_id(to).map_err(|e| format!("Invalid modifier ID: {}", e))?;
                    BaseKeyMapping::Modifier {
                        from: from_key,
                        modifier_id,
                    }
                } else if to.starts_with("LK_") {
                    // Custom lock
                    let lock_id =
                        parse_lock_id(to).map_err(|e| format!("Invalid lock ID: {}", e))?;
                    BaseKeyMapping::Lock {
                        from: from_key,
                        lock_id,
                    }
                } else {
                    return Err(format!(
                        "Output must have VK_, MD_, or LK_ prefix: {} → use VK_{} for virtual key",
                        to, to
                    )
                    .into());
                };

                // Add to current device
                if let Some(ref mut device) = state.current_device {
                    device.mappings.push(KeyMapping::Base(mapping));
                    Ok(())
                } else {
                    Err("map() must be called inside a device() block".into())
                }
            },
        );
    }

    /// Register the tap_hold() function
    fn register_tap_hold_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
        let state_clone = Arc::clone(&state);
        engine.register_fn(
            "tap_hold",
            move |key: &str,
                  tap: &str,
                  hold: &str,
                  threshold_ms: i64|
                  -> Result<(), Box<EvalAltResult>> {
                let mut state = state_clone.lock().unwrap();

                // Parse key (no prefix)
                let from_key = parse_key_name(key).map_err(|e| format!("Invalid key: {}", e))?;

                // Parse tap (must have VK_ prefix)
                if !tap.starts_with("VK_") {
                    return Err(format!(
                        "tap_hold tap parameter must have VK_ prefix, got: {}",
                        tap
                    )
                    .into());
                }
                let tap_key =
                    parse_virtual_key(tap).map_err(|e| format!("Invalid tap key: {}", e))?;

                // Parse hold (must have MD_ prefix)
                if !hold.starts_with("MD_") {
                    return Err(format!(
                        "tap_hold hold parameter must have MD_ prefix, got: {}",
                        hold
                    )
                    .into());
                }
                let hold_modifier =
                    parse_modifier_id(hold).map_err(|e| format!("Invalid hold modifier: {}", e))?;

                let mapping = BaseKeyMapping::TapHold {
                    from: from_key,
                    tap: tap_key,
                    hold_modifier,
                    threshold_ms: threshold_ms as u16,
                };

                // Add to current device
                if let Some(ref mut device) = state.current_device {
                    device.mappings.push(KeyMapping::Base(mapping));
                    Ok(())
                } else {
                    Err("tap_hold() must be called inside a device() block".into())
                }
            },
        );
    }

    /// Register when() and when_not() functions
    fn register_when_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
        // when() with single condition (string)
        let state_clone = Arc::clone(&state);
        engine.register_fn(
            "when",
            move |cond: &str, _mappings: Array| -> Result<(), Box<EvalAltResult>> {
                let condition = parse_condition_string(cond)
                    .map_err(|e| format!("Invalid condition: {}", e))?;

                // Convert mappings array to Vec<BaseKeyMapping>
                // (In practice, this would need proper implementation)
                // For now, we'll store the condition
                let mut state = state_clone.lock().unwrap();
                if let Some(ref mut device) = state.current_device {
                    // Note: This is simplified - proper implementation would parse the closure
                    device.mappings.push(KeyMapping::Conditional {
                        condition,
                        mappings: Vec::new(), // TODO: Parse from closure
                    });
                    Ok(())
                } else {
                    Err("when() must be called inside a device() block".into())
                }
            },
        );

        // when() with multiple conditions (array)
        let state_clone = Arc::clone(&state);
        engine.register_fn(
            "when",
            move |conds: Array, _mappings: Array| -> Result<(), Box<EvalAltResult>> {
                let mut condition_items = Vec::new();
                for cond_dyn in conds {
                    let cond_str = cond_dyn
                        .into_string()
                        .map_err(|_| "Condition must be a string")?;
                    let cond = parse_condition_string(&cond_str)
                        .map_err(|e| format!("Invalid condition: {}", e))?;

                    // Convert to ConditionItem
                    match cond {
                        Condition::ModifierActive(id) => {
                            condition_items.push(ConditionItem::ModifierActive(id))
                        }
                        Condition::LockActive(id) => {
                            condition_items.push(ConditionItem::LockActive(id))
                        }
                        _ => return Err("Only single modifiers/locks allowed in array".into()),
                    }
                }

                let condition = Condition::AllActive(condition_items);

                let mut state = state_clone.lock().unwrap();
                if let Some(ref mut device) = state.current_device {
                    device.mappings.push(KeyMapping::Conditional {
                        condition,
                        mappings: Vec::new(), // TODO: Parse from closure
                    });
                    Ok(())
                } else {
                    Err("when() must be called inside a device() block".into())
                }
            },
        );

        // when_not() with single condition
        let state_clone = Arc::clone(&state);
        engine.register_fn(
            "when_not",
            move |cond: &str, _mappings: Array| -> Result<(), Box<EvalAltResult>> {
                let condition = parse_condition_string(cond)
                    .map_err(|e| format!("Invalid condition: {}", e))?;

                // Convert to NotActive
                let condition_item = match condition {
                    Condition::ModifierActive(id) => ConditionItem::ModifierActive(id),
                    Condition::LockActive(id) => ConditionItem::LockActive(id),
                    _ => return Err("Only single modifiers/locks allowed in when_not".into()),
                };

                let condition = Condition::NotActive(vec![condition_item]);

                let mut state = state_clone.lock().unwrap();
                if let Some(ref mut device) = state.current_device {
                    device.mappings.push(KeyMapping::Conditional {
                        condition,
                        mappings: Vec::new(), // TODO: Parse from closure
                    });
                    Ok(())
                } else {
                    Err("when_not() must be called inside a device() block".into())
                }
            },
        );
    }

    /// Register with_shift(), with_ctrl(), with_alt(), with_mods() functions
    fn register_modifier_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
        // with_shift()
        let state_clone = Arc::clone(&state);
        engine.register_fn(
            "with_shift",
            move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
                let mut state = state_clone.lock().unwrap();

                let from_key =
                    parse_key_name(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;

                if !to.starts_with("VK_") {
                    return Err(format!("with_shift 'to' must have VK_ prefix, got: {}", to).into());
                }
                let to_key =
                    parse_virtual_key(to).map_err(|e| format!("Invalid 'to' key: {}", e))?;

                let mapping = BaseKeyMapping::ModifiedOutput {
                    from: from_key,
                    to: to_key,
                    shift: true,
                    ctrl: false,
                    alt: false,
                    win: false,
                };

                if let Some(ref mut device) = state.current_device {
                    device.mappings.push(KeyMapping::Base(mapping));
                    Ok(())
                } else {
                    Err("with_shift() must be called inside a device() block".into())
                }
            },
        );

        // with_ctrl()
        let state_clone = Arc::clone(&state);
        engine.register_fn(
            "with_ctrl",
            move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
                let mut state = state_clone.lock().unwrap();

                let from_key =
                    parse_key_name(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;

                if !to.starts_with("VK_") {
                    return Err(format!("with_ctrl 'to' must have VK_ prefix, got: {}", to).into());
                }
                let to_key =
                    parse_virtual_key(to).map_err(|e| format!("Invalid 'to' key: {}", e))?;

                let mapping = BaseKeyMapping::ModifiedOutput {
                    from: from_key,
                    to: to_key,
                    shift: false,
                    ctrl: true,
                    alt: false,
                    win: false,
                };

                if let Some(ref mut device) = state.current_device {
                    device.mappings.push(KeyMapping::Base(mapping));
                    Ok(())
                } else {
                    Err("with_ctrl() must be called inside a device() block".into())
                }
            },
        );

        // with_alt()
        let state_clone = Arc::clone(&state);
        engine.register_fn(
            "with_alt",
            move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
                let mut state = state_clone.lock().unwrap();

                let from_key =
                    parse_key_name(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;

                if !to.starts_with("VK_") {
                    return Err(format!("with_alt 'to' must have VK_ prefix, got: {}", to).into());
                }
                let to_key =
                    parse_virtual_key(to).map_err(|e| format!("Invalid 'to' key: {}", e))?;

                let mapping = BaseKeyMapping::ModifiedOutput {
                    from: from_key,
                    to: to_key,
                    shift: false,
                    ctrl: false,
                    alt: true,
                    win: false,
                };

                if let Some(ref mut device) = state.current_device {
                    device.mappings.push(KeyMapping::Base(mapping));
                    Ok(())
                } else {
                    Err("with_alt() must be called inside a device() block".into())
                }
            },
        );

        // with_mods()
        let state_clone = Arc::clone(&state);
        engine.register_fn(
            "with_mods",
            move |from: &str,
                  to: &str,
                  shift: bool,
                  ctrl: bool,
                  alt: bool,
                  win: bool|
                  -> Result<(), Box<EvalAltResult>> {
                let mut state = state_clone.lock().unwrap();

                let from_key =
                    parse_key_name(from).map_err(|e| format!("Invalid 'from' key: {}", e))?;

                if !to.starts_with("VK_") {
                    return Err(format!("with_mods 'to' must have VK_ prefix, got: {}", to).into());
                }
                let to_key =
                    parse_virtual_key(to).map_err(|e| format!("Invalid 'to' key: {}", e))?;

                let mapping = BaseKeyMapping::ModifiedOutput {
                    from: from_key,
                    to: to_key,
                    shift,
                    ctrl,
                    alt,
                    win,
                };

                if let Some(ref mut device) = state.current_device {
                    device.mappings.push(KeyMapping::Base(mapping));
                    Ok(())
                } else {
                    Err("with_mods() must be called inside a device() block".into())
                }
            },
        );
    }

    /// Register the device() function
    fn register_device_function(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
        // device() function starts a new device configuration
        let state_clone_start = Arc::clone(&state);
        engine.register_fn(
            "device_start",
            move |pattern: &str| -> Result<(), Box<EvalAltResult>> {
                let mut state = state_clone_start.lock().unwrap();

                // Finalize previous device if any
                if let Some(device) = state.current_device.take() {
                    state.devices.push(device);
                }

                // Start new device
                state.current_device = Some(DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: pattern.to_string(),
                    },
                    mappings: Vec::new(),
                });

                Ok(())
            },
        );

        // device_end() finalizes the current device
        let state_clone_end = Arc::clone(&state);
        engine.register_fn("device_end", move || -> Result<(), Box<EvalAltResult>> {
            let mut state = state_clone_end.lock().unwrap();

            if let Some(device) = state.current_device.take() {
                state.devices.push(device);
                Ok(())
            } else {
                Err("device_end() called without matching device_start()".into())
            }
        });
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses a virtual key string with VK_ prefix.
///
/// # Arguments
/// * `s` - Input string (e.g., "VK_A", "VK_Enter")
///
/// # Returns
/// * `Ok(KeyCode)` - The parsed key code
/// * `Err(ParseError)` - If the prefix is missing or the key name is invalid
///
/// # Examples
/// ```
/// # use keyrx_compiler::parser::parse_virtual_key;
/// # use keyrx_core::config::KeyCode;
/// let key = parse_virtual_key("VK_A").unwrap();
/// assert_eq!(key, KeyCode::A);
/// ```
pub fn parse_virtual_key(s: &str) -> Result<KeyCode, ParseError> {
    // Check for VK_ prefix
    if !s.starts_with("VK_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "virtual key".to_string(),
        });
    }

    // Extract key name after prefix
    let key_name = &s[3..];

    // Parse key name to KeyCode
    parse_key_name(key_name)
}

/// Parses a modifier ID string with MD_ prefix.
///
/// # Arguments
/// * `s` - Input string (e.g., "MD_00", "MD_FE")
///
/// # Returns
/// * `Ok(u8)` - The parsed modifier ID (0-254)
/// * `Err(ParseError)` - If the prefix is missing, format is invalid, or ID is out of range
///
/// # Errors
/// * `MissingPrefix` - If MD_ prefix is missing
/// * `PhysicalModifierInMD` - If a physical modifier name is used (e.g., "MD_LShift")
/// * `ModifierIdOutOfRange` - If the ID is > 0xFE (254)
/// * `InvalidPrefix` - If the format is invalid
///
/// # Examples
/// ```
/// # use keyrx_compiler::parser::parse_modifier_id;
/// let id = parse_modifier_id("MD_00").unwrap();
/// assert_eq!(id, 0);
///
/// let id = parse_modifier_id("MD_FE").unwrap();
/// assert_eq!(id, 254);
///
/// // Physical modifiers are rejected
/// assert!(parse_modifier_id("MD_LShift").is_err());
/// ```
pub fn parse_modifier_id(s: &str) -> Result<u8, ParseError> {
    // Check for MD_ prefix
    if !s.starts_with("MD_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "custom modifier".to_string(),
        });
    }

    // Extract ID part after prefix
    let id_part = &s[3..];

    // Check if it's a physical modifier name (not allowed)
    if PHYSICAL_MODIFIERS.contains(&id_part) {
        return Err(ParseError::PhysicalModifierInMD {
            name: id_part.to_string(),
        });
    }

    // Parse hex ID
    let id = u16::from_str_radix(id_part, 16).map_err(|_| ParseError::InvalidPrefix {
        expected: "MD_XX (hex, 00-FE)".to_string(),
        got: s.to_string(),
        context: "custom modifier ID".to_string(),
    })?;

    // Validate range (00-FE, max 254)
    if id > 0xFE {
        return Err(ParseError::ModifierIdOutOfRange { got: id, max: 0xFE });
    }

    Ok(id as u8)
}

/// Parses a lock ID string with LK_ prefix.
///
/// # Arguments
/// * `s` - Input string (e.g., "LK_00", "LK_FE")
///
/// # Returns
/// * `Ok(u8)` - The parsed lock ID (0-254)
/// * `Err(ParseError)` - If the prefix is missing, format is invalid, or ID is out of range
///
/// # Errors
/// * `MissingPrefix` - If LK_ prefix is missing
/// * `LockIdOutOfRange` - If the ID is > 0xFE (254)
/// * `InvalidPrefix` - If the format is invalid
///
/// # Examples
/// ```
/// # use keyrx_compiler::parser::parse_lock_id;
/// let id = parse_lock_id("LK_00").unwrap();
/// assert_eq!(id, 0);
///
/// let id = parse_lock_id("LK_FE").unwrap();
/// assert_eq!(id, 254);
/// ```
pub fn parse_lock_id(s: &str) -> Result<u8, ParseError> {
    // Check for LK_ prefix
    if !s.starts_with("LK_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "custom lock".to_string(),
        });
    }

    // Extract ID part after prefix
    let id_part = &s[3..];

    // Parse hex ID
    let id = u16::from_str_radix(id_part, 16).map_err(|_| ParseError::InvalidPrefix {
        expected: "LK_XX (hex, 00-FE)".to_string(),
        got: s.to_string(),
        context: "custom lock ID".to_string(),
    })?;

    // Validate range (00-FE, max 254)
    if id > 0xFE {
        return Err(ParseError::LockIdOutOfRange { got: id, max: 0xFE });
    }

    Ok(id as u8)
}

/// Parses a condition string (MD_XX or LK_XX) into a Condition variant.
///
/// # Arguments
/// * `s` - Input string (e.g., "MD_00", "LK_01")
///
/// # Returns
/// * `Ok(Condition)` - The parsed condition
/// * `Err(ParseError)` - If the format is invalid
///
/// # Examples
/// ```
/// # use keyrx_compiler::parser::parse_condition_string;
/// # use keyrx_core::config::Condition;
/// let cond = parse_condition_string("MD_00").unwrap();
/// assert_eq!(cond, Condition::ModifierActive(0));
///
/// let cond = parse_condition_string("LK_01").unwrap();
/// assert_eq!(cond, Condition::LockActive(1));
/// ```
pub fn parse_condition_string(s: &str) -> Result<Condition, ParseError> {
    if s.starts_with("MD_") {
        let id = parse_modifier_id(s)?;
        Ok(Condition::ModifierActive(id))
    } else if s.starts_with("LK_") {
        let id = parse_lock_id(s)?;
        Ok(Condition::LockActive(id))
    } else {
        Err(ParseError::InvalidPrefix {
            expected: "MD_XX or LK_XX".to_string(),
            got: s.to_string(),
            context: "condition".to_string(),
        })
    }
}

/// Parses a key name (without prefix) to KeyCode.
///
/// # Arguments
/// * `name` - Key name (e.g., "A", "Enter", "Space")
///
/// # Returns
/// * `Ok(KeyCode)` - The parsed key code
/// * `Err(ParseError)` - If the key name is unknown
fn parse_key_name(name: &str) -> Result<KeyCode, ParseError> {
    match name {
        // Letters
        "A" => Ok(KeyCode::A),
        "B" => Ok(KeyCode::B),
        "C" => Ok(KeyCode::C),
        "D" => Ok(KeyCode::D),
        "E" => Ok(KeyCode::E),
        "F" => Ok(KeyCode::F),
        "G" => Ok(KeyCode::G),
        "H" => Ok(KeyCode::H),
        "I" => Ok(KeyCode::I),
        "J" => Ok(KeyCode::J),
        "K" => Ok(KeyCode::K),
        "L" => Ok(KeyCode::L),
        "M" => Ok(KeyCode::M),
        "N" => Ok(KeyCode::N),
        "O" => Ok(KeyCode::O),
        "P" => Ok(KeyCode::P),
        "Q" => Ok(KeyCode::Q),
        "R" => Ok(KeyCode::R),
        "S" => Ok(KeyCode::S),
        "T" => Ok(KeyCode::T),
        "U" => Ok(KeyCode::U),
        "V" => Ok(KeyCode::V),
        "W" => Ok(KeyCode::W),
        "X" => Ok(KeyCode::X),
        "Y" => Ok(KeyCode::Y),
        "Z" => Ok(KeyCode::Z),

        // Numbers
        "Num0" | "0" => Ok(KeyCode::Num0),
        "Num1" | "1" => Ok(KeyCode::Num1),
        "Num2" | "2" => Ok(KeyCode::Num2),
        "Num3" | "3" => Ok(KeyCode::Num3),
        "Num4" | "4" => Ok(KeyCode::Num4),
        "Num5" | "5" => Ok(KeyCode::Num5),
        "Num6" | "6" => Ok(KeyCode::Num6),
        "Num7" | "7" => Ok(KeyCode::Num7),
        "Num8" | "8" => Ok(KeyCode::Num8),
        "Num9" | "9" => Ok(KeyCode::Num9),

        // Function keys
        "F1" => Ok(KeyCode::F1),
        "F2" => Ok(KeyCode::F2),
        "F3" => Ok(KeyCode::F3),
        "F4" => Ok(KeyCode::F4),
        "F5" => Ok(KeyCode::F5),
        "F6" => Ok(KeyCode::F6),
        "F7" => Ok(KeyCode::F7),
        "F8" => Ok(KeyCode::F8),
        "F9" => Ok(KeyCode::F9),
        "F10" => Ok(KeyCode::F10),
        "F11" => Ok(KeyCode::F11),
        "F12" => Ok(KeyCode::F12),

        // Physical modifiers
        "LShift" => Ok(KeyCode::LShift),
        "RShift" => Ok(KeyCode::RShift),
        "LCtrl" => Ok(KeyCode::LCtrl),
        "RCtrl" => Ok(KeyCode::RCtrl),
        "LAlt" => Ok(KeyCode::LAlt),
        "RAlt" => Ok(KeyCode::RAlt),
        "LMeta" => Ok(KeyCode::LMeta),
        "RMeta" => Ok(KeyCode::RMeta),

        // Special keys
        "Escape" | "Esc" => Ok(KeyCode::Escape),
        "Enter" | "Return" => Ok(KeyCode::Enter),
        "Backspace" => Ok(KeyCode::Backspace),
        "Tab" => Ok(KeyCode::Tab),
        "Space" => Ok(KeyCode::Space),
        "CapsLock" => Ok(KeyCode::CapsLock),
        "NumLock" => Ok(KeyCode::NumLock),
        "ScrollLock" => Ok(KeyCode::ScrollLock),
        "PrintScreen" => Ok(KeyCode::PrintScreen),
        "Pause" => Ok(KeyCode::Pause),
        "Insert" => Ok(KeyCode::Insert),
        "Delete" | "Del" => Ok(KeyCode::Delete),
        "Home" => Ok(KeyCode::Home),
        "End" => Ok(KeyCode::End),
        "PageUp" => Ok(KeyCode::PageUp),
        "PageDown" => Ok(KeyCode::PageDown),

        // Arrow keys
        "Left" => Ok(KeyCode::Left),
        "Right" => Ok(KeyCode::Right),
        "Up" => Ok(KeyCode::Up),
        "Down" => Ok(KeyCode::Down),

        // Additional special keys
        "LeftBracket" => Ok(KeyCode::LeftBracket),
        "RightBracket" => Ok(KeyCode::RightBracket),
        "Backslash" => Ok(KeyCode::Backslash),
        "Semicolon" => Ok(KeyCode::Semicolon),
        "Quote" => Ok(KeyCode::Quote),
        "Comma" => Ok(KeyCode::Comma),
        "Period" => Ok(KeyCode::Period),
        "Slash" => Ok(KeyCode::Slash),
        "Grave" => Ok(KeyCode::Grave),
        "Minus" => Ok(KeyCode::Minus),
        "Equal" => Ok(KeyCode::Equal),

        // Numpad keys
        "Numpad0" => Ok(KeyCode::Numpad0),
        "Numpad1" => Ok(KeyCode::Numpad1),
        "Numpad2" => Ok(KeyCode::Numpad2),
        "Numpad3" => Ok(KeyCode::Numpad3),
        "Numpad4" => Ok(KeyCode::Numpad4),
        "Numpad5" => Ok(KeyCode::Numpad5),
        "Numpad6" => Ok(KeyCode::Numpad6),
        "Numpad7" => Ok(KeyCode::Numpad7),
        "Numpad8" => Ok(KeyCode::Numpad8),
        "Numpad9" => Ok(KeyCode::Numpad9),
        "NumpadMultiply" => Ok(KeyCode::NumpadMultiply),
        "NumpadAdd" => Ok(KeyCode::NumpadAdd),
        "NumpadSubtract" => Ok(KeyCode::NumpadSubtract),
        "NumpadDivide" => Ok(KeyCode::NumpadDivide),
        "NumpadDecimal" => Ok(KeyCode::NumpadDecimal),
        "NumpadEnter" => Ok(KeyCode::NumpadEnter),

        // Unknown key name
        _ => Err(ParseError::InvalidPrefix {
            expected: "valid key name".to_string(),
            got: format!("VK_{}", name),
            context: "virtual key parsing".to_string(),
        }),
    }
}

#[cfg(test)]
mod prefix_tests {
    use super::*;

    #[test]
    fn test_parse_virtual_key_accepts_valid() {
        assert_eq!(parse_virtual_key("VK_A").unwrap(), KeyCode::A);
        assert_eq!(parse_virtual_key("VK_Enter").unwrap(), KeyCode::Enter);
        assert_eq!(parse_virtual_key("VK_Space").unwrap(), KeyCode::Space);
        assert_eq!(parse_virtual_key("VK_Escape").unwrap(), KeyCode::Escape);
        assert_eq!(parse_virtual_key("VK_F1").unwrap(), KeyCode::F1);
        assert_eq!(parse_virtual_key("VK_0").unwrap(), KeyCode::Num0);
    }

    #[test]
    fn test_parse_virtual_key_rejects_missing_prefix() {
        let result = parse_virtual_key("A");
        assert!(result.is_err());
        match result {
            Err(ParseError::MissingPrefix { key, context }) => {
                assert_eq!(key, "A");
                assert_eq!(context, "virtual key");
            }
            _ => panic!("Expected MissingPrefix error"),
        }
    }

    #[test]
    fn test_parse_virtual_key_rejects_unknown_key() {
        let result = parse_virtual_key("VK_Unknown");
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidPrefix { .. }) => {}
            _ => panic!("Expected InvalidPrefix error"),
        }
    }

    #[test]
    fn test_parse_modifier_id_accepts_valid() {
        assert_eq!(parse_modifier_id("MD_00").unwrap(), 0);
        assert_eq!(parse_modifier_id("MD_01").unwrap(), 1);
        assert_eq!(parse_modifier_id("MD_FE").unwrap(), 254);
        assert_eq!(parse_modifier_id("MD_0A").unwrap(), 10);
        assert_eq!(parse_modifier_id("MD_FF").is_err(), true); // FF is out of range
    }

    #[test]
    fn test_parse_modifier_id_rejects_physical_names() {
        let physical_names = ["MD_LShift", "MD_RShift", "MD_LCtrl", "MD_RCtrl"];
        for name in &physical_names {
            let result = parse_modifier_id(name);
            assert!(result.is_err());
            match result {
                Err(ParseError::PhysicalModifierInMD { .. }) => {}
                _ => panic!("Expected PhysicalModifierInMD error for {}", name),
            }
        }
    }

    #[test]
    fn test_parse_modifier_id_rejects_out_of_range() {
        let result = parse_modifier_id("MD_FF");
        assert!(result.is_err());
        match result {
            Err(ParseError::ModifierIdOutOfRange { got, max }) => {
                assert_eq!(got, 255);
                assert_eq!(max, 254);
            }
            _ => panic!("Expected ModifierIdOutOfRange error"),
        }
    }

    #[test]
    fn test_parse_modifier_id_rejects_missing_prefix() {
        let result = parse_modifier_id("00");
        assert!(result.is_err());
        match result {
            Err(ParseError::MissingPrefix { key, context }) => {
                assert_eq!(key, "00");
                assert_eq!(context, "custom modifier");
            }
            _ => panic!("Expected MissingPrefix error"),
        }
    }

    #[test]
    fn test_parse_modifier_id_rejects_invalid_format() {
        let result = parse_modifier_id("MD_XY");
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidPrefix { .. }) => {}
            _ => panic!("Expected InvalidPrefix error"),
        }
    }

    #[test]
    fn test_parse_lock_id_accepts_valid() {
        assert_eq!(parse_lock_id("LK_00").unwrap(), 0);
        assert_eq!(parse_lock_id("LK_01").unwrap(), 1);
        assert_eq!(parse_lock_id("LK_FE").unwrap(), 254);
        assert_eq!(parse_lock_id("LK_0A").unwrap(), 10);
    }

    #[test]
    fn test_parse_lock_id_rejects_out_of_range() {
        let result = parse_lock_id("LK_FF");
        assert!(result.is_err());
        match result {
            Err(ParseError::LockIdOutOfRange { got, max }) => {
                assert_eq!(got, 255);
                assert_eq!(max, 254);
            }
            _ => panic!("Expected LockIdOutOfRange error"),
        }
    }

    #[test]
    fn test_parse_lock_id_rejects_missing_prefix() {
        let result = parse_lock_id("00");
        assert!(result.is_err());
        match result {
            Err(ParseError::MissingPrefix { key, context }) => {
                assert_eq!(key, "00");
                assert_eq!(context, "custom lock");
            }
            _ => panic!("Expected MissingPrefix error"),
        }
    }

    #[test]
    fn test_parse_condition_string_handles_modifiers() {
        let cond = parse_condition_string("MD_00").unwrap();
        assert_eq!(cond, Condition::ModifierActive(0));

        let cond = parse_condition_string("MD_0A").unwrap();
        assert_eq!(cond, Condition::ModifierActive(10));
    }

    #[test]
    fn test_parse_condition_string_handles_locks() {
        let cond = parse_condition_string("LK_00").unwrap();
        assert_eq!(cond, Condition::LockActive(0));

        let cond = parse_condition_string("LK_0B").unwrap();
        assert_eq!(cond, Condition::LockActive(11));
    }

    #[test]
    fn test_parse_condition_string_rejects_invalid_prefix() {
        let result = parse_condition_string("VK_A");
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidPrefix {
                expected,
                got,
                context,
            }) => {
                assert_eq!(expected, "MD_XX or LK_XX");
                assert_eq!(got, "VK_A");
                assert_eq!(context, "condition");
            }
            _ => panic!("Expected InvalidPrefix error"),
        }
    }
}
