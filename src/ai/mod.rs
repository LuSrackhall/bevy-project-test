use bevy::prelude::*;
use rand::Rng;

use crate::core::*;
use crate::city::City;
use crate::soldier::Soldier;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiTimer>()
           .add_systems(Update, ai_decision_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, ai_defense_system.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
struct AiTimer(Timer);

impl Default for AiTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(AI_EVALUATION_INTERVAL, TimerMode::Repeating))
    }
}

/// Count nearby soldiers of a given faction using read-only iter
fn count_nearby(
    center: Vec2,
    radius: f32,
    faction: Faction,
    query: &Query<(&Transform, &mut Soldier)>,
) -> (u32, f32) {
    let mut count = 0u32;
    let mut attack = 0.0f32;
    for (t, s) in query.iter() {
        if s.faction == faction && t.translation.xy().distance(center) <= radius {
            count += 1;
            attack += s.attack + s.level as f32 * LEVEL_ATTACK_GAIN;
        }
    }
    (count, attack)
}

fn ai_decision_system(
    time: Res<Time>,
    mut timer: ResMut<AiTimer>,
    city_query: Query<(Entity, &City, &Transform)>,
    mut soldier_query: Query<(&Transform, &mut Soldier)>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() { return; }

    // Collect AI city data as owned values
    let ai_cities: Vec<(Entity, Vec2, u32)> = city_query.iter()
        .filter(|(_, c, _)| c.faction == Faction::Enemy)
        .map(|(e, c, t)| (e, t.translation.xy(), c.level))
        .collect();

    let player_cities: Vec<(Entity, Vec2)> = city_query.iter()
        .filter(|(_, c, _)| c.faction == Faction::Player)
        .map(|(e, _, t)| (e, t.translation.xy()))
        .collect();

    let neutral_cities: Vec<(Entity, Vec2, u32, f32)> = city_query.iter()
        .filter(|(_, c, _)| c.faction == Faction::Neutral)
        .map(|(e, c, t)| (e, t.translation.xy(), c.level, city_max_health(c.level)))
        .collect();

    if ai_cities.is_empty() { return; }
    let ai_highest = ai_cities.iter().map(|c| c.2).max().unwrap_or(1);

    for &(ai_entity, ai_pos, _ai_level) in &ai_cities {
        // Expansion
        if !neutral_cities.is_empty() {
            let mut by_dist: Vec<(usize, f32)> = neutral_cities.iter()
                .enumerate().map(|(i, n)| (i, ai_pos.distance(n.1))).collect();
            by_dist.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            let (te, tp, _lvl, mh) = neutral_cities[by_dist[0].0];
            let (cnt, atk) = count_nearby(tp, AI_SCOUT_RADIUS, Faction::Enemy, &soldier_query);
            if atk > mh * 1.5 && cnt > 0 {
                for (_t, mut s) in soldier_query.iter_mut() {
                    if s.faction == Faction::Enemy && s.target.is_none() {
                        s.target = Some(te); s.state = SoldierState::Moving;
                    }
                }
            }
        }

        // Attack
        if !player_cities.is_empty() {
            let mut by_dist: Vec<(usize, f32)> = player_cities.iter()
                .enumerate().map(|(i, p)| (i, ai_pos.distance(p.1))).collect();
            by_dist.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            for &(idx, _) in &by_dist {
                let (te, tp) = player_cities[idx];
                let ok = city_query.get(te).map_or(false, |(_, c, _)| c.level <= ai_highest);
                if ok {
                    let (ac, _) = count_nearby(tp, AI_SCOUT_RADIUS, Faction::Enemy, &soldier_query);
                    let (pc, _) = count_nearby(tp, AI_SCOUT_RADIUS, Faction::Player, &soldier_query);
                    if ac as f32 > pc as f32 * 1.3 && ac > 0 {
                        for (_t, mut s) in soldier_query.iter_mut() {
                            if s.faction == Faction::Enemy && s.target.is_none() {
                                s.target = Some(te); s.state = SoldierState::Moving;
                            }
                        }
                        break;
                    }
                }
            }
        }

        // Upgrade
        if let Ok((_, city, _)) = city_query.get(ai_entity) {
            if city.population as f32 > city.max_population as f32 * 0.6 && city.level < city.max_level {
                let surplus = city.population.saturating_sub((city.max_population as f32 * 0.6) as u32);
                let mut n = 0u32;
                for (_t, mut s) in soldier_query.iter_mut() {
                    if n >= surplus { break; }
                    if s.faction == Faction::Enemy && s.city_origin == Some(ai_entity) {
                        s.target = Some(ai_entity); n += 1;
                    }
                }
            }
        }
    }
}

fn ai_defense_system(
    mut city_query: Query<(Entity, &mut City, &Transform)>,
    mut soldier_query: Query<(&Transform, &mut Soldier)>,
) {
    for (ce, mut city, _) in city_query.iter_mut() {
        if city.faction == Faction::Enemy && city.health < city.max_health * 0.5 {
            let mut rng = rand::thread_rng();
            city.spawn_type = match rng.gen_range(0..3) {
                0 => SoldierType::Infantry,
                1 => SoldierType::Archer,
                _ => SoldierType::Cavalry,
            };
            let mut i = 0u32;
            for (_t, mut s) in soldier_query.iter_mut() {
                if s.faction == Faction::Enemy && s.city_origin == Some(ce) {
                    i += 1;
                    if i % 2 == 0 { s.target = Some(ce); }
                }
            }
        }
    }
}
