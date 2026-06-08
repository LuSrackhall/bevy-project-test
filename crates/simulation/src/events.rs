//! Simulation events — emitted by systems, consumed by bevy_adapter and other systems.

use bevy_ecs::prelude::Resource;
use crate::types::{Faction, FixedVec2, SoldierType, UnitId};

/// A new unit was spawned in the simulation.
#[derive(Clone, Debug)]
pub struct UnitSpawned {
    pub unit_id: UnitId,
    pub pos: FixedVec2,
    pub faction: Faction,
    pub unit_kind: UnitKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitKind {
    Soldier(SoldierType),
    City,
    Arrow,
    Waypoint,
}

/// A unit was destroyed.
#[derive(Clone, Debug)]
pub struct UnitDestroyed {
    pub unit_id: UnitId,
    pub killer_id: Option<UnitId>,
}

/// A city changed faction.
#[derive(Clone, Debug)]
pub struct CityCaptured {
    pub city_id: UnitId,
    pub old_faction: Faction,
    pub new_faction: Faction,
}

/// Damage was dealt to a target.
#[derive(Clone, Debug)]
pub struct DamageDealt {
    pub target_id: UnitId,
    pub amount: u32,
    pub from_faction: Faction,
}

/// A soldier leveled up.
#[derive(Clone, Debug)]
pub struct SoldierLeveledUp {
    pub unit_id: UnitId,
    pub new_level: u32,
}

/// Collection of events for a single tick.
#[derive(Clone, Debug, Default, Resource)]
pub struct SimulationEvents {
    pub spawned: Vec<UnitSpawned>,
    pub destroyed: Vec<UnitDestroyed>,
    pub captured: Vec<CityCaptured>,
    pub damage: Vec<DamageDealt>,
    pub leveled_up: Vec<SoldierLeveledUp>,
}

impl SimulationEvents {
    pub fn new() -> Self { Self::default() }
}
