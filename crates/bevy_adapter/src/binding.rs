use bevy::prelude::*;
use simulation::types::UnitId;

/// Component on Bevy entities that references their simulation counterpart.
#[derive(Component)]
pub struct LogicEntityRef(pub UnitId);

/// Smoothed visual position computed each frame by the presentation layer.
#[derive(Component, Default)]
pub struct PresentationPosition(pub Vec2);

/// Historical positions for interpolation between ticks.
#[derive(Component, Default)]
pub struct InterpolationData {
    pub previous_logical_pos: Vec2,
    pub current_logical_pos: Vec2,
    pub is_new: bool,
}
