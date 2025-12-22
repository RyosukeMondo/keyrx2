//! Compile subcommand handler.
//!
//! Handles the `compile` subcommand which parses Rhai scripts and compiles them
//! to binary .krx format.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use crate::error::ParseError;
use crate::error::SerializeError;
use crate::parser::Parser;
use crate::serialize::serialize;

/// Errors that can occur during the compile subcommand.
#[derive(Debug)]
pub enum CompileError {
    /// Failed to parse Rhai script.
    ParseError(ParseError),

    /// Failed to serialize configuration.
    SerializeError(SerializeError),

    /// I/O error during file operations.
    IoError(io::Error),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(err) => write!(f, "Parse error: {:?}", err),
            Self::SerializeError(err) => write!(f, "Serialization error: {:?}", err),
            Self::IoError(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<io::Error> for CompileError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<ParseError> for CompileError {
    fn from(err: ParseError) -> Self {
        Self::ParseError(err)
    }
}

impl From<SerializeError> for CompileError {
    fn from(err: SerializeError) -> Self {
        Self::SerializeError(err)
    }
}

/// Handles the compile subcommand.
///
/// # Arguments
///
/// * `input` - Path to the input .rhai script file.
/// * `output` - Path to the output .krx binary file.
///
/// # Returns
///
/// `Ok(())` on success, or `CompileError` on failure.
pub fn handle_compile(input: &Path, output: &Path) -> Result<(), CompileError> {
    eprintln!("Parsing {}...", input.display());

    // Parse the Rhai script
    let mut parser = Parser::new();
    let config = parser.parse_script(input)?;

    eprintln!("Serializing configuration...");

    // Serialize to .krx format
    let bytes = serialize(&config)?;

    eprintln!("Writing to {}...", output.display());

    // Write to output file
    fs::write(output, &bytes)?;

    // Extract hash from bytes (bytes 8-40 contain the SHA256 hash)
    let hash = &bytes[8..40];
    let hash_hex = hex::encode(hash);

    // Calculate file size
    let file_size = bytes.len();

    eprintln!("✓ Compilation successful");
    eprintln!("  Output: {}", output.display());
    eprintln!("  Size: {} bytes", file_size);
    eprintln!("  SHA256: {}", hash_hex);

    Ok(())
}
