use bevy::prelude::*;
use std::collections::HashMap;
use simulation::types::UnitId;

/// Bidirectional O(1) mapping between simulation UnitId and Bevy Entity.
#[derive(Resource, Default)]
pub struct UnitIdMapper {
    pub unit_to_entity: HashMap<UnitId, Entity>,
    pub entity_to_unit: HashMap<Entity, UnitId>,
}

impl UnitIdMapper {
    pub fn register(&mut self, unit_id: UnitId, entity: Entity) {
        self.unit_to_entity.insert(unit_id, entity);
        self.entity_to_unit.insert(entity, unit_id);
    }

    pub fn unregister(&mut self, entity: Entity) {
        if let Some(unit_id) = self.entity_to_unit.remove(&entity) {
            self.unit_to_entity.remove(&unit_id);
        }
    }

    pub fn entity_of(&self, unit_id: UnitId) -> Option<Entity> {
        self.unit_to_entity.get(&unit_id).copied()
    }

    pub fn unit_id_of(&self, entity: Entity) -> Option<UnitId> {
        self.entity_to_unit.get(&entity).copied()
    }
}
