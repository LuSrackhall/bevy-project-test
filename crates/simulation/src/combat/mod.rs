pub mod config;

use crate::combat::config::CombatGlobalConfig;
use crate::events::*;
use crate::facing;
use crate::soldier::config::SoldierConfig;
use crate::soldier::spatial_hash::{SpatialEntry, SpatialHash};
use crate::soldier::*;
use crate::types::*;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use std::collections::{BTreeMap, HashMap};

// ══════════ Arrow Types ══════════

/// Directional arrow — flies in a fixed direction, collides with enemies.
/// Flight phase (flight_remaining > 0): moves and deals collision damage.
/// Decay phase (decay_remaining > 0): visual only, no damage.
#[derive(Component, Clone, Debug)]
pub struct Arrow {
    pub direction: FixedVec2, // unit vector × speed (movement per tick)
    pub damage: u32,
    pub from_faction: Faction,
    pub shooter: Option<UnitId>,
    pub flight_remaining: u32, // ticks remaining in flight (damage phase)
    pub decay_remaining: u32,  // 0=flight, >0=decay (visual only)
    pub pierce_chance: u32,    // permyriad, pre-computed at launch
    pub stuck_to: Option<UnitId>, // follow target during decay
    pub hit_units: Vec<UnitId>, // already-hit units (prevent double-hit)
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
    if n <= 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Drop a shield on the ground if the dying entity has one.
/// Must be called BEFORE despawning the entity.
pub(crate) fn drop_shield_on_death(world: &mut World, dying_entity: Entity, current_tick: u32) {
    let has_shield = world
        .get::<ShieldItem>(dying_entity)
        .map(|s| s.hp > 0)
        .unwrap_or(false);
    if !has_shield {
        return;
    }

    let shield = world.get::<ShieldItem>(dying_entity).cloned().unwrap();
    let pos = world
        .get::<LogicalPosition>(dying_entity)
        .map(|p| p.0)
        .unwrap_or(FixedVec2::ZERO);
    let faction = world.get::<FactionComponent>(dying_entity).map(|f| f.0);

    world.spawn(DroppedShield {
        shield,
        position: pos,
        drop_tick: current_tick,
        owner_faction: faction,
    });
}

// ══════════ combat_engagement ══════════

/// Complexity: O(s*k), Memory: O(s), Hot Path: Yes
pub fn combat_engagement_system(world: &mut World) {
    let _soldier_config = world.resource::<SoldierConfig>().clone();

    // Ensure shared index exists (built once per tick or on-demand in tests)
    crate::soldier::TickCombatIndex::ensure_exists(world);

    // Build per-faction SpatialHash from shared pos_faction (eliminates faction_map)
    let (all_units, faction_spatial) = {
        let index = world.resource::<crate::soldier::TickCombatIndex>();
        let all_units = index.pos_faction.clone();
        let mut faction_spatial: HashMap<Faction, SpatialHash> = HashMap::new();
        for (&uid, &(pos, fac)) in &all_units {
            faction_spatial.entry(fac)
                .or_insert_with(|| SpatialHash::new(Fixed::from_int(64)))
                .insert(SpatialEntry { pos, radius: 0, unit_id: uid });
        }
        (all_units, faction_spatial)
    }; // Ref dropped here

    // Pre-sort soldier UnitIds for deterministic iteration
    let mut sorted_soldier_uids: Vec<UnitId> = Vec::new();
    // Use HashMap for O(1) lookup instead of Vec::find O(S²)
    let soldier_map: HashMap<UnitId, (Entity, FixedVec2, Faction, SoldierType, SoldierState, bool, Option<UnitId>, Option<UnitId>, u32, bool, u32)> = {
        let mut q = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &FactionComponent,
            &SoldierTypeComponent,
            &SoldierStateComponent,
            &Movement,
            Option<&SeekStance>,
        )>();
        q.iter(world)
            .map(|(e, id, pos, fac, st, sst, mov, seek)| {
                let uid = id.0;
                sorted_soldier_uids.push(uid);
                (uid, (e, pos.0, fac.0, st.0, sst.0, mov.force_move, mov.command_target, mov.target, mov.speed, seek.is_some_and(|s| s.active), seek.map_or(0, |s| s.seek_range)))
            })
            .collect()
    };
    sorted_soldier_uids.sort();

    for &suid in &sorted_soldier_uids {
        let &(entity, pos, faction, stype, state, force_move, cmd_target, target, speed, seek_active, seek_range) =
            soldier_map.get(&suid).unwrap();
        let uid = suid;

        if force_move {
            continue;
        }

        if !seek_active || seek_range == 0 {
            if state == SoldierState::Fighting && cmd_target.is_none() {
                let mut em = world.entity_mut(entity);
                em.insert(Movement {
                    speed,
                    target: cmd_target,
                    command_target: None,
                    waypoint: None,
                    force_move: false,
                });
                em.insert(SoldierStateComponent(SoldierState::Moving));
            }
            continue;
        }

        let aggro = Fixed::from_int(seek_range as i32);
        let aggro_sq = aggro * aggro;

        // Find nearest enemy using per-faction SpatialHash (no faction check needed)
        // For large ranges (>3 cells), use direct scan instead of query_range
        let mut best: Option<(UnitId, i64)> = None;
        let large_range = seek_range > 192; // >3 cells → query_range degenerates to full scan

        for (&ffac, enemy_spatial) in &faction_spatial {
            if ffac == faction { continue; }
            if large_range {
                // Direct scan: iterate all entries in enemy SpatialHash (flat Vec per cell)
                for entry in enemy_spatial.iter_all() {
                    if entry.unit_id == uid { continue; }
                    let ds = (pos - entry.pos).length_squared();
                    if ds <= aggro_sq && best.as_ref().is_none_or(|(_, d)| ds.0 < *d) {
                        best = Some((entry.unit_id, ds.0));
                    }
                }
            } else {
                let neighbors = enemy_spatial.query_range(pos, aggro.0);
                for entry in &neighbors {
                    if entry.unit_id == uid { continue; }
                    let ds = (pos - entry.pos).length_squared();
                    if ds <= aggro_sq && best.as_ref().is_none_or(|(_, d)| ds.0 < *d) {
                        best = Some((entry.unit_id, ds.0));
                    }
                }
            }
        }

        let mut em = world.entity_mut(entity);
        if let Some((enemy_id, _)) = best {
            let is_cav = stype == SoldierType::Cavalry;
            if !is_cav {
                let ct = if cmd_target.is_none() { target } else { cmd_target };
                em.insert(Movement {
                    speed,
                    target: Some(enemy_id),
                    command_target: ct,
                    waypoint: None,
                    force_move: false,
                });
            }
            em.insert(SoldierStateComponent(SoldierState::Fighting));
        } else if state == SoldierState::Fighting {
            let ct = cmd_target;
            em.insert(Movement {
                speed,
                target: ct,
                command_target: cmd_target,
                waypoint: None,
                force_move: false,
            });
            em.insert(SoldierStateComponent(SoldierState::Moving));
        }
    }
}

// ══════════ Shield block helper (manual + passive) ══════════

/// Attempt shield block (manual frontal + passive chance).
/// Returns remaining damage after shield absorption.
/// Call AFTER cavalry dodge, BEFORE applying damage to Health.
fn try_passive_block(
    world: &mut World,
    target_entity: Entity,
    mut damage: u32,
    attacker_pos: Option<FixedVec2>,
) -> u32 {
    if damage == 0 {
        return 0;
    }

    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // ── Manual block check (infantry in Blocking state) ──
    let is_blocking = world
        .get::<ShieldComponent>(target_entity)
        .is_some_and(|sc| sc.state == ShieldState::Blocking);

    if is_blocking {
        if let (Some(facing_comp), Some(atk_pos)) =
            (world.get::<FacingDirection>(target_entity), attacker_pos)
        {
            if let Some(target_pos) = world.get::<LogicalPosition>(target_entity).map(|p| p.0) {
                let attack_angle = facing::compute_angle_between(target_pos, atk_pos);
                let deviation = facing::angle_distance(facing_comp.angle, attack_angle);
                let frontal_half =
                    Fixed::from_int(combat_config.shield.frontal_angle_deg as i32 / 2);

                if deviation <= frontal_half {
                    // Frontal hit — 100% absorbed by shield
                    if let Some(mut shield_item) = world.get_mut::<ShieldItem>(target_entity) {
                        if shield_item.hp <= damage {
                            let absorbed = shield_item.hp;
                            shield_item.hp = 0;
                            damage -= absorbed;
                            world.entity_mut(target_entity).remove::<ShieldComponent>();
                            world.entity_mut(target_entity).remove::<ShieldItem>();
                        } else {
                            shield_item.hp -= damage;
                            return 0; // all damage absorbed
                        }
                    }
                    return damage;
                }
                // Non-frontal during Blocking: damage goes directly to HP, skip passive block
                return damage;
            }
        }
    }

    // ── Passive block check (40% chance, any direction) ──
    if world.get::<ShieldComponent>(target_entity).is_none() {
        return damage;
    }

    let passive_block_chance = combat_config.shield.passive_block_chance;
    let block_roll = world
        .resource_mut::<DeterministicRng>()
        .gen_probability_permyriad();

    if block_roll < passive_block_chance {
        if let Some(mut shield_item) = world.get_mut::<ShieldItem>(target_entity) {
            if shield_item.hp <= damage {
                let shield_damage = shield_item.hp;
                shield_item.hp = 0;
                damage -= shield_damage;
                world.entity_mut(target_entity).remove::<ShieldComponent>();
                world.entity_mut(target_entity).remove::<ShieldItem>();
            } else {
                shield_item.hp -= damage;
                damage = 0;
            }
        }
    }
    damage
}

// ══════════ melee_attack ══════════

