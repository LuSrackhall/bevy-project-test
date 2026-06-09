pub mod config;

use bevy_ecs::world::World;
use bevy_ecs::entity::Entity;
use bevy_ecs::component::Component;
use std::collections::{HashMap, HashSet};
use crate::types::*;
use crate::events::*;
use crate::soldier::*;
use crate::soldier::config::SoldierConfig;
use crate::combat::config::CombatGlobalConfig;

// ══════════ Arrow Types ══════════

/// Directional arrow — flies in a fixed direction, collides with enemies.
/// Flight phase (flight_remaining > 0): moves and deals collision damage.
/// Decay phase (decay_remaining > 0): visual only, no damage.
#[derive(Component, Clone, Debug)]
pub struct Arrow {
    pub direction: FixedVec2,     // unit vector × speed (movement per tick)
    pub damage: u32,
    pub from_faction: Faction,
    pub shooter: Option<UnitId>,
    pub flight_remaining: u32,    // ticks remaining in flight (damage phase)
    pub decay_remaining: u32,     // 0=flight, >0=decay (visual only)
    pub pierce_chance: f32,       // pre-computed at launch
    pub stuck_to: Option<UnitId>, // follow target during decay
    pub hit_units: Vec<UnitId>,   // already-hit units (prevent double-hit)
    pub start_pos: FixedVec2,
}

#[derive(Component, Clone, Debug)]
pub struct ArrowMarker;

/// Decay duration in ticks: 20 ticks at 20Hz = 1 second.
pub const ARROW_DECAY_TICKS: u32 = 20;
/// Collision radius for arrow hits — must be >= arrow_speed to prevent tunneling.
const ARROW_HIT_RADIUS: i64 = 22 * FIXED_ONE; // 22 units ≥ arrow_speed(20)

const FIXED_ONE: i64 = 256;

fn integer_sqrt(n: i64) -> i64 {
    if n <= 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + n / x) / 2; }
    x
}

// ══════════ combat_engagement ══════════

pub fn combat_engagement_system(world: &mut World) {
    let soldier_config = world.resource::<SoldierConfig>().clone();

    // Collect all entity positions & factions
    let all_units: HashMap<UnitId, (FixedVec2, Faction)> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent)>();
        q.iter(world).map(|(_, id, pos, fac)| (id.0, (pos.0, fac.0))).collect()
    };

    // Collect soldiers to process
    struct EngData { entity: Entity, uid: UnitId, pos: FixedVec2, faction: Faction, stype: SoldierType, state: SoldierState, force_move: bool, cmd_target: Option<UnitId>, target: Option<UnitId>, speed: u32 }
    let soldiers: Vec<EngData> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &SoldierTypeComponent, &SoldierStateComponent, &Movement)>();
        q.iter(world)
            .filter(|(_, _, _, _, st, _, _)| st.0 != SoldierType::Archer)
            .map(|(e, id, pos, fac, st, sst, mov)| EngData {
                entity: e, uid: id.0, pos: pos.0, faction: fac.0, stype: st.0,
                state: sst.0, force_move: mov.force_move,
                cmd_target: mov.command_target, target: mov.target, speed: mov.speed,
            }).collect()
    };

    for sd in soldiers {
        if sd.force_move { continue; }

        let unit_cfg = soldier_config.get(sd.stype);
        let aggro = Fixed::from_int(unit_cfg.aggression_range as i32);
        let aggro_sq = aggro * aggro;

        // Find nearest enemy
        let mut best: Option<(UnitId, i64)> = None;
        for (eid, (epos, efac)) in &all_units {
            if *efac == sd.faction { continue; }
            let ds = (sd.pos - *epos).length_squared();
            if ds <= aggro_sq && best.as_ref().map_or(true, |(_, d)| ds.0 < *d) {
                best = Some((*eid, ds.0));
            }
        }

        let mut em = world.entity_mut(sd.entity);
        if let Some((enemy_id, _)) = best {
            let is_cav = sd.stype == SoldierType::Cavalry;
            if !is_cav {
                let ct = if sd.cmd_target.is_none() { sd.target } else { sd.cmd_target };
                em.insert(Movement { speed: sd.speed, target: Some(enemy_id), command_target: ct, waypoint: None, force_move: false });
            }
            em.insert(SoldierStateComponent(SoldierState::Fighting));
        } else if sd.state == SoldierState::Fighting {
            let ct = sd.cmd_target;
            em.insert(Movement { speed: sd.speed, target: ct, command_target: None, waypoint: None, force_move: false });
            em.insert(SoldierStateComponent(SoldierState::Moving));
        }
    }
}

