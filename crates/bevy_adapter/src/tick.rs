use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use simulation::SimulationEvents;
use std::cell::UnsafeCell;

// Re-export TickClock from driver for backward compatibility
pub use crate::driver::TickClock;

/// Wrapper to hold the simulation World in Bevy's ECS.
/// World is !Send, so we use a resource accessible only from the main thread.
///
/// ## Architecture
/// - `world` field is `pub(crate)` — render_view cannot access it directly.
/// - Use `world_ref()` / `query_world()` for **read-only** access.
/// - Use `world_mut()` when **mutation** is truly required (e.g., rebuild,
///   backward seek).
/// - Use `submit_command()` to send commands via the `CommandSink` trait.
///
/// Uses `UnsafeCell` internally so `query()` can create `QueryState` from
/// `&self` (World::query requires &mut self due to a borrow-checker artifact
/// that cannot be worked around with safe code).
#[derive(Resource)]
pub struct SimulationWorld {
    pub(crate) world: UnsafeCell<simulation::World>,
}

// SAFETY: SimulationWorld is only used via NonSend/NonSendMut (main thread
// only). It is never sent or shared across threads. The UnsafeCell is purely
// for interior mutability during QueryState creation (a read-only operation).
unsafe impl Send for SimulationWorld {}
unsafe impl Sync for SimulationWorld {}

impl SimulationWorld {
    /// Create a new SimulationWorld from a simulation World.
    pub fn new(world: simulation::World) -> Self {
        Self {
            world: UnsafeCell::new(world),
        }
    }

    /// Read-only access to the inner simulation World state.
    pub fn world_ref(&self) -> &simulation::World {
        // SAFETY: This is safe because we never hand out &mut World through
        // this path, and no mutable reference exists simultaneously.
        unsafe { &*self.world.get() }
    }

    /// Mutable access to the inner simulation World.
    /// SAFETY: Caller must ensure no `world_ref()` borrow is active.
    pub fn world_mut(&mut self) -> &mut simulation::World {
        self.world.get_mut()
    }

    /// Create a read-only [`QueryState`] for the simulation World.
    ///
    /// SAFETY: `World::query()` requires `&mut self` due to a borrow-checker
    /// artifact — it only reads archetype metadata and never mutates entity
    /// data. The returned `QueryState` is `'static` (does not borrow the
    /// World) and must only be used with `QueryState::iter(world: &World)`.
    pub fn query<Q: QueryData>(&self) -> QueryState<Q, ()> {
        // SAFETY: QueryState::new only reads archetype metadata (component
        // counts, archetype layout) and does not mutate entity data. The
        // returned QueryState borrows nothing from the World.
        unsafe { QueryState::<Q, ()>::new(&mut *self.world.get()) }
    }
}

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
        // SAFETY: Closure receives a shared reference; no mutation occurs.
        let world: &simulation::World = unsafe { &*self.world.get() };
        f(world)
    }
}

impl CommandSink for SimulationWorld {
    fn submit_command(&mut self, cmd: simulation::command::GameCommand) {
        // SAFETY: &mut self ensures exclusive access. Handing &mut World to
        // resource_mut is safe here.
        let world: &mut simulation::World = unsafe { &mut *self.world.get() };
        world
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