/// Complexity: O(a*k), Memory: O(s), Hot Path: Yes
pub fn melee_attack_system(world: &mut World, current_tick: u32) {
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // Ensure shared index exists
    crate::soldier::TickCombatIndex::ensure_exists(world);

    // Tick cooldowns
    {
        let mut q = world.query::<(Entity, &mut Attack, &SoldierTypeComponent)>();
        for (_, mut atk, st) in q.iter_mut(world) {
            if st.0 != SoldierType::Archer {
                atk.cooldown_remaining = atk.cooldown_remaining.saturating_sub(1);
            }
        }
    }

    // Extract data from shared tick index then drop Ref
    let (faction_spatial, attackers) = {
        let index = world.resource::<crate::soldier::TickCombatIndex>();
        let soldiers = index.soldiers.clone();
        // Build per-faction SpatialHash (eliminates faction_map)
        let mut faction_spatial: HashMap<Faction, SpatialHash> = HashMap::new();
        for (&uid, ref s) in &soldiers {
            faction_spatial.entry(s.faction)
                .or_insert_with(|| SpatialHash::new(Fixed::from_int(32)))
                .insert(SpatialEntry { pos: s.pos, radius: 0, unit_id: uid });
        }
        let mut attackers: Vec<(UnitId, crate::soldier::SoldierSnapshot)> = soldiers
            .iter()
            .filter(|(_, s)| s.attack.cooldown_remaining == 0 && s.soldier_type != SoldierType::Archer)
            .map(|(&uid, s)| (uid, *s))
            .collect();
        attackers.sort_by_key(|(uid, _)| *uid);
        (faction_spatial, attackers)
    }; // Ref dropped here

    let mut pending_deaths: Vec<(Entity, Option<UnitId>, Option<UnitId>)> = Vec::new(); // (target, killer, city_origin)
    let mut xp_grants: Vec<(Entity, u32)> = Vec::new();
    let mut ls_hits: Vec<(Entity, u32, u32, bool)> = Vec::new();
    let mut ls_kills: Vec<(Entity, u32, u32, bool)> = Vec::new();

    let windup_config = combat_config.attack_windup.clone();

    for (uid, s) in &attackers {
        let uid = *uid;
        // Skip if entity was despawned earlier in this loop (stale snapshot)
        if world.get_entity(s.entity).is_err() {
            continue;
        }
        // ForceMove suppression: non-cavalry units skip attack during force_move
        if s.force_move && s.soldier_type != SoldierType::Cavalry {
            continue;
        }

        // Scan for nearest enemy in attack range
        // Blocking units only attack enemies in frontal 120° arc
        let is_blocking = world
            .get::<ShieldComponent>(s.entity)
            .is_some_and(|sc| sc.state == ShieldState::Blocking);
        let frontal_half = if is_blocking {
            let cfg = world.resource::<CombatGlobalConfig>();
            Fixed::from_int(cfg.shield.frontal_angle_deg as i32 / 2)
        } else {
            Fixed::from_int(180) // no restriction
        };

        let range_f = Fixed::from_int(s.attack.range as i32);
        let range_sq = range_f * range_f;
        let mut best_target: Option<(UnitId, FixedVec2, i64)> = None;
        // Use per-faction SpatialHash (no faction check per neighbor)
        for (&ffac, enemy_spatial) in &faction_spatial {
            if ffac == s.faction { continue; }
            let neighbors = enemy_spatial.query_range(s.pos, range_f.0);
            for entry in &neighbors {
                if entry.unit_id == uid { continue; }
                let dist_sq = (s.pos - entry.pos).length_squared();
                if dist_sq <= range_sq {
                    // Blocking: check frontal angle
                    if is_blocking {
                        let attack_angle = facing::compute_angle_between(s.pos, entry.pos);
                        let deviation = facing::angle_distance(s.facing, attack_angle);
                        if deviation > frontal_half {
                            continue;
                        }
                    }
                    if best_target
                        .as_ref()
                        .is_none_or(|(_, _, bd)| dist_sq.0 < *bd)
                    {
                        best_target = Some((entry.unit_id, entry.pos, dist_sq.0));
                    }
                }
            }
        }
        let Some((tid, tpos, _)) = best_target else {
            continue;
        };

        // Non-cavalry: start windup instead of attacking immediately
        if s.soldier_type != SoldierType::Cavalry || !windup_config.cavalry_no_windup {
            let already_winding = world
                .entity(s.entity)
                .get::<AttackWindup>()
                .is_some_and(|w| w.remaining_ticks > 0);
            if !already_winding {
                world.entity_mut(s.entity).insert(AttackWindup {
                    remaining_ticks: windup_config.windup_ticks,
                    target: Some(tid),
                });
            }
            continue;
        }

        // Cavalry with cavalry_no_windup: attack immediately
        let Some(te) = find_entity_by_unit_id(world, tid) else {
            continue;
        };

        let (thp, tmax, tst, _tfac, city_origin) = {
            let em = world.entity(te);
            let hp = em.get::<Health>();
            (
                hp.map(|h| h.current),
                hp.map(|h| h.max),
                em.get::<SoldierTypeComponent>().map(|s| s.0),
                em.get::<FactionComponent>().map(|f| f.0),
                em.get::<CityOrigin>().map(|c| c.0),
            )
        };
        let Some(hp_cur) = thp else { continue };
        let Some(hp_max) = tmax else { continue };

        let mut damage = s.attack.damage + s.level * combat_config.level_up.attack_gain;
        if s.has_fearless {
            damage += combat_config.fearless.attack_bonus;
        }

        // Cavalry dodge (permyriad arithmetic)
        if tst == Some(SoldierType::Cavalry) && hp_max > 0 {
            let hp_ratio_pm = (hp_cur as u64 * 10000) / hp_max as u64; // 0..10000
            let decay =
                ((10000 - hp_ratio_pm) * combat_config.cavalry.dodge_decay_rate as u64) / 10000;
            let dc = (combat_config.cavalry.dodge_max_chance as u64).saturating_sub(decay);
            if (world
                .resource_mut::<DeterministicRng>()
                .gen_probability_permyriad() as u64)
                < dc
            {
                damage = 0;
                world.entity_mut(te).insert(FearlessBuff {
                    remaining_ticks: combat_config.fearless.duration_ticks,
                });
            }
        }

        // Shield block check (after dodge, before damage application)
        damage = try_passive_block(world, te, damage, Some(s.pos));

        if damage > 0 {
            let died = {
                let mut em = world.entity_mut(te);
                if let Some(mut hp) = em.get_mut::<Health>() {
                    hp.current = hp.current.saturating_sub(damage);
                    hp.current == 0
                } else {
                    false
                }
            };

            if died {
                pending_deaths.push((te, Some(uid), city_origin));
                xp_grants.push((s.entity, combat_config.level_up.exp_per_kill));
                ls_kills.push((s.entity, damage, s.level, s.has_fearless));
            } else {
                ls_hits.push((s.entity, damage, s.level, s.has_fearless));
            }
        }

        // Reset attacker cooldown (penalized if blocking, affected by facing)
        let base_cooldown = if world
            .get::<ShieldComponent>(s.entity)
            .is_some_and(|sc| sc.state == ShieldState::Blocking)
        {
            combat_config.shield.attack_speed_penalty
        } else {
            s.attack.interval_ticks
        };
        // Apply facing attack speed factor (not for cavalry instant attacks)
        let cooldown = if s.soldier_type == SoldierType::Cavalry && windup_config.cavalry_no_windup {
            base_cooldown
        } else {
            let target_angle = facing::compute_angle_between(s.pos, tpos);
            let factor = facing::facing_atk_speed_factor(s.facing, target_angle);
            // effective_cooldown = base / factor
            // factor is Fixed with 8 fractional bits
            // base / factor = base * 256 / factor.0
            let factor_i64 = factor.0.max(1); // avoid division by zero
            let effective = (base_cooldown as i64 * 256) / factor_i64;
            effective.max(1) as u32
        };
        world.entity_mut(s.entity).insert(Attack {
            damage: s.attack.damage,
            range: s.attack.range,
            interval_ticks: s.attack.interval_ticks,
            cooldown_remaining: cooldown,
        });
    }

    // Apply lifesteal & XP (permyriad arithmetic)
    for (e, dmg, lvl, hf) in ls_kills.iter().chain(ls_hits.iter()) {
        let mut ls_pm = if *lvl >= combat_config.level_up.lifesteal_unlock_level {
            combat_config.level_up.lifesteal_rate
        } else {
            0
        };
        if *hf {
            ls_pm += combat_config.fearless.lifesteal_bonus;
        }
        if ls_pm > 0 {
            if let Some(mut hp) = world.entity_mut(*e).get_mut::<Health>() {
                let heal = (*dmg as u64 * ls_pm as u64 / 10000) as u32;
                hp.current = (hp.current + heal).min(hp.max);
            }
        }
    }
    for (e, xp) in xp_grants {
        if let Some(mut lvl) = world.entity_mut(e).get_mut::<Level>() {
            lvl.exp += xp;
        }
    }

    // Process deaths (dedup by entity — same target may be killed by multiple attackers)
    let mut seen = std::collections::HashSet::new();
    for (te, kid, origin) in pending_deaths {
        if seen.contains(&te) {
            continue;
        }
        seen.insert(te);
        let uid = find_unit_id(world, te).unwrap_or(UnitId(0));
        drop_shield_on_death(world, te, current_tick);
        world.despawn(te);
        // Update incremental index
        if let Some(mut index) = world.get_resource_mut::<crate::unit_index::UnitIdEntityIndex>() {
            index.remove(uid);
        }
        // Decrement origin city population when soldier dies in combat
        if let Some(origin_id) = origin {
            if let Some(oe) = find_entity_by_unit_id(world, origin_id) {
                if let Some(mut c) = world.entity_mut(oe).get_mut::<CityComponent>() {
                    c.population = c.population.saturating_sub(1);
                }
            }
        }
        let mut events = world.resource_mut::<SimulationEvents>();
        events.destroyed.push(UnitDestroyed {
            unit_id: uid,
            killer_id: kid,
        });
    }
}

// ══════════ melee_attack (end) ══════════

// ══════════ attack_windup ══════════

