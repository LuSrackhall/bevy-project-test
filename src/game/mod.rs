use bevy::prelude::*;

use crate::core::*;
use crate::city::City;
use crate::soldier::{Soldier, SoldierDiedEvent};
use crate::combat::Arrow;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameStats>()
           .add_observer(|_trigger: On<SoldierDiedEvent>, mut stats: ResMut<GameStats>| {
               if _trigger.faction != Faction::Player {
                   stats.total_kills += 1;
               }
           })
           .add_systems(OnEnter(GameState::Playing), start_game)
           .add_systems(OnExit(GameState::Playing), cleanup_game)
           .add_systems(Update, check_victory_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, handle_pause_input.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
pub struct GameStats {
    pub start_time: u64,
    pub total_kills: u32,
    pub player_cities_remaining: usize,
    pub winner: Option<Faction>,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            start_time: 0,
            total_kills: 0,
            player_cities_remaining: 0,
            winner: None,
        }
    }
}

fn start_game(mut stats: ResMut<GameStats>, time: Res<Time>, config: Res<GameConfig>) {
    info!("Starting new game with map size {}x{}", config.map_width, config.map_height);
    stats.start_time = time.elapsed().as_secs();
    stats.total_kills = 0;
    stats.winner = None;
}

fn cleanup_game(
    mut commands: Commands,
    city_query: Query<Entity, With<City>>,
    soldier_query: Query<Entity, With<Soldier>>,
    arrow_query: Query<Entity, With<Arrow>>,
) {
    for entity in city_query.iter() { commands.entity(entity).despawn(); }
    for entity in soldier_query.iter() { commands.entity(entity).despawn(); }
    for entity in arrow_query.iter() { commands.entity(entity).despawn(); }
}

fn check_victory_system(
    city_query: Query<&City>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stats: ResMut<GameStats>,
) {
    let player_cities: Vec<_> = city_query.iter().filter(|c| c.faction == Faction::Player).collect();
    let enemy_cities: Vec<_> = city_query.iter().filter(|c| c.faction == Faction::Enemy).collect();
    stats.player_cities_remaining = player_cities.len();

    if enemy_cities.is_empty() {
        info!("Player wins!");
        stats.winner = Some(Faction::Player);
        next_state.set(GameState::GameOver);
    } else if player_cities.is_empty() {
        info!("Player loses!");
        stats.winner = Some(Faction::Enemy);
        next_state.set(GameState::GameOver);
    }
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut selection: ResMut<crate::input::SelectionState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        // If soldiers selected, deselect first; otherwise pause
        if !selection.selected_soldiers.is_empty() || selection.selected_city.is_some() {
            selection.selected_soldiers.clear();
            selection.selected_city = None;
        } else {
            next_state.set(GameState::Paused);
        }
    }
}

