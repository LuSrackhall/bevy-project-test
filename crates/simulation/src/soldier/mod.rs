pub mod config;
pub mod spatial_hash;

use bevy_ecs::world::World;
use bevy_ecs::entity::Entity;
use bevy_ecs::component::Component;
use std::collections::{HashMap, HashSet};
use crate::types::*;
use crate::events::*;
use crate::command::*;
use crate::soldier::config::SoldierConfig;
use crate::city::config::CityGlobalConfig;
use crate::combat::config::CombatGlobalConfig;
use crate::soldier::spatial_hash::SpatialHash;
use crate::facing;

// ══════════ Components ══════════

#[derive(Component, Clone, Debug)]
pub struct UnitIdComponent(pub UnitId);
#[derive(Component, Clone, Debug)] pub struct SoldierMarker;
#[derive(Component, Clone, Debug)] pub struct CityMarker;
#[derive(Component, Clone, Debug)] pub struct WaypointMarker;
#[derive(Component, Clone, Debug)] pub struct LogicalPosition(pub FixedVec2);
#[derive(Component, Clone, Debug)] pub struct Movement { pub speed: u32, pub target: Option<UnitId>, pub command_target: Option<UnitId>, pub waypoint: Option<FixedVec2>, pub force_move: bool }
#[derive(Component, Clone, Debug)] pub struct SeekStance { pub active: bool, pub seek_range: u32 }
#[derive(Component, Clone, Debug)] pub struct Health { pub current: u32, pub max: u32 }
#[derive(Component, Clone, Debug)] pub struct Attack { pub damage: u32, pub range: u32, pub interval_ticks: u32, pub cooldown_remaining: u32 }
#[derive(Component, Clone, Debug)] pub struct FactionComponent(pub Faction);
#[derive(Component, Clone, Debug)] pub struct SoldierTypeComponent(pub SoldierType);
#[derive(Component, Clone, Debug)] pub struct Level { pub level: u32, pub exp: u32 }
/// Shield state on a soldier. Only present if the soldier has a shield.
#[derive(Component, Clone, Debug)]
pub struct ShieldComponent {
    pub state: ShieldState,
}
#[derive(Component, Clone, Debug)] pub struct CityOrigin(pub UnitId);
#[derive(Component, Clone, Debug)] pub struct SlowDebuff { pub stacks: u32, pub remaining_ticks: u32 }
#[derive(Component, Clone, Debug)] pub struct FearlessBuff { pub remaining_ticks: u32 }
#[derive(Component, Clone, Debug)] pub struct SoldierStateComponent(pub SoldierState);
#[derive(Component, Clone, Debug)] pub struct CityComponent { pub level: u32, pub max_level: u32, pub health_current: u32, pub health_max: u32, pub population: u32, pub max_population: u32, pub spawn_type: SoldierType, pub spawn_cooldown: u32, pub level_exp: u64, pub last_attacker_faction: Option<Faction>, pub arrow_damage_acc: u32 }
#[derive(Component, Clone, Debug)] pub struct CityRadius(pub u32);
#[derive(Component, Clone, Debug)] pub struct AuraHealComponent { pub base_heal: u32, pub per_level_heal: u32 }
#[derive(Component, Clone, Debug)] pub struct SpawnDirection(pub FixedVec2);

// ══════════ Helpers ══════════

const FIXED_ONE: i64 = 256;
const TICK_DURATION: Fixed = Fixed(12); // 256/20 ≈ 12.8

fn integer_sqrt(n: i64) -> i64 {
    if n <= 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + n / x) / 2; }
    x
}

pub fn find_entity_by_unit_id(world: &mut World, unit_id: UnitId) -> Option<Entity> {
    let mut query = world.query::<(Entity, &UnitIdComponent)>();
    for (entity, id_comp) in query.iter(world) {
        if id_comp.0 == unit_id { return Some(entity); }
    }
    None
}

pub fn find_unit_id(world: &World, entity: Entity) -> Option<UnitId> {
    world.entity(entity).get::<UnitIdComponent>().map(|c| c.0)
}

// ══════════ System: consume_commands ══════════

pub fn consume_commands_system(world: &mut World, tick: u32) {
    let commands: Vec<GameCommand> = {
        let mut buf = world.resource_mut::<CommandBuffer>();
        buf.take_for_tick(tick)
    };

    for cmd in commands {
        match cmd.action {
            Action::MoveTo { unit, target } => apply_movement(world, unit, target, true),
            Action::ForceMove { unit, target } => apply_movement(world, unit, target, true),
            Action::Attack { unit, target } => {
                if let Some(e) = find_entity_by_unit_id(world, unit) {
                    let s = get_speed(world, e);
                    world.entity_mut(e).insert(Movement { speed: s, target: Some(target), command_target: Some(target), waypoint: None, force_move: false });
                    world.entity_mut(e).insert(SoldierStateComponent(SoldierState::Fighting));
                }
            }
            Action::ReturnToCity { unit, city } => {
                if let Some(e) = find_entity_by_unit_id(world, unit) {
                    let s = get_speed(world, e);
                    world.entity_mut(e).insert(Movement { speed: s, target: Some(city), command_target: Some(city), waypoint: None, force_move: false });
                    world.entity_mut(e).insert(SoldierStateComponent(SoldierState::Moving));
                }
            }
            Action::SetShield { unit, state } => {
                if let Some(e) = find_entity_by_unit_id(world, unit) {
                    world.entity_mut(e).insert(ShieldComponent { state });
                }
            }
            Action::SetSpawnType { city, soldier_type } => {
                if let Some(e) = find_entity_by_unit_id(world, city) {
                    if let Some(mut cc) = world.entity_mut(e).get_mut::<CityComponent>() {
                        cc.spawn_type = soldier_type;
                    }
                }
            }
            Action::SetSeekStance { scope, seek_range, unit_ids } => {
                apply_seek_stance(world, cmd.tick, scope, seek_range, unit_ids);
            }
            Action::NoOp => {}
        }
    }
}

fn apply_movement(world: &mut World, unit: UnitId, target: FixedVec2, force: bool) {
    if let Some(e) = find_entity_by_unit_id(world, unit) {
        let s = get_speed(world, e);
        world.entity_mut(e).insert(Movement { speed: s, target: None, command_target: None, waypoint: Some(target), force_move: force });
        world.entity_mut(e).insert(SoldierStateComponent(SoldierState::Moving));
    }
}

fn get_speed(world: &World, entity: Entity) -> u32 {
    if let Some(st) = world.entity(entity).get::<SoldierTypeComponent>() {
        world.resource::<SoldierConfig>().get(st.0).speed
    } else { 80 }
}

/// Apply a SetSeekStance command.
/// - If `unit_ids` is non-empty: update only those units (selection mode).
/// - If `unit_ids` is empty: update GlobalSeekDirective and all matching soldiers.
fn apply_seek_stance(world: &mut World, tick: u32, scope: SeekScope, seek_range: u32, unit_ids: Vec<UnitId>) {
    let active = seek_range > 0;
    let new_stance = SeekStance { active, seek_range };

    if !unit_ids.is_empty() {
        // Selection mode: update only specified units, don't touch GlobalSeekDirective
        for uid in unit_ids {
            if let Some(e) = find_entity_by_unit_id(world, uid) {
                world.entity_mut(e).insert(new_stance.clone());
            }
        }
    } else {
        // Global mode: update GlobalSeekDirective and all matching Player soldiers
        {
            let mut directives = world.resource_mut::<GlobalSeekDirective>();
            directives.0.push(SeekDirective {
                scope: scope.clone(),
                seek_range,
                issue_tick: tick,
            });
        }

        // Collect matching Player-faction soldiers and update their SeekStance
        let matches: Vec<Entity> = {
            let mut q = world.query::<(Entity, &FactionComponent, &SoldierTypeComponent, &SoldierMarker)>();
            q.iter(world)
                .filter(|(_, fac, _, _)| fac.0 == Faction::Player)
                .filter(|(_, _, st, _)| match &scope {
                    SeekScope::All => true,
                    SeekScope::ByType(t) => st.0 == *t,
                })
                .map(|(e, _, _, _)| e)
                .collect()
        };
        for e in matches {
            world.entity_mut(e).insert(new_stance.clone());
        }
    }
}

// ══════════ System: soldier_movement (pure movement, single-pass) ══════════

