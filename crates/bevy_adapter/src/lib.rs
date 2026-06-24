pub mod binding;
pub mod input;
pub mod lifecycle;
pub mod mapper;
pub mod replay;
pub mod tick;
pub mod driver;

use crate::input::ForceMoveNext;
use crate::mapper::UnitIdMapper;
use crate::replay::{ReplayRecorder, ReplayStatus};
use crate::tick::PendingEvents;
use bevy::prelude::*;
use simulation::command::CommandBuffer;

/// Owned by bevy_adapter; set by render_view to gate tick/sync systems.
#[derive(Resource, Default, PartialEq)]
pub struct GameActive(pub bool);

/// Pause flag — when true, simulation tick stops even if GameActive is true.
#[derive(Resource, Default, PartialEq)]
pub struct Paused(pub bool);

/// Map dimensions in world units — bridged from MapGenConfig for render_view.
#[derive(Resource)]
pub struct MapBounds {
    pub width: f32,
    pub height: f32,
    /// Boundary wall min (x, y) — units can't pass
    pub wall_min_x: f32,
    pub wall_min_y: f32,
    /// Boundary wall max (x, y) — units can't pass
    pub wall_max_x: f32,
    pub wall_max_y: f32,
}

/// Currently selected map size — stored for SameSize restarts.
#[derive(Resource, Clone, Copy)]
pub struct CurrentMapSize(pub simulation::map::MapSize);

impl Default for CurrentMapSize {
    fn default() -> Self {
        Self(simulation::map::MapSize::Small)
    }
}

pub struct BevyAdapterPlugin;

impl Plugin for BevyAdapterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnitIdMapper>()
            .init_resource::<CommandBuffer>()
            .init_resource::<PendingEvents>()
            .init_resource::<ForceMoveNext>()
            .init_resource::<GameActive>()
            .init_resource::<Paused>()
            .init_resource::<CurrentMapSize>()
            .init_resource::<crate::driver::TickClock>()
            .init_resource::<ReplayRecorder>()
            .init_resource::<ReplayStatus>()
            // Unified driver: simulation_driver_system + sync_entities
            .add_systems(
                Update,
                (
                    crate::driver::simulation_driver_system.before(crate::lifecycle::sync_entities_system),
                    crate::lifecycle::sync_entities_system,
                )
                    .run_if(resource_exists_and_equals(GameActive(true))),
            );
    }
}
