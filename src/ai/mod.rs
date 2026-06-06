use bevy::prelude::*;
use rand::Rng;

use crate::core::*;
use crate::city::City;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiTimer>()
           .add_systems(Update, ai_decision_system.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
struct AiTimer(Timer);

impl Default for AiTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(AI_EVALUATION_INTERVAL, TimerMode::Repeating))
    }
}

fn ai_decision_system(
    time: Res<Time>,
    mut timer: ResMut<AiTimer>,
    mut city_query: Query<(Entity, &mut City, &Transform)>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();

    for (_entity, mut city, _transform) in city_query.iter_mut() {
        if city.faction == Faction::Enemy && city.health < city.max_health * 0.5 {
            city.spawn_type = match rng.gen_range(0..3) {
                0 => SoldierType::Infantry,
                1 => SoldierType::Archer,
                _ => SoldierType::Cavalry,
            };
        }
    }
}