pub fn soldier_movement_system(world: &mut World) {
    let combat_config = world.resource::<CombatGlobalConfig>().clone();
    let soldier_config = world.resource::<SoldierConfig>().clone();

    // Build position lookup from ALL entities with UnitIdComponent
    let positions: HashMap<UnitId, FixedVec2> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition)>();
        q.iter(world).map(|(_, id, pos)| (id.0, pos.0)).collect()
    };

    let mut soldier_updates: Vec<(Entity, FixedVec2)> = Vec::new();
    let mut arrivals: Vec<(Entity, u32)> = Vec::new();

    {
        let mut q = world.query::<(Entity, &LogicalPosition, &Movement, &SoldierTypeComponent, &SoldierStateComponent, Option<&SlowDebuff>, Option<&ShieldComponent>, Option<&FacingDirection>)>();
        for (e, pos, mov, st, sst, slow, shield, facing_dir) in q.iter(world) {
            if st.0 == SoldierType::Archer && sst.0 == SoldierState::Fighting {
                // Check if target is in attack range — if so, hold position and keep shooting
                if let Some(target_id) = mov.target {
                    if let Some(target_pos) = positions.get(&target_id) {
                        let archer_cfg = soldier_config.get(SoldierType::Archer);
                        let level = world.get::<Level>(e).map(|l| l.level).unwrap_or(1);
                        let attack_range = archer_cfg.compute_attack_range(level);
                        let range_sq = attack_range as i64 * attack_range as i64;
                        let dist_sq = (pos.0 - *target_pos).length_squared();
                        if dist_sq <= Fixed(range_sq) {
                            // Target in range — don't move, keep shooting
                            continue;
                        }
                        // Target out of range — fall through to normal movement (chase)
                    }
                } else {
                    // No target — stay put
                    continue;
                }
            }
            let mut speed = mov.speed as f32;
            if let Some(sl) = slow {
                let sc = &combat_config.slow_debuff;
                speed *= (sc.base_amount * sc.stack_mult.powi(sl.stacks as i32 - 1)).max(sc.max_reduction);
            }
            if let Some(sh) = shield {
                if sh.state == ShieldState::Blocking { speed -= combat_config.shield.speed_penalty as f32; }
            }
            if speed <= 0.0 { continue; }
            let mut speed_fixed = Fixed::from_float(speed);

            let is_cav = st.0 == SoldierType::Cavalry;
            let target_pos = if is_cav {
                mov.command_target.and_then(|tid| positions.get(&tid).copied())
                    .or_else(|| mov.target.and_then(|tid| positions.get(&tid).copied()))
            } else {
                mov.target.and_then(|tid| positions.get(&tid).copied())
            }.or(mov.waypoint);

            let Some(target_pos) = target_pos else { continue };

            // Facing-based speed reduction: moving sideways or backward is slower
            if let Some(facing) = facing_dir {
                let desired = facing::compute_angle_between(pos.0, target_pos);
                let deviation = facing::angle_distance(facing.angle, desired);
                let factor = Fixed::ONE - deviation / Fixed::from_int(180);
                let factor = factor.max(Fixed::ZERO);
                speed_fixed = speed_fixed * factor;
            }

            let delta = target_pos - pos.0;
            let dist_sq = delta.length_squared();
            let threshold = Fixed::from_int(5);
            if dist_sq < threshold * threshold {
                arrivals.push((e, mov.speed));
                continue;
            }

            let dist_internal = integer_sqrt(dist_sq.0 * FIXED_ONE);
            if dist_internal <= 0 { continue; }
            let distance = Fixed(dist_internal);
            let move_amount = speed_fixed * TICK_DURATION;
            // Use i128 to compute (delta * move_amount) / distance without truncation.
            // Fixed-point division would truncate to 0 when distance > move_amount
            // (e.g. clicking far away), causing the unit to stand still.
            let dx = (delta.x.0 as i128 * move_amount.0 as i128) / distance.0 as i128;
            let dy = (delta.y.0 as i128 * move_amount.0 as i128) / distance.0 as i128;
            let new_pos = FixedVec2::new(
                Fixed(pos.0.x.0 + dx as i64),
                Fixed(pos.0.y.0 + dy as i64),
            );
            soldier_updates.push((e, new_pos));
        }
    }

    for (e, new_pos) in soldier_updates {
        world.entity_mut(e).insert(LogicalPosition(new_pos));
    }
    for (e, speed) in arrivals {
        world.entity_mut(e).insert(Movement { speed, target: None, command_target: None, waypoint: None, force_move: false });
        world.entity_mut(e).insert(SoldierStateComponent(SoldierState::Moving));
    }
}

// ══════════ System: overlap_resolution (post-tick collision resolve) ══════════

pub fn overlap_resolution_system(world: &mut World) {
    let config = world.resource::<CombatGlobalConfig>().clone();
    let soldier_config = world.resource::<SoldierConfig>().clone();
    let max_iter = config.overlap_resolution.max_iterations;

    // Determine cell size from the max possible collision radius (cavalry=10)
    let max_radius = 10u32;
    let cell_size = Fixed::from_int(max_radius as i32 * 4);

    for _iter in 0..max_iter {
        // Build spatial hash from current positions + per-unit collision radii
        let mut hash = SpatialHash::new(cell_size);
        {
            let mut q = world.query::<(Entity, &LogicalPosition, &SoldierTypeComponent, &SoldierMarker)>();
            for (_, pos, st, _) in q.iter(world) {
                let cfg = soldier_config.get(st.0);
                hash.insert(pos.0, cfg.collision_radius);
            }
        }

        // Collect displacements using per-unit radii
        let mut displacements: Vec<(Entity, FixedVec2)> = Vec::new();
        {
            let mut q = world.query::<(Entity, &LogicalPosition, &SoldierTypeComponent, &SoldierMarker)>();
            for (e, pos, st, _) in q.iter(world) {
                let my_radius = soldier_config.get(st.0).collision_radius;
                let neighbors = hash.query_nearby(pos.0);
                let mut total_push = FixedVec2::ZERO;
                for (npos, nradius) in &neighbors {
                    if *npos == pos.0 { continue; } // skip self
                    let diff = pos.0 - *npos;
                    let dist_sq = diff.length_squared();
                    let dist = Fixed(integer_sqrt(dist_sq.0 * FIXED_ONE));
                    if dist.0 == 0 { continue; }
                    let min_dist = (my_radius + nradius) as i64 * FIXED_ONE;
                    let overlap = min_dist - dist.0;
                    if overlap > 0 {
                        let push = Fixed(overlap / 2);
                        total_push.x = total_push.x + (diff.x / dist) * push;
                        total_push.y = total_push.y + (diff.y / dist) * push;
                    }
                }
                if total_push.x.0 != 0 || total_push.y.0 != 0 {
                    let new_pos = FixedVec2::new(pos.0.x + total_push.x, pos.0.y + total_push.y);
                    displacements.push((e, new_pos));
                }
            }
        }

        if displacements.is_empty() { break; }

        for (e, new_pos) in displacements {
            world.entity_mut(e).insert(LogicalPosition(new_pos));
        }
    }
}

// ══════════ System: city_spawn ══════════

