//! Facing direction system — deterministic angle calculations.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use std::collections::HashMap;
use crate::types::{FacingDirection, Fixed, FixedVec2, SoldierType, UnitId};
use crate::soldier::{LogicalPosition, Movement, SoldierMarker, SoldierTypeComponent, UnitIdComponent};
use crate::combat::config::CombatGlobalConfig;

/// Atan approximation constant: 0.28 * 256 ≈ 72
const ATAN_K: Fixed = Fixed(72);

/// Normalize an angle in degrees to [0, 360) fixed-point range. O(1) via modulo.
pub fn normalize_angle(angle: Fixed) -> Fixed {
    let full = Fixed::from_int(360);
    let raw = angle.0 % full.0;
    Fixed(if raw < 0 { raw + full.0 } else { raw })
}

/// Compute the signed shortest angular difference between two angles.
/// Returns value in [-180, 180) degrees (fixed-point).
/// Positive = clockwise, negative = counter-clockwise.
pub fn angle_diff(from: Fixed, to: Fixed) -> Fixed {
    let full_circle = Fixed::from_int(360);
    let half_circle = Fixed::from_int(180);
    let diff = normalize_angle(to - normalize_angle(from));
    if diff > half_circle {
        diff - full_circle
    } else {
        diff
    }
}

/// Compute the absolute shortest angular distance between two angles.
/// Returns value in [0, 180] degrees (fixed-point).
pub fn angle_distance(a: Fixed, b: Fixed) -> Fixed {
    angle_diff(a, b).abs()
}

/// Compute the angle from point `from` to point `to` in degrees.
/// 0° = right (+x), 90° = up (+y).
/// Uses integer atan2 approximation.
pub fn compute_angle_between(from: FixedVec2, to: FixedVec2) -> Fixed {
    let dx = to.x - from.x;
    let dy = to.y - from.y;

    if dx == Fixed::ZERO && dy == Fixed::ZERO {
        return Fixed::ZERO;
    }

    let abs_dx = dx.abs();
    let abs_dy = dy.abs();

    // Determine the base angle from the ratio
    let base_angle = if abs_dx >= abs_dy {
        // Use dy/dx ratio (angle < 45°)
        if abs_dx == Fixed::ZERO {
            Fixed::from_int(90)
        } else {
            let ratio = abs_dy * Fixed::ONE / abs_dx;
            // atan approximation: angle ≈ 45 * ratio / (1 + 0.28 * ratio^2)
            let ratio_sq = ratio * ratio;
            let denom = Fixed::ONE + ratio_sq * ATAN_K;
            Fixed::from_int(45) * ratio / denom
        }
    } else {
        // Use dx/dy ratio (angle > 45°)
        if abs_dy == Fixed::ZERO {
            Fixed::ZERO
        } else {
            let ratio = abs_dx * Fixed::ONE / abs_dy;
            let ratio_sq = ratio * ratio;
            let denom = Fixed::ONE + ratio_sq * ATAN_K;
            let angle = Fixed::from_int(45) * ratio / denom;
            Fixed::from_int(90) - angle
        }
    };

    // Adjust for quadrant
    let angle = if dx.0 >= 0 && dy.0 >= 0 {
        base_angle
    } else if dx.0 < 0 && dy.0 >= 0 {
        Fixed::from_int(180) - base_angle
    } else if dx.0 < 0 && dy.0 < 0 {
        Fixed::from_int(180) + base_angle
    } else {
        Fixed::from_int(360) - base_angle
    };

    normalize_angle(angle)
}

/// Turn `current` angle toward `target` by at most `max_turn` degrees.
/// Returns the new angle (always normalized to [0, 360)).
pub fn turn_toward(current: Fixed, target: Fixed, max_turn: Fixed) -> Fixed {
    let diff = angle_diff(current, target);
    let abs_diff = diff.abs();

    if abs_diff <= max_turn {
        normalize_angle(target)
    } else {
        if diff.0 > 0 {
            normalize_angle(current + max_turn)
        } else {
            normalize_angle(current - max_turn)
        }
    }
}

