pub mod mapper;
pub mod tick;
pub mod lifecycle;
pub mod input;
pub mod binding;

use bevy::prelude::*;
use simulation::command::CommandBuffer;
use crate::mapper::UnitIdMapper;
use crate::tick::{TickClock, SimulationWorld, PendingEvents};
use crate::input::ForceMoveNext;

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
}

/// Currently selected map size — stored for SameSize restarts.
#[derive(Resource, Clone, Copy)]
pub struct CurrentMapSize(pub simulation::map::MapSize);

impl Default for CurrentMapSize {
    fn default() -> Self { Self(simulation::map::MapSize::Small) }
}

pub struct BevyAdapterPlugin;

impl Plugin for BevyAdapterPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UnitIdMapper>()
            .init_resource::<TickClock>()
            .init_resource::<CommandBuffer>()
            .init_resource::<PendingEvents>()
            .init_resource::<ForceMoveNext>()
            .init_resource::<GameActive>()
            .init_resource::<Paused>()
            .init_resource::<CurrentMapSize>()
            .add_systems(Update, (
                crate::tick::tick_driver_system,
                crate::lifecycle::sync_entities_system,
            ).run_if(resource_exists_and_equals(GameActive(true)).and(not(resource_exists_and_equals(Paused(true))))));
    }
}