pub fn city_spawn_system(world: &mut World) {
    let soldier_config = world.resource::<SoldierConfig>().clone();
    let combat_config = world.resource::<CombatGlobalConfig>().clone();
    let seek_directives = world.resource::<GlobalSeekDirective>().clone();

    // Collect city entities and check spawn conditions
    let cities: Vec<(Entity, FixedVec2, Faction, SoldierType, u32, bool)> = {
        let mut q = world.query::<(Entity, &LogicalPosition, &FactionComponent, &CityComponent)>();
        q.iter(world)
            .filter(|(_, _, fac, _)| fac.0 != Faction::Neutral)
            .map(|(e, pos, fac, city)| {
                let can = city.population < city.max_population && city.spawn_cooldown == 0;
                (e, pos.0, fac.0, city.spawn_type, city.spawn_cooldown, can)
            }).collect()
    };

    for (entity, pos, faction, spawn_type, _cooldown, can_spawn) in cities {
        if can_spawn {
            let new_id = { world.resource_mut::<IdGenerator>().next() };
            let jx = (new_id.0 % 16) as i32 - 8;
            let jy = ((new_id.0 >> 4) % 16) as i32 - 8;
            let spawn_pos = FixedVec2::new(pos.x + Fixed::from_int(30 + jx), pos.y + Fixed::from_int(jy));

            // Update city population + cooldown
            {
                let mut em = world.entity_mut(entity);
                if let Some(mut city) = em.get_mut::<CityComponent>() {
                    city.population += 1;
                    let mult = soldier_config.get(spawn_type).spawn_speed_mult;
                    city.spawn_cooldown = if mult > 0.0 { ((60.0 / mult) as u32).max(1) } else { 60 };
                }
            }

            // Determine SeekStance from global directives (latest matching by issue_tick)
            let mut seek_stance = SeekStance { active: false, seek_range: 0 };
            let mut best_tick: u32 = 0;
            for d in &seek_directives.0 {
                let matches = match &d.scope {
                    SeekScope::All => true,
                    SeekScope::ByType(t) => *t == spawn_type,
                };
                if matches && d.issue_tick >= best_tick {
                    seek_stance = SeekStance { active: d.seek_range > 0, seek_range: d.seek_range };
                    best_tick = d.issue_tick;
                }
            }

            // Create soldier
            let cfg = soldier_config.get(spawn_type);
            let origin = find_nearest_city_uid(world, spawn_pos, faction);
            let soldier_entity = world.spawn((
                UnitIdComponent(new_id), SoldierMarker, LogicalPosition(spawn_pos),
                Movement { speed: cfg.speed, target: None, command_target: None, waypoint: None, force_move: false },
                seek_stance,
                Health { current: cfg.health, max: cfg.health },
                Attack { damage: cfg.attack, range: cfg.attack_range, interval_ticks: cfg.attack_interval_ticks, cooldown_remaining: cfg.attack_interval_ticks },
                FactionComponent(faction), SoldierTypeComponent(spawn_type),
                Level { level: 1, exp: 0 },
                CityOrigin(origin), SoldierStateComponent(SoldierState::Moving),
                crate::types::FacingDirection { angle: Fixed::ZERO },
            )).id();
            // Only Infantry spawns with a shield
            if spawn_type == SoldierType::Infantry {
                world.entity_mut(soldier_entity).insert(ShieldComponent { state: ShieldState::Normal });
                world.entity_mut(soldier_entity).insert(ShieldItem { hp: combat_config.shield.initial_hp, max_hp: combat_config.shield.initial_hp });
            }
            world.entity_mut(soldier_entity).insert(
                crate::types::AttackWindup { remaining_ticks: 0, target: None },
            );

            let mut events = world.resource_mut::<SimulationEvents>();
            events.spawned.push(UnitSpawned { unit_id: new_id, pos: spawn_pos, faction, unit_kind: UnitKind::Soldier(spawn_type) });
        } else {
            // Decrement cooldown
            if let Some(mut city) = world.entity_mut(entity).get_mut::<CityComponent>() {
                city.spawn_cooldown = city.spawn_cooldown.saturating_sub(1);
            }
        }
    }
}

/// Find the nearest city entity of a given faction. Filters by CityMarker to exclude soldiers.
fn find_nearest_city_uid(world: &mut World, pos: FixedVec2, faction: Faction) -> UnitId {
    let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &CityMarker)>();
    let mut best = UnitId(0);
    let mut best_d = i64::MAX;
    for (_, id, cp, cf, _) in q.iter(world) {
        if cf.0 == faction {
            let d = (cp.0 - pos).length_squared().0;
            if d < best_d { best_d = d; best = id.0; }
        }
    }
    best
}

// ══════════ System: city_capture_check ══════════

pub fn city_capture_check_system(world: &mut World) {
    let city_config = world.resource::<CityGlobalConfig>().clone();

    let captures: Vec<(Entity, UnitId, Faction, Faction)> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &CityComponent, &FactionComponent)>();
        q.iter(world)
            .filter(|(_, _, city, _)| city.health_current == 0)
            .map(|(e, id, city, fac)| {
                let nf = match fac.0 {
                    Faction::Player => Faction::Enemy,
                    Faction::Enemy => Faction::Player,
                    Faction::Neutral => city.last_attacker_faction.unwrap_or(Faction::Player),
                };
                (e, id.0, fac.0, nf)
            }).collect()
    };

    for (entity, city_id, old_faction, new_faction) in captures {
        let mut em = world.entity_mut(entity);
        let (nl, _nmax) = {
            if let Some(mut c) = em.get_mut::<CityComponent>() {
                let nl = c.level.saturating_sub(1).max(1);
                let nm = nl * city_config.level_hp_multiplier;
                c.level = nl;
                c.health_max = nm;
                c.health_current = (nm as f32 * city_config.capture_hp_ratio) as u32;
                c.population = 0;
                c.level_exp = 0;
                c.last_attacker_faction = None;
                c.arrow_damage_acc = 0;
                (nl, nm)
            } else { continue; }
        };
        em.insert(FactionComponent(new_faction));
        let r = (city_config.visual_radius_base + nl as f32 * city_config.visual_radius_per_level) as u32;
        em.insert(CityRadius(r));

        let mut events = world.resource_mut::<SimulationEvents>();
        events.captured.push(CityCaptured { city_id, old_faction, new_faction });
    }
}

// ══════════ System: city_interaction ══════════

pub fn city_interaction_system(world: &mut World) {
    let combat_config = world.resource::<CombatGlobalConfig>().clone();
    let city_config = world.resource::<CityGlobalConfig>().clone();

    #[derive(Clone, Copy)]
    struct CData { entity: Entity, uid: UnitId, pos: FixedVec2, faction: Faction, level: u32, max_level: u32, hp: u32, max_hp: u32, radius: u32 }
    #[derive(Clone, Copy)]
    struct SData { entity: Entity, pos: FixedVec2, faction: Faction, attack: u32, cmd_target: Option<UnitId>, target: Option<UnitId> }

    let cities: Vec<CData> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &CityComponent, &CityRadius)>();
        q.iter(world).map(|(e, id, p, f, c, r)| CData {
            entity: e, uid: id.0, pos: p.0, faction: f.0,
            level: c.level, max_level: c.max_level, hp: c.health_current,
            max_hp: c.health_max, radius: r.0,
        }).collect()
    };

    let soldiers: Vec<SData> = {
        let mut q = world.query::<(Entity, &LogicalPosition, &FactionComponent, &Attack, &Movement)>();
        q.iter(world).map(|(e, p, f, a, m)| SData {
            entity: e, pos: p.0, faction: f.0, attack: a.damage,
            cmd_target: m.command_target, target: m.target,
        }).collect()
    };

    let mut to_despawn: Vec<(Entity, Option<UnitId>)> = Vec::new();
    let mut origin_decrements: Vec<UnitId> = Vec::new();

    for si in soldiers {
        for ci in &cities {
            let ds = (si.pos - ci.pos).length_squared();
            let t = Fixed::from_int(ci.radius as i32 + 5);
            if ds > t * t { continue; }

            if si.faction != ci.faction {
                let dmg = (si.attack as f32 * combat_config.city_damage_per_soldier_ratio) as u32;
                if let Some(mut c) = world.entity_mut(ci.entity).get_mut::<CityComponent>() {
                    c.health_current = c.health_current.saturating_sub(dmg);
                    c.last_attacker_faction = Some(si.faction);
                }
                if let Some(o) = world.entity(si.entity).get::<CityOrigin>() { origin_decrements.push(o.0); }
                to_despawn.push((si.entity, None));
                break;
            } else {
                let is_targeted = si.cmd_target == Some(ci.uid) || si.target == Some(ci.uid);
                if is_targeted {
                    if ci.hp < ci.max_hp {
                        let heal = (ci.max_hp as f32 * city_config.heal_ratio) as u32;
                        if let Some(mut c) = world.entity_mut(ci.entity).get_mut::<CityComponent>() {
                            c.health_current = (c.health_current + heal).min(c.health_max);
                        }
                    } else if ci.level < ci.max_level {
                        let eg = (ci.max_hp as f32 * city_config.level_up_gain_ratio) as u64;
                        let req = (ci.max_hp as f32 * city_config.level_up_cost_multiplier * ci.level as f32) as u64;
                        let mut new_radius: Option<u32> = None;
                        if let Some(mut c) = world.entity_mut(ci.entity).get_mut::<CityComponent>() {
                            c.level_exp += eg;
                            if c.level_exp >= req {
                                c.level_exp -= req; c.level += 1;
                                c.health_max = c.level * city_config.level_hp_multiplier;
                                c.health_current = c.health_max;
                                c.max_population = c.level * city_config.base_population_per_level;
                                new_radius = Some((city_config.visual_radius_base + c.level as f32 * city_config.visual_radius_per_level) as u32);
                            }
                        }
                        if let Some(r) = new_radius {
                            world.entity_mut(ci.entity).insert(CityRadius(r));
                        }
                    }
                    if let Some(o) = world.entity(si.entity).get::<CityOrigin>() { origin_decrements.push(o.0); }
                    to_despawn.push((si.entity, None));
                    break;
                }
            }
        }
    }

    for uid in &origin_decrements {
        if let Some(oe) = find_entity_by_unit_id(world, *uid) {
            if let Some(mut c) = world.entity_mut(oe).get_mut::<CityComponent>() {
                c.population = c.population.saturating_sub(1);
            }
        }
    }

    let mut seen = std::collections::HashSet::new();
    for (se, _) in &to_despawn {
        if seen.contains(se) { continue; }
        seen.insert(*se);
        world.despawn(*se);
        let mut events = world.resource_mut::<SimulationEvents>();
        events.destroyed.push(UnitDestroyed { unit_id: UnitId(0), killer_id: None });
    }
}