// ══════════ melee_attack ══════════

pub fn melee_attack_system(world: &mut World) {
    let soldier_config = world.resource::<SoldierConfig>().clone();
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // Tick cooldowns
    {
        let mut q = world.query::<(Entity, &mut Attack, &SoldierTypeComponent)>();
        for (_, mut atk, st) in q.iter_mut(world) {
            if st.0 != SoldierType::Archer {
                atk.cooldown_remaining = atk.cooldown_remaining.saturating_sub(1);
            }
        }
    }

    // Position lookup
    let positions: HashMap<UnitId, FixedVec2> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition)>();
        q.iter(world).map(|(_, id, pos)| (id.0, pos.0)).collect()
    };

    // Ready attackers
    struct AtkData { entity: Entity, uid: UnitId, pos: FixedVec2, faction: Faction, dmg: u32, range: u32, stype: SoldierType, level: u32, interval: u32, has_fearless: bool, target: Option<UnitId> }
    let attackers: Vec<AtkData> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &Attack, &SoldierTypeComponent, &Level, &Movement, Option<&FearlessBuff>)>();
        q.iter(world)
            .filter(|(_, _, _, _, atk, st, _, _, _)| atk.cooldown_remaining == 0 && st.0 != SoldierType::Archer)
            .map(|(e, id, pos, fac, atk, st, lvl, mov, fb)| AtkData {
                entity: e, uid: id.0, pos: pos.0, faction: fac.0,
                dmg: atk.damage, range: atk.range, stype: st.0,
                level: lvl.level, interval: atk.interval_ticks,
                has_fearless: fb.is_some(), target: mov.target,
            }).collect()
    };

    let mut pending_deaths: Vec<(Entity, Option<UnitId>, Option<UnitId>)> = Vec::new(); // (target, killer, city_origin)
    let mut xp_grants: Vec<(Entity, u32)> = Vec::new();
    let mut ls_hits: Vec<(Entity, u32, u32, bool)> = Vec::new();
    let mut ls_kills: Vec<(Entity, u32, u32, bool)> = Vec::new();

    for ad in attackers {
        let Some(tid) = ad.target else { continue };
        let Some(&tpos) = positions.get(&tid) else { continue };

        let range_f = Fixed::from_int(ad.range as i32);
        if (ad.pos - tpos).length_squared() > range_f * range_f { continue; }

        let Some(te) = find_entity_by_unit_id(world, tid) else { continue };

        let (thp, tmax, tst, _tfac, city_origin) = {
            let em = world.entity(te);
            let hp = em.get::<Health>();
            (hp.map(|h| h.current), hp.map(|h| h.max),
             em.get::<SoldierTypeComponent>().map(|s| s.0),
             em.get::<FactionComponent>().map(|f| f.0),
             em.get::<CityOrigin>().map(|c| c.0))
        };
        let Some(hp_cur) = thp else { continue };
        let Some(hp_max) = tmax else { continue };

        let mut damage = ad.dmg + ad.level * combat_config.level_up.attack_gain;
        if ad.has_fearless { damage += combat_config.fearless.attack_bonus; }

        // Cavalry dodge
        if tst == Some(SoldierType::Cavalry) && hp_max > 0 {
            let hp_r = hp_cur as f32 / hp_max as f32;
            let dc = (combat_config.cavalry.dodge_max_chance - (1.0 - hp_r) * combat_config.cavalry.dodge_decay_rate).max(0.0);
            if world.resource_mut::<DeterministicRng>().gen_probability() < dc {
                damage = 0;
                world.entity_mut(te).insert(FearlessBuff { remaining_ticks: combat_config.fearless.duration_ticks });
            }
        }

        if damage > 0 {
            let died = {
                let mut em = world.entity_mut(te);
                if let Some(mut hp) = em.get_mut::<Health>() {
                    hp.current = hp.current.saturating_sub(damage);
                    hp.current == 0
                } else { false }
            };

            if died {
                pending_deaths.push((te, Some(ad.uid), city_origin));
                xp_grants.push((ad.entity, combat_config.level_up.exp_per_kill));
                ls_kills.push((ad.entity, damage, ad.level, ad.has_fearless));
            } else {
                ls_hits.push((ad.entity, damage, ad.level, ad.has_fearless));
            }
        }

        // Reset attacker cooldown
        world.entity_mut(ad.entity).insert(Attack { damage: ad.dmg, range: ad.range, interval_ticks: ad.interval, cooldown_remaining: ad.interval });
    }

    // Apply lifesteal & XP
    for (e, dmg, lvl, hf) in ls_kills.iter().chain(ls_hits.iter()) {
        let ls = if *lvl >= combat_config.level_up.lifesteal_unlock_level { combat_config.level_up.lifesteal_rate } else { 0.0 }
            + if *hf { combat_config.fearless.lifesteal_bonus } else { 0.0 };
        if ls > 0.0 {
            if let Some(mut hp) = world.entity_mut(*e).get_mut::<Health>() {
                hp.current = (hp.current + (*dmg as f32 * ls) as u32).min(hp.max);
            }
        }
    }
    for (e, xp) in xp_grants {
        if let Some(mut lvl) = world.entity_mut(e).get_mut::<Level>() { lvl.exp += xp; }
    }

    // Process deaths (dedup by entity — same target may be killed by multiple attackers)
    let mut seen = std::collections::HashSet::new();
    for (te, kid, origin) in pending_deaths {
        if seen.contains(&te) { continue; }
        seen.insert(te);
        let uid = find_unit_id(world, te).unwrap_or(UnitId(0));
        world.despawn(te);
        // Decrement origin city population when soldier dies in combat
        if let Some(origin_id) = origin {
            if let Some(oe) = find_entity_by_unit_id(world, origin_id) {
                if let Some(mut c) = world.entity_mut(oe).get_mut::<CityComponent>() {
                    c.population = c.population.saturating_sub(1);
                }
            }
        }
        let mut events = world.resource_mut::<SimulationEvents>();
        events.destroyed.push(UnitDestroyed { unit_id: uid, killer_id: kid });
    }
}

