pub mod menu;
pub mod hud;
pub mod pause;
pub mod gameover;
pub mod observer;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<hud::HudTexts>()
            .init_resource::<hud::SeekPanelState>()
            .init_resource::<hud::ToastMessage>()
            .add_systems(OnEnter(crate::GameState::MainMenu), menu::setup_main_menu)
            .add_systems(OnExit(crate::GameState::MainMenu), menu::cleanup_main_menu)
            .add_systems(OnEnter(crate::GameState::Playing), hud::setup_hud)
            .add_systems(Update, (
                hud::update_top_bar,
                hud::update_bottom_panel,
                hud::soldier_type_button_system,
                hud::toolbar_button_system,
                hud::shield_button_visibility_system,
                hud::seek_panel_mode_system,
                hud::seek_panel_dropdown_system,
                hud::seek_panel_count_system,
                hud::seek_panel_input_system,
                hud::seek_panel_issue_system,
                hud::toast_tick_system,
                hud::toast_display_system,
                hud::selection_summary_toast_system,
            ).run_if(in_state(crate::GameState::Playing)))
            .add_systems(OnEnter(crate::GameState::Paused), pause::setup_pause)
            .add_systems(OnExit(crate::GameState::Paused), pause::cleanup_pause)
            .add_systems(OnEnter(crate::GameState::GameOver), gameover::setup_gameover)
            .add_systems(OnExit(crate::GameState::GameOver), gameover::cleanup_gameover)
            // Esc to pause (in Playing state)
            .add_systems(Update, handle_pause_input.run_if(in_state(crate::GameState::Playing)));
    }
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<crate::GameState>>,
    mut selection: ResMut<crate::selection::SelectionState>,
    mut seek_state: ResMut<hud::SeekPanelState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        // If input is active, deactivate instead of deselecting/pausing
        if seek_state.input_active {
            seek_state.input_active = false;
            return;
        }
        if seek_state.dropdown_open {
            seek_state.dropdown_open = false;
            return;
        }
        if !selection.selected_unit_ids.is_empty() || selection.selected_city.is_some() {
            selection.selected_unit_ids.clear();
            selection.selected_city = None;
        } else {
            next.set(crate::GameState::Paused);
        }
    }
}
