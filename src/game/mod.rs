use bevy::prelude::*;

use crate::core::*;
use crate::city::City;
use crate::soldier::Soldier;
use crate::combat::Arrow;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), start_game)
           .add_systems(OnExit(GameState::Playing), cleanup_game)
           .add_systems(Update, check_victory_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, handle_pause_input.run_if(in_state(GameState::Playing)));
    }
}

fn start_game(_commands: Commands, config: Res<GameConfig>) {
    info!("Starting new game with map size {}x{}", config.map_width, config.map_height);
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
) {
    let player_cities = city_query.iter().filter(|c| c.faction == Faction::Player).count();
    let enemy_cities = city_query.iter().filter(|c| c.faction == Faction::Enemy).count();
    if enemy_cities == 0 {
        info!("Player wins!");
        next_state.set(GameState::GameOver);
    } else if player_cities == 0 {
        info!("Player loses!");
        next_state.set(GameState::GameOver);
    }
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}
