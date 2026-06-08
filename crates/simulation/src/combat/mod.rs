pub mod config;

use bevy_ecs::world::World;
use bevy_ecs::entity::Entity;
use bevy_ecs::component::Component;
use std::collections::HashMap;
use crate::types::*;
use crate::events::*;
use crate::soldier::*;
use crate::soldier::config::SoldierConfig;
use crate::combat::config::CombatGlobalConfig;

// ══════════ Arrow Types ══════════

#[derive(Component, Clone, Debug)]
pub struct Arrow {
    pub target: UnitId,
    pub damage: u32,
    pub from_faction: Faction,
    pub shooter: Option<UnitId>,
    pub remaining_ticks: u32,
    pub start_pos: FixedVec2,
}

#[derive(Component, Clone, Debug)]
pub struct ArrowMarker;

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

    let mut pending_deaths: Vec<(Entity, Option<UnitId>)> = Vec::new();
    let mut xp_grants: Vec<(Entity, u32)> = Vec::new();
    let mut ls_hits: Vec<(Entity, u32, u32, bool)> = Vec::new();
    let mut ls_kills: Vec<(Entity, u32, u32, bool)> = Vec::new();

    for ad in attackers {
        let Some(tid) = ad.target else { continue };
        let Some(&tpos) = positions.get(&tid) else { continue };

        let range_f = Fixed::from_int(ad.range as i32);
        if (ad.pos - tpos).length_squared() > range_f * range_f { continue; }

        let Some(te) = find_entity_by_unit_id(world, tid) else { continue };

        let (thp, tmax, tst, _tfac, torg) = {
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
                pending_deaths.push((te, Some(ad.uid)));
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

    // Process deaths
    for (te, kid) in pending_deaths {
        let uid = find_unit_id(world, te).unwrap_or(UnitId(0));
        world.despawn(te);
        let mut events = world.resource_mut::<SimulationEvents>();
        events.destroyed.push(UnitDestroyed { unit_id: uid, killer_id: kid });
    }
}

// ══════════ archer_attack ══════════

pub fn archer_attack_system(world: &mut World) {
    let soldier_config = world.resource::<SoldierConfig>().clone();
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // Tick cooldowns
    {
        let mut q = world.query::<(Entity, &mut Attack, &SoldierTypeComponent)>();
        for (_, mut atk, st) in q.iter_mut(world) {
            if st.0 == SoldierType::Archer { atk.cooldown_remaining = atk.cooldown_remaining.saturating_sub(1); }
        }
    }

    // Enemy positions
    let enemies: Vec<(UnitId, FixedVec2, SoldierType, Faction)> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &SoldierTypeComponent, &FactionComponent)>();
        q.iter(world).map(|(_, id, pos, st, fac)| (id.0, pos.0, st.0, fac.0)).collect()
    };

    // Archer data
    struct ArcData { entity: Entity, uid: UnitId, pos: FixedVec2, faction: Faction, dmg: u32, range: u32, interval: u32, level: u32 }
    let archers: Vec<ArcData> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &Attack, &Level)>();
        q.iter(world)
            .filter(|(_, _, _, _, atk, _)| atk.cooldown_remaining == 0)
            .map(|(e, id, pos, fac, atk, lvl)| ArcData {
                entity: e, uid: id.0, pos: pos.0, faction: fac.0,
                dmg: atk.damage, range: atk.range, interval: atk.interval_ticks, level: lvl.level,
            }).collect()
    };

    let mut spawns: Vec<(UnitId, UnitId, FixedVec2, u32, Faction, Option<UnitId>)> = Vec::new();
    let mut to_reset: Vec<(Entity, u32, u32, u32)> = Vec::new();

    for ad in archers {
        let rng_f = Fixed::from_int(ad.range as i32);
        let rng_sq = rng_f * rng_f;

        let mut in_range: Vec<(UnitId, SoldierType)> = enemies.iter()
            .filter(|(_, epos, _, ef)| *ef != ad.faction && (ad.pos - *epos).length_squared() <= rng_sq)
            .map(|(eid, _, est, _)| (*eid, *est))
            .collect();
        if in_range.is_empty() { continue; }

        in_range.sort_by_key(|(_, _)| 0u32); // stable order for determinism

        let am = &combat_config.archer_multi_shot;
        let mc = (am.base_chance + ad.level as f32 * am.per_level_bonus).clamp(am.min_chance, am.max_chance);
        let count = if world.resource_mut::<DeterministicRng>().gen_probability() < mc {
            (world.resource_mut::<DeterministicRng>().next_u64() as usize % 4 + 2).min(in_range.len())
        } else { 1.min(in_range.len()) };

        for i in 0..count {
            let (tid, tst) = in_range[i];
            let mut damage = ad.dmg + ad.level * combat_config.level_up.attack_gain;
            if tst == SoldierType::Infantry { damage = (damage as f32 * 0.9) as u32; }

            let aid = { world.resource_mut::<IdGenerator>().next() };
            spawns.push((aid, tid, ad.pos, damage, ad.faction, Some(ad.uid)));
        }

        to_reset.push((ad.entity, ad.dmg, ad.range, ad.interval));
    }

    for (e, dmg, range, interval) in to_reset {
        world.entity_mut(e).insert(Attack { damage: dmg, range, interval_ticks: interval, cooldown_remaining: interval });
    }

    for (aid, tid, pos, dmg, fac, shooter) in spawns {
        world.spawn((
            UnitIdComponent(aid), ArrowMarker, LogicalPosition(pos),
            Arrow { target: tid, damage: dmg, from_faction: fac, shooter, remaining_ticks: 5, start_pos: pos },
        ));
        let mut events = world.resource_mut::<SimulationEvents>();
        events.spawned.push(UnitSpawned { unit_id: aid, pos, faction: fac, unit_kind: UnitKind::Arrow });
    }
}