/// Update each soldier's facing direction toward their movement target each tick.
///
/// Target priority depends on unit type:
/// - Cavalry: command target, then attack target, then waypoint
/// - Non-cavalry: attack target, then waypoint
pub fn facing_turn_system(world: &mut World) {
    let ticks_per_rotation = world.resource::<CombatGlobalConfig>().facing.turn_rate_ticks_per_full_rotation;
    if ticks_per_rotation == 0 {
        return; // degenerate config — no turning
    }
    // turn_rate = 360 / (ticks_per_rotation * 256)  in fixed-point internal units
    let turn_rate = Fixed::from_int(360) / Fixed((ticks_per_rotation as i64) * 256);

    // Build position lookup from ALL entities with UnitIdComponent + LogicalPosition
    let positions: HashMap<UnitId, FixedVec2> = {
        let mut q = world.query::<(&UnitIdComponent, &LogicalPosition)>();
        q.iter(world).map(|(id, pos)| (id.0, pos.0)).collect()
    };

    // Collect facing updates: (entity, new_angle)
    let mut updates: Vec<(Entity, Fixed)> = Vec::new();
    {
        let mut q = world.query::<(Entity, &LogicalPosition, &Movement, &SoldierMarker, Option<&SoldierTypeComponent>)>();
        for (e, pos, mov, _, stype) in q.iter(world) {
            // Determine target position based on unit type
            // Cavalry: command_target first, then target, then waypoint
            // Non-cavalry: target first, then waypoint (no command_target)
            let is_cav = stype.map_or(false, |s| s.0 == SoldierType::Cavalry);
            let target_pos = if is_cav {
                mov.command_target
                    .and_then(|tid| positions.get(&tid).copied())
                    .or_else(|| mov.target.and_then(|tid| positions.get(&tid).copied()))
            } else {
                mov.target.and_then(|tid| positions.get(&tid).copied())
            }
            .or(mov.waypoint);

            let Some(target_pos) = target_pos else { continue };

            // Skip if already at target (no meaningful angle)
            let delta = target_pos - pos.0;
            if delta.x == Fixed::ZERO && delta.y == Fixed::ZERO {
                continue;
            }

            let desired_angle = compute_angle_between(pos.0, target_pos);
            let facing = world.entity(e).get::<FacingDirection>();
            let current_angle = facing.map(|f| f.angle).unwrap_or(Fixed::ZERO);
            let new_angle = turn_toward(current_angle, desired_angle, turn_rate);

            updates.push((e, new_angle));
        }
    }

    for (e, angle) in updates {
        world.entity_mut(e).insert(FacingDirection { angle });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_angle() {
        assert_eq!(normalize_angle(Fixed::from_int(0)), Fixed::from_int(0));
        assert_eq!(normalize_angle(Fixed::from_int(360)), Fixed::from_int(0));
        assert_eq!(normalize_angle(Fixed::from_int(370)), Fixed::from_int(10));
        assert_eq!(normalize_angle(Fixed::from_int(-10)), Fixed::from_int(350));
        assert_eq!(normalize_angle(Fixed::from_int(-360)), Fixed::from_int(0));
    }

    #[test]
    fn test_angle_diff_same() {
        let diff = angle_diff(Fixed::from_int(90), Fixed::from_int(90));
        assert_eq!(diff, Fixed::ZERO);
    }

    #[test]
    fn test_angle_diff_clockwise() {
        // From 10° to 350°: shortest path is -20° (counter-clockwise)
        let diff = angle_diff(Fixed::from_int(10), Fixed::from_int(350));
        assert_eq!(diff, Fixed::from_int(-20));
    }

    #[test]
    fn test_angle_diff_counter_clockwise() {
        // From 350° to 10°: shortest path is +20° (clockwise)
        let diff = angle_diff(Fixed::from_int(350), Fixed::from_int(10));
        assert_eq!(diff, Fixed::from_int(20));
    }

    #[test]
    fn test_angle_distance() {
        let d = angle_distance(Fixed::from_int(10), Fixed::from_int(350));
        assert_eq!(d, Fixed::from_int(20));
    }

    #[test]
    fn test_compute_angle_right() {
        let from = FixedVec2::new(Fixed::ZERO, Fixed::ZERO);
        let to = FixedVec2::new(Fixed::from_int(10), Fixed::ZERO);
        let angle = compute_angle_between(from, to);
        assert_eq!(angle, Fixed::from_int(0));
    }

    #[test]
    fn test_compute_angle_up() {
        let from = FixedVec2::new(Fixed::ZERO, Fixed::ZERO);
        let to = FixedVec2::new(Fixed::ZERO, Fixed::from_int(10));
        let angle = compute_angle_between(from, to);
        // Should be close to 90° (tolerance for approximation)
        let diff = (angle.0 - Fixed::from_int(90).0).abs();
        assert!(diff <= 3 * 256 / 10, "expected ~90°, got {}", angle.to_float());
    }

    #[test]
    fn test_turn_toward_snap() {
        let result = turn_toward(Fixed::from_int(89), Fixed::from_int(90), Fixed::from_int(5));
        assert_eq!(result, Fixed::from_int(90));
    }

    #[test]
    fn test_turn_toward_partial() {
        let result = turn_toward(Fixed::from_int(0), Fixed::from_int(90), Fixed::from_int(10));
        assert_eq!(result, Fixed::from_int(10));
    }

    #[test]
    fn test_turn_toward_wrap() {
        // From 355° toward 5° with max_turn=10 → should reach 5°
        let result = turn_toward(Fixed::from_int(355), Fixed::from_int(5), Fixed::from_int(10));
        assert_eq!(result, Fixed::from_int(5));
    }
}