// ══════════ melee_attack (end) ══════════


// ══════════ archer_attack (direction-based) ══════════

pub fn archer_attack_system(world: &mut World) {
    let soldier_config = world.resource::<SoldierConfig>().clone();
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // Tick cooldowns for archers
    {
        let mut q = world.query::<(Entity, &mut Attack, &SoldierTypeComponent)>();
        for (_, mut atk, st) in q.iter_mut(world) {
            if st.0 == SoldierType::Archer { atk.cooldown_remaining = atk.cooldown_remaining.saturating_sub(1); }
        }
    }

    // Collect enemy soldier positions (filter by SoldierMarker + faction)
    let all_units: Vec<(UnitId, FixedVec2, Faction)> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &SoldierMarker)>();
        q.iter(world).map(|(_, id, pos, fac, _)| (id.0, pos.0, fac.0)).collect()
    };

    // Collect enemy city positions (filter by CityMarker + faction)
    let all_cities: Vec<(UnitId, FixedVec2, Faction)> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &CityMarker)>();
        q.iter(world).map(|(_, id, pos, fac, _)| (id.0, pos.0, fac.0)).collect()
    };

    // Collect archers ready to fire
    struct ArcData { entity: Entity, pos: FixedVec2, faction: Faction, dmg: u32, range: u32, interval: u32, level: u32, cfg: crate::soldier::config::SoldierUnitConfig }
    let archers: Vec<ArcData> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &Attack, &Level, &SoldierTypeComponent)>();
        q.iter(world)
            .filter(|(_, _, _, _, atk, _, st)| atk.cooldown_remaining == 0 && st.0 == SoldierType::Archer)
            .map(|(e, _id, pos, fac, atk, lvl, _st)| {
                let cfg = soldier_config.get(SoldierType::Archer).clone();
                let range = cfg.compute_attack_range(lvl.level);
                ArcData { entity: e, pos: pos.0, faction: fac.0, dmg: atk.damage, range, interval: cfg.attack_interval_ticks, level: lvl.level, cfg }
            }).collect()
    };

    for ad in archers {
        // Find nearest enemy in range
        let range_fixed = Fixed::from_int(ad.range as i32);
        let range_sq = range_fixed * range_fixed;
        let mut nearest: Option<(UnitId, FixedVec2)> = None;
        let mut nearest_d = i64::MAX;
        for (eid, epos, efac) in &all_units {
            if *efac == ad.faction { continue; } // only target enemies
            let ds = (ad.pos - *epos).length_squared();
            if ds <= range_sq && ds.0 < nearest_d {
                nearest = Some((*eid, *epos));
                nearest_d = ds.0;
            }
        }

        // Fallback: if no soldiers in range, search nearest enemy city
        let Some((target_id, target_pos)) = nearest.or_else(|| {
            let mut best: Option<(UnitId, FixedVec2)> = None;
            let mut best_d = i64::MAX;
            for (cid, cpos, cfac) in &all_cities {
                if *cfac == ad.faction { continue; }
                let ds = (ad.pos - *cpos).length_squared();
                if ds <= range_sq && ds.0 < best_d {
                    best = Some((*cid, *cpos));
                    best_d = ds.0;
                }
            }
            best
        }) else {
            // No enemy soldier or city in range — restore Moving state
            world.entity_mut(ad.entity).insert(SoldierStateComponent(SoldierState::Moving));
            continue;
        };

        // Compute flight direction with spread (65% dead-on, 35% ±0.1°–10°)
        let delta = target_pos - ad.pos;
        let dist_internal = integer_sqrt(delta.length_squared().0 * FIXED_ONE);
        if dist_internal <= 0 { continue; }
        let dist = Fixed(dist_internal);
        let dir_unit = FixedVec2::new(delta.x / dist, delta.y / dist);

        // Spread: 10° max in radians ≈ 0.1745 → Fixed(44) with 8-bit precision
        let max_spread = Fixed(44);
        let spread_angle = {
            let mut rng = world.resource_mut::<DeterministicRng>();
            if rng.gen_probability() < 0.65 {
                Fixed::ZERO // 65% perfect aim
            } else {
                let angle = Fixed(1 + (rng.next_u64() % 44) as i64); // 0.1°–10°
                if rng.gen_probability() < 0.5 { Fixed(-angle.0) } else { angle }
            }
        };

        // Apply rotation via small-angle approx: sin(θ)≈θ, cos(θ)≈1
        let sin_a = spread_angle;
        let cos_a = Fixed::ONE; // cos(small) ≈ 1
        let rotated_x = dir_unit.x * cos_a - dir_unit.y * sin_a;
        let rotated_y = dir_unit.x * sin_a + dir_unit.y * cos_a;

        let speed = Fixed::from_int(ad.cfg.arrow_speed as i32);
        let direction = FixedVec2::new(rotated_x * speed, rotated_y * speed);

        let flight_ticks = ad.cfg.compute_flight_ticks(ad.level);
        let pierce_chance = ad.cfg.compute_pierce_chance(ad.level);
        let damage = ad.dmg + ad.level * combat_config.level_up.attack_gain;

        // Spawn arrow
        let aid = { world.resource_mut::<IdGenerator>().next() };
        world.spawn((
            UnitIdComponent(aid), ArrowMarker, LogicalPosition(ad.pos),
            Arrow {
                direction, damage, from_faction: ad.faction,
                shooter: find_unit_id(world, ad.entity),
                flight_remaining: flight_ticks, decay_remaining: 0,
                pierce_chance, stuck_to: None, hit_units: Vec::new(),
                start_pos: ad.pos,
            },
        ));

        let mut events = world.resource_mut::<SimulationEvents>();
        events.spawned.push(UnitSpawned { unit_id: aid, pos: ad.pos, faction: ad.faction, unit_kind: UnitKind::Arrow });

        // Reset cooldown and set Fighting state (prevents movement while shooting)
        world.entity_mut(ad.entity).insert(Attack { damage: ad.dmg, range: ad.range, interval_ticks: ad.interval, cooldown_remaining: ad.interval });
        world.entity_mut(ad.entity).insert(SoldierStateComponent(SoldierState::Fighting));
    }
}