/// Process attack windups — when windup completes, apply the attack.
pub fn attack_windup_system(world: &mut World, current_tick: u32) {
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // Collect entities with active windups (remaining_ticks > 0)
    let mut windup_entities: Vec<(Entity, u32, Option<UnitId>)> = Vec::new();
    let mut cancel_windups: Vec<Entity> = Vec::new();
    {
        let mut q = world.query::<(Entity, &AttackWindup, &SoldierTypeComponent, &Movement)>();
        for (entity, windup, st, mov) in q.iter(world) {
            if windup.remaining_ticks > 0 {
                // ForceMove suppression: cancel windup for non-cavalry
                if mov.force_move && st.0 != SoldierType::Cavalry {
                    cancel_windups.push(entity);
                    continue;
                }
                windup_entities.push((entity, windup.remaining_ticks, windup.target));
            }
        }
    }
    // Apply windup cancellations
    for entity in cancel_windups {
        world.entity_mut(entity).insert(AttackWindup {
            remaining_ticks: 0,
            target: None,
        });
    }

    // Decrement windups and collect completed ones
    let mut ready: Vec<(Entity, UnitId)> = Vec::new();
    for (entity, remaining, target) in windup_entities {
        let new_remaining = remaining - 1;
        if new_remaining == 0 {
            if let Some(target_id) = target {
                ready.push((entity, target_id));
            }
            // Clear windup state
            world.entity_mut(entity).insert(AttackWindup {
                remaining_ticks: 0,
                target: None,
            });
        } else {
            world.entity_mut(entity).insert(AttackWindup {
                remaining_ticks: new_remaining,
                target,
            });
        }
    }

    // Position lookup for range checks
    let positions: HashMap<UnitId, FixedVec2> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition)>();
        q.iter(world).map(|(_, id, pos)| (id.0, pos.0)).collect()
    };

    // Apply attacks for completed windups
    let mut pending_deaths: Vec<(Entity, Option<UnitId>, Option<UnitId>)> = Vec::new();
    let mut xp_grants: Vec<(Entity, u32)> = Vec::new();
    let mut ls_hits: Vec<(Entity, u32, u32, bool)> = Vec::new();
    let mut ls_kills: Vec<(Entity, u32, u32, bool)> = Vec::new();

    for (attacker_entity, target_id) in ready {
        // Get attacker data
        let (ad_uid, ad_pos, ad_dmg, ad_range, ad_level, ad_interval, ad_has_fearless) = {
            let em = world.entity(attacker_entity);
            let uid = em.get::<UnitIdComponent>().map(|c| c.0);
            let pos = em.get::<LogicalPosition>().map(|p| p.0);
            let atk = em.get::<Attack>();
            let lvl = em.get::<Level>().map(|l| l.level);
            let fb = em.get::<FearlessBuff>().is_some();
            match (uid, pos, atk, lvl) {
                (Some(u), Some(p), Some(a), Some(l)) => {
                    (u, p, a.damage, a.range, l, a.interval_ticks, fb)
                }
                _ => continue,
            }
        };

        // Verify target is still in range
        let Some(&tpos) = positions.get(&target_id) else {
            continue;
        };
        let range_f = Fixed::from_int(ad_range as i32);
        if (ad_pos - tpos).length_squared() > range_f * range_f {
            continue;
        }

        let Some(te) = find_entity_by_unit_id(world, target_id) else {
            continue;
        };

        let (thp, tmax, tst, _tfac, city_origin) = {
            let em = world.entity(te);
            let hp = em.get::<Health>();
            (
                hp.map(|h| h.current),
                hp.map(|h| h.max),
                em.get::<SoldierTypeComponent>().map(|s| s.0),
                em.get::<FactionComponent>().map(|f| f.0),
                em.get::<CityOrigin>().map(|c| c.0),
            )
        };
        let Some(hp_cur) = thp else { continue };
        let Some(hp_max) = tmax else { continue };

        // Compute damage (same formula as melee_attack_system)
        let mut damage = ad_dmg + ad_level * combat_config.level_up.attack_gain;
        if ad_has_fearless {
            damage += combat_config.fearless.attack_bonus;
        }

        // Cavalry dodge (permyriad arithmetic)
        if tst == Some(SoldierType::Cavalry) && hp_max > 0 {
            let hp_ratio_pm = (hp_cur as u64 * 10000) / hp_max as u64;
            let decay =
                ((10000 - hp_ratio_pm) * combat_config.cavalry.dodge_decay_rate as u64) / 10000;
            let dc = (combat_config.cavalry.dodge_max_chance as u64).saturating_sub(decay);
            if (world
                .resource_mut::<DeterministicRng>()
                .gen_probability_permyriad() as u64)
                < dc
            {
                damage = 0;
                world.entity_mut(te).insert(FearlessBuff {
                    remaining_ticks: combat_config.fearless.duration_ticks,
                });
            }
        }

        // Shield block check (after dodge, before damage application)
        damage = try_passive_block(world, te, damage, Some(ad_pos));

        if damage > 0 {
            let died = {
                let mut em = world.entity_mut(te);
                if let Some(mut hp) = em.get_mut::<Health>() {
                    hp.current = hp.current.saturating_sub(damage);
                    hp.current == 0
                } else {
                    false
                }
            };

            if died {
                pending_deaths.push((te, Some(ad_uid), city_origin));
                xp_grants.push((attacker_entity, combat_config.level_up.exp_per_kill));
                ls_kills.push((attacker_entity, damage, ad_level, ad_has_fearless));
            } else {
                ls_hits.push((attacker_entity, damage, ad_level, ad_has_fearless));
            }
        }

        // Reset attacker cooldown (penalized if blocking, affected by facing)
        let base_cooldown = if world
            .get::<ShieldComponent>(attacker_entity)
            .is_some_and(|sc| sc.state == ShieldState::Blocking)
        {
            combat_config.shield.attack_speed_penalty
        } else {
            ad_interval
        };
        // Apply facing attack speed factor
        let cooldown = {
            let attacker_facing = world
                .get::<FacingDirection>(attacker_entity)
                .map(|f| f.angle)
                .unwrap_or(Fixed::ZERO);
            let target_pos = positions.get(&target_id).copied().unwrap_or(ad_pos);
            let target_angle = facing::compute_angle_between(ad_pos, target_pos);
            let factor = facing::facing_atk_speed_factor(attacker_facing, target_angle);
            let factor_i64 = factor.0.max(1);
            let effective = (base_cooldown as i64 * 256) / factor_i64;
            effective.max(1) as u32
        };
        world.entity_mut(attacker_entity).insert(Attack {
            damage: ad_dmg,
            range: ad_range,
            interval_ticks: ad_interval,
            cooldown_remaining: cooldown,
        });
    }

    // Apply lifesteal & XP (permyriad arithmetic)
    for (e, dmg, lvl, hf) in ls_kills.iter().chain(ls_hits.iter()) {
        let mut ls_pm = if *lvl >= combat_config.level_up.lifesteal_unlock_level {
            combat_config.level_up.lifesteal_rate
        } else {
            0
        };
        if *hf {
            ls_pm += combat_config.fearless.lifesteal_bonus;
        }
        if ls_pm > 0 {
            if let Some(mut hp) = world.entity_mut(*e).get_mut::<Health>() {
                let heal = (*dmg as u64 * ls_pm as u64 / 10000) as u32;
                hp.current = (hp.current + heal).min(hp.max);
            }
        }
    }
    for (e, xp) in xp_grants {
        if let Some(mut lvl) = world.entity_mut(e).get_mut::<Level>() {
            lvl.exp += xp;
        }
    }

    // Process deaths
    let mut seen = std::collections::HashSet::new();
    for (te, kid, origin) in pending_deaths {
        if seen.contains(&te) {
            continue;
        }
        seen.insert(te);
        let uid = find_unit_id(world, te).unwrap_or(UnitId(0));
        drop_shield_on_death(world, te, current_tick);
        world.despawn(te);
        if let Some(origin_id) = origin {
            if let Some(oe) = find_entity_by_unit_id(world, origin_id) {
                if let Some(mut c) = world.entity_mut(oe).get_mut::<CityComponent>() {
                    c.population = c.population.saturating_sub(1);
                }
            }
        }
        let mut events = world.resource_mut::<SimulationEvents>();
        events.destroyed.push(UnitDestroyed {
            unit_id: uid,
            killer_id: kid,
        });
    }
}

// ══════════ attack_windup (end) ══════════

// ══════════ archer_attack (direction-based) ══════════