// ══════════ System: aura_heal ══════════

pub fn aura_heal_system(world: &mut World) {
    let city_config = world.resource::<CityGlobalConfig>().clone();

    let cities: Vec<(FixedVec2, Faction, u32, u32)> = {
        let mut q = world.query::<(&LogicalPosition, &FactionComponent, &CityComponent, &CityRadius)>();
        q.iter(world).map(|(p, f, c, r)| (p.0, f.0, c.level, r.0)).collect()
    };

    let mut heals: Vec<(Entity, u32)> = Vec::new();
    {
        let mut q = world.query::<(Entity, &LogicalPosition, &FactionComponent, &Health)>();
        for (e, sp, sf, _) in q.iter(world) {
            let mut total = 0u32;
            for (cp, cf, cl, cr) in &cities {
                if *cf != sf.0 { continue; }
                let eff = Fixed::from_int(city_config.aura.base_radius as i32 + *cr as i32);
                if (sp.0 - *cp).length_squared() <= eff * eff {
                    total += city_config.aura.base_heal + (cl.saturating_sub(1)) * city_config.aura.per_level_heal;
                }
            }
            if total > 0 { heals.push((e, total)); }
        }
    }

    for (e, heal) in heals {
        if let Some(mut hp) = world.entity_mut(e).get_mut::<Health>() {
            hp.current = (hp.current + heal).min(hp.max);
        }
        if let Some(mut shield) = world.entity_mut(e).get_mut::<ShieldItem>() {
            shield.hp = (shield.hp + heal).min(shield.max_hp);
        }
    }
}

// ══════════ System: shield_pickup ══════════

pub fn shield_pickup_system(world: &mut World) {
    let soldier_config = world.resource::<SoldierConfig>().clone();

    // Collect all dropped shields
    let dropped: Vec<(Entity, FixedVec2, u32, u32)> = {
        let mut q = world.query::<(Entity, &DroppedShield)>();
        q.iter(world)
            .map(|(e, d)| (e, d.position, d.shield.hp, d.shield.max_hp))
            .collect()
    };

    if dropped.is_empty() { return; }

    // Collect soldier data for pickup checks
    struct SoldierData { entity: Entity, pos: FixedVec2, stype: SoldierType, has_shield: bool, shield_hp: u32 }
    let soldiers: Vec<SoldierData> = {
        let mut q = world.query::<(Entity, &LogicalPosition, &SoldierTypeComponent)>();
        q.iter(world)
            .filter(|(_, _, st)| st.0 != SoldierType::Archer)
            .map(|(e, pos, st)| {
                let has_shield = world.get::<ShieldItem>(e).is_some();
                let shield_hp = world.get::<ShieldItem>(e).map(|s| s.hp).unwrap_or(0);
                SoldierData { entity: e, pos: pos.0, stype: st.0, has_shield, shield_hp }
            })
            .collect()
    };

    let mut pickups: Vec<(Entity, Entity, u32, u32)> = Vec::new(); // (soldier, dropped, hp, max_hp)
    let mut claimed_dropped: HashSet<Entity> = HashSet::new();

    for sd in &soldiers {
        let collision_radius = soldier_config.get(sd.stype).collision_radius;
        let pickup_range = Fixed::from_int(collision_radius as i32);
        let range_sq = pickup_range * pickup_range;

        for &(dropped_e, dropped_pos, dropped_hp, dropped_max_hp) in &dropped {
            if claimed_dropped.contains(&dropped_e) { continue; }

            let dist_sq = (sd.pos - dropped_pos).length_squared();
            if dist_sq <= range_sq {
                if !sd.has_shield {
                    // No current shield — pick up directly
                    pickups.push((sd.entity, dropped_e, dropped_hp, dropped_max_hp));
                    claimed_dropped.insert(dropped_e);
                    break;
                } else if dropped_hp > sd.shield_hp {
                    // Has shield but dropped one is better — swap
                    pickups.push((sd.entity, dropped_e, dropped_hp, dropped_max_hp));
                    claimed_dropped.insert(dropped_e);
                    break;
                }
            }
        }
    }

    for (soldier_e, dropped_e, hp, max_hp) in pickups {
        world.entity_mut(soldier_e).insert(ShieldItem { hp, max_hp });
        world.entity_mut(soldier_e).insert(ShieldComponent { state: ShieldState::Normal });
        world.despawn(dropped_e);
    }
}

// ══════════ System: shield_decay ══════════

pub fn shield_decay_system(world: &mut World, current_tick: u32) {
    let config = world.resource::<CombatGlobalConfig>().clone();
    let survive_ticks = config.shield.drop_survive_ticks;
    let anim_ticks = config.shield.disappear_animation_ticks;

    let to_despawn: Vec<Entity> = {
        let mut q = world.query::<(Entity, &DroppedShield)>();
        q.iter(world)
            .filter(|(_, d)| {
                let age = current_tick.saturating_sub(d.drop_tick);
                age >= survive_ticks + anim_ticks
            })
            .map(|(e, _)| e)
            .collect()
    };

    for entity in to_despawn {
        world.despawn(entity);
    }
}

// ══════════ Tick systems ══════════

pub fn slow_debuff_tick_system(world: &mut World) {
    let expired: Vec<Entity> = {
        let mut q = world.query::<(Entity, &SlowDebuff)>();
        q.iter(world).filter(|(_, s)| s.remaining_ticks <= 1).map(|(e, _)| e).collect()
    };
    let mut q = world.query::<(Entity, &mut SlowDebuff)>();
    for (_, mut s) in q.iter_mut(world) { s.remaining_ticks = s.remaining_ticks.saturating_sub(1); }
    for e in expired { world.entity_mut(e).remove::<SlowDebuff>(); }
}

pub fn fearless_buff_tick_system(world: &mut World) {
    let expired: Vec<Entity> = {
        let mut q = world.query::<(Entity, &FearlessBuff)>();
        q.iter(world).filter(|(_, f)| f.remaining_ticks <= 1).map(|(e, _)| e).collect()
    };
    let mut q = world.query::<(Entity, &mut FearlessBuff)>();
    for (_, mut f) in q.iter_mut(world) { f.remaining_ticks = f.remaining_ticks.saturating_sub(1); }
    for e in expired { world.entity_mut(e).remove::<FearlessBuff>(); }
}

pub fn soldier_level_up_system(world: &mut World) {
    let lu = world.resource::<CombatGlobalConfig>().clone().level_up;
    let mut leveled: Vec<(Entity, UnitId, u32)> = Vec::new();
    {
        let mut q = world.query::<(Entity, &UnitIdComponent, &mut Level)>();
        for (e, id, mut lvl) in q.iter_mut(world) {
            let mut changed = false;
            while lvl.exp >= lu.exp_to_level { lvl.exp -= lu.exp_to_level; lvl.level += 1; changed = true; }
            if changed { leveled.push((e, id.0, lvl.level)); }
        }
    }
    for (e, uid, nl) in leveled {
        let mut em = world.entity_mut(e);
        if let Some(mut hp) = em.get_mut::<Health>() { hp.max += lu.hp_gain; hp.current = hp.max; }
        if let Some(mut atk) = em.get_mut::<Attack>() { atk.damage += lu.attack_gain; }
        let mut events = world.resource_mut::<SimulationEvents>();
        events.leveled_up.push(SoldierLeveledUp { unit_id: uid, new_level: nl });
    }
}

