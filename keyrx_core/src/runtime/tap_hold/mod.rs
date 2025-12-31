//! Tap-hold state machine implementation.
//!
//! This module provides the core tap-hold functionality where a key can act as
//! one key when tapped (quick press and release) and a modifier when held
//! (pressed beyond a threshold).
//!
//! # State Machine
//!
//! ```text
//!                    Press
//!     ┌─────────────────────────────────────┐
//!     │                                     ▼
//!  ┌──────┐    ┌─────────┐  timeout    ┌────────┐
//!  │ Idle │───▶│ Pending │────────────▶│  Hold  │
//!  └──────┘    └─────────┘             └────────┘
//!     ▲             │                       │
//!     │   quick     │    other key          │
//!     │   release   │    pressed            │
//!     │   (tap)     │  (permissive hold)    │
//!     │             ▼                       │
//!     │         emit tap                    │
//!     │         key event                   │
//!     │                                     │
//!     └─────────────────────────────────────┘
//!                   Release
//! ```
//!
//! # Debug Logging
//!
//! This module includes trace-level logging for state transitions when compiled
//! in debug mode (`cfg(debug_assertions)`). This logging is completely compiled
//! out in release builds, ensuring zero runtime overhead in production.
//!
//! To see the logs, initialize a logger (e.g., `env_logger`) and set the log
//! level to `trace`.
//!
//! # Example
//!
//! ```rust
//! use keyrx_core::runtime::tap_hold::{TapHoldPhase, TapHoldState, TapHoldConfig};
//! use keyrx_core::config::KeyCode;
//!
//! // Configure CapsLock as tap=Escape, hold=Ctrl (modifier 0)
//! let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
//!
//! // Create initial state
//! let state = TapHoldState::new(KeyCode::CapsLock, config);
//!
//! assert_eq!(state.phase(), TapHoldPhase::Idle);
//! assert_eq!(state.key(), KeyCode::CapsLock);
//! ```

mod event_processor;
mod state_machine;
mod timeout_handler;
mod types;

// Re-export public types
pub use event_processor::{TapHoldProcessor, MAX_OUTPUT_EVENTS};
pub use state_machine::TapHoldState;
pub use timeout_handler::{PendingKeyRegistry, TimeoutResult, DEFAULT_MAX_PENDING};
pub use types::{TapHoldConfig, TapHoldOutput, TapHoldPhase};

#[cfg(test)]
mod testing;
