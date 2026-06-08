use bevy::prelude::*;
use simulation::types::UnitId;

/// Component on Bevy entities that references their simulation counterpart.
/// Placed in bevy_adapter to avoid circular dependency with presentation.
#[derive(Component)]
pub struct LogicEntityRef(pub UnitId);
