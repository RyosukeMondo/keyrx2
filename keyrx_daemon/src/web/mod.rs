pub mod api;
pub mod events;
pub mod static_files;
pub mod ws;

#[cfg(test)]
mod ws_test;

use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;

pub use events::DaemonEvent;

use crate::macro_recorder::MacroRecorder;

/// Application state shared across all web handlers
///
/// This struct contains all dependencies needed by the web API and is injected
/// via axum's State extraction pattern. This enables testability by allowing
/// mock implementations to be injected during tests.
#[derive(Clone)]
pub struct AppState {
    /// Macro recorder for capturing keyboard event sequences
    pub macro_recorder: Arc<MacroRecorder>,
}

impl AppState {
    /// Creates a new AppState with the given dependencies
    pub fn new(macro_recorder: Arc<MacroRecorder>) -> Self {
        Self { macro_recorder }
    }
}

#[allow(dead_code)]
pub async fn create_app(event_tx: broadcast::Sender<DaemonEvent>, state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api", api::create_router(state))
        .nest("/ws", ws::create_router(event_tx))
        .fallback_service(static_files::serve_static())
}

#[allow(dead_code)]
pub async fn serve(
    addr: SocketAddr,
    event_tx: broadcast::Sender<DaemonEvent>,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(event_tx, state).await;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
