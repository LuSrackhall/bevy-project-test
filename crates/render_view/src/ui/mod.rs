pub mod menu;
pub mod hud;
pub mod pause;
pub mod gameover;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<hud::SelectedCity>()
            .init_resource::<hud::HudTexts>()
            .add_systems(OnEnter(crate::GameState::MainMenu), menu::setup_main_menu)
            .add_systems(OnExit(crate::GameState::MainMenu), menu::cleanup_main_menu)
            .add_systems(Update, menu::menu_button_system.run_if(in_state(crate::GameState::MainMenu)))
            .add_systems(OnEnter(crate::GameState::Playing), hud::setup_hud)
            .add_systems(Update, (
                hud::update_top_bar,
                hud::update_bottom_panel,
                hud::soldier_type_button_system,
                hud::toolbar_button_system,
                hud::city_click_system,
            ).run_if(in_state(crate::GameState::Playing)))
            .add_systems(OnEnter(crate::GameState::Paused), pause::setup_pause)
            .add_systems(OnExit(crate::GameState::Paused), pause::cleanup_pause)
            .add_systems(Update, pause::pause_button_system.run_if(in_state(crate::GameState::Paused)))
            .add_systems(OnEnter(crate::GameState::GameOver), gameover::setup_gameover)
            .add_systems(OnExit(crate::GameState::GameOver), gameover::cleanup_gameover)
            .add_systems(Update, gameover::gameover_button_system.run_if(in_state(crate::GameState::GameOver)))
            // Esc to pause (in Playing state)
            .add_systems(Update, handle_pause_input.run_if(in_state(crate::GameState::Playing)));
    }
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<crate::GameState>>,
    mut selection: ResMut<crate::selection::SelectionState>,
    mut selected_city: ResMut<crate::ui::hud::SelectedCity>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if !selection.selected_unit_ids.is_empty() || selected_city.0.is_some() {
            selection.selected_unit_ids.clear();
            selected_city.0 = None;
        } else {
            next.set(crate::GameState::Paused);
        }
    }
}