// ══════════ arrow_movement (flight + collision + decay) ══════════

pub fn arrow_movement_system(world: &mut World) {
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // Collect soldier positions for collision (filter by SoldierMarker)
    let all_soldiers: Vec<(UnitId, FixedVec2, Entity, Faction)> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &SoldierMarker)>();
        q.iter(world).map(|(e, id, pos, fac, _)| (id.0, pos.0, e, fac.0)).collect()
    };

    // Collect city positions for collision (filter by CityMarker)
    let all_cities: Vec<(UnitId, FixedVec2, Entity, Faction, u32)> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &CityMarker, &CityRadius)>();
        q.iter(world).map(|(e, id, pos, fac, _, r)| (id.0, pos.0, e, fac.0, r.0)).collect()
    };

    let arrow_building_damage_denom = (1.0_f32 / combat_config.arrow_building_damage_ratio).round() as u32;

    let mut to_despawn: Vec<Entity> = Vec::new();
    // Collect pierce decisions before query to avoid borrow conflict
    let pierce_rolls: Vec<f32> = {
        let mut rng = world.resource_mut::<DeterministicRng>();
        (0..100).map(|_| rng.gen_probability()).collect()
    };
    let mut pierce_idx = 0usize;

    let mut hits: Vec<(Entity, u32, Option<UnitId>, bool, Option<UnitId>)> = Vec::new();
    let mut city_hits: Vec<(Entity, u32)> = Vec::new(); // (city_entity, arrow_damage)

    // Process each arrow
    {
        let threshold = Fixed(ARROW_HIT_RADIUS);
        let threshold_sq = threshold * threshold;
        let mut q = world.query::<(Entity, &mut LogicalPosition, &mut Arrow)>();
        for (ae, mut arrow_pos, mut arrow) in q.iter_mut(world) {
            if arrow.decay_remaining > 0 {
                arrow.decay_remaining -= 1;
                if let Some(sid) = arrow.stuck_to {
                    for (eid, epos, _, _) in &all_soldiers {
                        if *eid == sid { arrow_pos.0 = *epos; break; }
                    }
                }
                if arrow.decay_remaining == 0 { to_despawn.push(ae); }
                continue;
            }

            if arrow.flight_remaining > 0 {
                arrow.flight_remaining -= 1;
                arrow_pos.0 = FixedVec2::new(
                    arrow_pos.0.x + arrow.direction.x,
                    arrow_pos.0.y + arrow.direction.y,
                );

                let mut stopped_by_soldier = false;
                for (eid, epos, _ee, efac) in &all_soldiers {
                    if *efac == arrow.from_faction { continue; } // don't hit friendlies
                    if arrow.hit_units.contains(eid) { continue; }
                    if (arrow_pos.0 - *epos).length_squared() <= threshold_sq {
                        arrow.hit_units.push(*eid);
                        let rolled = pierce_rolls[pierce_idx.min(pierce_rolls.len()-1)];
                        pierce_idx += 1;
                        if rolled < arrow.pierce_chance {
                            hits.push((ae, arrow.damage, None, true, arrow.shooter));
                        } else {
                            arrow.stuck_to = Some(*eid);
                            arrow.decay_remaining = ARROW_DECAY_TICKS;
                            hits.push((ae, arrow.damage, Some(*eid), false, arrow.shooter));
                            stopped_by_soldier = true;
                            break;
                        }
                    }
                }

                // City collision — only if not stopped by soldier hit
                if !stopped_by_soldier {
                    for (_cid, cpos, ce, cfac, cradius) in &all_cities {
                        if *cfac == arrow.from_faction { continue; } // friendly city: pass through
                        let radius = Fixed::from_int(*cradius as i32);
                        let radius_sq = radius * radius;
                        if (arrow_pos.0 - *cpos).length_squared() <= radius_sq {
                            // Hit city: accumulate damage, force decay (no pierce)
                            city_hits.push((*ce, arrow.damage));
                            arrow.decay_remaining = ARROW_DECAY_TICKS;
                            break;
                        }
                    }
                }

                if arrow.flight_remaining == 0 && arrow.decay_remaining == 0 {
                    arrow.decay_remaining = ARROW_DECAY_TICKS;
                }
            }
        }
    }

    // Apply damage from hits
    for (ae, dmg, stuck_to, _pierced, shooter) in &hits {
        if let Some(sid) = stuck_to {
            if let Some(te) = find_entity_by_unit_id(world, *sid) {
                let died = {
                    let mut em = world.entity_mut(te);
                    if let Some(mut hp) = em.get_mut::<Health>() {
                        hp.current = hp.current.saturating_sub(*dmg);
                        hp.current == 0
                    } else { false }
                };
                // Shield intercept check (simplified — shield handled in melee for now)
                if died {
                    let uid = find_unit_id(world, te).unwrap_or(UnitId(0));
                    if let Some(origin) = world.entity(te).get::<CityOrigin>().map(|c| c.0) {
                        if let Some(oe) = find_entity_by_unit_id(world, origin) {
                            if let Some(mut c) = world.entity_mut(oe).get_mut::<CityComponent>() {
                                c.population = c.population.saturating_sub(1);
                            }
                        }
                    }
                    world.despawn(te);
                    let mut events = world.resource_mut::<SimulationEvents>();
                    events.destroyed.push(UnitDestroyed { unit_id: uid, killer_id: *shooter });
                    // XP to shooter
                    if let Some(sid) = shooter {
                        if let Some(se) = find_entity_by_unit_id(world, *sid) {
                            if let Some(mut lvl) = world.entity_mut(se).get_mut::<Level>() {
                                lvl.exp += combat_config.level_up.exp_per_kill;
                            }
                        }
                    }
                }
            }
        }
    }

    // Apply city damage from arrow hits (accumulator: arrow_damage_acc / denom)
    if arrow_building_damage_denom > 0 {
        for (ce, dmg) in &city_hits {
            if let Some(mut city) = world.entity_mut(*ce).get_mut::<CityComponent>() {
                city.arrow_damage_acc += dmg;
                let integer_damage = city.arrow_damage_acc / arrow_building_damage_denom;
                if integer_damage > 0 {
                    city.health_current = city.health_current.saturating_sub(integer_damage);
                    city.arrow_damage_acc %= arrow_building_damage_denom;
                }
            }
        }
    }

    // Despawn arrows
    for ae in to_despawn { world.despawn(ae); }
}

