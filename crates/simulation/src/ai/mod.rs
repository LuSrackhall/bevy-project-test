use crate::command::*;
use crate::soldier::config::SoldierConfig;
use crate::soldier::*;
use crate::types::*;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;

fn rng_range(rng: &mut DeterministicRng, min: u32, max: u32) -> u32 {
    let range = max.wrapping_sub(min).wrapping_add(1);
    if range == 0 {
        return min;
    }
    (rng.next_u64() as u32) % range + min
}

const AI_TICK_INTERVAL: u32 = 40;

/// AI 决策入口。读取 PlayerSlots，为每个 AI Controller 的 faction 生成命令。
pub fn ai_decide(world: &mut World, current_tick: u32) {
    if !current_tick.is_multiple_of(AI_TICK_INTERVAL) {
        return;
    }

    let slots = world.get_resource::<PlayerSlots>();
    let ai_slots: Vec<FactionId> = match slots {
        Some(s) => s.slots.iter()
            .filter(|s| matches!(s.controller, Controller::AI(_)))
            .map(|s| s.faction)
            .collect(),
        None => vec![FactionId(1)], // fallback: legacy Enemy
    };

    let _soldier_config = world.resource::<SoldierConfig>().clone();

    for &ai_faction in &ai_slots {
        ai_decide_for_faction(world, current_tick, ai_faction);
    }
}

