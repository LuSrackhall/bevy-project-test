# 兵种行为重设计 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement three interconnected systems — Facing Direction, Shield Item redesign, and Archer fixes — to bring unit behavior in line with the design spec.

**Architecture:** All changes are in the `simulation` crate (pure ECS, fixed-point math, deterministic). The Facing Direction system is foundational and must be implemented first. Shield system depends on it for frontal angle checks. Archer fixes are mostly independent but interact with movement. Data changes go in `content/combat.ron`. No changes to `bevy_adapter`, `presentation`, or `render_view` in this plan.

**Tech Stack:** Rust, `bevy_ecs` (ECS only, no rendering), fixed-point `Fixed(i64)` with 8 fractional bits, RON config files.

**Spec:** `docs/superpowers/specs/2026-06-13-unit-behavior-redesign.md`

---

## File Map

### Files to Create
| File | Purpose |
|------|---------|
| `simulation/src/facing.rs` | `FacingDirection` component, `facing_turn_system`, `compute_angle_between`, angle normalization helpers |

### Files to Modify
| File | Changes |
|------|---------|
| `content/combat.ron` | Replace `shield` config, remove `archer_melee_*`, add `facing` config, add `attack_windup` config |
| `simulation/src/types.rs` | Add `ShieldState::Blocking` variant (rename `ShieldUp`), add `AttackWindup` component |
| `simulation/src/combat/config.rs` | Replace `ShieldConfig` fields, add `FacingConfig`, `AttackWindupConfig`, remove `archer_melee_*` fields |
| `simulation/src/soldier/config.rs` | No structural changes (collision_radius already present) |
| `simulation/src/soldier/mod.rs` | Add `FacingDirection` + `ShieldItem` to spawn, modify `ShieldComponent`, add `DroppedShield` entity handling, modify `soldier_movement_system` for facing-based speed + archer chase, add `attack_windup_system`, modify `shield_passive_block` + `shield_manual_block`, add `shield_pickup_system`, add `shield_decay_system`, modify `aura_heal_system` for shield HP, modify death handling for shield drop |
| `simulation/src/combat/mod.rs` | Modify `melee_attack_system` for attack windup + shield passive block + facing-based attack, modify `archer_attack_system` for multi-shot + remove close-range penalty + archer chase target setting, modify `combat_engagement_system` for archer inclusion |
| `simulation/src/command.rs` | Update `SetShield` action to use new `ShieldState` variants |
| `simulation/src/lib.rs` | Register `facing_turn_system`, `attack_windup_system`, `shield_pickup_system`, `shield_decay_system` in `run_tick` |

---

## Task 1: Facing Direction — Types & Config

**Files:**
- Modify: `simulation/src/types.rs`
- Modify: `simulation/src/combat/config.rs`
- Modify: `content/combat.ron`

### Step 1.1: Add `FacingDirection` component to types.rs

Add after the `ShieldState` enum (line ~256):

```rust
/// Unit facing direction — independent state updated by movement and attack targeting.
/// All 360° represented as fixed-point. 0° = right, 90° = up, etc.
#[derive(Component)]
pub struct FacingDirection {
    pub angle: Fixed, // 0..360 in fixed-point degrees
}
```

Also add `AttackWindup` component for the attack-while-moving mechanic:

```rust
/// Attack windup state — non-cavalry units pause briefly before attacking.
#[derive(Component)]
pub struct AttackWindup {
    pub remaining_ticks: u32, // ticks remaining in windup; 0 = not winding up
    pub target: Option<UnitId>, // target to attack when windup completes
}
```

### Step 1.2: Add `FacingConfig` and `AttackWindupConfig` to combat config

In `combat/config.rs`, add new config structs:

```rust
/// Facing direction mechanics.
#[derive(Clone, Debug, Deserialize)]
pub struct FacingConfig {
    pub turn_rate_ticks_per_full_rotation: u32, // ticks for 360°
}

/// Attack windup mechanics.
#[derive(Clone, Debug, Deserialize)]
pub struct AttackWindupConfig {
    pub windup_ticks: u32,          // 3 ticks = 0.15s for non-cavalry
    pub cavalry_no_windup: bool,    // true = cavalry attacks instantly
}
```

Add to `CombatGlobalConfig`:

```rust
pub facing: FacingConfig,
pub attack_windup: AttackWindupConfig,
```

Remove `archer_melee_range` and `archer_melee_damage_mult` fields from `CombatGlobalConfig`.

### Step 1.3: Update `content/combat.ron`

Add new sections and remove archer melee fields:

