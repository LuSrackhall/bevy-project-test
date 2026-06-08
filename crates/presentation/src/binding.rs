use bevy::prelude::*;
use simulation::types::UnitId;

/// Component on render entities that references their logical counterpart.
#[derive(Component)]
pub struct LogicEntityRef(pub UnitId);
