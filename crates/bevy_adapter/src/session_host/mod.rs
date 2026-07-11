//! Session lifecycle management — SessionController, RelayRuntime, Session.
//!
//! Manages the lifecycle of a local game session (room + relay instance).
//! See `openspec/changes/local-session-host/brainstorm-spec.md`.

mod controller;
mod error;
mod runtime;
mod thread;

pub use controller::*;
pub use error::*;
pub use runtime::*;
pub use thread::*;
