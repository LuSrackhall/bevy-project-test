use bevy::prelude::*;
use bevy_adapter::tick::SimulationWorld;
use simulation::soldier::*;
use simulation::combat::Arrow;

/// Render all simulation entities as colored circles using Gizmos.
pub fn draw_debug_shapes_system(
    mut gizmos: Gizmos,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    let world = &mut sim_world.0;

    // Draw cities
    {
        let mut query = world.query::<(&LogicalPosition, &CityRadius, &FactionComponent, &CityComponent)>();
        for (pos, radius, faction, city) in query.iter(world) {
            let color = match faction.0 {
                simulation::types::Faction::Player => Color::srgb(0.2, 0.6, 1.0),
                simulation::types::Faction::Enemy => Color::srgb(1.0, 0.2, 0.2),
                simulation::types::Faction::Neutral => Color::srgb(0.6, 0.6, 0.6),
            };
            let r = radius.0 as f32;
            gizmos.circle_2d(
                Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()),
                r,
                color,
            );
        }
    }

    // Draw soldiers
    {
        let mut query = world.query::<(&LogicalPosition, &FactionComponent, &SoldierTypeComponent)>();
        for (pos, faction, stype) in query.iter(world) {
            let color = match faction.0 {
                simulation::types::Faction::Player => Color::srgb(0.3, 0.5, 0.9),
                simulation::types::Faction::Enemy => Color::srgb(0.9, 0.3, 0.3),
                simulation::types::Faction::Neutral => Color::srgb(0.5, 0.5, 0.5),
            };
            let r = match stype.0 {
                simulation::types::SoldierType::Cavalry => 14.0,
                _ => 10.0,
            };
            gizmos.circle_2d(
                Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()),
                r,
                color,
            );
        }
    }

    // Draw arrows
    {
        let mut query = world.query::<(&LogicalPosition, &Arrow)>();
        for (pos, _arrow) in query.iter(world) {
            let p = Vec2::new(pos.0.x.to_float(), pos.0.y.to_float());
            let dir = Vec2::new(5.0, 0.0); // simplified
            gizmos.line_2d(p, p + dir, Color::srgb(0.9, 0.9, 0.3));
        }
    }
}
