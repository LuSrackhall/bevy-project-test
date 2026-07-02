use bevy::prelude::*;
use simulation::SimulationEvents;

// Re-export TickClock from driver for backward compatibility
pub use crate::driver::TickClock;

/// Wrapper to hold the simulation World in Bevy's ECS.
/// World is !Send, so we use a resource accessible only from the main thread.
#[derive(Resource)]
pub struct SimulationWorld(pub simulation::World);

// ═══════════════════════════════════════════════════════════════
// Simulation access traits — Simulation 外部模块的唯一交互接口
// ═══════════════════════════════════════════════════════════════

/// Read-only structural query into the simulation world.
///
/// Only provides `query_world(|world| ...)` — semantic query methods
/// (e.g. `get_unit_by_id`) are FORBIDDEN per §2.5.4 + ADR-006.
pub trait SimulationReader {
    /// Execute a read-only closure against the simulation World.
    fn query_world<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&simulation::World) -> R;
}

/// Command submission sink — pure transport interface (zero semantic branching).
///
/// All semantic processing (validation, dedup, priority) MUST happen
/// in `CommandScheduler`, NOT in `CommandSink`.
pub trait CommandSink {
    /// Submit a GameCommand into the pipeline.
    /// No guarantee of immediate acceptance — the Scheduler may reject.
    fn submit_command(&mut self, cmd: simulation::command::GameCommand);
}

// Implement on SimulationWorld — this is the only concrete impl.
impl SimulationReader for SimulationWorld {
    fn query_world<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&simulation::World) -> R,
    {
        f(&self.0)
    }
}

impl CommandSink for SimulationWorld {
    fn submit_command(&mut self, cmd: simulation::command::GameCommand) {
        self.0
            .resource_mut::<simulation::command::CommandBuffer>()
            .0
            .push(cmd);
    }
}

/// Pending events from the last simulation tick, to be consumed by lifecycle systems.
#[derive(Resource, Default, Clone)]
pub struct PendingEvents {
    pub events: Vec<SimulationEvents>,
}
