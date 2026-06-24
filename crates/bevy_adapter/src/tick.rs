use bevy::prelude::*;
use simulation::SimulationEvents;

// Re-export TickClock from driver for backward compatibility
pub use crate::driver::TickClock;

/// Wrapper to hold the simulation World in Bevy's ECS.
/// World is !Send, so we use a resource accessible only from the main thread.
#[derive(Resource)]
pub struct SimulationWorld(pub simulation::World);

/// Pending events from the last simulation tick, to be consumed by lifecycle systems.
#[derive(Resource, Default, Clone)]
pub struct PendingEvents {
    pub events: Vec<SimulationEvents>,
}
