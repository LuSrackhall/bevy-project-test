use bevy::prelude::*;
use rand::Rng;

use crate::core::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), generate_map);
    }
}

#[derive(Event)]
pub struct MapGenerated {
    pub cities: Vec<CitySpawnData>,
}

pub struct CitySpawnData {
    pub position: Vec2,
    pub level: u32,
    pub max_level: u32,
    pub faction: Faction,
}

fn generate_map(
    config: Res<GameConfig>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    let total = rng.gen_range(config.min_cities..=config.max_cities);

    let mut positions: Vec<(Vec2, u32, u32)> = Vec::new();
    for _ in 0..total {
        let mut attempts = 0;
        loop {
            let x = rng.gen_range(config.margin..config.map_width - config.margin);
            let y = rng.gen_range(config.margin..config.map_height - config.margin);
            let pos = Vec2::new(x, y);
            let too_close = positions.iter().any(|(p, _, _)| pos.distance(*p) < config.city_min_distance);
            if !too_close || attempts >= 100 {
                let base_max = rng.gen_range(3..=7);
                let max_level = (base_max as i32 + rng.gen_range(-1..=2)).clamp(1, 10) as u32;
                positions.push((pos, 1, max_level));
                break;
            }
            attempts += 1;
        }
    }

    let neutral_count = rng.gen_range((total as f32 * 0.3) as u32..=(total as f32 * 0.5) as u32);
    let per_side = ((total as usize).saturating_sub(neutral_count as usize) + 1) / 2;
    let mut cities: Vec<CitySpawnData> = Vec::new();
    let mut assigned = 0;

    for (i, (pos, level, max_level)) in positions.iter().enumerate() {
        let faction = if i < per_side {
            assigned += 1;
            Faction::Player
        } else if assigned < 2 * per_side && i.saturating_sub(per_side) < per_side {
            assigned += 1;
            Faction::Enemy
        } else {
            Faction::Neutral
        };
        cities.push(CitySpawnData {
            position: *pos,
            level: *level,
            max_level: *max_level,
            faction,
        });
    }

    warn!("Map generated: {} cities ({} player, {} enemy, {} neutral)",
        cities.len(),
        cities.iter().filter(|c| c.faction == Faction::Player).count(),
        cities.iter().filter(|c| c.faction == Faction::Enemy).count(),
        cities.iter().filter(|c| c.faction == Faction::Neutral).count(),
    );

    commands.trigger(MapGenerated { cities });
}
