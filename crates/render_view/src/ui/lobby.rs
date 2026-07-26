use crate::GameState;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

/// Marker component for lobby UI entities.
#[derive(Component)]
pub struct LobbyUI;

/// Container for the dynamic player list rows.
#[derive(Component)]
pub struct LobbyPlayerListContainer;

/// A single row in the player list.
#[derive(Component)]
pub struct LobbyPlayerRow;

/// Whether the local player has already clicked ready.
#[derive(Resource, Default)]
pub struct ReadyState(pub bool);

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
                padding: UiRect::axes(Val::Px(40.0), Val::Px(20.0)),
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

            // Column headers
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                margin: UiRect::top(Val::Px(20.0)),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            })
            .with_children(|row| {
                for (label, flex) in [("玩家", 1.0), ("状态", 1.0)] {
                    row.spawn((
                        Text::new(label),
                        TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() },
                        Node { flex_grow: flex, ..default() },
                    ));
                }
            });

            // Player list container (dynamically updated by update_lobby_player_list)
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
                LobbyPlayerListContainer,
                LobbyUI,
            ));

            // Status text — updated by lobby_update_system
            parent.spawn((
                Text::new("正在连接..."),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
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
                    margin: UiRect::top(Val::Px(15.0)),
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
                 mut commands: Commands| {
                    commands.remove_resource::<bevy_adapter::transport::NetworkClientHandle>();
                    commands.remove_resource::<bevy_adapter::network::NetworkEventReceiver>();
                    commands.remove_resource::<bevy_adapter::transport::NetworkReceiver>();
                    commands.remove_resource::<bevy_adapter::transport::NetworkSender>();
                    next.set(GameState::MainMenu);
                },
            );

            // Ready / Start Game button row
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(15.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|btn_row| {
                // Ready button (for non-host players)
                btn_row.spawn((
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.2, 0.6, 0.2, 1.0)),
                    LobbyActionButton::Ready,
                    LobbyUI,
                ))
                .with_child((
                    Text::new("就绪"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: FontSize::Px(18.0),
                        ..default()
                    },
                ))
                .observe(
                    |_ev: On<Activate>,
                     sender: Option<Res<bevy_adapter::transport::NetworkSender>>,
                     network_start: Option<Res<crate::NetworkGameStart>>,
                     is_host: Option<Res<crate::IsHost>>,
                     mut ready_state: ResMut<ReadyState>| {
                        // Host uses the Start Game button instead
                        let host = is_host.map(|h| h.0).unwrap_or(false);
                        if host { return; }
                        if ready_state.0 { return; }
                        if let (Some(s), Some(ns)) = (sender, network_start) {
                            bevy::log::info!("[LOBBY] Sending LobbyReady (player_id={})", ns.player_id);
                            s.send_lobby_ready(ns.player_id);
                            ready_state.0 = true;
                        }
                    },
                );

                // Start Game button (for host only)
                btn_row.spawn((
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.2, 0.6, 0.2, 1.0)),
                    LobbyActionButton::StartGame,
                    LobbyUI,
                ))
                .with_child((
                    Text::new("开始游戏"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: FontSize::Px(18.0),
                        ..default()
                    },
                ))
                .observe(
                    |_ev: On<Activate>,
                     sender: Option<Res<bevy_adapter::transport::NetworkSender>>,
                     network_start: Option<Res<crate::NetworkGameStart>>,
                     is_host: Option<Res<crate::IsHost>>,
                     mut ready_state: ResMut<ReadyState>| {
                        let host = is_host.map(|h| h.0).unwrap_or(false);
                        if !host { return; }
                        if ready_state.0 { return; }
                        if let (Some(s), Some(ns)) = (sender, network_start) {
                            bevy::log::info!("[LOBBY] Host starting game (player_id={})", ns.player_id);
                            s.send_lobby_ready(ns.player_id);
                            ready_state.0 = true;
                        }
                    },
                );
            });
        });
}

/// Dynamic player list: syncs LobbyPlayerList → player rows each frame.
pub fn update_lobby_player_list(
    mut commands: Commands,
    players: Res<crate::LobbyPlayerList>,
    list_container: Query<Entity, With<LobbyPlayerListContainer>>,
    mut existing_rows: Query<Entity, (With<LobbyPlayerRow>, Without<LobbyPlayerListContainer>)>,
    asset_server: Res<AssetServer>,
) {
    let list_entity = match list_container.iter().next() {
        Some(e) => e,
        None => return,
    };
    let font = asset_server.load("fonts/Arial Unicode.ttf");

    // Despawn existing rows
    for e in existing_rows.iter() {
        commands.entity(e).despawn();
    }

    // Spawn player rows
    for entry in &players.0 {
        let is_ready = entry.ready;
        let status_text = if is_ready { "✔ 就绪" } else { "✘ 未就绪" };
        commands.entity(list_entity).with_children(|parent| {
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                LobbyPlayerRow,
            ))
            .with_children(|row| {
                row.spawn((
                    Text::new(format!("玩家 {}", entry.player_id)),
                    TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() },
                    Node { flex_grow: 1.0, ..default() },
                ));
                row.spawn((
                    Text::new(status_text),
                    TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() },
                    Node { flex_grow: 1.0, ..default() },
                ));
            });
        });
    }
}

/// Update ready button text after local player clicks ready.
pub fn update_ready_button(
    ready_state: Res<ReadyState>,
    is_host: Option<Res<crate::IsHost>>,
    mut q: Query<&mut Text, (With<LobbyActionButton>, Without<LobbyPlayerListContainer>)>,
) {
    let host = is_host.map(|h| h.0).unwrap_or(false);
    for mut text in q.iter_mut() {
        if ready_state.0 {
            if host {
                text.0 = "已开始".to_string();
            } else {
                text.0 = "已就绪".to_string();
            }
        }
    }
}

/// Marker for lobby UI elements that need runtime updates.
#[derive(Component)]
pub enum LobbyUILayout {
    StatusText,
    CancelButton,
}

/// Marker to distinguish ready vs start game buttons for text updates.
#[derive(Component)]
pub enum LobbyActionButton {
    Ready,
    StartGame,
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
