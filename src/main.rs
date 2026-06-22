use bevy::prelude::*;
use bevy::window::MonitorSelection;

use bevy_adapter::tick::SimulationWorld;
use bevy_adapter::BevyAdapterPlugin;
use presentation::PresentationPlugin;
use render_view::RenderViewPlugin;

use bevy::log::LogPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "城池争霸".to_string(),
                    mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
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
