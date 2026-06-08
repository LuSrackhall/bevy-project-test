use bevy::prelude::*;
use simulation::events::*;
use simulation::soldier::*;
use crate::mapper::UnitIdMapper;
use crate::tick::PendingEvents;
use crate::binding::{LogicEntityRef, PresentationPosition, InterpolationData};

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

            let float_pos = Vec2::new(ev.pos.x.to_float(), ev.pos.y.to_float());
            let entity = commands.spawn((
                LogicEntityRef(ev.unit_id),
                PresentationPosition(float_pos),
                InterpolationData {
                    previous_logical_pos: float_pos,
                    current_logical_pos: float_pos,
                    is_new: true,
                },
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

/// Backfill: sync simulation entities that existed before any ticks ran.
pub fn backfill_entities_system(
    mut commands: Commands,
    mut mapper: ResMut<UnitIdMapper>,
    mut sim_world: bevy::ecs::system::NonSendMut<crate::tick::SimulationWorld>,
) {
    let world = &mut sim_world.0;
    let to_spawn: Vec<(simulation::types::UnitId, Vec2)> = {
        let mut query = world.query::<(Entity, &UnitIdComponent, &LogicalPosition)>();
        query.iter(world)
            .filter(|(_, id, _)| mapper.entity_of(id.0).is_none())
            .map(|(_, id, pos)| (id.0, Vec2::new(pos.0.x.to_float(), pos.0.y.to_float())))
            .collect()
    };

    for (unit_id, float_pos) in to_spawn {
        let entity = commands.spawn((
            LogicEntityRef(unit_id),
            PresentationPosition(float_pos),
            InterpolationData {
                previous_logical_pos: float_pos,
                current_logical_pos: float_pos,
                is_new: true,
            },
        )).id();
        mapper.register(unit_id, entity);
    }
}
