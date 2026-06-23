use bevy::prelude::*;
use bevy::window::MonitorSelection;

use bevy_adapter::tick::SimulationWorld;
use bevy_adapter::BevyAdapterPlugin;
use presentation::PresentationPlugin;
use render_view::RenderViewPlugin;

use bevy::log::LogPlugin;

fn main() {
    let window_mode = if cfg!(target_arch = "wasm32") {
        bevy::window::WindowMode::Windowed
    } else {
        bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
    };

    App::new()
        .add_plugins((DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "城池争霸".to_string(),
                    mode: window_mode,
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                filter: "warn,icu_provider=error".to_string(),
                ..default()
            }),))
        .insert_non_send(SimulationWorld(simulation::init_simulation_world(0)))
        .add_plugins((BevyAdapterPlugin, PresentationPlugin, RenderViewPlugin))
        .run();
}