/// Complexity: O(archers*k), Memory: O(s+c), Hot Path: Yes
pub fn archer_attack_system(world: &mut World) {
    let soldier_config = world.resource::<SoldierConfig>().clone();
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // Ensure shared index exists
    crate::soldier::TickCombatIndex::ensure_exists(world);

    // Tick cooldowns for archers
    {
        let mut q = world.query::<(Entity, &mut Attack, &SoldierTypeComponent)>();
        for (_, mut atk, st) in q.iter_mut(world) {
            if st.0 == SoldierType::Archer {
                atk.cooldown_remaining = atk.cooldown_remaining.saturating_sub(1);
            }
        }
    }

    // Extract data from shared tick index then drop Ref — per-faction SpatialHash
    let soldier_faction_spatial = {
        let index = world.resource::<crate::soldier::TickCombatIndex>();
        let mut faction_spatial: HashMap<Faction, SpatialHash> = HashMap::new();
        for (&uid, &(pos, fac)) in &index.pos_faction {
            faction_spatial.entry(fac)
                .or_insert_with(|| SpatialHash::new(Fixed::from_int(200)))
                .insert(SpatialEntry { pos, radius: 0, unit_id: uid });
        }
        faction_spatial
    }; // Ref dropped here

    // Collect enemy city positions + build SpatialHash
    let all_cities: Vec<(UnitId, FixedVec2, Faction)> = {
        let mut q = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &FactionComponent,
            &CityMarker,
        )>();
        q.iter(world)
            .map(|(_, id, pos, fac, _)| (id.0, pos.0, fac.0))
            .collect()
    };
    let city_faction_map: HashMap<UnitId, Faction> =
        all_cities.iter().map(|(id, _, fac)| (*id, *fac)).collect();
    let mut city_spatial = SpatialHash::new(Fixed::from_int(200));
    for &(uid, pos, _) in &all_cities {
        city_spatial.insert(SpatialEntry { pos, radius: 0, unit_id: uid });
    }

    // Collect archers ready to fire
    struct ArcData {
        entity: Entity,
        uid: UnitId,
        pos: FixedVec2,
        faction: Faction,
        dmg: u32,
        range: u32,
        interval: u32,
        level: u32,
        cfg: crate::soldier::config::SoldierUnitConfig,
    }
    let archers: Vec<ArcData> = {
        let mut q = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &FactionComponent,
            &Attack,
            &Level,
            &SoldierTypeComponent,
        )>();
        q.iter(world)
            .filter(|(_, _, _, _, atk, _, st)| {
                atk.cooldown_remaining == 0 && st.0 == SoldierType::Archer
            })
            .map(|(e, id, pos, fac, atk, lvl, _st)| {
                let cfg = soldier_config.get(SoldierType::Archer).clone();
                let range = cfg.compute_attack_range(lvl.level);
                ArcData {
                    entity: e,
                    uid: id.0,
                    pos: pos.0,
                    faction: fac.0,
                    dmg: atk.damage,
                    range,
                    interval: cfg.attack_interval_ticks,
                    level: lvl.level,
                    cfg,
                }
            })
            .collect()
    };

    for ad in archers {
        // Find all enemy soldiers in range using SpatialHash
        let range_fixed = Fixed::from_int(ad.range as i32);
        let range_sq = range_fixed * range_fixed;
        let mut enemy_soldiers_in_range: Vec<(UnitId, FixedVec2)> = Vec::new();
        let mut nearest: Option<(UnitId, FixedVec2)> = None;
        let mut nearest_d = i64::MAX;
        for (&ffac, enemy_spatial) in &soldier_faction_spatial {
            if ffac == ad.faction { continue; }
            for entry in &enemy_spatial.query_nearby(ad.pos) {
                if entry.unit_id == ad.uid { continue; }
                let ds = (ad.pos - entry.pos).length_squared();
                if ds <= range_sq {
                    enemy_soldiers_in_range.push((entry.unit_id, entry.pos));
                    if ds.0 < nearest_d {
                        nearest = Some((entry.unit_id, entry.pos));
                        nearest_d = ds.0;
                    }
                }
            }
        }

        // Fallback: if no soldiers in range, search nearest enemy city using SpatialHash
        let Some((target_id, target_pos)) = nearest.or_else(|| {
            let mut best: Option<(UnitId, FixedVec2)> = None;
            let mut best_d = i64::MAX;
            for entry in &city_spatial.query_nearby(ad.pos) {
                if let Some(&cfac) = city_faction_map.get(&entry.unit_id) {
                    if cfac == ad.faction { continue; }
                    let ds = (ad.pos - entry.pos).length_squared();
                    if ds <= range_sq && ds.0 < best_d {
                        best = Some((entry.unit_id, entry.pos));
                        best_d = ds.0;
                    }
                }
            }
            best
        }) else {
            // No enemy soldier or city in range — restore Moving state
            world
                .entity_mut(ad.entity)
                .insert(SoldierStateComponent(SoldierState::Moving));
            continue;
        };

        // ── Multi-shot check (permyriad) ──
        let multi_cfg = &combat_config.archer_multi_shot;
        let multi_chance = (multi_cfg.base_chance + ad.level * multi_cfg.per_level_bonus)
            .min(multi_cfg.max_chance)
            .max(multi_cfg.min_chance);

        let targets_to_hit: Vec<(UnitId, FixedVec2)> = {
            let mut rng = world.resource_mut::<DeterministicRng>();
            let roll = rng.gen_probability_permyriad();

            if roll < multi_chance && enemy_soldiers_in_range.len() > 1 {
                // Multi-shot: pick 2-5 random enemies in range
                let num_shots = 2 + (rng.next_u64() % 4) as u32; // 2-5
                let num_shots = num_shots.min(enemy_soldiers_in_range.len() as u32);

                // Collect candidates excluding primary target
                let mut candidates: Vec<(UnitId, FixedVec2)> = enemy_soldiers_in_range
                    .iter()
                    .filter(|(id, _)| *id != target_id)
                    .cloned()
                    .collect();

                // Shuffle using Fisher-Yates with deterministic RNG
                for i in (1..candidates.len()).rev() {
                    let j = (rng.next_u64() as usize) % (i + 1);
                    candidates.swap(i, j);
                }

                // Take num_shots - 1 extra targets + primary
                let mut targets = vec![(target_id, target_pos)];
                targets.extend(candidates.into_iter().take((num_shots - 1) as usize));
                targets
            } else {
                // Single shot
                vec![(target_id, target_pos)]
            }
        };
        // RNG borrow dropped here before spawning arrows

        // Fire arrows at all targets
        let flight_ticks = ad.cfg.compute_flight_ticks(ad.level);
        let pierce_chance = ad.cfg.compute_pierce_chance(ad.level);
        let damage = ad.dmg + ad.level * combat_config.level_up.attack_gain;
        let speed = Fixed::from_int(ad.cfg.arrow_speed as i32);
        let shooter_id = find_unit_id(world, ad.entity);

        for (_t_id, t_pos) in &targets_to_hit {
            // Compute flight direction with spread (65% dead-on, 35% ±0.1°–10°)
            let delta = *t_pos - ad.pos;
            let dist_internal = integer_sqrt(delta.length_squared().0 * FIXED_ONE);
            if dist_internal <= 0 {
                continue;
            }
            let dist = Fixed(dist_internal);
            let dir_unit = FixedVec2::new(delta.x / dist, delta.y / dist);

            // Spread: 65% dead-on, 35% ±0.1°–10°
            let spread_angle = {
                let mut rng = world.resource_mut::<DeterministicRng>();
                if rng.gen_probability_permyriad() < 6500 {
                    // 65%
                    Fixed::ZERO // perfect aim
                } else {
                    let angle = Fixed(1 + (rng.next_u64() % 44) as i64); // 0.1°–10°
                    if rng.gen_probability_permyriad() < 5000 {
                        Fixed(-angle.0)
                    } else {
                        angle
                    }
                }
            };

            // Apply rotation via small-angle approx: sin(θ)≈θ, cos(θ)≈1
            let sin_a = spread_angle;
            let cos_a = Fixed::ONE; // cos(small) ≈ 1
            let rotated_x = dir_unit.x * cos_a - dir_unit.y * sin_a;
            let rotated_y = dir_unit.x * sin_a + dir_unit.y * cos_a;

            let direction = FixedVec2::new(rotated_x * speed, rotated_y * speed);

            // Spawn arrow
            let aid = { world.resource_mut::<IdGenerator>().next_id() };
            world.spawn((
                UnitIdComponent(aid),
                ArrowMarker,
                LogicalPosition(ad.pos),
                Arrow {
                    direction,
                    damage,
                    from_faction: ad.faction,
                    shooter: shooter_id,
                    flight_remaining: flight_ticks,
                    decay_remaining: 0,
                    pierce_chance,
                    stuck_to: None,
                    hit_units: Vec::new(),
                    start_pos: ad.pos,
                },
            ));

            let mut events = world.resource_mut::<SimulationEvents>();
            events.spawned.push(UnitSpawned {
                unit_id: aid,
                pos: ad.pos,
                faction: ad.faction,
                unit_kind: UnitKind::Arrow,
            });
        }

        // Reset cooldown and set Fighting state (prevents movement while shooting)
        world.entity_mut(ad.entity).insert(Attack {
            damage: ad.dmg,
            range: ad.range,
            interval_ticks: ad.interval,
            cooldown_remaining: ad.interval,
        });
        world
            .entity_mut(ad.entity)
            .insert(SoldierStateComponent(SoldierState::Fighting));
    }
}

// ══════════ arrow_movement (flight + collision + decay) ══════════