// ══════════ Tests: arrow-city collision ══════════

#[cfg(test)]
mod arrow_city_tests {
    use super::*;
    use crate::init_simulation_world;
    use crate::soldier::{CityMarker, CityComponent, CityRadius, UnitIdComponent, LogicalPosition, FactionComponent, SoldierMarker, Health, SoldierTypeComponent, SoldierStateComponent, Attack, Movement, Level, CityOrigin};

    /// Helper: create a minimal arrow flying toward a position
    fn spawn_test_arrow(world: &mut World, pos: FixedVec2, dir: FixedVec2, dmg: u32, faction: Faction) -> Entity {
        let aid = world.resource_mut::<IdGenerator>().next();
        world.spawn((
            UnitIdComponent(aid), ArrowMarker, LogicalPosition(pos),
            Arrow {
                direction: dir, damage: dmg, from_faction: faction,
                shooter: None, flight_remaining: 50, decay_remaining: 0,
                pierce_chance: 0.0, stuck_to: None, hit_units: Vec::new(),
                start_pos: pos,
            },
        )).id()
    }

    /// Helper: create a city entity at a position
    fn spawn_test_city(world: &mut World, pos: FixedVec2, faction: Faction, radius: u32, hp: u32) -> Entity {
        let cid = world.resource_mut::<IdGenerator>().next();
        world.spawn((
            UnitIdComponent(cid), CityMarker, LogicalPosition(pos),
            CityComponent {
                level: 1, max_level: 5, health_current: hp, health_max: hp,
                population: 0, max_population: 10, spawn_type: SoldierType::Militia,
                spawn_cooldown: 0, level_exp: 0, last_attacker_faction: None,
                arrow_damage_acc: 0,
            },
            CityRadius(radius), FactionComponent(faction),
        )).id()
    }

