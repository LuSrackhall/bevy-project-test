use bevy::prelude::*;
use bevy_adapter::tick::SimulationWorld;
use simulation::soldier::*;
use simulation::soldier::config::SoldierConfig;
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

    // Draw soldiers — circle radius from collision_radius config
    {
        let soldier_config = world.resource::<SoldierConfig>().clone();
        let mut query = world.query::<(Entity, &LogicalPosition, &FactionComponent, &SoldierTypeComponent, Option<&simulation::types::FacingDirection>)>();
        for (_entity, pos, faction, stype, facing) in query.iter(world) {
            let color = match faction.0 {
                simulation::types::Faction::Player => Color::srgb(0.3, 0.5, 0.9),
                simulation::types::Faction::Enemy => Color::srgb(0.9, 0.3, 0.3),
                simulation::types::Faction::Neutral => Color::srgb(0.5, 0.5, 0.5),
            };
            let r = soldier_config.get(stype.0).collision_radius as f32;
            let p = Vec2::new(pos.0.x.to_float(), pos.0.y.to_float());
            gizmos.circle_2d(p, r, color);

            // Facing direction line
            if let Some(facing) = facing {
                let angle_deg = facing.angle.to_float();
                let angle_rad = angle_deg * std::f32::consts::PI / 180.0;
                let line_len = r * 1.5;
                let dir = Vec2::new(angle_rad.cos(), angle_rad.sin());
                let line_color = match faction.0 {
                    simulation::types::Faction::Player => Color::srgb(0.5, 0.7, 1.0),
                    simulation::types::Faction::Enemy => Color::srgb(1.0, 0.5, 0.5),
                    simulation::types::Faction::Neutral => Color::srgb(0.7, 0.7, 0.7),
                };
                gizmos.line_2d(p, p + dir * line_len, line_color);
            }
        }
    }

    // Draw arrows — bright yellow, with decay-phase alpha/shrink
    {
        let mut query = world.query::<(&LogicalPosition, &Arrow)>();
        for (pos, arrow) in query.iter(world) {
            let p = Vec2::new(pos.0.x.to_float(), pos.0.y.to_float());

            // In decay phase: shrink and fade; in flight: full bright yellow
            let (radius, alpha) = if arrow.decay_remaining > 0 {
                let t = arrow.decay_remaining as f32 / simulation::combat::ARROW_DECAY_TICKS as f32;
                (1.0 + 3.0 * t, 0.3 + 0.7 * t)
            } else {
                (4.0, 1.0)
            };
            let color = Color::srgba(1.0, 0.93, 0.2, alpha);

            // Circle
            gizmos.circle_2d(p, radius, color);

            // Direction line (shorter in decay)
            let dir_len = if arrow.decay_remaining > 0 { 4.0 } else { 10.0 };
            let dir = Vec2::new(arrow.direction.x.to_float(), arrow.direction.y.to_float()).normalize_or_zero();
            gizmos.line_2d(p, p + dir * dir_len, color);
        }
    }
}
