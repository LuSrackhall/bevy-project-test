use bevy_ecs::world::World;
use bevy_ecs::entity::Entity;
use crate::types::*;
use crate::command::*;
use crate::soldier::*;
use crate::soldier::config::SoldierConfig;


fn rng_range(rng: &mut DeterministicRng, min: u32, max: u32) -> u32 {
    let range = max.wrapping_sub(min).wrapping_add(1);
    if range == 0 { return min; }
    (rng.next_u64() as u32) % range + min
}

const AI_TICK_INTERVAL: u32 = 40;

pub fn ai_decide(world: &mut World, current_tick: u32) {
    if current_tick % AI_TICK_INTERVAL != 0 { return; }

    let soldier_config = world.resource::<SoldierConfig>().clone();

    // Collect AI (Enemy) cities
    let ai_cities: Vec<(UnitId, FixedVec2, u32, u32)> = {
        let mut query = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &CityComponent, &FactionComponent)>();
        query.iter(world)
            .filter(|(_, _, _, _, fac)| fac.0 == Faction::Enemy)
            .map(|(_, id, pos, city, _)| (id.0, pos.0, city.level, city.max_level))
            .collect()
    };

    // Collect Player cities
    let player_cities: Vec<(UnitId, FixedVec2, u32)> = {
        let mut query = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &CityComponent, &FactionComponent)>();
        query.iter(world)
            .filter(|(_, _, _, _, fac)| fac.0 == Faction::Player)
            .map(|(_, id, pos, city, _)| (id.0, pos.0, city.level))
            .collect()
    };

    // Collect Neutral cities
    let neutral_cities: Vec<(UnitId, FixedVec2, u32, u32)> = {
        let mut query = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &CityComponent, &FactionComponent)>();
        query.iter(world)
            .filter(|(_, _, _, _, fac)| fac.0 == Faction::Neutral)
            .map(|(_, id, pos, city, _)| (id.0, pos.0, city.level, city.health_max))
            .collect()
    };

    // Collect all soldiers
    let soldiers: Vec<(UnitId, FixedVec2, Faction, bool, Option<UnitId>)> = {
        let mut query = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &FactionComponent, &Movement)>();
        query.iter(world)
            .map(|(_, id, pos, fac, mov)| (id.0, pos.0, fac.0, mov.target.is_some(), mov.command_target))
            .collect()
    };

    // Expansion + Attack + Upgrade
    let mut commands: Vec<GameCommand> = Vec::new();

    for &(ai_city_id, ai_pos, ai_level, _ai_max_level) in &ai_cities {
        // Expansion: target nearest neutral city
        if !neutral_cities.is_empty() {
            let mut by_dist: Vec<(usize, i64)> = neutral_cities.iter()
                .enumerate()
                .map(|(i, (_, npos, _, _))| (i, (ai_pos - *npos).length_squared().0))
                .collect();
            by_dist.sort_by_key(|(_, d)| *d);

            let (idx, _) = by_dist[0];
            let (target_city_id, target_pos, _, target_hp) = neutral_cities[idx];
            let radius_sq = Fixed::from_int(500) * Fixed::from_int(500);

            let ai_nearby = soldiers.iter()
                .filter(|(_, pos, fac, _, _)| *fac == Faction::Enemy && (*pos - target_pos).length_squared() <= radius_sq)
                .count();

            if ai_nearby > 0 {
                for (sid, spos, sfac, has_target, _) in &soldiers {
                    if *sfac == Faction::Enemy && !*has_target && (*spos - ai_pos).length_squared() <= radius_sq {
                        commands.push(GameCommand {
                            tick: current_tick + 1,
                            player_id: 1,
                            action: Action::MoveTo { unit: *sid, target: target_pos },
                        });
                    }
                }
            }
        }

        // Attack: target nearest player city
        if !player_cities.is_empty() {
            let mut by_dist: Vec<(usize, i64)> = player_cities.iter()
                .enumerate()
                .map(|(i, (_, ppos, _))| (i, (ai_pos - *ppos).length_squared().0))
                .collect();
            by_dist.sort_by_key(|(_, d)| *d);

            for &(idx, _) in &by_dist {
                let (target_city_id, target_pos, player_level) = player_cities[idx];
                if ai_level >= player_level {
                    let radius_sq = Fixed::from_int(500) * Fixed::from_int(500);
                    let ai_nearby = soldiers.iter()
                        .filter(|(_, pos, fac, _, _)| *fac == Faction::Enemy && (*pos - target_pos).length_squared() <= radius_sq)
                        .count();
                    let player_nearby = soldiers.iter()
                        .filter(|(_, pos, fac, _, _)| *fac == Faction::Player && (*pos - target_pos).length_squared() <= radius_sq)
                        .count();

                    if ai_nearby as f32 > player_nearby as f32 * 1.3 && ai_nearby > 0 {
                        for &(sid, spos, sfac, has_target, _) in &soldiers {
                            if sfac == Faction::Enemy && !has_target {
                                commands.push(GameCommand {
                                    tick: current_tick + 1,
                                    player_id: 1,
                                    action: Action::MoveTo { unit: sid, target: target_pos },
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
        let low_hp_cities: Vec<UnitId> = {
            let mut query = world.query::<(Entity, &UnitIdComponent, &CityComponent, &FactionComponent)>();
            query.iter(world)
                .filter(|(_, _, city, fac)| fac.0 == Faction::Enemy && city.health_current < city.health_max / 2)
                .map(|(_, id, _, _)| id.0)
                .collect()
        };
        for city_id in low_hp_cities {
            let st = match {
                let mut rng = world.resource_mut::<DeterministicRng>();
                rng_range(&mut rng, 0, 3)
            } {
                0 => SoldierType::Infantry,
                1 => SoldierType::Archer,
                _ => SoldierType::Cavalry,
            };
            commands.push(GameCommand {
                tick: current_tick + 1,
                player_id: 1,
                action: Action::SetSpawnType { city: city_id, soldier_type: st },
            });
        }
    }

    // Push all commands
    let mut cmd_buf = world.resource_mut::<CommandBuffer>();
    for cmd in commands {
        cmd_buf.push(cmd);
    }
}
