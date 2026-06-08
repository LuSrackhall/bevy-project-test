use bevy::prelude::*;
use simulation::events::*;
use simulation::types::*;
use crate::mapper::UnitIdMapper;

/// Sync spawned units from simulation to Bevy entities.
pub fn sync_spawn_system(
    mut commands: Commands,
    mut mapper: ResMut<UnitIdMapper>,
) {
    // This system is called after tick_driver has run.
    // Events are in the simulation world — we need a bridge.
    // For now, placeholder: the actual sync happens in the tick_driver.
}

/// Sync destroyed units.
pub fn sync_destroy_system(
    mut commands: Commands,
    mut mapper: ResMut<UnitIdMapper>,
) {
    // Placeholder
}

/// Listen for city captured events for HUD updates (placeholder).
pub fn sync_city_captured_system() {
    // HUD refresh triggered by Changed<CityComponent> in render_view
}

/// Bevy event mirroring simulation events (for cross-layer communication).
#[derive(Event, Clone)]
pub struct CityCapturedEvent {
    pub city_id: UnitId,
    pub old_faction: Faction,
    pub new_faction: Faction,
}
