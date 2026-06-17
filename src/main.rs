use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_prototype_lyon::prelude::*;

use bevy_adapter::BevyAdapterPlugin;
use bevy_adapter::tick::SimulationWorld;
use presentation::PresentationPlugin;
use render_view::RenderViewPlugin;

fn main() {
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
            ShapePlugin,
        ))
        .insert_non_send_resource(SimulationWorld(simulation::init_simulation_world(0)))
        .add_plugins((BevyAdapterPlugin, PresentationPlugin, RenderViewPlugin))
        .run();
}