/// Complexity: O(arrows*k), Memory: O(s+c), Hot Path: Yes
pub fn arrow_movement_system(world: &mut World, current_tick: u32) {
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // Ensure shared index exists
    crate::soldier::TickCombatIndex::ensure_exists(world);

    // Extract data from shared tick index then drop Ref
    let (soldier_faction_map, soldier_entity_map, soldier_pos_map, soldier_spatial) = {
        let index = world.resource::<crate::soldier::TickCombatIndex>();
        let soldiers = &index.soldiers;
        let faction_map: HashMap<UnitId, Faction> =
            soldiers.iter().map(|(&id, s)| (id, s.faction)).collect();
        let entity_map: HashMap<UnitId, Entity> =
            soldiers.iter().map(|(&id, s)| (id, s.entity)).collect();
        let pos_map: HashMap<UnitId, FixedVec2> =
            soldiers.iter().map(|(&id, s)| (id, s.pos)).collect();
        let mut spatial = SpatialHash::new(Fixed::from_int(32));
        for (&uid, s) in soldiers {
            spatial.insert(SpatialEntry { pos: s.pos, radius: 0, unit_id: uid });
        }
        (faction_map, entity_map, pos_map, spatial)
    }; // Ref dropped here

    // Collect city positions + SpatialHash for collision
    let all_cities: Vec<(UnitId, FixedVec2, Entity, Faction, u32)> = {
        let mut q = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &FactionComponent,
            &CityMarker,
            &CityRadius,
        )>();
        q.iter(world)
            .map(|(e, id, pos, fac, _, r)| (id.0, pos.0, e, fac.0, r.0))
            .collect()
    };
    let city_faction_map: HashMap<UnitId, Faction> =
        all_cities.iter().map(|(id, _, _, fac, _)| (*id, *fac)).collect();
    let city_entity_map: HashMap<UnitId, (Entity, u32)> =
        all_cities.iter().map(|(id, _, e, _, r)| (*id, (*e, *r))).collect();
    let mut city_spatial = SpatialHash::new(Fixed::from_int(200));
    for &(uid, pos, _, _, _) in &all_cities {
        city_spatial.insert(SpatialEntry { pos, radius: 0, unit_id: uid });
    }

    // arrow_building_damage_ratio is permyriad (e.g. 50 = 0.5%). Denom = 10000 / ratio.
    let arrow_building_damage_denom = 10000u32
        .checked_div(combat_config.arrow_building_damage_ratio)
        .unwrap_or(0);

    let mut to_despawn: Vec<Entity> = Vec::new();
    // Collect pierce decisions before query to avoid borrow conflict
    let pierce_rolls: Vec<u32> = {
        let mut rng = world.resource_mut::<DeterministicRng>();
        (0..100).map(|_| rng.gen_probability_permyriad()).collect()
    };
    let mut pierce_idx = 0usize;

    // (arrow_entity, damage, stuck_to_unit, pierced, shooter, hit_position)
    type ArrowHit = (Entity, u32, Option<UnitId>, bool, Option<UnitId>, FixedVec2);
    let mut hits: Vec<ArrowHit> = Vec::new();
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
                    if let Some(&epos) = soldier_pos_map.get(&sid) {
                        arrow_pos.0 = epos;
                    }
                }
                if arrow.decay_remaining == 0 {
                    to_despawn.push(ae);
                }
                continue;
            }

            if arrow.flight_remaining > 0 {
                arrow.flight_remaining -= 1;
                arrow_pos.0 = FixedVec2::new(
                    arrow_pos.0.x + arrow.direction.x,
                    arrow_pos.0.y + arrow.direction.y,
                );

                // Soldier collision using SpatialHash
                let mut stopped_by_soldier = false;
                let neighbors = soldier_spatial.query_nearby(arrow_pos.0);
                for entry in &neighbors {
                    if let Some(&efac) = soldier_faction_map.get(&entry.unit_id) {
                        if efac == arrow.from_faction { continue; }
                    } else { continue; }
                    if arrow.hit_units.contains(&entry.unit_id) { continue; }
                    if (arrow_pos.0 - entry.pos).length_squared() <= threshold_sq {
                        arrow.hit_units.push(entry.unit_id);
                        let rolled = pierce_rolls[pierce_idx.min(pierce_rolls.len() - 1)];
                        pierce_idx += 1;
                        if rolled < arrow.pierce_chance {
                            hits.push((ae, arrow.damage, None, true, arrow.shooter, arrow_pos.0));
                        } else {
                            arrow.stuck_to = Some(entry.unit_id);
                            arrow.decay_remaining = ARROW_DECAY_TICKS;
                            hits.push((
                                ae,
                                arrow.damage,
                                Some(entry.unit_id),
                                false,
                                arrow.shooter,
                                arrow_pos.0,
                            ));
                            stopped_by_soldier = true;
                            break;
                        }
                    }
                }

                // City collision using SpatialHash
                if !stopped_by_soldier {
                    let city_neighbors = city_spatial.query_nearby(arrow_pos.0);
                    for entry in &city_neighbors {
                        if let Some(&cfac) = city_faction_map.get(&entry.unit_id) {
                            if cfac == arrow.from_faction { continue; }
                        } else { continue; }
                        if let Some(&(ce, cradius)) = city_entity_map.get(&entry.unit_id) {
                            let radius = Fixed::from_int(cradius as i32);
                            let radius_sq = radius * radius;
                            if (arrow_pos.0 - entry.pos).length_squared() <= radius_sq {
                                city_hits.push((ce, arrow.damage));
                                arrow.decay_remaining = ARROW_DECAY_TICKS;
                                break;
                            }
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
    for (_ae, dmg, stuck_to, _pierced, shooter, arrow_hit_pos) in &hits {
        if let Some(sid) = stuck_to {
            if let Some(te) = find_entity_by_unit_id(world, *sid) {
                // Shield block check (before damage application)
                let remaining_dmg = try_passive_block(world, te, *dmg, Some(*arrow_hit_pos));
                let died = {
                    let mut em = world.entity_mut(te);
                    if let Some(mut hp) = em.get_mut::<Health>() {
                        hp.current = hp.current.saturating_sub(remaining_dmg);
                        hp.current == 0
                    } else {
                        false
                    }
                };
                if died {
                    let uid = find_unit_id(world, te).unwrap_or(UnitId(0));
                    if let Some(origin) = world.entity(te).get::<CityOrigin>().map(|c| c.0) {
                        if let Some(oe) = find_entity_by_unit_id(world, origin) {
                            if let Some(mut c) = world.entity_mut(oe).get_mut::<CityComponent>() {
                                c.population = c.population.saturating_sub(1);
                            }
                        }
                    }
                    drop_shield_on_death(world, te, current_tick);
                    world.despawn(te);
                    // Update incremental index
                    if let Some(mut index) = world.get_resource_mut::<crate::unit_index::UnitIdEntityIndex>() {
                        index.remove(uid);
                    }
                    let mut events = world.resource_mut::<SimulationEvents>();
                    events.destroyed.push(UnitDestroyed {
                        unit_id: uid,
                        killer_id: *shooter,
                    });
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
                let integer_damage = city
                    .arrow_damage_acc
                    .saturating_div(arrow_building_damage_denom);
                if integer_damage > 0 {
                    city.health_current = city.health_current.saturating_sub(integer_damage);
                    city.arrow_damage_acc %= arrow_building_damage_denom;
                }
            }
        }
    }

    // Despawn arrows with UnitDestroyed events
    for ae in to_despawn {
        let uid = world.entity(ae).get::<UnitIdComponent>().map(|c| c.0).unwrap_or(UnitId(0));
        world.despawn(ae);
        // Update incremental index
        if let Some(mut index) = world.get_resource_mut::<crate::unit_index::UnitIdEntityIndex>() {
            index.remove(uid);
        }
        world.resource_mut::<SimulationEvents>().destroyed.push(UnitDestroyed {
            unit_id: uid,
            killer_id: None,
        });
    }
}

// ══════════ Tests: arrow-city collision ══════════

#[cfg(test)]
mod arrow_city_tests {
    use super::*;
    use crate::init_simulation_world;
    use crate::soldier::{
        Attack, CityComponent, CityMarker, CityOrigin, CityRadius, FactionComponent, Health, Level,
        LogicalPosition, Movement, SoldierMarker, SoldierStateComponent, SoldierTypeComponent,
        UnitIdComponent,
    };

    /// Helper: create a minimal arrow flying toward a position
    fn spawn_test_arrow(
        world: &mut World,
        pos: FixedVec2,
        dir: FixedVec2,
        dmg: u32,
        faction: Faction,
    ) -> Entity {
        let aid = world.resource_mut::<IdGenerator>().next_id();
        world
            .spawn((
                UnitIdComponent(aid),
                ArrowMarker,
                LogicalPosition(pos),
                Arrow {
                    direction: dir,
                    damage: dmg,
                    from_faction: faction,
                    shooter: None,
                    flight_remaining: 50,
                    decay_remaining: 0,
                    pierce_chance: 0,
                    stuck_to: None,
                    hit_units: Vec::new(),
                    start_pos: pos,
                },
            ))
            .id()
    }

    /// Helper: create a city entity at a position
    fn spawn_test_city(
        world: &mut World,
        pos: FixedVec2,
        faction: Faction,
        radius: u32,
        hp: u32,
    ) -> Entity {
        let cid = world.resource_mut::<IdGenerator>().next_id();
        world
            .spawn((
                UnitIdComponent(cid),
                CityMarker,
                LogicalPosition(pos),
                CityComponent {
                    level: 1,
                    max_level: 5,
                    health_current: hp,
                    health_max: hp,
                    population: 0,
                    max_population: 10,
                    spawn_type: SoldierType::Militia,
                    spawn_cooldown: 0,
                    level_exp: 0,
                    last_attacker_faction: None,
                    arrow_damage_acc: 0,
                },
                CityRadius(radius),
                FactionComponent(faction),
            ))
            .id()
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

        arrow_movement_system(&mut world, 0);

        // Find city, check accumulator
        let mut q = world.query::<(Entity, &CityComponent)>();
        let (_, city) = q.iter(&world).next().unwrap();
        assert_eq!(
            city.arrow_damage_acc, 16,
            "Accumulator should store 16 damage"
        );
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

        arrow_movement_system(&mut world, 0);

        let mut q = world.query::<(Entity, &CityComponent)>();
        let (_, city) = q.iter(&world).next().unwrap();
        assert_eq!(city.health_current, 499, "Should lose 1 HP (206/200 = 1)");
        assert_eq!(
            city.arrow_damage_acc, 6,
            "Remainder should be 6 (206 % 200)"
        );
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

        arrow_movement_system(&mut world, 0);

        let mut q = world.query::<(Entity, &Arrow)>();
        let (_, arrow) = q.iter(&world).next().unwrap();
        assert!(
            arrow.decay_remaining > 0,
            "Arrow should enter decay after city hit"
        );
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

        arrow_movement_system(&mut world, 0);

        let mut q = world.query::<(Entity, &CityComponent)>();
        let (_, city) = q.iter(&world).next().unwrap();
        assert_eq!(
            city.arrow_damage_acc, 0,
            "Friendly arrow should not accumulate damage"
        );
        assert_eq!(city.health_current, 500, "Friendly city health unchanged");

        let mut q = world.query::<(Entity, &Arrow)>();
        let (_, arrow) = q.iter(&world).next().unwrap();
        assert_eq!(
            arrow.decay_remaining, 0,
            "Arrow should still be in flight (not decay)"
        );
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
        let sid = world.resource_mut::<IdGenerator>().next_id();
        world.spawn((
            UnitIdComponent(sid),
            SoldierMarker,
            LogicalPosition(soldier_pos),
            FactionComponent(Faction::Enemy),
            Health {
                current: 100,
                max: 100,
            },
            SoldierTypeComponent(SoldierType::Infantry),
            SoldierStateComponent(SoldierState::Moving),
            Attack {
                damage: 10,
                range: 30,
                interval_ticks: 10,
                cooldown_remaining: 0,
            },
            Movement {
                speed: 50,
                target: None,
                command_target: None,
                waypoint: None,
                force_move: false,
            },
            Level { level: 1, exp: 0 },
            CityOrigin(UnitId(0)),
        ));

        // Arrow with 100% pierce chance (will pierce soldier and continue to city)
        let arrow_start = FixedVec2::new(Fixed::from_int(100), Fixed::from_int(60));
        let dir = FixedVec2::new(Fixed::ZERO, Fixed::from_int(20));
        let aid = world.resource_mut::<IdGenerator>().next_id();
        world.spawn((
            UnitIdComponent(aid),
            ArrowMarker,
            LogicalPosition(arrow_start),
            Arrow {
                direction: dir,
                damage: 200,
                from_faction: Faction::Player,
                shooter: None,
                flight_remaining: 50,
                decay_remaining: 0,
                pierce_chance: 10000,
                stuck_to: None,
                hit_units: Vec::new(),
                start_pos: arrow_start,
            },
        ));

        arrow_movement_system(&mut world, 0);

        // Check city took damage (200 damage → 200/200 = 1 HP)
        let mut q = world.query::<(Entity, &CityComponent)>();
        let (_, city) = q.iter(&world).next().unwrap();
        assert_eq!(
            city.health_current, 499,
            "City should lose 1 HP (200/200) on same tick after pierce"
        );

        // Arrow should be in decay (stopped by city)
        let mut q = world.query::<(Entity, &Arrow)>();
        let (_, arrow) = q.iter(&world).next().unwrap();
        assert!(
            arrow.decay_remaining > 0,
            "Arrow should enter decay after hitting city"
        );
    }
}

// ══════════ Tests: Facing, Block, Multi-shot, Archer Chase ══════════

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::facing;
    use crate::init_simulation_world;
    use crate::soldier;
    use crate::soldier::{
        Attack, CityOrigin, FactionComponent, Health, Level, LogicalPosition, Movement, SeekStance,
        ShieldComponent, SoldierMarker, SoldierStateComponent, SoldierTypeComponent,
        UnitIdComponent,
    };

    /// Spawn a full soldier entity with all components needed for combat/facing systems.
    /// Follows the same pattern as shield_lifecycle_tests: 14 components in the spawn
    /// tuple, then FacingDirection and AttackWindup via separate `.insert()` calls.
    fn spawn_test_soldier(
        world: &mut World,
        pos: FixedVec2,
        faction: Faction,
        stype: SoldierType,
        facing_angle: Fixed,
    ) -> (UnitId, Entity) {
        let uid = world.resource_mut::<IdGenerator>().next_id();
        let cfg = world.resource::<SoldierConfig>().get(stype).clone();
        let shield_hp = world.resource::<CombatGlobalConfig>().shield.initial_hp;
        let e = world
            .spawn((
                UnitIdComponent(uid),
                SoldierMarker,
                LogicalPosition(pos),
                Movement {
                    speed: cfg.speed,
                    target: None,
                    command_target: None,
                    waypoint: None,
                    force_move: false,
                },
                SeekStance {
                    active: false,
                    seek_range: 0,
                },
                Health {
                    current: cfg.health,
                    max: cfg.health,
                },
                Attack {
                    damage: cfg.attack,
                    range: cfg.attack_range,
                    interval_ticks: cfg.attack_interval_ticks,
                    cooldown_remaining: 0,
                },
                FactionComponent(faction),
                SoldierTypeComponent(stype),
                Level { level: 1, exp: 0 },
                ShieldComponent {
                    state: ShieldState::Normal,
                },
                ShieldItem {
                    hp: shield_hp,
                    max_hp: shield_hp,
                },
                CityOrigin(UnitId(0)),
                SoldierStateComponent(SoldierState::Moving),
            ))
            .id();
        world.entity_mut(e).insert(crate::types::FacingDirection {
            angle: facing_angle,
        });
        world.entity_mut(e).insert(crate::types::AttackWindup {
            remaining_ticks: 0,
            target: None,
        });
        (uid, e)
    }

    /// Spawn an archer entity (no shield components).
    fn spawn_test_archer(
        world: &mut World,
        pos: FixedVec2,
        faction: Faction,
        facing_angle: Fixed,
    ) -> (UnitId, Entity) {
        let uid = world.resource_mut::<IdGenerator>().next_id();
        let cfg = world
            .resource::<SoldierConfig>()
            .get(SoldierType::Archer)
            .clone();
        let e = world
            .spawn((
                UnitIdComponent(uid),
                SoldierMarker,
                LogicalPosition(pos),
                Movement {
                    speed: cfg.speed,
                    target: None,
                    command_target: None,
                    waypoint: None,
                    force_move: false,
                },
                SeekStance {
                    active: false,
                    seek_range: 0,
                },
                Health {
                    current: cfg.health,
                    max: cfg.health,
                },
                Attack {
                    damage: cfg.attack,
                    range: cfg.attack_range,
                    interval_ticks: cfg.attack_interval_ticks,
                    cooldown_remaining: 0,
                },
                FactionComponent(faction),
                SoldierTypeComponent(SoldierType::Archer),
                Level { level: 1, exp: 0 },
                CityOrigin(UnitId(0)),
                SoldierStateComponent(SoldierState::Moving),
            ))
            .id();
        world.entity_mut(e).insert(crate::types::FacingDirection {
            angle: facing_angle,
        });
        world.entity_mut(e).insert(crate::types::AttackWindup {
            remaining_ticks: 0,
            target: None,
        });
        (uid, e)
    }

    // ── Test 1: Facing direction turn toward target ──

    #[test]
    fn test_facing_turn_toward_target() {
        let mut world = init_simulation_world(42);

        // Soldier at origin facing right (0°), waypoint above (→ 90°)
        let (_uid, entity) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            SoldierType::Infantry,
            Fixed::ZERO, // facing 0° (right)
        );

        // Set waypoint above so facing_turn_system computes target angle = 90°
        {
            let mut mov = world.get_mut::<Movement>(entity).unwrap();
            mov.waypoint = Some(FixedVec2::new(Fixed::ZERO, Fixed::from_int(100)));
        }

        // Run facing_turn_system for several ticks
        for _ in 0..15 {
            facing::facing_turn_system(&mut world);
        }

        // Verify facing angle has approached 90°
        let facing = world.get::<crate::types::FacingDirection>(entity).unwrap();
        let deviation = facing::angle_distance(facing.angle, Fixed::from_int(90));
        assert!(
            deviation < Fixed::from_int(5),
            "Facing angle should approach 90°, got ~{}°, deviation ~{}°",
            facing.angle.0 / 256,
            deviation.0 / 256,
        );
    }

    // ── Test 2: Passive block with 100% block chance ──

    #[test]
    fn test_passive_block_full_chance_shield_absorbs() {
        let mut world = init_simulation_world(42);

        // Override passive_block_chance to 1.0 (100%)
        {
            let mut cfg = world.resource_mut::<CombatGlobalConfig>();
            cfg.shield.passive_block_chance = 10000;
        }

        // Infantry in Normal state (not Blocking) at origin
        let (_uid, entity) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            SoldierType::Infantry,
            Fixed::ZERO,
        );

        let initial_shield_hp = world.get::<ShieldItem>(entity).unwrap().hp;
        let initial_soldier_hp = world.get::<Health>(entity).unwrap().current;
        let damage = 50u32;

        // Attacker to the right (irrelevant for passive block — direction-independent)
        let attacker_pos = FixedVec2::new(Fixed::from_int(100), Fixed::ZERO);
        let remaining = try_passive_block(&mut world, entity, damage, Some(attacker_pos));

        // All damage should be absorbed by shield
        assert_eq!(remaining, 0, "All damage should be absorbed by shield");
        let shield = world.get::<ShieldItem>(entity).unwrap();
        assert_eq!(
            shield.hp,
            initial_shield_hp - damage,
            "Shield HP should decrease by damage amount"
        );
        let hp = world.get::<Health>(entity).unwrap();
        assert_eq!(
            hp.current, initial_soldier_hp,
            "Soldier HP should be unchanged"
        );
    }

    // ── Test 3: Manual block — frontal absorbs, behind does not ──

    #[test]
    fn test_manual_block_frontal_absorbs_behind_does_not() {
        let mut world = init_simulation_world(42);

        // Disable passive block so only manual (Blocking state) matters
        {
            let mut cfg = world.resource_mut::<CombatGlobalConfig>();
            cfg.shield.passive_block_chance = 0;
        }

        // Infantry facing right (0°) in Blocking state
        let (_uid, entity) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            SoldierType::Infantry,
            Fixed::ZERO, // facing 0° (right)
        );
        {
            let mut sc = world.get_mut::<ShieldComponent>(entity).unwrap();
            sc.state = ShieldState::Blocking;
        }

        let initial_shield_hp = world.get::<ShieldItem>(entity).unwrap().hp;
        let initial_soldier_hp = world.get::<Health>(entity).unwrap().current;
        let damage = 50u32;

        // Case A: Frontal attack (attacker to the right, facing = 0°, attack angle = 0°)
        let frontal_attacker = FixedVec2::new(Fixed::from_int(100), Fixed::ZERO);
        let remaining = try_passive_block(&mut world, entity, damage, Some(frontal_attacker));
        assert_eq!(
            remaining, 0,
            "Frontal attack should be fully absorbed by shield"
        );
        let shield = world.get::<ShieldItem>(entity).unwrap();
        assert_eq!(
            shield.hp,
            initial_shield_hp - damage,
            "Shield should absorb frontal damage"
        );

        // Case B: Rear attack (attacker behind — facing = 0°, attack angle = 180°)
        let rear_attacker = FixedVec2::new(Fixed::from_int(-100), Fixed::ZERO);
        let remaining = try_passive_block(&mut world, entity, damage, Some(rear_attacker));
        assert_eq!(
            remaining, damage,
            "Rear attack should NOT be absorbed (pass through to HP)"
        );
        // Apply remaining damage to Health (same as melee_attack_system does)
        {
            let mut hp = world.get_mut::<Health>(entity).unwrap();
            hp.current = hp.current.saturating_sub(remaining);
        }
        let hp = world.get::<Health>(entity).unwrap();
        assert_eq!(
            hp.current,
            initial_soldier_hp - damage,
            "Soldier should take full damage from behind"
        );
    }

    /// 测试：手动格挡时非正面伤害跳过被动格挡
    ///
    /// 场景：步兵 Blocking 状态，被动格挡 100%，背面受击
    /// 预期：非正面伤害直接扣士兵 HP，不触发被动格挡
    /// 验证：Blocking + 非正面 → 跳过被动格挡
    #[test]
    fn test_manual_block_non_frontal_skips_passive() {
        let mut world = init_simulation_world(42);

        // Set passive block to 100% — if it triggers, damage goes to shield
        {
            let mut cfg = world.resource_mut::<CombatGlobalConfig>();
            cfg.shield.passive_block_chance = 10000;
        }

        // Infantry facing right (0°) in Blocking state
        let (_uid, entity) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            SoldierType::Infantry,
            Fixed::ZERO,
        );
        {
            let mut sc = world.get_mut::<ShieldComponent>(entity).unwrap();
            sc.state = ShieldState::Blocking;
        }

        let initial_shield_hp = world.get::<ShieldItem>(entity).unwrap().hp;
        let initial_soldier_hp = world.get::<Health>(entity).unwrap().current;
        let damage = 50u32;

        // Rear attack (behind the infantry)
        let rear_attacker = FixedVec2::new(Fixed::from_int(-100), Fixed::ZERO);
        let remaining = try_passive_block(&mut world, entity, damage, Some(rear_attacker));

        // Damage should pass through to soldier HP (not absorbed by shield)
        assert_eq!(
            remaining, damage,
            "Non-frontal damage should pass through to HP"
        );

        // Apply damage to soldier
        {
            let mut hp = world.get_mut::<Health>(entity).unwrap();
            hp.current = hp.current.saturating_sub(remaining);
        }

        // Shield HP should be unchanged (passive block was skipped)
        let shield = world.get::<ShieldItem>(entity).unwrap();
        assert_eq!(
            shield.hp, initial_shield_hp,
            "Shield HP should not change (passive block skipped for non-frontal)"
        );

        // Soldier HP should decrease
        let hp = world.get::<Health>(entity).unwrap();
        assert_eq!(
            hp.current,
            initial_soldier_hp - damage,
            "Soldier should take full damage from behind"
        );
    }

    /// 测试：手动格挡时移速变为 15
    ///
    /// 场景：步兵 Blocking 状态，基线移速 80
    /// 预期：移速变为 15（不是 80-15=65）
    /// 验证：speed_penalty 是设为值，不是减去值
    #[test]
    fn test_manual_block_speed_set_to_penalty() {
        let mut world = init_simulation_world(42);

        // Infantry at origin
        let (_uid, entity) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            SoldierType::Infantry,
            Fixed::ZERO,
        );

        // Set waypoint and Blocking state
        world.entity_mut(entity).insert(Movement {
            speed: 80,
            target: None,
            command_target: None,
            waypoint: Some(FixedVec2::new(Fixed::from_int(200), Fixed::ZERO)),
            force_move: false,
        });
        {
            let mut sc = world.get_mut::<ShieldComponent>(entity).unwrap();
            sc.state = ShieldState::Blocking;
        }

        // Run one tick of movement
        crate::run_tick_default(&mut world, 1);

        // Check position — with speed 15 and tick_duration 0.05:
        // movement per tick = 15 * 0.05 = 0.75 units
        let pos = world.get::<LogicalPosition>(entity).unwrap().0;
        let move_per_tick = Fixed::from_float(15.0 * 0.05); // 0.75

        // Position should be approximately 0.75 (not 80 * 0.05 = 4.0)
        let diff = (pos.x - move_per_tick).abs();
        assert!(
            diff.0 < Fixed::from_float(0.1).0,
            "Blocking speed should be 15, not 65. Position after 1 tick: {}, expected ~{}",
            pos.x.to_float(),
            move_per_tick.to_float()
        );
    }

    // ── Test 4: Multi-shot spawns multiple arrows ──

    #[test]
    fn test_multi_shot_spawns_multiple_arrows() {
        let mut world = init_simulation_world(42);

        // Override multi-shot to 100% chance
        {
            let mut cfg = world.resource_mut::<CombatGlobalConfig>();
            cfg.archer_multi_shot.base_chance = 10000;
            cfg.archer_multi_shot.min_chance = 10000;
            cfg.archer_multi_shot.max_chance = 10000;
        }

        // Archer at origin (cooldown starts at 0 → fires immediately)
        let (_archer_uid, _archer_e) = spawn_test_archer(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            Fixed::ZERO,
        );

        // Spawn 5 enemies in range (archer range at level 1 = 380)
        for i in 0..5 {
            let enemy_pos = FixedVec2::new(Fixed::from_int(50 + i * 30), Fixed::from_int(50));
            let eid = world.resource_mut::<IdGenerator>().next_id();
            world.spawn((
                UnitIdComponent(eid),
                SoldierMarker,
                LogicalPosition(enemy_pos),
                FactionComponent(Faction::Enemy),
                SoldierTypeComponent(SoldierType::Militia),
                Health {
                    current: 100,
                    max: 100,
                },
                Movement {
                    speed: 80,
                    target: None,
                    command_target: None,
                    waypoint: None,
                    force_move: false,
                },
                Level { level: 1, exp: 0 },
                Attack {
                    damage: 10,
                    range: 30,
                    interval_ticks: 10,
                    cooldown_remaining: 0,
                },
                SoldierStateComponent(SoldierState::Moving),
            ));
        }

        archer_attack_system(&mut world);

        // Verify multiple arrows were spawned
        let mut arrow_query = world.query::<&Arrow>();
        let arrow_count = arrow_query.iter(&world).count();
        assert!(
            arrow_count > 1,
            "Multi-shot should spawn multiple arrows, got {}",
            arrow_count,
        );
    }

    // ── Test 5: Archer chases target out of attack range ──

    #[test]
    fn test_facing_affects_attack_speed() {
        // Setup: two militia attacking the same target
        // One faces the target (frontal), one faces away (rear)
        let mut world = init_simulation_world(42);
        let cfg = world.resource::<SoldierConfig>().clone();

        // Spawn enemy target at (100, 0)
        let (_enemy_uid, _enemy) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::from_int(100), Fixed::ZERO),
            Faction::Enemy,
            SoldierType::Militia,
            Fixed::ZERO,
        );

        // Spawn frontal attacker at (75, 0) — facing right (0°) toward enemy
        let (_frontal_uid, frontal) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::from_int(75), Fixed::ZERO),
            Faction::Player,
            SoldierType::Militia,
            Fixed::from_int(0), // facing 0° (toward enemy)
        );
        world.entity_mut(frontal).insert(Movement {
            speed: 80,
            target: Some(_enemy_uid),
            command_target: None,
            waypoint: None,
            force_move: false,
        });
        world
            .entity_mut(frontal)
            .insert(SoldierStateComponent(SoldierState::Fighting));

        // Spawn rear attacker at (75, 0) — facing left (180°) away from enemy
        let (_rear_uid, rear) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::from_int(75), Fixed::from_int(1)),
            Faction::Player,
            SoldierType::Militia,
            Fixed::from_int(180), // facing 180° (away from enemy)
        );
        world.entity_mut(rear).insert(Movement {
            speed: 80,
            target: Some(_enemy_uid),
            command_target: None,
            waypoint: None,
            force_move: false,
        });
        world
            .entity_mut(rear)
            .insert(SoldierStateComponent(SoldierState::Fighting));

        // Run enough ticks for windup to complete (3 ticks) + attack
        for tick in 1..=5 {
            crate::run_tick_default(&mut world, tick);
        }

        // Check cooldowns — frontal should have shorter cooldown than rear
        let frontal_atk = world.get::<Attack>(frontal).unwrap();
        let rear_atk = world.get::<Attack>(rear).unwrap();

        // Frontal: cooldown ≈ base / 1.3 ≈ 0.77 * base
        // Rear: cooldown ≈ base / 0.7 ≈ 1.43 * base
        let base_interval = cfg.get(SoldierType::Militia).attack_interval_ticks;
        println!("Base interval: {}", base_interval);
        println!("Frontal cooldown: {}", frontal_atk.cooldown_remaining);
        println!("Rear cooldown: {}", rear_atk.cooldown_remaining);

        // Both should have attacked (cooldown > 0 means attack happened)
        assert!(
            frontal_atk.cooldown_remaining > 0,
            "Frontal attacker should have attacked (cooldown > 0)"
        );
        assert!(
            rear_atk.cooldown_remaining > 0,
            "Rear attacker should have attacked (cooldown > 0)"
        );
        assert!(
            frontal_atk.cooldown_remaining < rear_atk.cooldown_remaining,
            "Frontal cooldown ({}) should be less than rear cooldown ({})",
            frontal_atk.cooldown_remaining,
            rear_atk.cooldown_remaining
        );
        assert!(
            frontal_atk.cooldown_remaining < base_interval,
            "Frontal cooldown ({}) should be less than base ({})",
            frontal_atk.cooldown_remaining,
            base_interval
        );
        assert!(
            rear_atk.cooldown_remaining > base_interval,
            "Rear cooldown ({}) should be greater than base ({})",
            rear_atk.cooldown_remaining,
            base_interval
        );
    }

    // ═══════════════════════════════════════════════════════════════
    // 近战自动攻击测试用例
    //
    // 测试说明：
    // 1. 站立不动时自动攻击范围内敌人
    // 2. 移动过程中自动攻击范围内敌人
    // 3. ForceMove 时不攻击（非骑兵）
    // 4. 骑兵 ForceMove 时仍可攻击
    // 5. 移动中攻击不影响移动进度
    // ═══════════════════════════════════════════════════════════════

    /// 测试：站立不动的单位自动攻击范围内敌人
    ///
    /// 场景：玩家步兵站立不动，敌人步兵进入攻击范围（30 单位）
    /// 预期：玩家步兵自动攻击敌人，造成伤害
    /// 验证：近战系统不依赖 Movement.target，直接扫描范围内敌人
    #[test]
    fn test_standing_soldier_attacks_enemy_in_range() {
        let mut world = init_simulation_world(42);
        let _cfg = world.resource::<SoldierConfig>().clone();

        // 玩家步兵站立不动在 (50, 0)
        let (_player_uid, player) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::from_int(50), Fixed::ZERO),
            Faction::Player,
            SoldierType::Infantry,
            Fixed::from_int(0),
        );
        // 无目标，无移动命令，SeekStance 未激活
        world.entity_mut(player).insert(Movement {
            speed: 80,
            target: None,
            command_target: None,
            waypoint: None,
            force_move: false,
        });

        // 敌人步兵在 (60, 0)，距离 10，在攻击范围 30 内
        let (_enemy_uid, enemy) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::from_int(60), Fixed::ZERO),
            Faction::Enemy,
            SoldierType::Infantry,
            Fixed::from_int(0),
        );

        let enemy_hp_before = world.get::<Health>(enemy).unwrap().current;

        // 运行多个 tick（等待前摇完成 + 攻击生效）
        for tick in 1..=10 {
            crate::run_tick_default(&mut world, tick);
        }

        let enemy_hp_after = world.get::<Health>(enemy).unwrap().current;
        assert!(
            enemy_hp_after < enemy_hp_before,
            "Standing soldier should damage enemy. HP before: {}, after: {}",
            enemy_hp_before,
            enemy_hp_after
        );
    }

    /// 测试：移动中的单位自动攻击范围内敌人
    ///
    /// 场景：玩家步兵从 (0,0) 移动到 (200,0)，途中经过敌人 (50,0)
    /// 预期：步兵在接近敌人时自动攻击，然后继续移动
    /// 验证：移动过程中攻击不中断移动命令
    #[test]
    fn test_moving_soldier_attacks_enemy_in_range() {
        let mut world = init_simulation_world(42);
        let _cfg = world.resource::<SoldierConfig>().clone();

        // 玩家步兵从 (0,0) 移动到 (200,0)
        let (_player_uid, player) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            SoldierType::Infantry,
            Fixed::from_int(0),
        );
        world.entity_mut(player).insert(Movement {
            speed: 80,
            target: None,
            command_target: None,
            waypoint: Some(FixedVec2::new(Fixed::from_int(200), Fixed::ZERO)),
            force_move: false,
        });

        // 敌人步兵在 (50, 0)
        let (_enemy_uid, enemy) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::from_int(50), Fixed::ZERO),
            Faction::Enemy,
            SoldierType::Infantry,
            Fixed::from_int(0),
        );

        let enemy_hp_before = world.get::<Health>(enemy).unwrap().current;

        // 运行多个 tick
        for tick in 1..=20 {
            crate::run_tick_default(&mut world, tick);
        }

        let enemy_hp_after = world.get::<Health>(enemy).unwrap().current;
        let player_pos = world.get::<LogicalPosition>(player).unwrap().0;

        // 敌人应该受到伤害
        assert!(
            enemy_hp_after < enemy_hp_before,
            "Moving soldier should damage enemy. HP before: {}, after: {}",
            enemy_hp_before,
            enemy_hp_after
        );

        // 玩家应该已经移动了一段距离（移动未被中断）
        assert!(
            player_pos.x.0 > Fixed::from_int(30).0,
            "Soldier should have moved past enemy. Position: {}",
            player_pos.x.to_float()
        );
    }

    /// 测试：ForceMove 时不攻击（非骑兵）
    ///
    /// 场景：玩家步兵 ForceMove 经过敌人
    /// 预期：步兵不攻击敌人，纯移动
    /// 验证：force_move=true 时非骑兵跳过攻击
    #[test]
    fn test_force_move_no_attack() {
        let mut world = init_simulation_world(42);

        // 玩家步兵 ForceMove 从 (0,0) 到 (200,0)
        let (_player_uid, player) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            SoldierType::Infantry,
            Fixed::from_int(0),
        );
        world.entity_mut(player).insert(Movement {
            speed: 80,
            target: None,
            command_target: None,
            waypoint: Some(FixedVec2::new(Fixed::from_int(200), Fixed::ZERO)),
            force_move: true, // 强制移动
        });

        // 敌人在 (50, 0)
        let (_enemy_uid, enemy) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::from_int(50), Fixed::ZERO),
            Faction::Enemy,
            SoldierType::Infantry,
            Fixed::from_int(0),
        );

        let enemy_hp_before = world.get::<Health>(enemy).unwrap().current;

        // 运行多个 tick
        for tick in 1..=20 {
            crate::run_tick_default(&mut world, tick);
        }

        let enemy_hp_after = world.get::<Health>(enemy).unwrap().current;
        assert_eq!(
            enemy_hp_after, enemy_hp_before,
            "ForceMove should NOT damage enemy. HP before: {}, after: {}",
            enemy_hp_before, enemy_hp_after
        );
    }

    /// 测试：骑兵 ForceMove 时仍可攻击
    ///
    /// 场景：骑兵 ForceMove 经过敌人
    /// 预期：骑兵仍然攻击敌人（骑兵攻击不影响移动）
    /// 验证：骑兵不受 ForceMove 攻击抑制
    #[test]
    fn test_cavalry_force_move_attacks() {
        let mut world = init_simulation_world(42);

        // 骑兵 ForceMove 从 (0,0) 到 (200,0)
        let (_cav_uid, cavalry) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            SoldierType::Cavalry,
            Fixed::from_int(0),
        );
        world.entity_mut(cavalry).insert(Movement {
            speed: 200,
            target: None,
            command_target: None,
            waypoint: Some(FixedVec2::new(Fixed::from_int(200), Fixed::ZERO)),
            force_move: true,
        });

        // 敌人在 (50, 0)
        let (_enemy_uid, enemy) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::from_int(50), Fixed::ZERO),
            Faction::Enemy,
            SoldierType::Infantry,
            Fixed::from_int(0),
        );

        let enemy_hp_before = world.get::<Health>(enemy).unwrap().current;

        // 运行多个 tick（共享索引有 1-tick 位置滞后，需要更多 tick）
        for tick in 1..=30 {
            crate::run_tick_default(&mut world, tick);
        }

        let enemy_hp_after = world.get::<Health>(enemy).unwrap().current;
        assert!(
            enemy_hp_after < enemy_hp_before,
            "Cavalry ForceMove should still damage enemy. HP before: {}, after: {}",
            enemy_hp_before,
            enemy_hp_after
        );
    }

    /// 测试：移动中攻击后继续移动（不卡住）
    ///
    /// 场景：步兵移动到 (200,0)，途中攻击敌人
    /// 预期：攻击后步兵继续移动，最终超过敌人位置
    /// 验证：攻击不中断移动命令，攻击完成后继续移动
    /// 注意：非骑兵有 0.15s 前摇停顿，这是设计行为，不影响移动命令的继续执行
    #[test]
    fn test_attack_does_not_interrupt_movement_command() {
        let mut world = init_simulation_world(42);

        // 玩家步兵从 (0,0) 移动到 (200,0)
        let (_player_uid, player) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            SoldierType::Infantry,
            Fixed::from_int(0),
        );
        world.entity_mut(player).insert(Movement {
            speed: 80,
            target: None,
            command_target: None,
            waypoint: Some(FixedVec2::new(Fixed::from_int(200), Fixed::ZERO)),
            force_move: false,
        });

        // 敌人在 (50, 0) — 路径上
        let (_enemy_uid, _enemy) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::from_int(50), Fixed::ZERO),
            Faction::Enemy,
            SoldierType::Infantry,
            Fixed::from_int(0),
        );

        // 运行足够 tick（步兵速度 80，走 200 距离约需 25 tick，加上攻击停顿）
        for tick in 1..=50 {
            crate::run_tick_default(&mut world, tick);
        }

        let player_pos = world.get::<LogicalPosition>(player).unwrap().0.x;
        let player_state = world.get::<SoldierStateComponent>(player).unwrap().0;

        // 步兵应该已经越过敌人（位置 > 50），说明攻击后继续移动了
        assert!(
            player_pos.0 > Fixed::from_int(50).0,
            "Soldier should have moved past enemy after attacking. Position: {}",
            player_pos.to_float()
        );

        // 步兵应该还在移动状态（因为 waypoint 还没到）
        // 如果攻击中断了移动命令，状态会卡在 Fighting
        assert_eq!(
            player_state,
            SoldierState::Moving,
            "Soldier should be in Moving state after passing enemy"
        );
    }

    /// 测试：骑兵移动中攻击且继续移动
    ///
    /// 场景：骑兵从 (0,0) 移动到 (500,0)，途中攻击敌人 (50,0)
    /// 预期：骑兵攻击敌人（造成伤害）且继续移动（越过敌人位置）
    /// 验证：骑兵攻击不中断移动命令
    #[test]
    fn test_cavalry_attacks_while_moving() {
        let mut world = init_simulation_world(42);

        // 骑兵从 (0,0) 移动到 (500,0)
        let (_cav_uid, cav) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            SoldierType::Cavalry,
            Fixed::from_int(0),
        );
        world.entity_mut(cav).insert(Movement {
            speed: 200,
            target: None,
            command_target: None,
            waypoint: Some(FixedVec2::new(Fixed::from_int(500), Fixed::ZERO)),
            force_move: false,
        });

        // 敌人在 (50, 0)
        let (_enemy_uid, enemy) = spawn_test_soldier(
            &mut world,
            FixedVec2::new(Fixed::from_int(50), Fixed::ZERO),
            Faction::Enemy,
            SoldierType::Infantry,
            Fixed::from_int(0),
        );

        let enemy_hp_before = world.get::<Health>(enemy).unwrap().current;

        for tick in 1..=30 {
            crate::run_tick_default(&mut world, tick);
        }

        let enemy_hp_after = world.get::<Health>(enemy).unwrap().current;
        let cav_pos = world.get::<LogicalPosition>(cav).unwrap().0.x;

        // 骑兵应该对敌人造成伤害
        assert!(
            enemy_hp_after < enemy_hp_before,
            "Cavalry should damage enemy while moving. HP before: {}, after: {}",
            enemy_hp_before,
            enemy_hp_after
        );

        // 骑兵应该已经越过敌人（位置 > 50）
        assert!(
            cav_pos.0 > Fixed::from_int(50).0,
            "Cavalry should have moved past enemy. Position: {}",
            cav_pos.to_float()
        );
    }

    #[test]
    fn test_archer_chases_target_out_of_range() {
        let mut world = init_simulation_world(42);

        // Archer at origin facing up (90°), in Fighting state with a target out of range
        let (_archer_uid, archer_e) = spawn_test_archer(
            &mut world,
            FixedVec2::new(Fixed::ZERO, Fixed::ZERO),
            Faction::Player,
            Fixed::from_int(90), // facing up
        );
        {
            let mut sc = world.get_mut::<SoldierStateComponent>(archer_e).unwrap();
            sc.0 = SoldierState::Fighting;
        }

        // Enemy far above — out of archer attack range (level 1 range = 380)
        let enemy_pos = FixedVec2::new(Fixed::ZERO, Fixed::from_int(500));
        let enemy_uid = {
            let eid = world.resource_mut::<IdGenerator>().next_id();
            world.spawn((
                UnitIdComponent(eid),
                SoldierMarker,
                LogicalPosition(enemy_pos),
                FactionComponent(Faction::Enemy),
                SoldierTypeComponent(SoldierType::Militia),
                Health {
                    current: 100,
                    max: 100,
                },
                Movement {
                    speed: 80,
                    target: None,
                    command_target: None,
                    waypoint: None,
                    force_move: false,
                },
                Level { level: 1, exp: 0 },
                Attack {
                    damage: 10,
                    range: 30,
                    interval_ticks: 10,
                    cooldown_remaining: 0,
                },
                SoldierStateComponent(SoldierState::Moving),
            ));
            eid
        };

        // Set the archer's target to the enemy
        {
            let mut mov = world.get_mut::<Movement>(archer_e).unwrap();
            mov.target = Some(enemy_uid);
        }

        let initial_pos = world.get::<LogicalPosition>(archer_e).unwrap().0;

        // Run movement system for 20 ticks — archer should chase
        for _ in 0..20 {
            soldier::soldier_movement_system(&mut world);
        }

        let final_pos = world.get::<LogicalPosition>(archer_e).unwrap().0;
        let moved = (final_pos.y - initial_pos.y).abs();
        assert!(
            moved > Fixed::from_int(1),
            "Archer should have moved toward out-of-range target. Moved ~{} units",
            moved.0 / 256,
        );
        assert!(
            final_pos.y > initial_pos.y,
            "Archer should have moved upward toward target (y should increase)",
        );
    }
}
