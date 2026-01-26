//! Parser error types for Rhai DSL parsing.
//!
//! This module defines error types that can be used by both the compiler
//! and WASM parser, avoiding the need for std::path types in WASM.

use alloc::string::String;
use alloc::vec::Vec;

/// Error type for parsing operations.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Missing required prefix (VK_, MD_, LK_)
    MissingPrefix { key: String, context: String },
    /// Invalid prefix format
    InvalidPrefix {
        expected: String,
        got: String,
        context: String,
    },
    /// Physical modifier used where custom modifier expected
    PhysicalModifierInMD { name: String },
    /// Modifier ID out of range
    ModifierIdOutOfRange { got: u16, max: u16 },
    /// Lock ID out of range
    LockIdOutOfRange { got: u16, max: u16 },
    /// Unknown key name
    UnknownKey {
        name: String,
        suggestions: Vec<String>,
    },
    /// Syntax error in Rhai script
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },
    /// Generic error message
    Other(String),
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::MissingPrefix { key, context } => {
                write!(f, "{} '{}' is missing required prefix", context, key)
            }
            ParseError::InvalidPrefix {
                expected,
                got,
                context,
            } => {
                write!(
                    f,
                    "Invalid {} prefix: expected {}, got '{}'",
                    context, expected, got
                )
            }
            ParseError::PhysicalModifierInMD { name } => {
                write!(
                    f,
                    "Physical modifier '{}' cannot be used as MD_ identifier",
                    name
                )
            }
            ParseError::ModifierIdOutOfRange { got, max } => {
                write!(f, "Modifier ID {} exceeds maximum value {}", got, max)
            }
            ParseError::LockIdOutOfRange { got, max } => {
                write!(f, "Lock ID {} exceeds maximum value {}", got, max)
            }
            ParseError::UnknownKey { name, suggestions } => {
                let mut msg = alloc::format!("Unknown key name: '{}'", name);
                if !suggestions.is_empty() {
                    msg.push_str("\n\nDid you mean one of these?\n");
                    for suggestion in suggestions {
                        msg.push_str(&alloc::format!("  - {}\n", suggestion));
                    }
                }
                write!(f, "{}", msg)
            }
            ParseError::SyntaxError {
                line,
                column,
                message,
            } => {
                write!(f, "Line {}, column {}: {}", line, column, message)
            }
            ParseError::Other(msg) => write!(f, "{}", msg),
        }
    }
}
