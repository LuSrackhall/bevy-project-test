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
/// Collision radius for arrow hits — must be >= arrow_speed (40) to prevent tunneling.
const ARROW_HIT_RADIUS: i64 = 45 * FIXED_ONE; // 45 units > arrow_speed(40)

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

        let Some((target_id, target_pos)) = nearest else {
            // No enemy in range — restore Moving state
            world.entity_mut(ad.entity).insert(SoldierStateComponent(SoldierState::Moving));
            continue;
        };

        // Compute flight direction and ticks
        let delta = target_pos - ad.pos;
        let dist_internal = integer_sqrt(delta.length_squared().0 * FIXED_ONE);
        if dist_internal <= 0 { continue; }
        let dist = Fixed(dist_internal);
        let dir_unit = FixedVec2::new(delta.x / dist, delta.y / dist);
        let speed = Fixed::from_int(ad.cfg.arrow_speed as i32);
        let direction = FixedVec2::new(dir_unit.x * speed, dir_unit.y * speed);

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

    let mut to_despawn: Vec<Entity> = Vec::new();
    // Collect pierce decisions before query to avoid borrow conflict
    let pierce_rolls: Vec<f32> = {
        let mut rng = world.resource_mut::<DeterministicRng>();
        (0..100).map(|_| rng.gen_probability()).collect()
    };
    let mut pierce_idx = 0usize;

    let mut hits: Vec<(Entity, u32, Option<UnitId>, bool, Option<UnitId>)> = Vec::new();

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

    // Despawn arrows
    for ae in to_despawn { world.despawn(ae); }
}