```ron
(
    city_damage_per_soldier_ratio: 0.5,
    arrow_building_damage_ratio: 0.005,
    // archer_melee_range and archer_melee_damage_mult REMOVED

    facing: (
        turn_rate_ticks_per_full_rotation: 20, // 360°/s at 20Hz
    ),

    attack_windup: (
        windup_ticks: 3,          // 0.15s for non-cavalry
        cavalry_no_windup: true,  // cavalry attacks instantly
    ),

    shield: (
        speed_penalty: 15,
        attack_speed_penalty: 60,
        passive_block_chance: 0.40,
        frontal_angle_deg: 120,
        initial_hp: 1500,
        drop_survive_ticks: 600,
        disappear_animation_ticks: 60,
    ),

    // ... rest unchanged
)
```

### Step 1.4: Update `ShieldConfig` struct

Replace the existing `ShieldConfig` fields:

```rust
#[derive(Clone, Debug, Deserialize)]
pub struct ShieldConfig {
    pub speed_penalty: u32,
    pub attack_speed_penalty: u32,
    pub passive_block_chance: f32,
    pub frontal_angle_deg: u32,
    pub initial_hp: u32,
    pub drop_survive_ticks: u32,
    pub disappear_animation_ticks: u32,
}
```

Remove `damage_reduction` and `intercept_chance` fields.

### Step 1.5: Verify compilation

Run: `cargo check -p simulation`
Expected: compile errors for references to removed fields — will fix in later tasks.

**Commit:** `feat(types): add FacingDirection, AttackWindup, update combat config`

---

## Task 2: Facing Direction — Angle Utilities

**Files:**
- Create: `simulation/src/facing.rs`
- Modify: `simulation/src/lib.rs` (add `mod facing`)

### Step 2.1: Create `facing.rs` with angle utilities

```rust
//! Facing direction system — deterministic angle calculations.

use crate::types::{Fixed, FixedVec2, FIXED_ONE};

/// Normalize an angle in degrees to [0, 360) fixed-point range.
pub fn normalize_angle(angle: Fixed) -> Fixed {
    let full_circle = Fixed::from_int(360);
    let mut a = angle;
    while a.0 >= full_circle.0 {
        a = a - full_circle;
    }
    while a.0 < 0 {
        a = a + full_circle;
    }
    a
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
    let diff = angle_diff(a, b).abs();
    diff
}

/// Compute the angle from point `from` to point `to` in degrees.
/// 0° = right (+x), 90° = up (+y).
/// Uses integer atan2 approximation.
pub fn compute_angle_between(from: FixedVec2, to: FixedVec2) -> Fixed {
    let dx = to.x - from.x;
    let dy = to.y - from.y;

    // Fast atan2 approximation using fixed-point
    // Based on: atan2(y,x) ≈ (x*abs(y)/(x^2+0.28*abs(y)^2)) * 180/π
    // But for determinism and simplicity, use a lookup-free approximation.
    //
    // We use the standard approximation:
    //   angle = atan2(y, x) in [0, 360)
    //
    // For fixed-point: use the ratio-based approach.
    // angle = atan(y/x) adjusted for quadrant.
    //
    // Simpler approach: use the fact that atan2 can be approximated by
    // comparing |x| and |y| ratios with a polynomial.

    if dx == Fixed::ZERO && dy == Fixed::ZERO {
        return Fixed::ZERO;
    }

    let abs_dx = dx.abs();
    let abs_dy = dy.abs();

    // Determine the base angle from the ratio
    let (base_angle, is_y_dominant) = if abs_dx >= abs_dy {
        // Use dy/dx ratio (angle < 45°)
        if abs_dx == Fixed::ZERO {
            (Fixed::from_int(90), true)
        } else {
            let ratio = abs_dy * Fixed::ONE / abs_dx;
            // atan approximation: angle ≈ ratio * 45° (for ratio in [0,1])
            // More accurate: angle = 45 * ratio / (1 + 0.28 * ratio^2)
            let ratio_sq = ratio * ratio;
            let denom = Fixed::ONE + ratio_sq * Fixed::from_float(0.28);
            let angle = Fixed::from_int(45) * ratio / denom;
            (angle, false)
        }
    } else {
        // Use dx/dy ratio (angle > 45°)
        if abs_dy == Fixed::ZERO {
            (Fixed::ZERO, false)
        } else {
            let ratio = abs_dx * Fixed::ONE / abs_dy;
            let ratio_sq = ratio * ratio;
            let denom = Fixed::ONE + ratio_sq * Fixed::from_float(0.28);
            let angle = Fixed::from_int(45) * ratio / denom;
            (Fixed::from_int(90) - angle, true)
        }
    };

    // Adjust for quadrant
    let angle = if dx.0 >= 0 && dy.0 >= 0 {
        // Quadrant 1: [0, 90)
        base_angle
    } else if dx.0 < 0 && dy.0 >= 0 {
        // Quadrant 2: [90, 180)
        Fixed::from_int(180) - base_angle
    } else if dx.0 < 0 && dy.0 < 0 {
        // Quadrant 3: [180, 270)
        Fixed::from_int(180) + base_angle
    } else {
        // Quadrant 4: [270, 360)
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
        // Close enough — snap to target
        normalize_angle(target)
    } else {
        // Turn by max_turn in the correct direction
        if diff.0 > 0 {
            normalize_angle(current + max_turn)
        } else {
            normalize_angle(current - max_turn)
        }
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
        assert_eq!(angle, Fixed::from_int(0)); // 0° = right
    }

    #[test]
    fn test_compute_angle_up() {
        let from = FixedVec2::new(Fixed::ZERO, Fixed::ZERO);
        let to = FixedVec2::new(Fixed::ZERO, Fixed::from_int(10));
        let angle = compute_angle_between(from, to);
        // Should be close to 90°
        let diff = (angle.0 - Fixed::from_int(90).0).abs();
        assert!(diff <= 2 * 256 / 10, "expected ~90°, got {}", angle.to_float());
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
```

