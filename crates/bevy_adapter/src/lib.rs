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
            .add_systems(Update, (
                crate::tick::tick_driver_system,
                crate::lifecycle::sync_entities_system,
            ).run_if(resource_exists_and_equals(GameActive(true))));
    }
}
