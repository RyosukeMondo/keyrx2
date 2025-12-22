//! keyrx_daemon - OS-level keyboard remapping daemon
//!
//! This binary provides the main daemon interface for keyboard remapping.
//! It intercepts keyboard events via platform-specific APIs and injects
//! remapped events back to the system.
//!
//! # Subcommands
//!
//! - `run`: Start the daemon with a .krx configuration file
//! - `list-devices`: List available input devices
//! - `validate`: Validate configuration and device matching without grabbing

mod platform;

#[cfg(feature = "web")]
mod web;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

/// KeyRx daemon for OS-level keyboard remapping.
///
/// Intercepts keyboard events and applies remapping rules defined in .krx
/// configuration files compiled by keyrx_compiler.
#[derive(Parser)]
#[command(name = "keyrx_daemon")]
#[command(version, about = "OS-level keyboard remapping daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for the daemon.
#[derive(Subcommand)]
enum Commands {
    /// Start the daemon with the specified configuration file.
    ///
    /// The daemon will intercept keyboard events from matched devices and apply
    /// the remapping rules from the .krx configuration file.
    Run {
        /// Path to the .krx configuration file compiled by keyrx_compiler.
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,

        /// Enable debug logging for verbose output.
        ///
        /// This increases log verbosity to show individual key events and
        /// detailed processing information. Useful for troubleshooting.
        #[arg(short, long)]
        debug: bool,
    },

    /// List available input devices on the system.
    ///
    /// Displays all input devices with their names, paths, and serial numbers.
    /// Keyboards are clearly marked to help with configuration setup.
    ListDevices,

    /// Validate configuration and device matching without grabbing devices.
    ///
    /// This performs a dry-run that loads the configuration, enumerates devices,
    /// and shows which devices would be matched. No devices are grabbed, so normal
    /// keyboard input continues.
    Validate {
        /// Path to the .krx configuration file to validate.
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
    },
}

/// Exit codes following Unix conventions.
mod exit_codes {
    /// Successful execution.
    pub const SUCCESS: i32 = 0;
    /// Configuration error (file not found, parse error).
    pub const CONFIG_ERROR: i32 = 1;
    /// Permission error (cannot access devices, cannot create uinput).
    #[cfg(feature = "linux")]
    pub const PERMISSION_ERROR: i32 = 2;
    /// Runtime error (device disconnected with no fallback).
    #[cfg(feature = "linux")]
    pub const RUNTIME_ERROR: i32 = 3;
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Run { config, debug } => handle_run(&config, debug),
        Commands::ListDevices => handle_list_devices(),
        Commands::Validate { config } => handle_validate(&config),
    };

    match result {
        Ok(()) => process::exit(exit_codes::SUCCESS),
        Err((code, message)) => {
            eprintln!("Error: {}", message);
            process::exit(code);
        }
    }
}

/// Handles the `run` subcommand - starts the daemon.
#[cfg(feature = "linux")]
fn handle_run(config_path: &std::path::Path, debug: bool) -> Result<(), (i32, String)> {
    use keyrx_daemon::daemon::Daemon;

    // Initialize logging
    init_logging(debug);

    log::info!(
        "Starting keyrx daemon with config: {}",
        config_path.display()
    );

    // Create and run the daemon
    let mut daemon = Daemon::new(config_path).map_err(daemon_error_to_exit)?;

    log::info!(
        "Daemon initialized with {} device(s)",
        daemon.device_count()
    );

    // Run the event loop
    daemon.run().map_err(daemon_error_to_exit)?;

    log::info!("Daemon stopped gracefully");
    Ok(())
}

#[cfg(not(feature = "linux"))]
fn handle_run(_config_path: &std::path::Path, _debug: bool) -> Result<(), (i32, String)> {
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'run' command is only available on Linux. \
         Build with --features linux to enable."
            .to_string(),
    ))
}

/// Handles the `list-devices` subcommand - lists input devices.
#[cfg(feature = "linux")]
fn handle_list_devices() -> Result<(), (i32, String)> {
    use keyrx_daemon::device_manager::enumerate_keyboards;

    // Get all keyboard devices
    let keyboards = enumerate_keyboards().map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to enumerate devices: {}", e),
        )
    })?;

    if keyboards.is_empty() {
        println!("No keyboard devices found.");
        println!();
        println!("This could mean:");
        println!("  - No keyboards are connected");
        println!("  - Permission denied to read /dev/input/event* devices");
        println!();
        println!("To fix permission issues, either:");
        println!("  - Run as root (for testing only)");
        println!("  - Add your user to the 'input' group: sudo usermod -aG input $USER");
        println!("  - Install the udev rules: see docs/LINUX_SETUP.md");
        return Ok(());
    }

    println!("Available keyboard devices:");
    println!();
    println!("{:<30} {:<25} SERIAL", "PATH", "NAME");
    println!("{}", "-".repeat(80));

    for keyboard in &keyboards {
        let serial_display = keyboard.serial.as_deref().unwrap_or("-");
        println!(
            "{:<30} {:<25} {}",
            keyboard.path.display(),
            truncate_string(&keyboard.name, 24),
            serial_display
        );
    }

    println!();
    println!("Found {} keyboard device(s).", keyboards.len());
    println!();
    println!("Tip: Use patterns in your configuration to match devices:");
    println!("  - \"*\" matches all keyboards");
    println!("  - \"USB*\" matches devices with USB in name/serial");
    println!("  - Exact name match for specific devices");

    Ok(())
}

