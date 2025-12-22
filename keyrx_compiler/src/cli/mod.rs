//! CLI subcommand handlers.
//!
//! This module contains the implementation of all CLI subcommands:
//! - `compile`: Compile Rhai scripts to .krx binary format
//! - `verify`: Verify .krx binary file integrity
//! - `hash`: Extract and verify SHA256 hash from .krx files
//! - `parse`: Parse Rhai scripts and display configuration structure

pub mod compile;
pub mod hash;
pub mod parse;
pub mod verify;

// Re-export handler functions for easy access
#[allow(unused_imports)]
pub use compile::handle_compile;
#[allow(unused_imports)]
pub use hash::handle_hash;
#[allow(unused_imports)]
pub use parse::handle_parse;
#[allow(unused_imports)]
pub use verify::handle_verify;

// Re-export error types for external use
#[allow(unused_imports)]
pub use compile::CompileError;
#[allow(unused_imports)]
pub use hash::HashError;
#[allow(unused_imports)]
pub use parse::ParseCommandError;
#[allow(unused_imports)]
pub use verify::VerifyError;
