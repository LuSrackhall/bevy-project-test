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

pub struct BevyAdapterPlugin;

impl Plugin for BevyAdapterPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UnitIdMapper>()
            .init_resource::<TickClock>()
            .init_resource::<CommandBuffer>()
            .init_resource::<PendingEvents>()
            .init_resource::<ForceMoveNext>()
            .add_systems(Startup, crate::lifecycle::backfill_entities_system)
            .add_systems(Update, (
                crate::tick::tick_driver_system,
                crate::lifecycle::sync_entities_system,
            ));
    }
}
