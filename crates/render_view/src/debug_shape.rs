use bevy::prelude::*;
use bevy_adapter::tick::{SimulationWorld, TickClock};
use simulation::combat::Arrow;
use simulation::soldier::config::SoldierConfig;
use simulation::soldier::*;
use std::collections::HashMap;

/// Per-entity interpolation state for smoothing 20Hz sim ticks to 60fps render.
/// Maintained by `draw_debug_shapes_system` (no separate capture system — the
/// draw pass already iterates the sim world every frame).
///
/// Standard one-tick-lag interpolation: `prev` holds positions at the start of
/// the current inter-tick interval, `cur` holds positions at the interval end
/// (captured last frame). At each tick boundary `prev`/`cur` are swapped.
#[derive(Resource, Default)]
pub struct RenderInterpolation {
    /// Positions at the previous tick boundary (interval start).
    pub prev: HashMap<Entity, Vec2>,
    /// Positions at the current tick boundary (interval end), captured last frame.
    pub cur: HashMap<Entity, Vec2>,
    /// Tick whose positions are in `cur`.
    pub last_tick: u32,
}

impl RenderInterpolation {
    /// Advance the interval if a tick completed this frame: promote `cur` → `prev`.
    fn advance(&mut self, current_tick: u32) {
        if current_tick != self.last_tick {
            std::mem::swap(&mut self.prev, &mut self.cur);
            self.cur.clear();
            self.last_tick = current_tick;
        }
    }

    /// Interpolated draw position for one entity; records `current` for the
    /// next tick boundary. Entities without a `prev` record fall back to `current`.
    fn sample(&mut self, entity: Entity, current: Vec2, alpha: f32) -> Vec2 {
        let prev_pos = self.prev.get(&entity).copied().unwrap_or(current);
        let pos = prev_pos.lerp(current, alpha);
        self.cur.insert(entity, current);
        pos
    }
}

fn is_player_faction(f: simulation::types::FactionId, lid: u8) -> bool {
    f == simulation::types::FactionId(lid)
}
fn faction_is_active_enemy(f: simulation::types::FactionId, lid: u8) -> bool {
    f != simulation::types::FactionId(lid)
        && (f == simulation::types::FactionId(0) || f == simulation::types::FactionId(1))
}