// ══════════ Tests: SeekStance ══════════

#[cfg(test)]
mod seek_stance_tests {
    use super::*;
    use crate::init_simulation_world;
    use crate::map;
    use crate::combat::combat_engagement_system;

    /// Spawn a player soldier at a position with a given type.
    fn spawn_player_soldier(world: &mut World, pos: FixedVec2, stype: SoldierType) -> UnitId {
        let uid = world.resource_mut::<IdGenerator>().next();
        let cfg = world.resource::<SoldierConfig>().get(stype).clone();
        let shield_hp = world.resource::<CombatGlobalConfig>().shield.initial_hp;
        world.spawn((
            UnitIdComponent(uid), SoldierMarker, LogicalPosition(pos),
            Movement { speed: cfg.speed, target: None, command_target: None, waypoint: None, force_move: false },
            SeekStance { active: false, seek_range: 0 },
            Health { current: cfg.health, max: cfg.health },
            Attack { damage: cfg.attack, range: cfg.attack_range, interval_ticks: cfg.attack_interval_ticks, cooldown_remaining: 0 },
            FactionComponent(Faction::Player), SoldierTypeComponent(stype),
            Level { level: 1, exp: 0 }, ShieldComponent { state: ShieldState::Normal },
            ShieldItem { hp: shield_hp, max_hp: shield_hp },
            CityOrigin(UnitId(0)), SoldierStateComponent(SoldierState::Moving),
        ));
        uid
    }

    /// Spawn an enemy soldier at a position.
    fn spawn_enemy_soldier(world: &mut World, pos: FixedVec2) -> UnitId {
        let uid = world.resource_mut::<IdGenerator>().next();
        let shield_hp = world.resource::<CombatGlobalConfig>().shield.initial_hp;
        world.spawn((
            UnitIdComponent(uid), SoldierMarker, LogicalPosition(pos),
            Movement { speed: 80, target: None, command_target: None, waypoint: None, force_move: false },
            SeekStance { active: false, seek_range: 0 },
            Health { current: 100, max: 100 },
            Attack { damage: 10, range: 30, interval_ticks: 10, cooldown_remaining: 0 },
            FactionComponent(Faction::Enemy), SoldierTypeComponent(SoldierType::Militia),
            Level { level: 1, exp: 0 }, ShieldComponent { state: ShieldState::Normal },
            ShieldItem { hp: shield_hp, max_hp: shield_hp },
            CityOrigin(UnitId(0)), SoldierStateComponent(SoldierState::Moving),
        ));
        uid
    }

    // ── 4.1: SeekStance default values ──

    #[test]
    fn test_seek_stance_default_values() {
        let stance = SeekStance { active: false, seek_range: 0 };
        assert!(!stance.active);
        assert_eq!(stance.seek_range, 0);
    }

    #[test]
    fn test_seek_stance_active_values() {
        let stance = SeekStance { active: true, seek_range: 150 };
        assert!(stance.active);
        assert_eq!(stance.seek_range, 150);
    }

    // ── 4.2: GlobalSeekDirective inheritance ──

    #[test]
    fn test_spawn_inherits_global_seek_all() {
        let mut world = init_simulation_world(42);
        map::generate_map(&mut world);

        // Issue a global All directive
        world.resource_mut::<GlobalSeekDirective>().0.push(SeekDirective {
            scope: SeekScope::All,
            seek_range: 10,
            issue_tick: 1,
        });

        // Run city spawn
        city_spawn_system(&mut world);

        // Find a newly spawned player soldier
        let mut q = world.query::<(&FactionComponent, &SoldierTypeComponent, &SeekStance)>();
        let found = q.iter(&world)
            .filter(|(fac, _, _)| fac.0 == Faction::Player)
            .any(|(_, _, stance)| stance.active && stance.seek_range == 10);
        assert!(found, "Newly spawned soldier should inherit All seek directive");
    }

    #[test]
    fn test_spawn_ignores_non_matching_bytype() {
        let mut world = init_simulation_world(42);
        map::generate_map(&mut world);

        // Issue a ByType(Archer) directive only
        world.resource_mut::<GlobalSeekDirective>().0.push(SeekDirective {
            scope: SeekScope::ByType(SoldierType::Archer),
            seek_range: 50,
            issue_tick: 1,
        });

        // Run city spawn (cities default to Militia)
        city_spawn_system(&mut world);

        // Newly spawned militia should NOT inherit Archer directive
        let mut q = world.query::<(&FactionComponent, &SoldierTypeComponent, &SeekStance)>();
        let militia_with_seek = q.iter(&world)
            .filter(|(fac, st, _)| fac.0 == Faction::Player && st.0 == SoldierType::Militia)
            .any(|(_, _, stance)| stance.active);
        assert!(!militia_with_seek, "Militia should not inherit Archer-only directive");
    }

    #[test]
    fn test_spawn_latest_matching_directive_wins() {
        let mut world = init_simulation_world(42);
        map::generate_map(&mut world);

        // Push All(10) at tick 5, then ByType(Militia, 30) at tick 8
        {
            let mut gd = world.resource_mut::<GlobalSeekDirective>();
            gd.0.push(SeekDirective { scope: SeekScope::All, seek_range: 10, issue_tick: 5 });
            gd.0.push(SeekDirective { scope: SeekScope::ByType(SoldierType::Militia), seek_range: 30, issue_tick: 8 });
        }

        city_spawn_system(&mut world);

        let mut q = world.query::<(&FactionComponent, &SoldierTypeComponent, &SeekStance)>();
        let found = q.iter(&world)
            .filter(|(fac, st, _)| fac.0 == Faction::Player && st.0 == SoldierType::Militia)
            .any(|(_, _, stance)| stance.active && stance.seek_range == 30);
        assert!(found, "Militia should use latest matching directive (tick 8, range 30)");
    }

    // ── 4.3: consume_commands_system — SetSeekStance ──

