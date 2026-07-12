use std::net::SocketAddr;

use crate::discovery::{RelayId, RoomMetadata};

use super::error::RelayError;

/// Relay creation strategy. Default implementation is `ThreadRelayRuntime`.
///
/// `start()` creates a new relay instance. The relay's lifetime is managed
/// through the returned `RelayHandle`. There is no `stop()` on the runtime
/// itself — lifecycle is fully delegated to the handle.
pub trait RelayRuntime: Send + Sync {
    /// Start a relay for the given room.
    ///
    /// Returns a handle to the running relay, or an error if startup fails.
    fn start(&mut self, room: &RoomMetadata) -> Result<Box<dyn RelayHandle>, RelayError>;
}

/// Handle to a running relay instance.
///
/// Provides access to the relay's identity and connection endpoint,
/// and the ability to shut it down.
pub trait RelayHandle: Send + Sync {
    /// Unique identifier for this relay instance.
    fn relay_id(&self) -> RelayId;
    /// Network endpoint where the relay is listening.
    fn endpoint(&self) -> SocketAddr;
    /// Shut down the relay, releasing all resources.
    fn shutdown(self: Box<Self>) -> Result<(), RelayError>;
}
