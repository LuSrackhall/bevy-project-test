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

    // Build UnitId → position map for arrow target lookup
    let positions: std::collections::HashMap<simulation::types::UnitId, Vec2> = {
        let mut q = world.query::<(&UnitIdComponent, &LogicalPosition)>();
        q.iter(world)
            .map(|(id, pos)| (id.0, Vec2::new(pos.0.x.to_float(), pos.0.y.to_float())))
            .collect()
    };

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

    // Draw arrows — bright yellow circles with direction lines toward targets
    {
        let mut query = world.query::<(&LogicalPosition, &Arrow)>();
        for (pos, arrow) in query.iter(world) {
            let p = Vec2::new(pos.0.x.to_float(), pos.0.y.to_float());
            // Bright yellow for visibility
            let color = Color::srgb(1.0, 0.93, 0.2);

            // Draw arrow as circle
            gizmos.circle_2d(p, 4.0, color);

            // Direction line toward target
            if let Some(&target_pos) = positions.get(&arrow.target) {
                let dir = (target_pos - p).normalize_or_zero();
                gizmos.line_2d(p, p + dir * 8.0, color);
            }
        }
    }
}
