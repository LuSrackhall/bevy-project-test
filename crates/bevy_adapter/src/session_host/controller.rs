use crate::discovery::{RelayId, RoomMetadata};

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

    /// Returns the relay_id of the current session, if active.
    /// Used by UI to determine if a discovered room is "our own" room.
    pub fn current_relay_id(&self) -> Option<RelayId> {
        self.session.as_ref().map(|s| s.relay.relay_id())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{RelayId, RoomId, RoomMetadata, RoomState};
    use std::net::SocketAddr;

    struct MockRelayHandle {
        relay_id: RelayId,
        endpoint: SocketAddr,
    }

    impl RelayHandle for MockRelayHandle {
        fn relay_id(&self) -> RelayId {
            self.relay_id
        }
        fn endpoint(&self) -> SocketAddr {
            self.endpoint
        }
        fn shutdown(self: Box<Self>) -> Result<(), RelayError> {
            Ok(())
        }
    }

    struct MockRelayRuntime {
        fail_on_start: bool,
    }

    impl RelayRuntime for MockRelayRuntime {
        fn start(&mut self, _room: &RoomMetadata) -> Result<Box<dyn RelayHandle>, RelayError> {
            if self.fail_on_start {
                Err(RelayError::StartFailed("mock failure".into()))
            } else {
                Ok(Box::new(MockRelayHandle {
                    relay_id: RelayId(42),
                    endpoint: ([127, 0, 0, 1], 9999).into(),
                }))
            }
        }
    }

    fn make_room() -> RoomMetadata {
        RoomMetadata {
            room_id: RoomId(1),
            room_name: "测试房间".into(),
            map_id: "grassland_small".into(),
            current_players: 1,
            max_players: 2,
            state: RoomState::Waiting,
        }
    }

    #[test]
    fn test_create_session() {
        let mut ctrl = SessionController::new(Box::new(MockRelayRuntime { fail_on_start: false }));
        assert!(!ctrl.is_active());
        let result = ctrl.create_session(make_room());
        assert!(result.is_ok());
        assert!(ctrl.is_active());
        assert!(ctrl.current_session().is_some());
    }

    #[test]
    fn test_destroy_session() {
        let mut ctrl = SessionController::new(Box::new(MockRelayRuntime { fail_on_start: false }));
        ctrl.create_session(make_room()).unwrap();
        assert!(ctrl.is_active());
        ctrl.destroy_session().unwrap();
        assert!(!ctrl.is_active());
        assert!(ctrl.current_session().is_none());
    }

    #[test]
    fn test_destroy_when_no_session() {
        let mut ctrl = SessionController::new(Box::new(MockRelayRuntime { fail_on_start: false }));
        let result = ctrl.destroy_session();
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_session_replaces_existing() {
        let mut ctrl = SessionController::new(Box::new(MockRelayRuntime { fail_on_start: false }));
        ctrl.create_session(make_room()).unwrap();
        let _first_id = ctrl.current_session().unwrap().room.room_id;

        let mut room2 = make_room();
        room2.room_id = RoomId(2);
        ctrl.create_session(room2).unwrap();

        assert_eq!(ctrl.current_session().unwrap().room.room_id, RoomId(2));
        // Old session was replaced, not stacked
    }

    #[test]
    fn test_create_session_failure() {
        let mut ctrl = SessionController::new(Box::new(MockRelayRuntime { fail_on_start: true }));
        let result = ctrl.create_session(make_room());
        assert!(result.is_err());
        assert!(!ctrl.is_active());
    }

    #[test]
    fn test_relay_error_display() {
        let err = RelayError::StartFailed("port in use".into());
        assert_eq!(format!("{}", err), "Relay start failed: port in use");
        let err = RelayError::ShutdownFailed("timeout".into());
        assert_eq!(format!("{}", err), "Relay shutdown failed: timeout");
    }

    #[test]
    fn test_current_relay_id() {
        let mut ctrl = SessionController::new(Box::new(MockRelayRuntime { fail_on_start: false }));
        assert_eq!(ctrl.current_relay_id(), None);

        ctrl.create_session(make_room()).unwrap();
        assert_eq!(ctrl.current_relay_id(), Some(RelayId(42)));
    }

    #[test]
    fn test_session_fields_accessible() {
        let room = make_room();
        let relay = Box::new(MockRelayHandle {
            relay_id: RelayId(99),
            endpoint: ([127, 0, 0, 1], 8888).into(),
        });
        let session = Session { room, relay };
        assert_eq!(session.room.room_id, RoomId(1));
        assert_eq!(session.relay.relay_id(), RelayId(99));
        assert_eq!(session.relay.endpoint().port(), 8888);
    }
}
