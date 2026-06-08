pub mod mapper;
pub mod tick;
pub mod lifecycle;
pub mod input;

use bevy::prelude::*;
use simulation::command::CommandBuffer;
use crate::mapper::UnitIdMapper;
use crate::tick::{TickClock, SimulationWorld};
use crate::input::ForceMoveNext;

pub struct BevyAdapterPlugin;

impl Plugin for BevyAdapterPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UnitIdMapper>()
            .init_resource::<TickClock>()
            .init_resource::<CommandBuffer>()
            .init_resource::<ForceMoveNext>()
            .add_systems(Update, crate::tick::tick_driver_system);
    }
}
