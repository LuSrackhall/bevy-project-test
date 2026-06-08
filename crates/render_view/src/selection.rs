use bevy::prelude::*;
use simulation::types::UnitId;

#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected_unit_ids: Vec<UnitId>,
}

// Migrate existing selection logic from src/input/mod.rs here.
// For now, provide the basic resource and placeholder systems.
