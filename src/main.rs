use bevy::prelude::*;
use bevy::window::WindowResolution;

use bevy_adapter::BevyAdapterPlugin;
use bevy_adapter::tick::SimulationWorld;
use presentation::PresentationPlugin;
use render_view::RenderViewPlugin;

// Keep old state for now
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, States, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

fn main() {
    // Initialize simulation world with map
    let mut sim_world = simulation::init_simulation_world(42);
    simulation::map::generate_map(&mut sim_world);

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "城池争霸".to_string(),
                    resolution: WindowResolution::new(1280, 720),
                    ..default()
                }),
                ..default()
            }),
        ))
        .init_state::<GameState>()
        .insert_non_send_resource(SimulationWorld(sim_world))
        .add_plugins(BevyAdapterPlugin)
        .add_plugins(PresentationPlugin)
        .add_plugins(RenderViewPlugin)
        .run();
}
