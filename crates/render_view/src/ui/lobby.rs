use crate::GameState;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

/// Marker component for lobby UI entities.
#[derive(Component)]
pub struct LobbyUI;

/// 设置 Lobby 等待室 UI。
/// 显示连接状态（Connecting / Connected / Failed），由 lobby_update_system 驱动阶段。
pub fn setup_lobby_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            LobbyUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("联机大厅"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
                LobbyUI,
            ));

            // Status text — updated by lobby_update_system
            parent.spawn((
                Text::new("正在连接..."),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                LobbyUILayout::StatusText,
                LobbyUI,
            ));

            // Cancel button
            parent.spawn((
                Button,
                Node {
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.5, 0.5, 0.6, 1.0)),
                LobbyUILayout::CancelButton,
                LobbyUI,
            ))
            .with_child((
                Text::new("取消"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
            ))
            .observe(
                |_ev: On<Activate>,
                 mut next: ResMut<NextState<GameState>>,
                 mut commands: Commands,
                 mut network_active: Option<ResMut<bevy_adapter::NetworkActive>>| {
                    // Clean up all Lobby-scoped resources
                    if let Some(ref mut na) = network_active {
                        na.0 = false;
                    }
                    commands.remove_resource::<crate::LobbyConnectionState>();
                    commands.remove_resource::<crate::ConnectionPollRx>();
                    commands.remove_resource::<bevy_adapter::transport::NetworkClientHandle>();
                    commands.remove_resource::<bevy_adapter::network::NetworkEventReceiver>();
                    commands.remove_resource::<bevy_adapter::transport::NetworkReceiver>();
                    commands.remove_resource::<bevy_adapter::transport::NetworkSender>();
                    // State transition drops handle → stop thread → clean exit
                    next.set(GameState::MainMenu);
                },
            );

            // Ready button
            parent.spawn((
                bevy::ui_widgets::Button,
                Node {
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.2, 0.6, 0.2, 1.0)),
                LobbyUI,
            ))
            .with_child((
                Text::new("就绪"),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf").into(),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
            ))
            .observe(
                |_ev: On<Activate>,
                 sender: Option<Res<bevy_adapter::transport::NetworkSender>>,
                 network_start: Option<Res<crate::NetworkGameStart>>| {
                    if let (Some(s), Some(ns)) = (sender, network_start) {
                        bevy::log::info!("[LOBBY] Sending LobbyReady (player_id={})", ns.player_id);
                        s.send_lobby_ready(ns.player_id);
                    }
                },
            );
        });
}

/// Marker for lobby UI elements that need runtime updates.
#[derive(Component)]
pub enum LobbyUILayout {
    StatusText,
    CancelButton,
}

/// 更新 Lobby 状态文本（由 lobby_update_system 调用）。
pub fn update_lobby_status(
    status: Option<Res<crate::LobbyConnectionState>>,
    mut q: Query<&mut Text, With<LobbyUILayout>>,
) {
    let Some(status) = status else { return };
    let phase = &status.phase;
    for mut text in q.iter_mut() {
        match phase {
            crate::LobbyPhase::Connecting => {
                text.0 = "正在连接...".to_string();
            }
            crate::LobbyPhase::Ready => {
                text.0 = "已准备，等待开始".to_string();
            }
            crate::LobbyPhase::Connected => {
                text.0 = "已连接，等待其他玩家...".to_string();
            }
            crate::LobbyPhase::Failed(err) => {
                text.0 = format!("连接失败: {}", err);
            }
        }
    }
}

/// 清理 Lobby UI 实体。
pub fn cleanup_lobby_ui(mut commands: Commands, q: Query<Entity, With<LobbyUI>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}