    #[test]
    fn test_consume_seek_stance_global_all() {
        let mut world = init_simulation_world(42);
        let p1 = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0)), SoldierType::Militia);
        let p2 = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(50), Fixed::from_int(0)), SoldierType::Archer);
        spawn_enemy_soldier(&mut world, FixedVec2::new(Fixed::from_int(100), Fixed::from_int(0)));

        // Push global All(range=15) command
        world.resource_mut::<CommandBuffer>().push(GameCommand {
            tick: 1, player_id: 0,
            action: Action::SetSeekStance { scope: SeekScope::All, seek_range: 15, unit_ids: vec![] },
        });

        consume_commands_system(&mut world, 1);

        // Verify GlobalSeekDirective updated
        let gd = world.resource::<GlobalSeekDirective>();
        assert_eq!(gd.0.len(), 1);
        assert_eq!(gd.0[0].seek_range, 15);

        // Verify both soldiers have SeekStance updated
        let e1 = find_entity_by_unit_id(&mut world, p1).unwrap();
        let s1 = world.entity(e1).get::<SeekStance>().unwrap();
        assert!(s1.active && s1.seek_range == 15, "Militia should have active seek range 15");

        let e2 = find_entity_by_unit_id(&mut world, p2).unwrap();
        let s2 = world.entity(e2).get::<SeekStance>().unwrap();
        assert!(s2.active && s2.seek_range == 15, "Archer should have active seek range 15");
    }

    #[test]
    fn test_consume_seek_stance_by_type() {
        let mut world = init_simulation_world(42);
        let p_inf = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0)), SoldierType::Infantry);
        let p_arch = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(50), Fixed::from_int(0)), SoldierType::Archer);
        spawn_enemy_soldier(&mut world, FixedVec2::new(Fixed::from_int(100), Fixed::from_int(0)));

        // Push ByType(Infantry, range=20)
        world.resource_mut::<CommandBuffer>().push(GameCommand {
            tick: 1, player_id: 0,
            action: Action::SetSeekStance { scope: SeekScope::ByType(SoldierType::Infantry), seek_range: 20, unit_ids: vec![] },
        });

        consume_commands_system(&mut world, 1);

        // Infantry gets updated
        let e_inf = find_entity_by_unit_id(&mut world, p_inf).unwrap();
        let s_inf = world.entity(e_inf).get::<SeekStance>().unwrap();
        assert!(s_inf.active && s_inf.seek_range == 20, "Infantry should have active seek range 20");

        // Archer stays default
        let e_arch = find_entity_by_unit_id(&mut world, p_arch).unwrap();
        let s_arch = world.entity(e_arch).get::<SeekStance>().unwrap();
        assert!(!s_arch.active && s_arch.seek_range == 0, "Archer should remain default (not Infantry type)");
    }

    #[test]
    fn test_consume_seek_stance_selection() {
        let mut world = init_simulation_world(42);
        let p1 = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0)), SoldierType::Cavalry);
        let p2 = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(50), Fixed::from_int(0)), SoldierType::Cavalry);
        spawn_enemy_soldier(&mut world, FixedVec2::new(Fixed::from_int(100), Fixed::from_int(0)));

        // Push selection command for p1 only
        world.resource_mut::<CommandBuffer>().push(GameCommand {
            tick: 1, player_id: 0,
            action: Action::SetSeekStance { scope: SeekScope::All, seek_range: 60, unit_ids: vec![p1] },
        });

        consume_commands_system(&mut world, 1);

        // p1 gets updated
        let e1 = find_entity_by_unit_id(&mut world, p1).unwrap();
        let s1 = world.entity(e1).get::<SeekStance>().unwrap();
        assert!(s1.active && s1.seek_range == 60, "Selected cavalry should have active seek range 60");

        // p2 stays default
        let e2 = find_entity_by_unit_id(&mut world, p2).unwrap();
        let s2 = world.entity(e2).get::<SeekStance>().unwrap();
        assert!(!s2.active && s2.seek_range == 0, "Unselected cavalry should remain default");

        // GlobalSeekDirective NOT modified for selection commands
        let gd = world.resource::<GlobalSeekDirective>();
        assert!(gd.0.is_empty(), "Selection command should not modify GlobalSeekDirective");
    }

    #[test]
    fn test_consume_seek_stance_range_zero_disables() {
        let mut world = init_simulation_world(42);
        let p1 = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0)), SoldierType::Militia);
        spawn_enemy_soldier(&mut world, FixedVec2::new(Fixed::from_int(100), Fixed::from_int(0)));

        // First enable
        world.resource_mut::<CommandBuffer>().push(GameCommand {
            tick: 1, player_id: 0,
            action: Action::SetSeekStance { scope: SeekScope::All, seek_range: 10, unit_ids: vec![] },
        });
        consume_commands_system(&mut world, 1);

        // Then disable with range=0
        world.resource_mut::<CommandBuffer>().push(GameCommand {
            tick: 2, player_id: 0,
            action: Action::SetSeekStance { scope: SeekScope::All, seek_range: 0, unit_ids: vec![] },
        });
        consume_commands_system(&mut world, 2);

        let e1 = find_entity_by_unit_id(&mut world, p1).unwrap();
        let s1 = world.entity(e1).get::<SeekStance>().unwrap();
        assert!(!s1.active && s1.seek_range == 0, "range=0 should disable seek");
    }

    // ── 4.4: combat_engagement_system — SeekStance behavior ──

    #[test]
    fn test_engagement_inactive_seek_no_target() {
        let mut world = init_simulation_world(42);
        let p1 = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0)), SoldierType::Infantry);
        let e1 = find_entity_by_unit_id(&mut world, p1).unwrap();
        world.entity_mut(e1).insert(SeekStance { active: false, seek_range: 0 });

        // Enemy at distance 100 (within old default aggression range but seek is off)
        spawn_enemy_soldier(&mut world, FixedVec2::new(Fixed::from_int(100), Fixed::from_int(0)));

        combat_engagement_system(&mut world);

        let mov = world.entity(e1).get::<Movement>().unwrap();
        assert!(mov.target.is_none(), "Inactive seek should not set target");
    }

    #[test]
    fn test_engagement_active_seek_in_range() {
        let mut world = init_simulation_world(42);
        let p1 = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0)), SoldierType::Infantry);
        let e1 = find_entity_by_unit_id(&mut world, p1).unwrap();
        world.entity_mut(e1).insert(SeekStance { active: true, seek_range: 150 });

        let enemy_uid = spawn_enemy_soldier(&mut world, FixedVec2::new(Fixed::from_int(100), Fixed::from_int(0)));

        combat_engagement_system(&mut world);

        let mov = world.entity(e1).get::<Movement>().unwrap();
        assert_eq!(mov.target, Some(enemy_uid), "Active seek in range should target enemy");
        let state = world.entity(e1).get::<SoldierStateComponent>().unwrap();
        assert_eq!(state.0, SoldierState::Fighting, "Should be Fighting after seeking");
    }

    #[test]
    fn test_engagement_active_seek_out_of_range() {
        let mut world = init_simulation_world(42);
        let p1 = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0)), SoldierType::Infantry);
        let e1 = find_entity_by_unit_id(&mut world, p1).unwrap();
        world.entity_mut(e1).insert(SeekStance { active: true, seek_range: 50 });

        // Enemy at distance 200 (beyond seek_range 50)
        spawn_enemy_soldier(&mut world, FixedVec2::new(Fixed::from_int(200), Fixed::from_int(0)));

        combat_engagement_system(&mut world);

        let mov = world.entity(e1).get::<Movement>().unwrap();
        assert!(mov.target.is_none(), "Out-of-range enemy should not be targeted");
    }

    #[test]
    fn test_engagement_force_move_skips_seek() {
        let mut world = init_simulation_world(42);
        let p1 = spawn_player_soldier(&mut world, FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0)), SoldierType::Infantry);
        let e1 = find_entity_by_unit_id(&mut world, p1).unwrap();
        world.entity_mut(e1).insert(SeekStance { active: true, seek_range: 150 });
        world.entity_mut(e1).insert(Movement { speed: 80, target: None, command_target: None, waypoint: Some(FixedVec2::new(Fixed::from_int(500), Fixed::from_int(0))), force_move: true });

        spawn_enemy_soldier(&mut world, FixedVec2::new(Fixed::from_int(50), Fixed::from_int(0)));

        combat_engagement_system(&mut world);

        let mov = world.entity(e1).get::<Movement>().unwrap();
        assert!(mov.target.is_none(), "force_move should skip auto-engagement");
    }

    // ── 4.5: Backward compatibility — aggression_range removed ──

    #[test]
    fn test_units_ron_loads_without_aggression_range() {
        // This test verifies that content/units.ron loads successfully
        // after aggression_range was removed (missing field uses serde default).
        let world = init_simulation_world(42);
        let config = world.resource::<SoldierConfig>();

        // All 4 types should be loaded
        assert!(config.units.contains_key(&SoldierType::Militia));
        assert!(config.units.contains_key(&SoldierType::Infantry));
        assert!(config.units.contains_key(&SoldierType::Archer));
        assert!(config.units.contains_key(&SoldierType::Cavalry));

        // Verify basic values are still correct
        let militia = config.get(SoldierType::Militia);
        assert_eq!(militia.health, 100);
        assert_eq!(militia.attack, 16);
        assert_eq!(militia.speed, 80);
    }
}

// ══════════ Tests: Shield Lifecycle ══════════

#[cfg(test)]
mod shield_lifecycle_tests {
    use super::*;
    use crate::init_simulation_world;
    use crate::map;
    use crate::combat::drop_shield_on_death;

    fn spawn_infantry(world: &mut World, pos: FixedVec2, faction: Faction) -> UnitId {
        let uid = world.resource_mut::<IdGenerator>().next();
        let cfg = world.resource::<SoldierConfig>().get(SoldierType::Infantry).clone();
        let shield_hp = world.resource::<CombatGlobalConfig>().shield.initial_hp;
        let e = world.spawn((
            UnitIdComponent(uid), SoldierMarker, LogicalPosition(pos),
            Movement { speed: cfg.speed, target: None, command_target: None, waypoint: None, force_move: false },
            SeekStance { active: false, seek_range: 0 },
            Health { current: cfg.health, max: cfg.health },
            Attack { damage: cfg.attack, range: cfg.attack_range, interval_ticks: cfg.attack_interval_ticks, cooldown_remaining: 0 },
            FactionComponent(faction), SoldierTypeComponent(SoldierType::Infantry),
            Level { level: 1, exp: 0 },
            ShieldComponent { state: ShieldState::Normal },
            ShieldItem { hp: shield_hp, max_hp: shield_hp },
            CityOrigin(UnitId(0)), SoldierStateComponent(SoldierState::Moving),
        )).id();
        world.entity_mut(e).insert(crate::types::FacingDirection { angle: Fixed::ZERO });
        world.entity_mut(e).insert(crate::types::AttackWindup { remaining_ticks: 0, target: None });
        uid
    }

