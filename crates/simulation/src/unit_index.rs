//! Per-tick index mapping UnitId → Entity for O(1) lookups.
//!
//! Maintained incrementally: insert on spawn, remove on despawn.
//! This is simulation-internal derived data (like hash_world_state), not
//! shared state — does not conflict with bevy_adapter's UnitIdMapper (§17).

use crate::soldier::UnitIdComponent;
use crate::types::*;
use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct UnitIdEntityIndex(pub HashMap<UnitId, bevy_ecs::entity::Entity>);

impl UnitIdEntityIndex {
    /// Rebuild the index from all entities with UnitIdComponent.
    /// Used for initial construction; during a tick, use insert/remove instead.
    pub fn rebuild(world: &mut World) -> Self {
        let mut q = world.query::<(bevy_ecs::entity::Entity, &UnitIdComponent)>();
        let map = q.iter(world).map(|(e, id)| (id.0, e)).collect();
        UnitIdEntityIndex(map)
    }

    /// Look up an Entity by UnitId. O(1).
    pub fn get(&self, unit_id: UnitId) -> Option<bevy_ecs::entity::Entity> {
        self.0.get(&unit_id).copied()
    }

    /// Insert a (UnitId, Entity) pair. Called when a new entity is spawned.
    pub fn insert(&mut self, unit_id: UnitId, entity: bevy_ecs::entity::Entity) {
        self.0.insert(unit_id, entity);
    }

    /// Remove a UnitId from the index. Called when an entity is despawned.
    pub fn remove(&mut self, unit_id: UnitId) {
        self.0.remove(&unit_id);
    }
}
