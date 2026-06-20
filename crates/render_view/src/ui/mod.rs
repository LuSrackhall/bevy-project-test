pub mod menu;
pub mod hud;
pub mod pause;
pub mod gameover;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<hud::HudTexts>()
            .init_resource::<hud::SeekPanelState>()
            .init_resource::<hud::ToastMessage>()
            .init_resource::<hud::HoveredSoldierType>()
            // Button visual feedback — runs in all states
            .add_systems(Update, hud::button_style_system)
            .add_systems(OnEnter(crate::GameState::MainMenu), menu::setup_main_menu)
            .add_systems(OnExit(crate::GameState::MainMenu), menu::cleanup_main_menu)
            // HUD setup is registered in RenderViewPlugin (after reset_game_system)
            .add_systems(Update, (
                hud::update_top_bar,
                hud::update_bottom_panel,
                hud::shield_button_visibility_system,
                hud::seek_panel_mode_system,
                hud::seek_panel_count_system,
                hud::seek_panel_input_system,
                hud::scope_popup_close_system,
                hud::toast_tick_system,
                hud::toast_display_system,
                hud::selection_summary_toast_system,
            ).run_if(in_state(crate::GameState::Playing).and(not(resource_exists_and_equals(bevy_adapter::Paused(true))))))
            .add_systems(OnEnter(crate::GameState::GameOver), gameover::setup_gameover)
            .add_systems(OnExit(crate::GameState::GameOver), gameover::cleanup_gameover)
            // Pause visibility toggle
            .add_systems(Update, update_pause_visibility.run_if(in_state(crate::GameState::Playing)))
            // Esc to pause (only when Playing and not paused)
            .add_systems(Update, handle_pause_input
                .run_if(in_state(crate::GameState::Playing).and(not(resource_exists_and_equals(bevy_adapter::Paused(true))))));
    }
}

fn update_pause_visibility(
    paused: Res<bevy_adapter::Paused>,
    mut query: Query<&mut Visibility, With<pause::PauseUI>>,
) {
    for mut vis in &mut query {
        *vis = if paused.0 { Visibility::Visible } else { Visibility::Hidden };
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
