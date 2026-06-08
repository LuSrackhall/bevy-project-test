use bevy::prelude::*;
use bevy_adapter::binding::LogicEntityRef;
use bevy_adapter::tick::SimulationWorld;
use crate::interpolation::{PresentationPosition, InterpolationData};
use simulation::soldier::*;

/// When a new Bevy entity is created with LogicEntityRef,
/// add interpolation components and initialize from simulation position.
pub fn bind_new_entities_system(
    mut commands: Commands,
    new_entities: Query<(Entity, &LogicEntityRef), Added<LogicEntityRef>>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    let world = &mut sim_world.0;

    for (entity, logic_ref) in new_entities.iter() {
        let float_pos = {
            let id_comp = UnitIdComponent(logic_ref.0);
            let mut query = world.query::<(Entity, &UnitIdComponent, &LogicalPosition)>();
            query.iter(world)
                .find(|(_, uid, _)| uid.0 == id_comp.0)
                .map(|(_, _, pos)| Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()))
                .unwrap_or(Vec2::ZERO)
        };

        commands.entity(entity).insert((
            PresentationPosition(float_pos),
            InterpolationData {
                previous_logical_pos: float_pos,
                current_logical_pos: float_pos,
                is_new: true,
            },
        ));
    }
}
