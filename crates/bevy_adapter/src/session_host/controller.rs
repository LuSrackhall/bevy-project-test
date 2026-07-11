use crate::discovery::RoomMetadata;

use super::error::RelayError;
use super::runtime::{RelayHandle, RelayRuntime};

/// A running session: room metadata + active relay handle.
pub struct Session {
    pub room: RoomMetadata,
    pub relay: Box<dyn RelayHandle>,
}

/// Manages the lifecycle of the current local session.
///
/// I1: A SessionController manages at most one Session at a time
/// (Option<Session>, not Vec<Session>).
///
/// I2: SessionController does NOT maintain runtime room state.
/// All runtime fields (current_players, game_state) are authoritatively
/// updated by the relay, not by the controller.
pub struct SessionController {
    runtime: Box<dyn RelayRuntime>,
    session: Option<Session>,
}

impl SessionController {
    /// Create a new controller with the given relay runtime strategy.
    pub fn new(runtime: Box<dyn RelayRuntime>) -> Self {
        Self {
            runtime,
            session: None,
        }
    }

    /// Whether a session is currently active.
    pub fn is_active(&self) -> bool {
        self.session.is_some()
    }

    /// Create a new session. If a session is already active, it is
    /// destroyed first (I1: single session).
    pub fn create_session(&mut self, room: RoomMetadata) -> Result<&Session, RelayError> {
        // I1: Destroy existing session first
        if let Some(old) = self.session.take() {
            let _ = old.relay.shutdown();
        }
        let relay = self.runtime.start(&room)?;
        self.session = Some(Session { room, relay });
        Ok(self.session.as_ref().unwrap())
    }

    /// Returns a reference to the current session, if any.
    pub fn current_session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// Destroy the current session, stopping the relay.
    /// No-op if no session is active.
    pub fn destroy_session(&mut self) -> Result<(), RelayError> {
        if let Some(session) = self.session.take() {
            session.relay.shutdown()?;
        }
        Ok(())
    }
}
