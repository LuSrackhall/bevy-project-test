use bevy::prelude::*;
use bevy::window::WindowResolution;

use bevy_adapter::BevyAdapterPlugin;
use bevy_adapter::tick::SimulationWorld;
use presentation::PresentationPlugin;
use render_view::RenderViewPlugin;

/// Initialize the simulation world with map pre-generated.
fn init_sim_world() -> SimulationWorld {
    let mut world = simulation::init_simulation_world(42);
    simulation::map::generate_map(&mut world);
    SimulationWorld(world)
}

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
        ))
        .insert_non_send_resource(init_sim_world())
        .add_plugins(BevyAdapterPlugin)
        .add_plugins(PresentationPlugin)
        .add_plugins(RenderViewPlugin)
        .run();
}
