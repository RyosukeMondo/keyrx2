//! KeyRx Compiler Library
//!
//! This library provides the compilation infrastructure for KeyRx configuration files.
//! It parses Rhai DSL scripts and compiles them to binary .krx format.

use std::path::Path;

pub mod cli;
pub mod error;
pub mod import_resolver;
pub mod parser;
pub mod serialize;

// Re-export common types
pub use cli::compile::CompileError;

/// Compile a Rhai script to .krx binary format.
///
/// # Arguments
///
/// * `input` - Path to the input .rhai script file.
/// * `output` - Path to the output .krx binary file.
///
/// # Errors
///
/// Returns `CompileError` if parsing, serialization, or I/O fails.
pub fn compile_file(input: &Path, output: &Path) -> Result<(), CompileError> {
    cli::compile::handle_compile(input, output)
}