// ══════════ arrow_hit ══════════

pub fn arrow_hit_system(world: &mut World) {
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // Decrement ticks
    {
        let mut q = world.query::<(Entity, &mut Arrow)>();
        for (_, mut a) in q.iter_mut(world) { a.remaining_ticks = a.remaining_ticks.saturating_sub(1); }
    }

    let ready: Vec<(Entity, UnitId, u32, Faction, Option<UnitId>)> = {
        let mut q = world.query::<(Entity, &Arrow)>();
        q.iter(world)
            .filter(|(_, a)| a.remaining_ticks == 0)
            .map(|(e, a)| (e, a.target, a.damage, a.from_faction, a.shooter))
            .collect()
    };

    let mut deaths: Vec<(Entity, Option<UnitId>)> = Vec::new();
    let mut xp: Vec<(Entity, u32)> = Vec::new();
    let mut to_despawn: Vec<Entity> = Vec::new();

    for (ae, tid, dmg, from_fac, shooter) in ready {
        let Some(te) = find_entity_by_unit_id(world, tid) else {
            to_despawn.push(ae); continue;
        };

        let mut final_dmg = dmg;

        // Shield intercept
        if let Some(sh) = world.entity(te).get::<ShieldComponent>() {
            if sh.0 == ShieldState::ShieldUp {
                if world.resource_mut::<DeterministicRng>().gen_probability() < combat_config.shield.intercept_chance {
                    final_dmg = (final_dmg as f32 * (1.0 - combat_config.shield.damage_reduction)) as u32;
                }
            }
        }

        // Slow
        let apply_slow = world.entity(te).get::<FactionComponent>().map(|f| f.0 != from_fac).unwrap_or(false);
        if apply_slow {
            let sd = &combat_config.slow_debuff;
            let st = world.entity(te).get::<SlowDebuff>().map(|s| s.stacks).unwrap_or(0);
            let ns = (st + 1).min(sd.max_stacks);
            world.entity_mut(te).insert(SlowDebuff { stacks: ns, remaining_ticks: sd.duration_ticks });
        }

        // Damage
        let died = {
            let mut em = world.entity_mut(te);
            if let Some(mut hp) = em.get_mut::<Health>() {
                hp.current = hp.current.saturating_sub(final_dmg);
                hp.current == 0
            } else { false }
        };

        if died {
            deaths.push((te, shooter));
            if let Some(sid) = shooter {
                if let Some(se) = find_entity_by_unit_id(world, sid) {
                    xp.push((se, combat_config.level_up.exp_per_kill));
                }
            }
        }

        to_despawn.push(ae);
    }

    for (e, x) in xp {
        if let Some(mut l) = world.entity_mut(e).get_mut::<Level>() { l.exp += x; }
    }
    for (te, kid) in deaths {
        let uid = find_unit_id(world, te).unwrap_or(UnitId(0));
        world.despawn(te);
        let mut ev = world.resource_mut::<SimulationEvents>();
        ev.destroyed.push(UnitDestroyed { unit_id: uid, killer_id: kid });
    }
    for ae in to_despawn { world.despawn(ae); }
}

// ══════════ arrow_expire ══════════

pub fn arrow_expire_system(world: &mut World) {
    let to_despawn: Vec<Entity> = {
        let mut q = world.query::<(Entity, &Arrow)>();
        q.iter(world).filter(|(_, a)| a.remaining_ticks == 0).map(|(e, _)| e).collect()
    };
    for e in to_despawn { world.despawn(e); }
}