    fn spawn_militia(world: &mut World, pos: FixedVec2, faction: Faction) -> UnitId {
        let uid = world.resource_mut::<IdGenerator>().next();
        let cfg = world.resource::<SoldierConfig>().get(SoldierType::Militia).clone();
        world.spawn((
            UnitIdComponent(uid), SoldierMarker, LogicalPosition(pos),
            Movement { speed: cfg.speed, target: None, command_target: None, waypoint: None, force_move: false },
            SeekStance { active: false, seek_range: 0 },
            Health { current: cfg.health, max: cfg.health },
            Attack { damage: cfg.attack, range: cfg.attack_range, interval_ticks: cfg.attack_interval_ticks, cooldown_remaining: 0 },
            FactionComponent(faction), SoldierTypeComponent(SoldierType::Militia),
            Level { level: 1, exp: 0 },
            CityOrigin(UnitId(0)), SoldierStateComponent(SoldierState::Moving),
            crate::types::FacingDirection { angle: Fixed::ZERO },
            crate::types::AttackWindup { remaining_ticks: 0, target: None },
        ));
        uid
    }

    // ── Infantry spawns with shield ──

    #[test]
    fn test_infantry_spawns_with_shield() {
        let mut world = init_simulation_world(42);
        let uid = spawn_infantry(&mut world, FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0)), Faction::Player);
        let e = find_entity_by_unit_id(&mut world, uid).unwrap();
        assert!(world.get::<ShieldItem>(e).is_some(), "Infantry should have ShieldItem");
        assert!(world.get::<ShieldComponent>(e).is_some(), "Infantry should have ShieldComponent");
        let shield = world.get::<ShieldItem>(e).unwrap();
        assert_eq!(shield.hp, 1500);
        assert_eq!(shield.max_hp, 1500);
    }

    #[test]
    fn test_militia_spawns_without_shield() {
        let mut world = init_simulation_world(42);
        let uid = spawn_militia(&mut world, FixedVec2::new(Fixed::from_int(0), Fixed::from_int(0)), Faction::Player);
        let e = find_entity_by_unit_id(&mut world, uid).unwrap();
        assert!(world.get::<ShieldItem>(e).is_none(), "Militia should NOT have ShieldItem");
        assert!(world.get::<ShieldComponent>(e).is_none(), "Militia should NOT have ShieldComponent");
    }

    #[test]
    fn test_city_spawn_infantry_has_shield_militia_does_not() {
        let mut world = init_simulation_world(42);
        map::generate_map(&mut world);

        // Change a city to spawn Infantry
        {
            let mut q = world.query::<(Entity, &FactionComponent, &mut CityComponent)>();
            for (_, fac, mut city) in q.iter_mut(&mut world) {
                if fac.0 == Faction::Player {
                    city.spawn_type = SoldierType::Infantry;
                    city.population = 0;
                    city.max_population = 100;
                    city.spawn_cooldown = 0;
                    break;
                }
            }
        }

        city_spawn_system(&mut world);

        // Check that spawned infantry has shield
        let mut found_infantry_with_shield = false;
        {
            let mut q = world.query::<(&FactionComponent, &SoldierTypeComponent, Option<&ShieldItem>, Option<&ShieldComponent>)>();
            for (fac, st, shield_item, shield_comp) in q.iter(&world) {
                if fac.0 == Faction::Player && st.0 == SoldierType::Infantry {
                    assert!(shield_item.is_some(), "Spawned Infantry should have ShieldItem");
                    assert!(shield_comp.is_some(), "Spawned Infantry should have ShieldComponent");
                    found_infantry_with_shield = true;
                }
            }
        }
        assert!(found_infantry_with_shield, "Should have spawned at least one Infantry");
    }

    // ── Shield drop on death ──

    #[test]
    fn test_shield_drops_on_death() {
        let mut world = init_simulation_world(42);
        let shield_hp = world.resource::<CombatGlobalConfig>().shield.initial_hp;

        // Spawn an infantry with a shield
        let uid = spawn_infantry(&mut world, FixedVec2::new(Fixed::from_int(50), Fixed::from_int(50)), Faction::Player);
        let e = find_entity_by_unit_id(&mut world, uid).unwrap();

        // Set HP to 0 to simulate death
        {
            let mut hp = world.get_mut::<Health>(e).unwrap();
            hp.current = 0;
        }

        // Drop shield before despawn
        drop_shield_on_death(&mut world, e, 100);
        world.despawn(e);

        // Verify DroppedShield was created
        let mut q = world.query::<&DroppedShield>();
        let dropped: Vec<&DroppedShield> = q.iter(&world).collect();
        assert_eq!(dropped.len(), 1, "Should have exactly one DroppedShield");
        assert_eq!(dropped[0].shield.hp, shield_hp);
        assert_eq!(dropped[0].drop_tick, 100);
        assert_eq!(dropped[0].position, FixedVec2::new(Fixed::from_int(50), Fixed::from_int(50)));
    }

    #[test]
    fn test_no_shield_drop_if_no_shield() {
        let mut world = init_simulation_world(42);

        // Spawn militia (no shield)
        let uid = spawn_militia(&mut world, FixedVec2::new(Fixed::from_int(50), Fixed::from_int(50)), Faction::Player);
        let e = find_entity_by_unit_id(&mut world, uid).unwrap();

        // Set HP to 0
        {
            let mut hp = world.get_mut::<Health>(e).unwrap();
            hp.current = 0;
        }

        drop_shield_on_death(&mut world, e, 100);
        world.despawn(e);

        // Verify no DroppedShield was created
        let mut q = world.query::<&DroppedShield>();
        let dropped: Vec<&DroppedShield> = q.iter(&world).collect();
        assert_eq!(dropped.len(), 0, "Should have no DroppedShield for militia");
    }

    // ── Shield pickup ──

    #[test]
    fn test_shield_pickup_no_existing_shield() {
        let mut world = init_simulation_world(42);

        // Spawn militia at origin (no shield)
        let uid = spawn_militia(&mut world, FixedVec2::new(Fixed::from_int(10), Fixed::from_int(10)), Faction::Player);
        let e = find_entity_by_unit_id(&mut world, uid).unwrap();

        // Spawn a dropped shield nearby
        world.spawn(DroppedShield {
            shield: ShieldItem { hp: 800, max_hp: 1500 },
            position: FixedVec2::new(Fixed::from_int(12), Fixed::from_int(10)),
            drop_tick: 50,
            owner_faction: Some(Faction::Enemy),
        });

        shield_pickup_system(&mut world);

        // Militia should now have a shield
        assert!(world.get::<ShieldItem>(e).is_some(), "Militia should have picked up ShieldItem");
        assert!(world.get::<ShieldComponent>(e).is_some(), "Militia should have ShieldComponent");
        let shield = world.get::<ShieldItem>(e).unwrap();
        assert_eq!(shield.hp, 800);
        assert_eq!(shield.max_hp, 1500);

        // DroppedShield should be gone
        let mut q = world.query::<&DroppedShield>();
        assert_eq!(q.iter(&world).count(), 0, "DroppedShield should be consumed");
    }

    #[test]
    fn test_shield_pickup_swap_better_shield() {
        let mut world = init_simulation_world(42);

        // Spawn infantry at origin (has shield with 1500 HP)
        let uid = spawn_infantry(&mut world, FixedVec2::new(Fixed::from_int(10), Fixed::from_int(10)), Faction::Player);
        let e = find_entity_by_unit_id(&mut world, uid).unwrap();

        // Reduce their shield HP
        {
            let mut shield = world.get_mut::<ShieldItem>(e).unwrap();
            shield.hp = 200;
        }

        // Spawn a dropped shield with higher HP nearby
        world.spawn(DroppedShield {
            shield: ShieldItem { hp: 1000, max_hp: 1500 },
            position: FixedVec2::new(Fixed::from_int(12), Fixed::from_int(10)),
            drop_tick: 50,
            owner_faction: Some(Faction::Enemy),
        });

        shield_pickup_system(&mut world);

        // Infantry should now have the better shield
        let shield = world.get::<ShieldItem>(e).unwrap();
        assert_eq!(shield.hp, 1000, "Should have swapped to the better shield");
        assert_eq!(shield.max_hp, 1500);
    }

    #[test]
    fn test_shield_pickup_ignores_worse_shield() {
        let mut world = init_simulation_world(42);

        // Spawn infantry at origin (has shield with 1500 HP)
        let uid = spawn_infantry(&mut world, FixedVec2::new(Fixed::from_int(10), Fixed::from_int(10)), Faction::Player);
        let e = find_entity_by_unit_id(&mut world, uid).unwrap();

        // Spawn a dropped shield with lower HP nearby
        world.spawn(DroppedShield {
            shield: ShieldItem { hp: 500, max_hp: 1500 },
            position: FixedVec2::new(Fixed::from_int(12), Fixed::from_int(10)),
            drop_tick: 50,
            owner_faction: Some(Faction::Enemy),
        });

        shield_pickup_system(&mut world);

        // Infantry should keep their original shield (1500 HP)
        let shield = world.get::<ShieldItem>(e).unwrap();
        assert_eq!(shield.hp, 1500, "Should keep the better shield");

        // DroppedShield should remain
        let mut q = world.query::<&DroppedShield>();
        assert_eq!(q.iter(&world).count(), 1, "DroppedShield should remain (not picked up)");
    }

    #[test]
    fn test_shield_pickup_archer_ignored() {
        let mut world = init_simulation_world(42);

        // Spawn an archer
        let uid = world.resource_mut::<IdGenerator>().next();
        let cfg = world.resource::<SoldierConfig>().get(SoldierType::Archer).clone();
        world.spawn((
            UnitIdComponent(uid), SoldierMarker, LogicalPosition(FixedVec2::new(Fixed::from_int(10), Fixed::from_int(10))),
            Movement { speed: cfg.speed, target: None, command_target: None, waypoint: None, force_move: false },
            SeekStance { active: false, seek_range: 0 },
            Health { current: cfg.health, max: cfg.health },
            Attack { damage: cfg.attack, range: cfg.attack_range, interval_ticks: cfg.attack_interval_ticks, cooldown_remaining: 0 },
            FactionComponent(Faction::Player), SoldierTypeComponent(SoldierType::Archer),
            Level { level: 1, exp: 0 },
            CityOrigin(UnitId(0)), SoldierStateComponent(SoldierState::Moving),
        ));
        let e = find_entity_by_unit_id(&mut world, uid).unwrap();

        // Spawn a dropped shield right on top
        world.spawn(DroppedShield {
            shield: ShieldItem { hp: 1000, max_hp: 1500 },
            position: FixedVec2::new(Fixed::from_int(10), Fixed::from_int(10)),
            drop_tick: 50,
            owner_faction: Some(Faction::Enemy),
        });

        shield_pickup_system(&mut world);

        // Archer should NOT have a shield
        assert!(world.get::<ShieldItem>(e).is_none(), "Archer should not pick up shields");
    }

    // ── Shield decay ──

    #[test]
    fn test_shield_decay_not_yet_expired() {
        let mut world = init_simulation_world(42);

        // Spawn a dropped shield at tick 100
        world.spawn(DroppedShield {
            shield: ShieldItem { hp: 800, max_hp: 1500 },
            position: FixedVec2::new(Fixed::from_int(50), Fixed::from_int(50)),
            drop_tick: 100,
            owner_faction: Some(Faction::Player),
        });

        // At tick 500 (age = 400, need 600+60=660 to expire)
        shield_decay_system(&mut world, 500);

        let mut q = world.query::<&DroppedShield>();
        assert_eq!(q.iter(&world).count(), 1, "Shield should not have decayed yet");
    }

    #[test]
    fn test_shield_decay_expired() {
        let mut world = init_simulation_world(42);

        // Spawn a dropped shield at tick 100
        world.spawn(DroppedShield {
            shield: ShieldItem { hp: 800, max_hp: 1500 },
            position: FixedVec2::new(Fixed::from_int(50), Fixed::from_int(50)),
            drop_tick: 100,
            owner_faction: Some(Faction::Player),
        });

        // At tick 760 (age = 660 = 600 + 60, exactly at threshold)
        shield_decay_system(&mut world, 760);

        let mut q = world.query::<&DroppedShield>();
        assert_eq!(q.iter(&world).count(), 0, "Shield should have decayed");
    }

    #[test]
    fn test_shield_decay_multiple_mixed_ages() {
        let mut world = init_simulation_world(42);

        // Spawn two shields: one fresh, one old
        world.spawn(DroppedShield {
            shield: ShieldItem { hp: 800, max_hp: 1500 },
            position: FixedVec2::new(Fixed::from_int(50), Fixed::from_int(50)),
            drop_tick: 100, // age at tick 800 = 700, expired
            owner_faction: Some(Faction::Player),
        });
        world.spawn(DroppedShield {
            shield: ShieldItem { hp: 900, max_hp: 1500 },
            position: FixedVec2::new(Fixed::from_int(60), Fixed::from_int(60)),
            drop_tick: 500, // age at tick 800 = 300, not expired
            owner_faction: Some(Faction::Enemy),
        });

        shield_decay_system(&mut world, 800);

        let mut q = world.query::<(Entity, &DroppedShield)>();
        let remaining: Vec<(Entity, &DroppedShield)> = q.iter(&world).collect();
        assert_eq!(remaining.len(), 1, "Only the expired shield should be removed");
        assert_eq!(remaining[0].1.drop_tick, 500, "The remaining shield should be the newer one");
    }

    // ── Aura heal heals shield ──

    #[test]
    fn test_aura_heal_heals_shield() {
        let mut world = init_simulation_world(42);
        map::generate_map(&mut world);

        // Find a player city position to place infantry nearby
        let city_pos = {
            let mut q = world.query::<(&LogicalPosition, &FactionComponent, &CityMarker)>();
            let mut found = None;
            for (pos, fac, _) in q.iter(&world) {
                if fac.0 == Faction::Player {
                    found = Some(pos.0);
                    break;
                }
            }
            found.expect("Should have a player city")
        };

        // Spawn infantry right at the player city position
        let uid = spawn_infantry(&mut world, city_pos, Faction::Player);
        let e = find_entity_by_unit_id(&mut world, uid).unwrap();

        // Damage the shield
        {
            let mut shield = world.get_mut::<ShieldItem>(e).unwrap();
            shield.hp = 1000; // damaged from 1500
        }

        // Run aura heal
        aura_heal_system(&mut world);

        // Shield should be healed
        let shield = world.get::<ShieldItem>(e).unwrap();
        assert!(shield.hp > 1000, "Shield HP should have been healed (was 1000, now {})", shield.hp);
        assert!(shield.hp <= shield.max_hp, "Shield HP should not exceed max");
    }

    // ── Integration: full lifecycle ──

    #[test]
    fn test_full_shield_lifecycle() {
        let mut world = init_simulation_world(42);

        // 1. Infantry spawns with shield
        let uid = spawn_infantry(&mut world, FixedVec2::new(Fixed::from_int(100), Fixed::from_int(100)), Faction::Player);
        let e = find_entity_by_unit_id(&mut world, uid).unwrap();
        assert!(world.get::<ShieldItem>(e).is_some());

        // 2. Infantry dies — shield drops
        {
            let mut hp = world.get_mut::<Health>(e).unwrap();
            hp.current = 0;
        }
        drop_shield_on_death(&mut world, e, 100);
        world.despawn(e);

        // 3. Verify DroppedShield exists
        let mut q = world.query::<&DroppedShield>();
        assert_eq!(q.iter(&world).count(), 1);

        // 4. Another soldier picks it up
        let uid2 = spawn_militia(&mut world, FixedVec2::new(Fixed::from_int(102), Fixed::from_int(100)), Faction::Player);
        let e2 = find_entity_by_unit_id(&mut world, uid2).unwrap();
        shield_pickup_system(&mut world);

        assert!(world.get::<ShieldItem>(e2).is_some(), "Militia should have picked up the dropped shield");

        // 5. DroppedShield consumed
        let mut q = world.query::<&DroppedShield>();
        assert_eq!(q.iter(&world).count(), 0);

        // 6. Shield does not decay (it's been picked up, no DroppedShield left)
        shield_decay_system(&mut world, 800);
        assert!(world.get::<ShieldItem>(e2).is_some(), "Picked-up shield should not decay");
    }
}