### Step 2.2: Register `facing` module in lib.rs

Add `pub mod facing;` after the existing module declarations.

### Step 2.3: Run tests

Run: `cargo test -p simulation -- facing`
Expected: all tests pass.

**Commit:** `feat(facing): add angle utility functions with fixed-point atan2`

---

## Task 3: Facing Direction — Turn System

**Files:**
- Modify: `simulation/src/facing.rs` (add `facing_turn_system`)
- Modify: `simulation/src/soldier/mod.rs` (add `FacingDirection` to soldier spawn)
- Modify: `simulation/src/lib.rs` (register system in `run_tick`)

### Step 3.1: Add `facing_turn_system` to facing.rs

```rust
use bevy_ecs::world::World;
use crate::soldier::{LogicalPosition, Movement, SoldierMarker};
use crate::combat::config::CombatGlobalConfig;

/// Each tick, update each soldier's facing direction toward their movement target.
/// Turn rate is fixed at `turn_rate_ticks_per_full_rotation` ticks per full rotation.
pub fn facing_turn_system(world: &mut World) {
    let turn_rate = {
        let config = world.resource::<CombatGlobalConfig>();
        let ticks_per_rotation = config.facing.turn_rate_ticks_per_full_rotation as i64;
        if ticks_per_rotation == 0 {
            Fixed::from_int(360)
        } else {
            Fixed::from_int(360) / Fixed(ticks_per_rotation * FIXED_ONE)
        }
    };

    // Collect soldiers that need to turn
    let mut updates: Vec<(bevy_ecs::entity::Entity, Fixed)> = Vec::new();

    for (entity, pos, movement, _marker) in world
        .query::<(bevy_ecs::entity::Entity, &LogicalPosition, &Movement, &SoldierMarker)>()
        .iter(world)
    {
        // Determine target position to face toward
        let target_pos = if let Some(target_id) = movement.target {
            // Face toward attack target
            let mut found = None;
            for (other_entity, other_pos, other_id) in world
                .query::<(&LogicalPosition, &crate::soldier::UnitIdComponent)>()
                .iter(world)
            {
                if other_id.0 == target_id {
                    found = Some(other_pos.0);
                    break;
                }
            }
            found
        } else if let Some(cmd_target) = movement.command_target {
            let mut found = None;
            for (other_entity, other_pos, other_id) in world
                .query::<(&LogicalPosition, &crate::soldier::UnitIdComponent)>()
                .iter(world)
            {
                if other_id.0 == cmd_target {
                    found = Some(other_pos.0);
                    break;
                }
            }
            found
        } else if let Some(waypoint) = movement.waypoint {
            Some(waypoint)
        } else {
            None
        };

        if let Some(target) = target_pos {
            let desired_angle = compute_angle_between(pos.0, target);
            let facing = world.get::<FacingDirection>(entity);
            if let Some(facing) = facing {
                let new_angle = turn_toward(facing.angle, desired_angle, turn_rate);
                updates.push((entity, new_angle));
            }
        }
    }

    for (entity, new_angle) in updates {
        if let Some(mut facing) = world.get_mut::<FacingDirection>(entity) {
            facing.angle = new_angle;
        }
    }
}
```

### Step 3.2: Add `FacingDirection` to soldier spawn in `city_spawn_system`

In `soldier/mod.rs`, in `city_spawn_system` where the soldier entity is spawned (around line 330), add:

```rust
.insert(FacingDirection { angle: Fixed::ZERO }); // face right by default
```

### Step 3.3: Register `facing_turn_system` in `run_tick`

In `lib.rs`, add after `combat_engagement_system` (phase 2):

