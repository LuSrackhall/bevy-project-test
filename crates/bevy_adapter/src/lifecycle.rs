use bevy::prelude::*;
use simulation::events::*;
use simulation::soldier::*;
use crate::mapper::UnitIdMapper;
use crate::tick::PendingEvents;
use crate::binding::LogicEntityRef;

/// Sync simulation entities to Bevy entities based on pending tick events.
pub fn sync_entities_system(
    mut commands: Commands,
    mut mapper: ResMut<UnitIdMapper>,
    mut pending: ResMut<PendingEvents>,
) {
    for events in pending.events.drain(..) {
        // Spawn new entities
        for ev in &events.spawned {
            if mapper.entity_of(ev.unit_id).is_some() { continue; }

            let entity = commands.spawn((
                LogicEntityRef(ev.unit_id),
            )).id();
            mapper.register(ev.unit_id, entity);
        }

        // Despawn destroyed entities
        for ev in &events.destroyed {
            if let Some(entity) = mapper.entity_of(ev.unit_id) {
                commands.entity(entity).despawn();
                mapper.unregister(entity);
            }
        }
    }
}

/// Backfill: sync any simulation entities that exist but have no Bevy counterpart.
/// Runs every frame but is cheap after first run (entities already registered).
pub fn backfill_entities_system(
    mut commands: Commands,
    mut mapper: ResMut<UnitIdMapper>,
    mut sim_world: bevy::ecs::system::NonSendMut<crate::tick::SimulationWorld>,
    mut has_run: Local<bool>,
) {
    let world = &mut sim_world.0;
    let to_spawn: Vec<simulation::types::UnitId> = {
        let mut query = world.query::<(Entity, &UnitIdComponent)>();
        query.iter(world)
            .filter(|(_, id)| mapper.entity_of(id.0).is_none())
            .map(|(_, id)| id.0)
            .collect()
    };

    for unit_id in to_spawn {
        let entity = commands.spawn((LogicEntityRef(unit_id),)).id();
        mapper.register(unit_id, entity);
    }

    *has_run = true;
}