#[cfg(not(feature = "linux"))]
fn handle_list_devices() -> Result<(), (i32, String)> {
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'list-devices' command is only available on Linux. \
         Build with --features linux to enable."
            .to_string(),
    ))
}

/// Handles the `validate` subcommand - validates config without grabbing.
#[cfg(feature = "linux")]
fn handle_validate(config_path: &std::path::Path) -> Result<(), (i32, String)> {
    use keyrx_daemon::config_loader::load_config;
    use keyrx_daemon::device_manager::{enumerate_keyboards, match_device};

    println!("Validating configuration: {}", config_path.display());
    println!();

    // Step 1: Load and validate the configuration
    println!("1. Loading configuration...");
    let config = load_config(config_path).map_err(|e| {
        (
            exit_codes::CONFIG_ERROR,
            format!("Failed to load configuration: {}", e),
        )
    })?;

    println!(
        "   Configuration loaded: {} device pattern(s)",
        config.devices.len()
    );

    // Print the device patterns
    for (i, device_config) in config.devices.iter().enumerate() {
        println!(
            "   [{:>2}] Pattern: \"{}\" ({} mapping(s))",
            i + 1,
            device_config.identifier.pattern,
            device_config.mappings.len()
        );
    }
    println!();

    // Step 2: Enumerate keyboard devices
    println!("2. Enumerating keyboard devices...");
    let keyboards = enumerate_keyboards().map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to enumerate devices: {}", e),
        )
    })?;

    if keyboards.is_empty() {
        println!("   No keyboard devices found.");
        println!();
        println!("RESULT: Configuration is valid, but no devices to match.");
        return Ok(());
    }

    println!("   Found {} keyboard device(s)", keyboards.len());
    println!();

    // Step 3: Match devices against patterns
    println!("3. Matching devices to configuration patterns...");
    println!();

    let mut matched_count = 0;
    let mut unmatched_devices = Vec::new();

    for keyboard in &keyboards {
        // Check each pattern in order (priority)
        let mut matched_pattern: Option<&str> = None;

        for device_config in config.devices.iter() {
            let pattern = device_config.identifier.pattern.as_str();
            if match_device(keyboard, pattern) {
                matched_pattern = Some(pattern);
                break; // First match wins (priority ordering)
            }
        }

        if let Some(pattern) = matched_pattern {
            println!(
                "   [MATCH] {} -> pattern \"{}\"",
                keyboard.path.display(),
                pattern
            );
            println!("           Name: {}", keyboard.name);
            if let Some(ref serial) = keyboard.serial {
                println!("           Serial: {}", serial);
            }
            matched_count += 1;
        } else {
            unmatched_devices.push(keyboard);
        }
    }

    println!();

    // Show unmatched devices as warnings
    if !unmatched_devices.is_empty() {
        println!("   Unmatched devices (will not be remapped):");
        for device in &unmatched_devices {
            println!("   [SKIP]  {}", device.path.display());
            println!("           Name: {}", device.name);
        }
        println!();
    }

    // Final result
    println!("{}", "=".repeat(60));
    if matched_count > 0 {
        println!(
            "RESULT: Configuration is valid. {} of {} device(s) matched.",
            matched_count,
            keyboards.len()
        );
        println!();
        println!(
            "Run 'keyrx_daemon run --config {}' to start remapping.",
            config_path.display()
        );
    } else {
        println!("WARNING: Configuration is valid, but no devices matched any pattern.");
        println!();
        println!(
            "Check your device patterns. Use 'keyrx_daemon list-devices' to see available devices."
        );
    }

    Ok(())
}

#[cfg(not(feature = "linux"))]
fn handle_validate(_config_path: &std::path::Path) -> Result<(), (i32, String)> {
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'validate' command is only available on Linux. \
         Build with --features linux to enable."
            .to_string(),
    ))
}

/// Initializes the logging system.
#[cfg(feature = "linux")]
fn init_logging(debug: bool) {
    use env_logger::Builder;
    use log::LevelFilter;

    let level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    Builder::new()
        .filter_level(level)
        .format_timestamp_millis()
        .init();
}

/// Converts a DaemonError to an exit code and message.
#[cfg(feature = "linux")]
fn daemon_error_to_exit(error: keyrx_daemon::daemon::DaemonError) -> (i32, String) {
    use keyrx_daemon::daemon::DaemonError;

    match &error {
        DaemonError::Config(_) => (exit_codes::CONFIG_ERROR, error.to_string()),
        DaemonError::PermissionError(_) => (exit_codes::PERMISSION_ERROR, error.to_string()),
        DaemonError::Device(dev_err) => {
            // Check if it's a permission error
            if dev_err.to_string().contains("permission")
                || dev_err.to_string().contains("Permission")
            {
                (exit_codes::PERMISSION_ERROR, error.to_string())
            } else {
                (exit_codes::CONFIG_ERROR, error.to_string())
            }
        }
        DaemonError::DiscoveryError(_) => (exit_codes::CONFIG_ERROR, error.to_string()),
        _ => (exit_codes::RUNTIME_ERROR, error.to_string()),
    }
}

/// Truncates a string to the specified length, adding "..." if truncated.
#[cfg(feature = "linux")]
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