```rust
facing::facing_turn_system(world);
```

This goes between engagement and movement, so units face the right direction before moving.

### Step 3.4: Run tests

Run: `cargo test -p simulation`
Expected: existing tests still pass.

**Commit:** `feat(facing): implement facing_turn_system with deterministic turn rate`

---

## Task 4: Facing Direction — Movement Speed Modifier

**Files:**
- Modify: `simulation/src/soldier/mod.rs` (`soldier_movement_system`)

### Step 4.1: Modify `soldier_movement_system` to use facing-based speed

In the movement system (line ~171), after computing the movement direction and before applying movement, add facing-based speed reduction:

```rust
// After computing desired_direction and before applying movement:

// Get facing angle and compute deviation
if let Some(facing) = world.get::<FacingDirection>(entity) {
    let desired_angle = compute_angle_between(pos.0, target_pos);
    let deviation = angle_distance(facing.angle, desired_angle);
    // Linear speed reduction: 0° deviation = 100% speed, 180° = 0% speed
    let speed_factor = Fixed::ONE - (deviation / Fixed::from_int(180));
    let speed_factor = speed_factor.max(Fixed::ZERO); // clamp to 0
    effective_speed = effective_speed * speed_factor;
}
```

This replaces the existing movement speed calculation. The `effective_speed` is computed after all other modifiers (SlowDebuff, ShieldUp penalty) and then further reduced by facing deviation.

### Step 4.2: Verify cavalry movement

Cavalry has speed 200. With facing deviation, turning will slow them down proportionally. This is correct per spec — cavalry also need to turn.

**Commit:** `feat(movement): apply facing-based speed reduction to soldier movement`

---

## Task 5: Archer Chase Behavior

**Files:**
- Modify: `simulation/src/combat/mod.rs` (`combat_engagement_system`)
- Modify: `simulation/src/soldier/mod.rs` (`soldier_movement_system`)

### Step 5.1: Modify `combat_engagement_system` to include archers

Currently archers are excluded from engagement (line ~67). Change this so archers are included but with special behavior:

```rust
// Remove the archer exclusion:
// OLD: if stype.0 == SoldierType::Archer { continue; }
// NEW: Don't skip archers — they now participate in engagement
```

