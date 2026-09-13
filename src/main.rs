use bevy::prelude::*;
use bevy::window::MonitorSelection;

use bevy_adapter::discovery::RelayId;
use bevy_adapter::tick::SimulationWorld;
use bevy_adapter::BevyAdapterPlugin;
use presentation::PresentationPlugin;
use render_view::RenderViewPlugin;

use bevy::log::LogPlugin;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let windowed = args.iter().any(|a| a == "--windowed");

    let window_mode = if windowed || cfg!(target_arch = "wasm32") {
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

    // 运行时可观测（P0.4）：`--features remote` 时开放 BRP。
    // agent 可用 JSON-RPC 查询运行中的仿真状态（`city_conquest/probe`），
    // 也可调用官方内置方法（world.query / world.trigger_event / Screenshot）做 UI 验收。
    //
    // `--brp-port <port>`：同机跑多个客户端时必须错开（默认 15702），否则第二个实例
    // 绑定失败——这是"双客户端联机测量/验收"的前提。
    #[cfg(feature = "remote")]
    {
        let brp_port: u16 = args
            .iter()
            .position(|a| a == "--brp-port")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(bevy::remote::http::DEFAULT_PORT);
        app.add_plugins((
            bevy_adapter::remote::plugin(),
            bevy::remote::http::RemoteHttpPlugin::default().with_port(brp_port),
        ));
    }

    // CLI args for network mode (Phase 1 testing)
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--relay") {
        if let Some(relay_addr) = args.get(pos + 1) {
            let player_id: u8 = args
                .iter()
                .position(|a| a == "--player-id")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let player_count: u8 = args
                .iter()
                .position(|a| a == "--players")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            // 必须与 relay 的 `--relay-id` 一致，否则被拒（Relay identity mismatch）。
            let relay_id: u64 = args
                .iter()
                .position(|a| a == "--relay-id")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            app.insert_resource(render_view::NeedsGameReset::Network {
                relay_addr: relay_addr.clone(),
                player_count,
                player_id: Some(player_id),
                relay_id: RelayId(relay_id),
            });
            app.add_systems(
                Startup,
                |mut next: ResMut<NextState<render_view::GameState>>| {
                    next.set(render_view::GameState::Lobby);
                },
            );
        }
    }

    app.run();
}
