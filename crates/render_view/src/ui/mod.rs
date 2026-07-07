pub mod gameover;
pub mod hud;
pub mod lobby;
pub mod menu;
pub mod pause;
pub mod replay_player;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<hud::HudTexts>()
            .init_resource::<hud::SeekPanelState>()
            .init_resource::<hud::ToastMessage>()
            .init_resource::<hud::HoveredSoldierType>()
            // Button visual feedback — runs in all states
            .add_systems(Update, hud::button_style_system)
            .add_systems(OnEnter(crate::GameState::MainMenu), menu::setup_main_menu)
            .add_systems(OnExit(crate::GameState::MainMenu), menu::cleanup_main_menu)
            // Lobby UI
            .add_systems(OnEnter(crate::GameState::Lobby), lobby::setup_lobby_ui)
            .add_systems(OnExit(crate::GameState::Lobby), (lobby::cleanup_lobby_ui, crate::cleanup_lobby_network))
            .add_systems(Update, lobby::update_lobby_status.run_if(in_state(crate::GameState::Lobby)))
            // HUD setup is registered in RenderViewPlugin (after reset_game_system)
            // update_top_bar runs even during replay (top bar shows real-time faction stats)
            // It has NO gate — not even Paused — since the replay seek driver updates world state
            .add_systems(
                Update,
                hud::update_top_bar.run_if(in_state(crate::GameState::Playing)),
            )
            // Other HUD update systems: gated from replay for performance
            .add_systems(
                Update,
                (
                    hud::update_bottom_panel,
                    hud::shield_button_visibility_system,
                    hud::seek_panel_mode_system,
                    hud::seek_panel_count_system,
                    hud::seek_panel_input_system,
                    hud::scope_popup_close_system,
                    hud::toast_tick_system,
                    hud::toast_display_system,
                    hud::selection_summary_toast_system,
                )
                    .run_if(
                        in_state(crate::GameState::Playing)
                            .and_then(not(resource_exists_and_equals(bevy_adapter::GameMode::Replay)))
                            .and_then(not(resource_exists_and_equals(bevy_adapter::Paused(true)))),
                    ),
            )
            // Hide interactive HUD areas in replay (no Paused gate — replay player must stay functional)
            .add_systems(
                Update,
                hud::hide_interactive_in_replay.run_if(in_state(crate::GameState::Playing)),
            )
            .add_systems(
                OnEnter(crate::GameState::GameOver),
                gameover::setup_gameover,
            )
            .add_systems(
                OnExit(crate::GameState::GameOver),
                gameover::cleanup_gameover,
            )
            // Pause visibility toggle
            .add_systems(
                Update,
                update_pause_visibility.run_if(in_state(crate::GameState::Playing)),
            )
            // Esc to pause (only when Playing and not paused)
            .add_systems(
                Update,
                handle_pause_input.run_if(
                    in_state(crate::GameState::Playing)
                        .and_then(not(resource_exists_and_equals(bevy_adapter::Paused(true)))),
                ),
            )
            // Replay player UI
            .add_systems(
                OnEnter(crate::GameState::Playing),
                replay_player::setup_replay_player.run_if(replay_player::in_replay_mode),
            )
            .add_systems(
                OnExit(crate::GameState::Playing),
                replay_player::cleanup_replay_player,
            )
            .add_systems(
                Update,
                (
                    replay_player::update_replay_player,
                    replay_player::replay_seek_system,
                )
                    .run_if(in_state(crate::GameState::Playing)),
            );
    }
}

fn update_pause_visibility(
    paused: Res<bevy_adapter::Paused>,
    mut query: Query<&mut Visibility, With<pause::PauseUI>>,
) {
    for mut vis in &mut query {
        *vis = if paused.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<bevy_adapter::Paused>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        paused.0 = true;
    }
}