pub fn draw_debug_shapes_system(
    mut gizmos: Gizmos,
    sim_world: bevy::ecs::system::NonSend<SimulationWorld>,
    tick_clock: Res<TickClock>,
    mut interp: ResMut<RenderInterpolation>,
    q_windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform), With<crate::camera::MainCamera>>,
    q_proj: Query<&Projection, With<crate::camera::MainCamera>>,
) {
    let world = sim_world.world_ref();
    let lid = crate::local_player_id(&*sim_world);

    interp.advance(tick_clock.current_tick);
    let alpha = (tick_clock.accumulator / tick_clock.tick_duration).clamp(0.0, 1.0);


    // Compute viewport AABB for culling
    let scale = q_proj.iter().next().and_then(|p| {
        if let Projection::Orthographic(ref o) = p { Some(o.scale) } else { None }
    }).unwrap_or(1.0);
    let aabb = q_windows.single().ok().zip(q_camera.single().ok()).map(|(w, (_, t))| {
        crate::camera::viewport_aabb(t, w, scale)
    });

    let in_view = |x: f32, y: f32| -> bool {
        if let Some((min_x, min_y, max_x, max_y)) = aabb {
            x >= min_x && x <= max_x && y >= min_y && y <= max_y
        } else {
            true
        }
    };

    // Draw cities
    {
        let mut q = sim_world.query::<(
            Entity,
            &LogicalPosition,
            &CityRadius,
            &FactionComponent,
            &CityComponent,
        )>();
        for (entity, pos, radius, faction, _city) in q.iter(world) {
            let p = interp.sample(
                entity,
                Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()),
                alpha,
            );
            let px = p.x;
            let py = p.y;
            if !in_view(px, py) { continue; }
                        let color = if is_player_faction(faction.0, lid) {
                Color::srgb(0.2, 0.6, 1.0)
            } else if faction_is_active_enemy(faction.0, lid) {
                Color::srgb(1.0, 0.2, 0.2)
            } else {
                Color::srgb(0.6, 0.6, 0.6)
            };

            let r = radius.0 as f32;
            gizmos.circle_2d(Vec2::new(px, py), r, color);
        }
    }

    // Draw soldiers — circle radius from collision_radius config
    {
        let soldier_config = world.resource::<SoldierConfig>().clone();
        let mut q = sim_world.query::<(
            Entity,
            &LogicalPosition,
            &FactionComponent,
            &SoldierTypeComponent,
            Option<&simulation::types::FacingDirection>,
        )>();
        for (entity, pos, faction, stype, facing) in q.iter(world) {
            let p = interp.sample(
                entity,
                Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()),
                alpha,
            );
            if !in_view(p.x, p.y) { continue; }
                        let color = if is_player_faction(faction.0, lid) {
                Color::srgb(0.3, 0.5, 0.9)
            } else if faction_is_active_enemy(faction.0, lid) {
                Color::srgb(0.9, 0.3, 0.3)
            } else {
                Color::srgb(0.5, 0.5, 0.5)
            };

            let r = soldier_config.get(stype.0).collision_radius as f32;
            gizmos.circle_2d(p, r, color);

            // Facing direction line
            if let Some(facing) = facing {
                let angle_deg = facing.angle.to_float();
                let angle_rad = angle_deg * std::f32::consts::PI / 180.0;
                let line_len = r * 1.5;
                let dir = Vec2::new(angle_rad.cos(), angle_rad.sin());
                                let line_color = if is_player_faction(faction.0, lid) {
                    Color::srgb(0.5, 0.7, 1.0)
                } else if faction_is_active_enemy(faction.0, lid) {
                    Color::srgb(1.0, 0.5, 0.5)
                } else {
                    Color::srgb(0.7, 0.7, 0.7)
                };

                gizmos.line_2d(p, p + dir * line_len, line_color);
            }

            // Shield visual — small rectangle to the right of the soldier
            if let Some(shield) = world.get::<simulation::types::ShieldItem>(entity) {
                if shield.hp > 0 {
                    let shield_offset = r + 3.0;
                    let shield_pos = p + Vec2::new(shield_offset, 0.0);
                                        let shield_color = if is_player_faction(faction.0, lid) {
                        Color::srgb(0.4, 0.6, 1.0)
                    } else if faction_is_active_enemy(faction.0, lid) {
                        Color::srgb(1.0, 0.4, 0.4)
                    } else {
                        Color::srgb(0.6, 0.6, 0.6)
                    };

                    let hw = 2.0;
                    let hh = 2.5;
                    let corners = [
                        shield_pos + Vec2::new(-hw, -hh),
                        shield_pos + Vec2::new(hw, -hh),
                        shield_pos + Vec2::new(hw, hh),
                        shield_pos + Vec2::new(-hw, hh),
                    ];
                    gizmos.line_2d(corners[0], corners[1], shield_color);
                    gizmos.line_2d(corners[1], corners[2], shield_color);
                    gizmos.line_2d(corners[2], corners[3], shield_color);
                    gizmos.line_2d(corners[3], corners[0], shield_color);
                }
            }
        }
    }

    // Draw arrows — bright yellow, with decay-phase alpha/shrink
    {
        let mut q = sim_world.query::<(Entity, &LogicalPosition, &Arrow)>();
        for (entity, pos, arrow) in q.iter(world) {
            let p = interp.sample(
                entity,
                Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()),
                alpha,
            );

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
            let dir = Vec2::new(arrow.direction.x.to_float(), arrow.direction.y.to_float())
                .normalize_or_zero();
            gizmos.line_2d(p, p + dir * dir_len, color);
        }
    }
}

