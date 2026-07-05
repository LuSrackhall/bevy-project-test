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

    let mut app = App::new();
    app.add_plugins((DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "城池争霸".to_string(),
                mode: window_mode,
                ..default()
            }),
            ..default()
        })
        .set(LogPlugin {
            filter: "info,simulation=warn,relay=warn,wgpu=warn,naga=warn,accesskit=warn,cargo=warn,icu_provider=error".to_string(),
            ..default()
        }),))
        .insert_non_send(SimulationWorld::new(simulation::init_simulation_world(0)))
        .add_plugins((BevyAdapterPlugin, PresentationPlugin, RenderViewPlugin));

    // CLI args for network mode (Phase 1 testing)
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--relay") {
        if let Some(relay_addr) = args.get(pos + 1) {
            let player_id: u8 = args.iter()
                .position(|a| a == "--player-id")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let player_count: u8 = args.iter()
                .position(|a| a == "--players")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            app.insert_resource(render_view::NeedsGameReset::Network {
                relay_addr: relay_addr.clone(),
                player_count,
                player_id,
            });
            app.add_systems(Startup, |mut next: ResMut<NextState<render_view::GameState>>| {
                next.set(render_view::GameState::Lobby);
            });
        }
    }

    app.run();
}
