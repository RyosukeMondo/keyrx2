//! Daemon control RPC method handlers.
//!
//! This module implements daemon control operations like restart and status.

use serde::Serialize;
use serde_json::Value;
use typeshare::typeshare;

use crate::web::rpc_types::RpcError;

/// Result of restart_daemon command
#[typeshare]
#[derive(Debug, Serialize)]
pub struct RestartResult {
    pub success: bool,
    pub message: String,
}

/// Restart the daemon process.
///
/// This performs a full process restart using exec() on Unix or spawn+exit on Windows.
/// The WebSocket connection will be lost and the client should reconnect.
pub async fn restart_daemon(_params: Value) -> Result<Value, RpcError> {
    log::info!("Daemon restart requested via RPC");

    // Spawn a thread to perform the restart after a brief delay
    // This allows the RPC response to be sent before the process restarts
    std::thread::spawn(|| {
        // Brief delay to allow response to be sent
        std::thread::sleep(std::time::Duration::from_millis(100));
        perform_restart();
    });

    let result = RestartResult {
        success: true,
        message: "Daemon restart initiated. Reconnect in a moment.".to_string(),
    };

    serde_json::to_value(result).map_err(|e| RpcError::internal_error(e.to_string()))
}

/// Perform the actual process restart
fn perform_restart() {
    log::info!("Performing daemon restart...");

    let exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            log::error!("Failed to get current executable path: {}", e);
            return;
        }
    };

    let args: Vec<String> = std::env::args().skip(1).collect();

    log::info!("Restarting with: {:?} {:?}", exe, args);

    // Spawn new process and exit current one (cleaner than exec)
    // This ensures all resources are properly released
    match std::process::Command::new(&exe).args(&args).spawn() {
        Ok(_) => {
            log::info!("New daemon process spawned, exiting current process");
            std::process::exit(0);
        }
        Err(e) => {
            log::error!("Failed to spawn new daemon process: {}", e);
        }
    }
}
