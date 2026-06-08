pub mod config;

use bevy_ecs::world::World;
use bevy_ecs::entity::Entity;
use bevy_ecs::component::Component;
use std::collections::HashMap;
use crate::types::*;
use crate::events::*;
use crate::command::*;
use crate::soldier::config::SoldierConfig;
use crate::city::config::CityGlobalConfig;
use crate::combat::config::CombatGlobalConfig;

// ══════════ Components ══════════

#[derive(Component, Clone, Debug)]
pub struct UnitIdComponent(pub UnitId);
#[derive(Component, Clone, Debug)] pub struct SoldierMarker;
#[derive(Component, Clone, Debug)] pub struct CityMarker;
#[derive(Component, Clone, Debug)] pub struct WaypointMarker;
#[derive(Component, Clone, Debug)] pub struct LogicalPosition(pub FixedVec2);
#[derive(Component, Clone, Debug)] pub struct Movement { pub speed: u32, pub target: Option<UnitId>, pub command_target: Option<UnitId>, pub waypoint: Option<FixedVec2>, pub force_move: bool }
#[derive(Component, Clone, Debug)] pub struct Health { pub current: u32, pub max: u32 }
#[derive(Component, Clone, Debug)] pub struct Attack { pub damage: u32, pub range: u32, pub interval_ticks: u32, pub cooldown_remaining: u32 }
#[derive(Component, Clone, Debug)] pub struct FactionComponent(pub Faction);
#[derive(Component, Clone, Debug)] pub struct SoldierTypeComponent(pub SoldierType);
#[derive(Component, Clone, Debug)] pub struct Level { pub level: u32, pub exp: u32 }
#[derive(Component, Clone, Debug)] pub struct ShieldComponent(pub ShieldState);
#[derive(Component, Clone, Debug)] pub struct CityOrigin(pub UnitId);
#[derive(Component, Clone, Debug)] pub struct SlowDebuff { pub stacks: u32, pub remaining_ticks: u32 }
#[derive(Component, Clone, Debug)] pub struct FearlessBuff { pub remaining_ticks: u32 }
#[derive(Component, Clone, Debug)] pub struct SoldierStateComponent(pub SoldierState);
#[derive(Component, Clone, Debug)] pub struct CityComponent { pub level: u32, pub max_level: u32, pub health_current: u32, pub health_max: u32, pub population: u32, pub max_population: u32, pub spawn_type: SoldierType, pub spawn_cooldown: u32, pub level_exp: u64, pub last_attacker_faction: Option<Faction> }
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
            Action::MoveTo { unit, target } => apply_movement(world, unit, target, false),
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
                    world.entity_mut(e).insert(ShieldComponent(state));
                }
            }
            Action::SetSpawnType { city, soldier_type } => {
                if let Some(e) = find_entity_by_unit_id(world, city) {
                    if let Some(mut cc) = world.entity_mut(e).get_mut::<CityComponent>() {
                        cc.spawn_type = soldier_type;
                    }
                }
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

// ══════════ System: soldier_movement ══════════

pub fn soldier_movement_system(world: &mut World) {
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    // Build position lookup from ALL entities with UnitIdComponent
    let positions: HashMap<UnitId, FixedVec2> = {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition)>();
        q.iter(world).map(|(_, id, pos)| (id.0, pos.0)).collect()
    };

    // Collect soldier data as owned values (avoid borrow issues)
    let mut soldier_updates: Vec<(Entity, FixedVec2)> = Vec::new();
    let mut arrivals: Vec<(Entity, u32)> = Vec::new();

    {
        let mut q = world.query::<(Entity, &LogicalPosition, &Movement, &SoldierTypeComponent, &SoldierStateComponent, Option<&SlowDebuff>, Option<&ShieldComponent>)>();
        for (e, pos, mov, st, _sst, slow, shield) in q.iter(world) {
            let mut speed = mov.speed as f32;
            if let Some(sl) = slow {
                let sc = &combat_config.slow_debuff;
                speed *= (sc.base_amount * sc.stack_mult.powi(sl.stacks as i32 - 1)).max(sc.max_reduction);
            }
            if let Some(sh) = shield {
                if sh.0 == ShieldState::ShieldUp { speed -= combat_config.shield.speed_penalty as f32; }
            }
            if speed <= 0.0 { continue; }
            let speed_fixed = Fixed::from_float(speed);

            let is_cav = st.0 == SoldierType::Cavalry;
            let target_pos = if is_cav {
                mov.command_target.and_then(|tid| positions.get(&tid).copied())
                    .or_else(|| mov.target.and_then(|tid| positions.get(&tid).copied()))
            } else {
                mov.target.and_then(|tid| positions.get(&tid).copied())
            }.or(mov.waypoint);

            let Some(target_pos) = target_pos else { continue };

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
            let ratio = if distance > Fixed::ZERO { move_amount / distance } else { Fixed::ZERO };
            let new_pos = FixedVec2::new(pos.0.x + delta.x * ratio, pos.0.y + delta.y * ratio);
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

// ══════════ System: city_spawn ══════════

pub fn city_spawn_system(world: &mut World) {
    let soldier_config = world.resource::<SoldierConfig>().clone();

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
            let spawn_pos = FixedVec2::new(pos.x + Fixed::from_int(30), pos.y);

            // Update city population + cooldown
            {
                let mut em = world.entity_mut(entity);
                if let Some(mut city) = em.get_mut::<CityComponent>() {
                    city.population += 1;
                    let mult = soldier_config.get(spawn_type).spawn_speed_mult;
                    city.spawn_cooldown = if mult > 0.0 { ((60.0 / mult) as u32).max(1) } else { 60 };
                }
            }

            // Create soldier
            let cfg = soldier_config.get(spawn_type);
            let origin = find_nearest_city_uid(world, spawn_pos, faction);
            world.spawn((
                UnitIdComponent(new_id), SoldierMarker, LogicalPosition(spawn_pos),
                Movement { speed: cfg.speed, target: None, command_target: None, waypoint: None, force_move: false },
                Health { current: cfg.health, max: cfg.health },
                Attack { damage: cfg.attack, range: cfg.attack_range, interval_ticks: cfg.attack_interval_ticks, cooldown_remaining: cfg.attack_interval_ticks },
                FactionComponent(faction), SoldierTypeComponent(spawn_type),
                Level { level: 1, exp: 0 }, ShieldComponent(ShieldState::Normal),
                CityOrigin(origin), SoldierStateComponent(SoldierState::Moving),
            ));

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

fn find_nearest_city_uid(world: &mut World, pos: FixedVec2, faction: Faction) -> UnitId {
    let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent)>();
    let mut best = UnitId(0);
    let mut best_d = i64::MAX;
    for (_, id, cp, cf) in q.iter(world) {
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
                        let req = (ci.max_hp as f32 * city_config.level_up_cost_multiplier) as u64;
                        if let Some(mut c) = world.entity_mut(ci.entity).get_mut::<CityComponent>() {
                            c.level_exp += eg;
                            if c.level_exp >= req {
                                c.level_exp -= req; c.level += 1;
                                c.health_max = c.level * city_config.level_hp_multiplier;
                                c.health_current = c.health_max;
                            }
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

    for (se, _) in &to_despawn {
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
