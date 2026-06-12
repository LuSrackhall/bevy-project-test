pub mod debug_shape;
pub mod camera;
pub mod selection;
pub mod ui;
pub mod unit_info_bar;

use bevy::prelude::*;

/// Game state enum — shared across the render view.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, States, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

pub struct RenderViewPlugin;

impl Plugin for RenderViewPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .init_resource::<crate::selection::SelectionState>()
            .add_plugins(crate::ui::UiPlugin)
            .add_systems(Startup, crate::camera::setup_camera)
            .init_resource::<crate::unit_info_bar::UnitInfoBarSettings>()
            .add_systems(Update, (
                crate::debug_shape::draw_debug_shapes_system,
                crate::unit_info_bar::unit_info_bar_system,
                crate::unit_info_bar::info_bar_mode_toggle_system,
                crate::camera::camera_drag_system,
                crate::camera::camera_zoom_system,
                crate::camera::center_on_player_city,
                crate::selection::selection_click_system,
                crate::selection::drag_select_system,
                crate::selection::selection_shortcut_system,
                crate::selection::selection_visual_system,
                crate::selection::drag_visual_system,
                crate::selection::command_issue_system,
                crate::selection::seek_stance_shortcut_system,
                crate::selection::waypoint_cleanup_system,
                check_victory_system,
            ).run_if(in_state(GameState::Playing)));
    }
}

/// Check if all cities of one faction are gone.
fn check_victory_system(
    mut sim_world: bevy::ecs::system::NonSendMut<bevy_adapter::tick::SimulationWorld>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let world = &mut sim_world.0;
    let mut query = world.query::<(&simulation::soldier::FactionComponent,)>();
    let mut player = false;
    let mut enemy = false;
    for (f,) in query.iter(world) {
        match f.0 {
            simulation::types::Faction::Player => player = true,
            simulation::types::Faction::Enemy => enemy = true,
            _ => {}
        }
    }
    if !player || !enemy {
        next_state.set(GameState::GameOver);
    }
}