For archers, the engagement system should set `Movement.target` to the nearest enemy but keep state as `Fighting` (they'll handle movement in the movement system).

### Step 5.2: Modify `soldier_movement_system` for archer chase

Replace the current archer skip logic:

```rust
// OLD: if stype.0 == SoldierType::Archer && state.0 == SoldierState::Fighting { continue; }
// NEW:
if stype.0 == SoldierType::Archer && state.0 == SoldierState::Fighting {
    // Archer in combat: check if target is in range
    if let Some(target_id) = movement.target {
        // Find target position
        let target_pos = /* lookup target position */;
        let archer_config = soldier_config.get(SoldierType::Archer);
        let attack_range = archer_config.compute_attack_range(level.level);
        let dist_sq = (target_pos - pos.0).length_squared();
        let range_sq = Fixed::from_int(attack_range as i32 * attack_range as i32);

        if dist_sq <= range_sq {
            // Target in range — don't move, continue shooting
            continue;
        }
        // Target out of range — fall through to normal movement (chase)
    }
}
```

This allows archers to chase targets that leave their attack range.

**Commit:** `feat(archer): implement chase-to-range-edge behavior`

---

## Task 6: Archer Multi-Shot

**Files:**
- Modify: `simulation/src/combat/mod.rs` (`archer_attack_system`)

### Step 6.1: Add multi-shot logic to `archer_attack_system`

After the archer finds its primary target and before spawning the arrow, add multi-shot check:

```rust
// After computing primary target and before spawning arrow:

let multi_shot_config = &combat_config.archer_multi_shot;
let multi_shot_chance = (multi_shot_config.base_chance
    + level.level as f32 * multi_shot_config.per_level_bonus)
    .min(multi_shot_config.max_chance);

let rng = world.resource_mut::<DeterministicRng>();
let roll = rng.gen_probability();

if roll < multi_shot_chance {
    // Multi-shot: fire at 2-5 random different enemies
    let num_extra = 2 + (rng.gen_probability() * 4.0) as u32; // 2-5 total shots
    let num_extra = num_extra.min(enemy_soldiers_in_range.len() as u32);

    // Collect all enemies in range (excluding primary target)
    let mut candidates: Vec<(UnitId, FixedVec2)> = enemy_soldiers_in_range
        .iter()
        .filter(|(id, _)| *id != primary_target_id)
        .cloned()
        .collect();

    // Shuffle using deterministic RNG
    for i in (1..candidates.len()).rev() {
        let j = (rng.gen_probability() * (i + 1) as f32) as usize;
        candidates.swap(i, j);
    }

    // Fire at up to num_extra - 1 additional targets (primary already fires)
    for (target_id, target_pos) in candidates.iter().take((num_extra - 1) as usize) {
        // Spawn arrow toward this target (same logic as primary arrow)
        let direction = /* compute direction with spread */;
        // ... spawn Arrow entity
    }
}
```

### Step 6.2: Verify cooldown

After multi-shot, the archer's cooldown is reset to `interval_ticks` (same as single shot). This is already handled by the existing code after arrow spawn.

**Commit:** `feat(archer): implement multi-shot with random target selection`

---

## Task 7: Attack Windup System (边走边打)

**Files:**
- Modify: `simulation/src/soldier/mod.rs` (add `attack_windup_system`)
- Modify: `simulation/src/combat/mod.rs` (modify `melee_attack_system` and `archer_attack_system`)
- Modify: `simulation/src/lib.rs` (register system)

### Step 7.1: Add `AttackWindup` to soldier spawn

In `city_spawn_system`, add to the soldier entity builder:

```rust
.insert(AttackWindup { remaining_ticks: 0, target: None });
```

### Step 7.2: Modify `melee_attack_system` for windup

Replace the current attack logic:

```rust
// When attacker's cooldown reaches 0:
// 1. Check if target is in range
// 2. If non-cavalry: start windup (set AttackWindup.remaining_ticks = windup_ticks)
// 3. If cavalry: attack immediately (no windup)

if let Some(windup) = world.get::<AttackWindup>(attacker_entity) {
    let is_cavalry = world.get::<SoldierTypeComponent>(attacker_entity).map(|s| s.0 == SoldierType::Cavalry).unwrap_or(false);
    let windup_config = &combat_config.attack_windup;

    if is_cavalry && windup_config.cavalry_no_windup {
        // Cavalry: attack immediately, no windup
        // ... apply damage as before
    } else if windup.remaining_ticks == 0 && !is_cavalry {
        // Non-cavalry: start windup
        if let Some(mut windup) = world.get_mut::<AttackWindup>(attacker_entity) {
            windup.remaining_ticks = windup_config.windup_ticks;
            windup.target = Some(target_id);
        }
        // Don't attack yet — will attack when windup completes
    }
}
```

### Step 7.3: Add `attack_windup_system`

New system that decrements windup timers and fires the attack when windup completes:

```rust
pub fn attack_windup_system(world: &mut World) {
    // Collect entities with active windups
    let mut completions: Vec<(Entity, UnitId)> = Vec::new();

    for (entity, windup) in world.query::<(Entity, &AttackWindup)>().iter(world) {
        if windup.remaining_ticks > 0 {
            if let Some(mut w) = world.get_mut::<AttackWindup>(entity) {
                w.remaining_ticks -= 1;
                if w.remaining_ticks == 0 {
                    if let Some(target) = windup.target {
                        completions.push((entity, target));
                    }
                }
            }
        }
    }

    // Apply attacks for completed windups
    for (attacker_entity, target_id) in completions {
        // Apply melee damage (same logic as melee_attack_system)
        // ... find target, compute damage, apply
        // Reset cooldown
        if let Some(mut attack) = world.get_mut::<Attack>(attacker_entity) {
            attack.cooldown_remaining = attack.interval_ticks;
        }
    }
}
```

### Step 7.4: Register in `run_tick`

Add `attack_windup_system` after `melee_attack_system` in the tick order.

**Commit:** `feat(combat): implement attack windup system for non-cavalry units`

---

## Task 8: Shield System — Core Data Structures

**Files:**
- Modify: `simulation/src/types.rs` (update `ShieldState`)
- Modify: `simulation/src/soldier/mod.rs` (add `ShieldItem`, `DroppedShield`, modify `ShieldComponent`)

### Step 8.1: Update `ShieldState` enum

Rename `ShieldUp` to `Blocking`:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ShieldState {
    Normal,
    Blocking, // was ShieldUp
}
```

### Step 8.2: Add `ShieldItem` component

```rust
/// Shield item with independent HP. Can be held by a soldier or dropped on ground.
#[derive(Component, Clone, Debug)]
pub struct ShieldItem {
    pub hp: u32,
    pub max_hp: u32,
}
```

### Step 8.3: Add `DroppedShield` component

```rust
/// A shield dropped on the ground after a soldier dies.
#[derive(Component, Clone, Debug)]
pub struct DroppedShield {
    pub shield: ShieldItem,
    pub position: FixedVec2,
    pub drop_tick: u32,
    pub owner_faction: Option<Faction>,
}
```

### Step 8.4: Modify `ShieldComponent`

Change from `ShieldComponent(pub ShieldState)` to:

```rust
/// Shield state on a soldier. Only present if the soldier has a shield.
#[derive(Component)]
pub struct ShieldComponent {
    pub state: ShieldState,
}
```

### Step 8.5: Update all references to `ShieldComponent`

Find all uses of `ShieldComponent(ShieldState::ShieldUp)` and update to use the new struct syntax.

**Commit:** `feat(shield): add ShieldItem, DroppedShield, update ShieldState`

---

## Task 9: Shield System — Passive Block

**Files:**
- Modify: `simulation/src/combat/mod.rs` (`melee_attack_system`)

### Step 9.1: Add passive block check to damage application

**Ordering rule (from spec):** Cavalry dodge is checked FIRST. Passive block is checked ONLY if dodge did not trigger. This ensures dodge can trigger Fearless buff correctly.

In `melee_attack_system`, the damage pipeline becomes:
1. Cavalry dodge check (existing) → if dodged, skip all further checks, grant Fearless
2. Shield passive block check (new) → if blocked, damage goes to shield HP
3. Manual block frontal check (if Blocking state, see Task 10)
4. Apply remaining damage to soldier HP

In `melee_attack_system`, before applying damage to the target, check for passive block:

```rust
// After cavalry dodge check, before applying damage:

// Shield passive block check
let shield_config = &combat_config.shield;
let has_shield = world.get::<ShieldComponent>(target_entity).is_some();

if has_shield {
    let mut rng = world.resource_mut::<DeterministicRng>();
    let block_roll = rng.gen_probability();

    if block_roll < shield_config.passive_block_chance {
        // Passive block triggered — damage goes to shield HP
        if let Some(mut shield) = world.get_mut::<ShieldItem>(target_entity) {
            let shield_damage = damage;
            if shield.hp <= shield_damage {
                // Shield breaks
                shield.hp = 0;
                // Remove shield components
                world.entity_mut(target_entity).remove::<ShieldComponent>();
                world.entity_mut(target_entity).remove::<ShieldItem>();
            } else {
                shield.hp -= shield_damage;
            }
            damage = 0; // soldier takes no damage
        }
    }
}
```

### Step 9.2: Apply same logic to arrow damage

In `arrow_movement_system`, when an arrow hits a soldier, add the same passive block check before applying damage.

**Commit:** `feat(shield): implement passive block with 40% chance`

---

## Task 10: Shield System — Manual Block (Infantry)

**Files:**
- Modify: `simulation/src/combat/mod.rs` (damage application)
- Modify: `simulation/src/soldier/mod.rs` (movement speed modifier)

### Step 10.1: Add frontal angle check for manual block

In damage application (melee and arrow), when the target has `ShieldState::Blocking`:

```rust
if let Some(shield_comp) = world.get::<ShieldComponent>(target_entity) {
    if shield_comp.state == ShieldState::Blocking {
        // Manual block: check if damage comes from frontal 120° arc
        let facing = world.get::<FacingDirection>(target_entity);
        if let Some(facing) = facing {
            // Compute angle from target to attacker
            let attack_angle = compute_angle_between(target_pos, attacker_pos);
            let deviation = angle_distance(facing.angle, attack_angle);
            let frontal_half = Fixed::from_int(shield_config.frontal_angle_deg as i32 / 2);

            if deviation <= frontal_half {
                // Frontal hit — 100% absorbed by shield
                if let Some(mut shield) = world.get_mut::<ShieldItem>(target_entity) {
                    if shield.hp <= damage {
                        shield.hp = 0;
                        world.entity_mut(target_entity).remove::<ShieldComponent>();
                        world.entity_mut(target_entity).remove::<ShieldItem>();
                    } else {
                        shield.hp -= damage;
                    }
                    damage = 0;
                }
            }
            // Non-frontal: damage goes through (passive block still applies separately)
        }
    }
}
```

### Step 10.2: Apply speed and attack speed penalties for Blocking state

In `soldier_movement_system`, when the soldier has `ShieldState::Blocking`:

```rust
if let Some(shield_comp) = world.get::<ShieldComponent>(entity) {
    if shield_comp.state == ShieldState::Blocking {
        effective_speed = effective_speed.min(Fixed::from_int(shield_config.speed_penalty as i32));
    }
}
```

In `melee_attack_system`, when attacker has `ShieldState::Blocking`:

```rust
if let Some(shield_comp) = world.get::<ShieldComponent>(attacker_entity) {
    if shield_comp.state == ShieldState::Blocking {
        // Override cooldown to attack_speed_penalty
        attack.interval_ticks = shield_config.attack_speed_penalty;
    }
}
```

**Commit:** `feat(shield): implement manual block with frontal 120° damage absorption`

---

## Task 11: Shield System — Spawn, Drop, Pickup, Decay

**Files:**
- Modify: `simulation/src/soldier/mod.rs`

### Step 11.1: Infantry spawns with shield

In `city_spawn_system`, when spawning an Infantry:

```rust
if stype == SoldierType::Infantry {
    entity_commands
        .insert(ShieldItem { hp: shield_config.initial_hp, max_hp: shield_config.initial_hp })
        .insert(ShieldComponent { state: ShieldState::Normal });
}
```

### Step 11.2: Shield drop on death

In the death handling code (where soldiers are despawned on kill), before despawning:

```rust
// Check if dying soldier has a shield
if let Some(shield_item) = world.get::<ShieldItem>(dying_entity) {
    let pos = world.get::<LogicalPosition>(dying_entity).map(|p| p.0).unwrap_or(FixedVec2::ZERO);
    let faction = world.get::<FactionComponent>(dying_entity).map(|f| f.0);

    // Spawn DroppedShield entity
    let dropped = DroppedShield {
        shield: shield_item.clone(),
        position: pos,
        drop_tick: current_tick,
        owner_faction: faction,
    };
    // Insert as new entity
    world.spawn(dropped);
}
```

### Step 11.3: Add `shield_pickup_system`

```rust
pub fn shield_pickup_system(world: &mut World) {
    // Collect all dropped shields
    let dropped_shields: Vec<(Entity, FixedVec2, ShieldItem)> = world
        .query::<(Entity, &DroppedShield)>()
        .iter(world)
        .map(|(e, d)| (e, d.position, d.shield.clone()))
        .collect();

    // For each soldier without an archer type, check nearby dropped shields
    for (soldier_entity, pos, stype, _marker) in world
        .query::<(Entity, &LogicalPosition, &SoldierTypeComponent, &SoldierMarker)>()
        .iter(world)
    {
        if stype.0 == SoldierType::Archer {
            continue; // Archers can't pick up shields
        }

        let collision_radius = soldier_config.get(stype.0).collision_radius;
        let pickup_range = Fixed::from_int(collision_radius as i32);

        let has_shield = world.get::<ShieldItem>(soldier_entity).is_some();

        for (dropped_entity, dropped_pos, dropped_shield) in &dropped_shields {
            let dist_sq = (pos.0 - *dropped_pos).length_squared();
            let range_sq = pickup_range * pickup_range;

            if dist_sq <= range_sq {
                if !has_shield {
                    // Pick up directly
                    world.entity_mut(soldier_entity).insert(ShieldItem {
                        hp: dropped_shield.hp,
                        max_hp: dropped_shield.max_hp,
                    });
                    world.entity_mut(soldier_entity).insert(ShieldComponent {
                        state: ShieldState::Normal,
                    });
                    world.despawn(*dropped_entity);
                    break;
                } else {
                    // Compare HP — keep the better one
                    let current_shield = world.get::<ShieldItem>(soldier_entity).unwrap();
                    if dropped_shield.hp > current_shield.hp {
                        world.entity_mut(soldier_entity).insert(ShieldItem {
                            hp: dropped_shield.hp,
                            max_hp: dropped_shield.max_hp,
                        });
                        world.despawn(*dropped_entity);
                        break;
                    }
                }
            }
        }
    }
}
```

### Step 11.4: Add `shield_decay_system`

```rust
pub fn shield_decay_system(world: &mut World, current_tick: u32) {
    let config = world.resource::<CombatGlobalConfig>();
    let survive_ticks = config.shield.drop_survive_ticks;
    let anim_ticks = config.shield.disappear_animation_ticks;

    let mut to_despawn: Vec<Entity> = Vec::new();

    for (entity, dropped) in world.query::<(Entity, &DroppedShield)>().iter(world) {
        let age = current_tick.saturating_sub(dropped.drop_tick);
        if age >= survive_ticks + anim_ticks {
            to_despawn.push(entity);
        }
    }

    for entity in to_despawn {
        world.despawn(entity);
    }
}
```

### Step 11.5: Modify `aura_heal_system` for shield HP

In `aura_heal_system`, after healing soldier HP, also heal shield HP:

```rust
if let Some(mut shield) = world.get_mut::<ShieldItem>(soldier_entity) {
    shield.hp = (shield.hp + heal_amount).min(shield.max_hp);
}
```

### Step 11.6: Register systems in `run_tick`

Add to `lib.rs`:
- `shield_pickup_system` after `city_interaction_system`
- `shield_decay_system` before `ai_decide`

**Commit:** `feat(shield): implement spawn, drop, pickup, decay, and aura heal for shields`

---

## Task 12: Archer — Remove Close-Range Penalty

**Files:**
- Modify: `simulation/src/combat/mod.rs`
- Modify: `simulation/src/combat/config.rs`
- Modify: `content/combat.ron`

### Step 12.1: Remove `archer_melee_range` and `archer_melee_damage_mult` from config

In `combat/config.rs`, remove these fields from `CombatGlobalConfig`:
```rust
// DELETE:
pub archer_melee_range: u32,
pub archer_melee_damage_mult: f32,
```

### Step 12.2: Remove close-range penalty from `archer_attack_system`

In `combat/mod.rs`, remove any code that checks `archer_melee_range` and applies `archer_melee_damage_mult` to arrow damage.

### Step 12.3: Update `content/combat.ron`

Remove the two fields:
```ron
// DELETE:
archer_melee_range: 50,
archer_melee_damage_mult: 0.85,
```

**Commit:** `refactor(archer): remove close-range damage penalty`

---

## Task 13: Update Command Handling

**Files:**
- Modify: `simulation/src/command.rs`
- Modify: `simulation/src/soldier/mod.rs` (`consume_commands_system`)

### Step 13.1: Update `SetShield` command handling

In `consume_commands_system`, update the `SetShield` handler:

```rust
Action::SetShield { unit, state } => {
    // Find the soldier entity
    // Only infantry can toggle shield
    if let Some(entity) = find_entity_by_unit_id(world, unit) {
        if let Some(mut shield) = world.get_mut::<ShieldComponent>(entity) {
            shield.state = state;
        }
    }
}
```

The `ShieldState` in the `Action` enum should now use the updated variants (`Normal`, `Blocking`).

**Commit:** `refactor(command): update SetShield handling for new ShieldState`

---

## Task 14: Integration Tests

**Files:**
- Create: `simulation/src/tests/shield_integration_test.rs` (or add to existing test file)

### Step 14.1: Test facing direction turn

```rust
#[test]
fn test_facing_turn_toward_target() {
    let mut world = init_test_world();
    // Spawn a soldier facing 0° (right)
    // Set target to be above (90°)
    // Run several ticks
    // Verify facing angle approaches 90°
}
```

### Step 14.2: Test passive block

```rust
#[test]
fn test_passive_block_reduces_shield_hp() {
    let mut world = init_test_world();
    // Spawn infantry with shield (HP 1500)
    // Deal damage with 100% passive block chance (config override)
    // Verify shield HP decreased, soldier HP unchanged
}
```

### Step 14.3: Test manual block frontal

```rust
#[test]
fn test_manual_block_frontal_absorbs_damage() {
    let mut world = init_test_world();
    // Spawn infantry in Blocking state facing 0°
    // Attack from 0° (frontal) — verify shield absorbs
    // Attack from 180° (behind) — verify soldier takes damage
}
```

### Step 14.4: Test shield drop and pickup

```rust
#[test]
fn test_shield_drops_on_death_and_can_be_picked_up() {
    let mut world = init_test_world();
    // Spawn infantry with shield
    // Kill the infantry
    // Verify DroppedShield entity exists
    // Spawn militia near the dropped shield
    // Run one tick
    // Verify militia now has ShieldItem
}
```

### Step 14.5: Test multi-shot

```rust
#[test]
fn test_multi_shot_fires_at_multiple_targets() {
    let mut world = init_test_world();
    // Spawn archer with 100% multi-shot chance
    // Spawn 5 enemies in range
    // Run one tick
    // Verify multiple Arrow entities created
}
```

### Step 14.6: Test archer chase

```rust
#[test]
fn test_archer_chases_target_out_of_range() {
    let mut world = init_test_world();
    // Spawn archer in Fighting state with target
    // Move target out of attack range
    // Run one tick
    // Verify archer position changed (moved toward target)
}
```

**Commit:** `test: add integration tests for shield, facing, multi-shot, and archer chase`

---

## Task 15: Design Document Sync

**Files:**
- Modify: `docs/superpowers/specs/2026-06-05-rts-game-design.md`

### Step 15.1: Update arrow flight description

Change "400px/s 追踪速度" to "固定方向飞行，速度 400 单位/秒".

### Step 15.2: Update archer range

Change "600px" to "380px (Lv.1) → 600px (Lv.4)，线性缩放".

### Step 15.3: Remove archer close-range penalty

Remove "≤50px 伤害减少 15%" section.

### Step 15.4: Remove infantry archer damage reduction

Remove "受到弓兵伤害减少 10%" section.

### Step 15.5: Update shield mode description

Replace old shield description with new shield system summary.

### Step 15.6: Add facing direction and attack-while-moving sections

Add new sections describing the facing system and attack windup mechanics.

**Commit:** `docs: sync game design doc with confirmed spec changes`