/// AI 为单个 faction 生成命令（从原 ai_decide 提取）。
fn ai_decide_for_faction(world: &mut World, current_tick: u32, ai_faction: FactionId) {

    // Collect AI (Enemy) cities — sorted for determinism (§0.1)
    let mut ai_cities: Vec<(UnitId, FixedVec2, u32, u32)> = {
        let mut query = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &CityComponent,
            &FactionComponent,
        )>();
        query
            .iter(world)
            .filter(|(_, _, _, _, fac)| fac.0 == ai_faction)
            .map(|(_, id, pos, city, _)| (id.0, pos.0, city.level, city.max_level))
            .collect()
    };
    ai_cities.sort_by_key(|(uid, _, _, _)| *uid);

    // Collect Player cities — sorted for determinism (§0.1)
    let mut player_cities: Vec<(UnitId, FixedVec2, u32)> = {
        let mut query = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &CityComponent,
            &FactionComponent,
        )>();
        query
            .iter(world)
            .filter(|(_, _, _, _, fac)| fac.0 == FactionId(0))
            .map(|(_, id, pos, city, _)| (id.0, pos.0, city.level))
            .collect()
    };
    player_cities.sort_by_key(|(uid, _, _)| *uid);

    // Collect Neutral cities — sorted for determinism (§0.1)
    let mut neutral_cities: Vec<(UnitId, FixedVec2, u32, u32)> = {
        let mut query = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &CityComponent,
            &FactionComponent,
        )>();
        query
            .iter(world)
            .filter(|(_, _, _, _, fac)| fac.0 == FactionId(2))
            .map(|(_, id, pos, city, _)| (id.0, pos.0, city.level, city.health_max))
            .collect()
    };
    neutral_cities.sort_by_key(|(uid, _, _, _)| *uid);

    // Collect all soldiers — sorted for determinism (§0.1)
    let mut soldiers: Vec<(UnitId, FixedVec2, FactionId, bool, Option<UnitId>)> = {
        let mut query = world.query::<(
            Entity,
            &UnitIdComponent,
            &LogicalPosition,
            &FactionComponent,
            &Movement,
        )>();
        query
            .iter(world)
            .map(|(_, id, pos, fac, mov)| {
                (id.0, pos.0, fac.0, mov.target.is_some(), mov.command_target)
            })
            .collect()
    };
    soldiers.sort_by_key(|(uid, _, _, _, _)| *uid);

    // Expansion + Attack + Upgrade
    let mut commands: Vec<GameCommand> = Vec::new();

    for &(_ai_city_id, ai_pos, ai_level, _ai_max_level) in &ai_cities {
        // Expansion: target nearest neutral city
        if !neutral_cities.is_empty() {
            let mut by_dist: Vec<(usize, i64)> = neutral_cities
                .iter()
                .enumerate()
                .map(|(i, (uid, npos, _, _))| (i, (ai_pos - *npos).length_squared().0))
                .collect();
            by_dist.sort_by_key(|(i, d)| (*d, neutral_cities[*i].0));

            let (idx, _) = by_dist[0];
            let (_target_city_id, target_pos, _, _target_hp) = neutral_cities[idx];
            let radius_sq = Fixed::from_int(500) * Fixed::from_int(500);

            let ai_nearby = soldiers
                .iter()
                .filter(|(_, pos, fac, _, _)| {
                    *fac == ai_faction && (*pos - target_pos).length_squared() <= radius_sq
                })
                .count();

            if ai_nearby > 0 {
                for (sid, spos, sfac, has_target, _) in &soldiers {
                    if *sfac == ai_faction
                        && !*has_target
                        && (*spos - ai_pos).length_squared() <= radius_sq
                    {
                        commands.push(GameCommand {
                            tick: current_tick + 1,
                            player_id: 1,
                            action: Action::MoveTo {
                                unit: *sid,
                                target: target_pos,
                            },
                        });
                    }
                }
            }
        }

        // Attack: target nearest player city
        if !player_cities.is_empty() {
            let mut by_dist: Vec<(usize, i64)> = player_cities
                .iter()
                .enumerate()
                .map(|(i, (uid, ppos, _))| (i, (ai_pos - *ppos).length_squared().0))
                .collect();
            by_dist.sort_by_key(|(i, d)| (*d, player_cities[*i].0));

            for &(idx, _) in &by_dist {
                let (_target_city_id, target_pos, player_level) = player_cities[idx];
                if ai_level >= player_level {
                    let radius_sq = Fixed::from_int(500) * Fixed::from_int(500);
                    let ai_nearby = soldiers
                        .iter()
                        .filter(|(_, pos, fac, _, _)| {
                            *fac == ai_faction
                                && (*pos - target_pos).length_squared() <= radius_sq
                        })
                        .count();
                    let player_nearby = soldiers
                        .iter()
                        .filter(|(_, pos, fac, _, _)| {
                            *fac == FactionId(0)
                                && (*pos - target_pos).length_squared() <= radius_sq
                        })
                        .count();

                    if (ai_nearby as u64) * 10 > (player_nearby as u64) * 13 && ai_nearby > 0 {
                        for &(sid, _spos, sfac, has_target, _) in &soldiers {
                            if sfac == ai_faction && !has_target {
                                commands.push(GameCommand {
                                    tick: current_tick + 1,
                                    player_id: 1,
                                    action: Action::MoveTo {
                                        unit: sid,
                                        target: target_pos,
                                    },
                                });
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    // Defense: low HP cities switch spawn and recall
    {
        let mut low_hp_cities: Vec<UnitId> = {
            let mut query =
                world.query::<(Entity, &UnitIdComponent, &CityComponent, &FactionComponent)>();
            query
                .iter(world)
                .filter(|(_, _, city, fac)| {
                    fac.0 == ai_faction && city.health_current < city.health_max / 2
                })
                .map(|(_, id, _, _)| id.0)
                .collect()
        };
        low_hp_cities.sort();
        for city_id in low_hp_cities {
            let rng_val = {
                let mut rng = world.resource_mut::<DeterministicRng>();
                rng_range(&mut rng, 0, 3)
            };
            let st = match rng_val {
                0 => SoldierType::Infantry,
                1 => SoldierType::Archer,
                _ => SoldierType::Cavalry,
            };
            commands.push(GameCommand {
                tick: current_tick + 1,
                player_id: 1,
                action: Action::SetSpawnType {
                    city: city_id,
                    soldier_type: st,
                },
            });
        }
    }

    // Push all commands
    let mut cmd_buf = world.resource_mut::<CommandBuffer>();
    for cmd in commands {
        cmd_buf.push(cmd);
    }
}
