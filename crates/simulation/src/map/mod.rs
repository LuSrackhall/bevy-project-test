pub mod config;

use bevy_ecs::world::World;
use crate::types::*;
use crate::events::*;
use crate::soldier::*;
use crate::map::config::MapGenConfig;
use crate::city::config::CityGlobalConfig;


fn rng_range(rng: &mut DeterministicRng, min: u32, max: u32) -> u32 {
    let range = max.wrapping_sub(min).wrapping_add(1);
    if range == 0 { return min; }
    (rng.next_u64() as u32) % range + min
}

pub fn generate_map(world: &mut World) {
    let map_config = world.resource::<MapGenConfig>().clone();
    let city_config = world.resource::<CityGlobalConfig>().clone();

    let total_cities = {
        let mut rng = world.resource_mut::<DeterministicRng>();
        rng_range(&mut rng, map_config.min_cities, map_config.max_cities)
    };

    // Generate positions with level
    let mut positions: Vec<(FixedVec2, u32)> = Vec::new();
    for _ in 0..total_cities {
        let mut attempts = 0;
        loop {
            let (x, y, max_lvl) = {
                let mut rng = world.resource_mut::<DeterministicRng>();
                let rx = rng_range(&mut rng, map_config.margin, map_config.width - map_config.margin) as i32;
                let ry = rng_range(&mut rng, map_config.margin, map_config.height - map_config.margin) as i32;
                let rl = rng_range(&mut rng, map_config.city_level_range[0], map_config.city_level_range[1]);
                (rx, ry, rl)
            };
            let pos = FixedVec2::new(Fixed::from_int(x), Fixed::from_int(y));
            let min_dist_sq = Fixed::from_int(map_config.city_min_distance as i32);
            let min_sq = min_dist_sq * min_dist_sq;
            let too_close = positions.iter().any(|(p, _)| {
                let d = (*p - pos).length_squared();
                d < min_sq
            });
            if !too_close || attempts >= 100 {
                let max_level = (max_lvl as i32 + {
                    let mut rng = world.resource_mut::<DeterministicRng>();
                    rng_range(&mut rng, 0, 3) as i32 - 1
                }).clamp(1, 10) as u32;
                positions.push((pos, max_level));
                break;
            }
            attempts += 1;
        }
    }

    // Assign factions
    let neutral_count = {
        let mut rng = world.resource_mut::<DeterministicRng>();
        let min = (total_cities as f32 * map_config.neutral_city_ratio[0]) as u32;
        let max = (total_cities as f32 * map_config.neutral_city_ratio[1]) as u32;
        rng_range(&mut rng, min, max)
    };
    let per_side = (total_cities as usize).saturating_sub(neutral_count as usize + 1) / 2;

    let mut cities: Vec<(FixedVec2, u32, Faction)> = Vec::new();
    let mut assigned = 0;
    for (i, (pos, max_level)) in positions.iter().enumerate() {
        let faction = if i < per_side {
            assigned += 1;
            Faction::Player
        } else if assigned < 2 * per_side && i >= per_side {
            assigned += 1;
            Faction::Enemy
        } else {
            Faction::Neutral
        };
        cities.push((*pos, *max_level, faction));
    }

    // Spawn cities
    for (pos, max_level, faction) in &cities {
        let level = 1u32;
        let health_max = level * city_config.level_hp_multiplier;
        let pop_extra = {
            let mut rng = world.resource_mut::<DeterministicRng>();
            rng_range(&mut rng, level * 2, level * 5)
        };
        let max_population = level * city_config.base_population_per_level + pop_extra;
        let visual_radius = (city_config.visual_radius_base + level as f32 * city_config.visual_radius_per_level) as u32;

        let unit_id = {
            let mut id_gen = world.resource_mut::<IdGenerator>();
            id_gen.next()
        };

        world.spawn((
            UnitIdComponent(unit_id),
            CityMarker,
            LogicalPosition(*pos),
            CityComponent {
                level: 1,
                max_level: *max_level,
                health_current: health_max,
                health_max,
                population: 0,
                max_population,
                spawn_type: SoldierType::Militia,
                spawn_cooldown: 0,
                level_exp: 0,
                last_attacker_faction: None,
            },
            CityRadius(visual_radius),
            AuraHealComponent {
                base_heal: city_config.aura.base_heal,
                per_level_heal: city_config.aura.per_level_heal,
            },
            SpawnDirection(FixedVec2::new(Fixed::ONE, Fixed::ZERO)),
            FactionComponent(*faction),
        ));

        let mut events = world.resource_mut::<SimulationEvents>();
        events.spawned.push(UnitSpawned {
            unit_id,
            pos: *pos,
            faction: *faction,
            unit_kind: UnitKind::City,
        });
    }
}