    // ── 4.3: Arrow hits city, accumulator increments but no health damage (value insufficient) ──

    #[test]
    fn test_arrow_hits_city_accumulator_increments_no_health_loss() {
        let mut world = init_simulation_world(42);
        let city_pos = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(100));
        spawn_test_city(&mut world, city_pos, Faction::Enemy, 30, 500);

        // Arrow at city edge (distance 25), flying toward city center
        let arrow_start = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(95)); // 5 units from center
        let dir = FixedVec2::new(Fixed::ZERO, Fixed::from_int(20)); // fly upward into city
        let dmg = 16u32;
        spawn_test_arrow(&mut world, arrow_start, dir, dmg, Faction::Player);

        arrow_movement_system(&mut world);

        // Find city, check accumulator
        let mut q = world.query::<(Entity, &CityComponent)>();
        let (_, city) = q.iter(&world).next().unwrap();
        assert_eq!(city.arrow_damage_acc, 16, "Accumulator should store 16 damage");
        assert_eq!(city.health_current, 500, "Health unchanged (16 < 200)");
    }

    // ── 4.4: Accumulated hits reach 200 → health reduced ──

    #[test]
    fn test_arrow_accumulation_triggers_health_damage() {
        let mut world = init_simulation_world(42);
        let city_pos = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(100));
        let city_e = spawn_test_city(&mut world, city_pos, Faction::Enemy, 30, 500);

        // Pre-set accumulator close to threshold
        {
            let mut em = world.entity_mut(city_e);
            if let Some(mut c) = em.get_mut::<CityComponent>() {
                c.arrow_damage_acc = 190;
            }
        }

        // Arrow with damage 16 → 190+16=206 → 1 health damage, 6 remainder
        let arrow_start = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(95));
        let dir = FixedVec2::new(Fixed::ZERO, Fixed::from_int(20));
        spawn_test_arrow(&mut world, arrow_start, dir, 16, Faction::Player);

        arrow_movement_system(&mut world);

        let mut q = world.query::<(Entity, &CityComponent)>();
        let (_, city) = q.iter(&world).next().unwrap();
        assert_eq!(city.health_current, 499, "Should lose 1 HP (206/200 = 1)");
        assert_eq!(city.arrow_damage_acc, 6, "Remainder should be 6 (206 % 200)");
    }

    // ── 4.5: Arrow enters decay after hitting city ──

    #[test]
    fn test_arrow_enters_decay_after_city_hit() {
        let mut world = init_simulation_world(42);
        let city_pos = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(100));
        spawn_test_city(&mut world, city_pos, Faction::Enemy, 30, 500);

        let arrow_start = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(95));
        let dir = FixedVec2::new(Fixed::ZERO, Fixed::from_int(20));
        spawn_test_arrow(&mut world, arrow_start, dir, 16, Faction::Player);

        arrow_movement_system(&mut world);

        let mut q = world.query::<(Entity, &Arrow)>();
        let (_, arrow) = q.iter(&world).next().unwrap();
        assert!(arrow.decay_remaining > 0, "Arrow should enter decay after city hit");
    }

    // ── 4.6: Friendly arrow passes through friendly city ──

    #[test]
    fn test_friendly_arrow_passes_through_friendly_city() {
        let mut world = init_simulation_world(42);
        let city_pos = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(100));
        spawn_test_city(&mut world, city_pos, Faction::Player, 30, 500);

        let arrow_start = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(95));
        let dir = FixedVec2::new(Fixed::ZERO, Fixed::from_int(20));
        spawn_test_arrow(&mut world, arrow_start, dir, 16, Faction::Player);

        arrow_movement_system(&mut world);

        let mut q = world.query::<(Entity, &CityComponent)>();
        let (_, city) = q.iter(&world).next().unwrap();
        assert_eq!(city.arrow_damage_acc, 0, "Friendly arrow should not accumulate damage");
        assert_eq!(city.health_current, 500, "Friendly city health unchanged");

        let mut q = world.query::<(Entity, &Arrow)>();
        let (_, arrow) = q.iter(&world).next().unwrap();
        assert_eq!(arrow.decay_remaining, 0, "Arrow should still be in flight (not decay)");
    }

    // ── 4.7: Pierced soldier → same tick hits city behind ──

    #[test]
    fn test_pierced_arrow_hits_city_same_tick() {
        let mut world = init_simulation_world(42);

        // Enemy city at center
        let city_pos = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(100));
        spawn_test_city(&mut world, city_pos, Faction::Enemy, 35, 500);

        // Enemy soldier between arrow and city
        let soldier_pos = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(80));
        let sid = world.resource_mut::<IdGenerator>().next();
        world.spawn((
            UnitIdComponent(sid), SoldierMarker, LogicalPosition(soldier_pos),
            FactionComponent(Faction::Enemy), Health { current: 100, max: 100 },
            SoldierTypeComponent(SoldierType::Infantry),
            SoldierStateComponent(SoldierState::Moving),
            Attack { damage: 10, range: 30, interval_ticks: 10, cooldown_remaining: 0 },
            Movement { speed: 50, target: None, command_target: None, waypoint: None, force_move: false },
            Level { level: 1, exp: 0 }, CityOrigin(UnitId(0)),
        ));

        // Arrow with 100% pierce chance (will pierce soldier and continue to city)
        let arrow_start = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(60));
        let dir = FixedVec2::new(Fixed::ZERO, Fixed::from_int(20));
        let aid = world.resource_mut::<IdGenerator>().next();
        world.spawn((
            UnitIdComponent(aid), ArrowMarker, LogicalPosition(arrow_start),
            Arrow {
                direction: dir, damage: 200, from_faction: Faction::Player,
                shooter: None, flight_remaining: 50, decay_remaining: 0,
                pierce_chance: 1.0, stuck_to: None, hit_units: Vec::new(),
                start_pos: arrow_start,
            },
        ));

        arrow_movement_system(&mut world);

        // Check city took damage (200 damage → 200/200 = 1 HP)
        let mut q = world.query::<(Entity, &CityComponent)>();
        let (_, city) = q.iter(&world).next().unwrap();
        assert_eq!(city.health_current, 499, "City should lose 1 HP (200/200) on same tick after pierce");

        // Arrow should be in decay (stopped by city)
        let mut q = world.query::<(Entity, &Arrow)>();
        let (_, arrow) = q.iter(&world).next().unwrap();
        assert!(arrow.decay_remaining > 0, "Arrow should enter decay after hitting city");
    }
}