/// Render boundary walls as red lines.
pub fn draw_boundary_walls_system(
    mut gizmos: Gizmos,
    map_bounds: Option<Res<bevy_adapter::MapBounds>>,
) {
    if let Some(b) = map_bounds.as_ref() {
        let color = Color::srgb(0.8, 0.2, 0.2);
        let tl = Vec2::new(b.wall_min_x, b.wall_max_y);
        let tr = Vec2::new(b.wall_max_x, b.wall_max_y);
        let br = Vec2::new(b.wall_max_x, b.wall_min_y);
        let bl = Vec2::new(b.wall_min_x, b.wall_min_y);
        gizmos.line_2d(tl, tr, color);
        gizmos.line_2d(tr, br, color);
        gizmos.line_2d(br, bl, color);
        gizmos.line_2d(bl, tl, color);
    }
}

/// Render dropped shields on the ground as gray rectangles.
pub fn draw_dropped_shields_system(
    mut gizmos: Gizmos,
    sim_world: bevy::ecs::system::NonSend<SimulationWorld>,
) {
    let world = sim_world.world_ref();
    let mut q = sim_world.query::<(&simulation::types::DroppedShield,)>();
    for (dropped,) in q.iter(world) {
        let p = Vec2::new(dropped.position.x.to_float(), dropped.position.y.to_float());
        let color = Color::srgb(0.6, 0.6, 0.6);

        // 6x8 rectangle outline
        let hw = 3.0;
        let hh = 4.0;
        let corners = [
            p + Vec2::new(-hw, -hh),
            p + Vec2::new(hw, -hh),
            p + Vec2::new(hw, hh),
            p + Vec2::new(-hw, hh),
        ];
        gizmos.line_2d(corners[0], corners[1], color);
        gizmos.line_2d(corners[1], corners[2], color);
        gizmos.line_2d(corners[2], corners[3], color);
        gizmos.line_2d(corners[3], corners[0], color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_swaps_cur_to_prev_on_new_tick() {
        let e = Entity::from_raw_u32(1).unwrap();
        let mut interp = RenderInterpolation {
            cur: HashMap::from([(e, Vec2::new(10.0, 10.0))]),
            last_tick: 5,
            ..Default::default()
        };
        interp.advance(6);
        assert_eq!(interp.prev.get(&e), Some(&Vec2::new(10.0, 10.0)));
        assert!(interp.cur.is_empty(), "cur must be cleared after swap");
        assert_eq!(interp.last_tick, 6);
    }

    #[test]
    fn test_no_advance_without_tick() {
        let e = Entity::from_raw_u32(1).unwrap();
        let mut interp = RenderInterpolation {
            prev: HashMap::from([(e, Vec2::new(1.0, 1.0))]),
            cur: HashMap::from([(e, Vec2::new(2.0, 2.0))]),
            last_tick: 5,
        };
        interp.advance(5);
        assert_eq!(interp.prev.get(&e), Some(&Vec2::new(1.0, 1.0)));
        assert_eq!(interp.cur.get(&e), Some(&Vec2::new(2.0, 2.0)));
    }

    #[test]
    fn test_sample_lerps_between_prev_and_current() {
        let e = Entity::from_raw_u32(1).unwrap();
        let mut interp = RenderInterpolation {
            prev: HashMap::from([(e, Vec2::new(0.0, 0.0))]),
            ..Default::default()
        };
        let pos = interp.sample(e, Vec2::new(100.0, 0.0), 0.5);
        assert_eq!(pos, Vec2::new(50.0, 0.0));
        // current recorded for the next tick boundary
        assert_eq!(interp.cur.get(&e), Some(&Vec2::new(100.0, 0.0)));
    }

    #[test]
    fn test_sample_new_entity_falls_back_to_current() {
        let e = Entity::from_raw_u32(1).unwrap();
        let mut interp = RenderInterpolation::default();
        let pos = interp.sample(e, Vec2::new(7.0, 9.0), 0.5);
        assert_eq!(pos, Vec2::new(7.0, 9.0), "no prev record → render at current");
        assert_eq!(interp.cur.get(&e), Some(&Vec2::new(7.0, 9.0)));
    }
}
