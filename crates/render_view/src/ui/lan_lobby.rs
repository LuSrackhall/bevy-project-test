use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui_widgets::{Activate, Button as WidgetButton};

use crate::ui::hud::ButtonTheme;
use bevy_adapter::discovery::{LanDiscoveryPacket, RelayId, RoomState};

#[derive(Component)]
pub struct LanLobbyUI;

#[derive(Component)]
pub struct LanLobbyRoomList;

#[derive(Component)]
pub struct LanLobbyEmptyText;

#[derive(Component)]
pub struct LanLobbyRow;

#[derive(Component)]
pub struct LanLobbyRowData(pub RelayId);

#[derive(Component)]
pub struct RoomNameLabel;

#[derive(Component)]
pub struct MapLabel;

#[derive(Component)]
pub struct PlayersLabel;

#[derive(Component)]
pub struct StateLabel;

#[derive(Component)]
pub struct LanLobbyModal;

#[derive(Component)]
pub struct LanLobbyCreateBtn;

#[derive(Component)]
pub struct RoomRowJoinBtn(pub LanDiscoveryPacket);

/// Build the static LanLobby UI skeleton.
pub fn setup_lan_lobby(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            LanLobbyUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("局域网大厅"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
            ));

            // Column headers
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                margin: UiRect::top(Val::Px(20.0)),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            })
            .with_children(|row| {
                for (label, flex) in [
                    ("房间名称", 3.0),
                    ("地图", 2.0),
                    ("人数", 1.0),
                    ("状态", 1.0),
                    ("操作", 1.0),
                ] {
                    row.spawn((
                        Text::new(label),
                        TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() },
                        Node { flex_grow: flex, ..default() },
                    ));
                }
            });

            // Room list container (dynamically updated)
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
                LanLobbyRoomList,
            ));

            // Empty state (toggled by update_room_list)
            parent.spawn((
                Text::new("没有找到局域网房间"),
                TextFont { font: font.clone().into(), font_size: FontSize::Px(18.0), ..default() },
                Node { margin: UiRect::top(Val::Px(20.0)), display: Display::None, ..default() },
                LanLobbyEmptyText,
            ));

            // Bottom bar: return + create buttons
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            })
            .with_children(|row| {
                // Return button
                row.spawn((
                    WidgetButton,
                    Node { padding: UiRect::all(Val::Px(12.0)), border: UiRect::all(Val::Px(2.0)), ..default() },
                    ButtonTheme::dark(),
                    BorderColor::all(Color::srgba(0.5, 0.5, 0.6, 1.0)),
                ))
                .with_child((Text::new("返回"), TextFont { font: font.clone().into(), font_size: FontSize::Px(18.0), ..default() }))
                .observe(|_ev: On<Activate>, mut next: ResMut<NextState<crate::GameState>>| {
                    next.set(crate::GameState::MainMenu);
                });

                // Create room button
                row.spawn((
                    WidgetButton,
                    Node { padding: UiRect::all(Val::Px(12.0)), border: UiRect::all(Val::Px(2.0)), ..default() },
                    LanLobbyCreateBtn,
                    ButtonTheme::default(),
                    BorderColor::all(Color::srgba(0.2, 0.6, 0.2, 1.0)),
                ))
                .with_child((Text::new("创建房间"), TextFont { font: font.clone().into(), font_size: FontSize::Px(18.0), ..default() }))
                .observe(open_create_room_modal);
            });
        });
}

/// Dynamic room list: syncs LanServers → room rows each frame.
/// Uses incremental update (not full rebuild) to keep button entities stable
/// across frames, preserving WidgetButton's Pressed state for Activate events.
pub fn update_room_list(
    mut commands: Commands,
    servers: Res<crate::ui::lan::LanServers>,
    controller: Option<Res<bevy_adapter::session_host::SessionController>>,
    room_list: Query<Entity, With<LanLobbyRoomList>>,
    existing_rows: Query<(Entity, &LanLobbyRowData), (With<LanLobbyRow>, Without<LanLobbyRoomList>)>,
    mut empty_text: Query<&mut Node, With<LanLobbyEmptyText>>,
    asset_server: Res<AssetServer>,
    children_q: Query<&Children>,
) {
    let list_entity = match room_list.iter().next() {
        Some(e) => e,
        None => return,
    };
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    let own_relay_id = controller.as_ref().and_then(|c| c.current_relay_id());

    // Sort servers by relay_id for stable row ordering
    let mut sorted_servers: Vec<_> = servers.servers.iter().collect();
    sorted_servers.sort_by_key(|s| s.packet.advertisement.relay_id.0);

    // Build map of existing rows by relay_id
    let existing_map: std::collections::HashMap<RelayId, Entity> = existing_rows
        .iter()
        .map(|(e, d)| (d.0, e))
        .collect();

    // Toggle empty state
    if sorted_servers.is_empty() {
        if let Some(mut node) = empty_text.iter_mut().next() {
            node.display = Display::Flex;
        }
        // Despawn all rows
        for (_, &entity) in existing_map.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }
    if let Some(mut node) = empty_text.iter_mut().next() {
        node.display = Display::None;
    }

    // Remove stale rows (disappeared from discovery)
    for (&relay_id, &entity) in &existing_map {
        if !sorted_servers.iter().any(|s| s.packet.advertisement.relay_id == relay_id) {
            commands.entity(entity).despawn();
        }
    }

    // Add new rows / update existing rows
    for entry in &sorted_servers {
        let pkt = &entry.packet;
        let adv = &pkt.advertisement;
        let room = &adv.room;
        let relay_id = adv.relay_id;
        let is_own = own_relay_id == Some(relay_id);
        let is_playing = room.state == RoomState::Playing;
        let is_full = room.current_players >= room.max_players;
        let can_join = !is_own && !is_playing && !is_full;

        let state_label = match room.state {
            RoomState::Waiting => "等待中",
            RoomState::Starting => "启动中",
            RoomState::Playing => "游戏中",
        };

        if let Some(&existing_entity) = existing_map.get(&relay_id) {
            // Update text on existing row children (keeps button entity stable)
            let players_text = format!("{}/{}", room.current_players, room.max_players);
            if let Ok(children) = children_q.get(existing_entity) {
                for (i, child) in children.iter().enumerate() {
                    let new_text = match i {
                        0 => &room.room_name,
                        1 => &room.map_id,
                        2 => &players_text,
                        3 => state_label,
                        _ => continue,
                    };
                    commands.entity(child).insert(Text::new(new_text.to_string()));
                }
            }
        } else {
            // Spawn new row
            let pkt_clone = pkt.clone();
            commands.entity(list_entity).with_children(|parent| {
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    LanLobbyRow,
                    LanLobbyRowData(relay_id),
                ))
                .with_children(|row| {
                    // Room name
                    row.spawn((Text::new(&room.room_name), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }, Node { flex_grow: 3.0, ..default() }, RoomNameLabel));
                    // Map
                    row.spawn((Text::new(&room.map_id), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }, Node { flex_grow: 2.0, ..default() }, MapLabel));
                    // Players
                    row.spawn((Text::new(format!("{}/{}", room.current_players, room.max_players)), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }, Node { flex_grow: 1.0, ..default() }, PlayersLabel));
                    // State
                    row.spawn((Text::new(state_label), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }, Node { flex_grow: 1.0, ..default() }, StateLabel));

                    // Action button
                    if is_own {
                        row.spawn((Text::new("⚡"), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }, Node { flex_grow: 1.0, ..default() }));
                    } else if can_join {
                        let pkt_for_join = pkt_clone.clone();
                        row.spawn((
                            WidgetButton,
                            Node { padding: UiRect::all(Val::Px(4.0)), border: UiRect::all(Val::Px(1.0)), flex_grow: 1.0, ..default() },
                            ButtonTheme::default(),
                            RoomRowJoinBtn(pkt_clone.clone()),
                            BorderColor::all(Color::srgba(0.2, 0.6, 0.2, 1.0)),
                        ))
                        .with_child((Text::new("加入"), TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() }))
                        .observe(move |_ev: On<Activate>,
                            mut request: ResMut<crate::JoinRoomRequest>| {
                            eprintln!("[JOIN_BTN] activate — setting JoinRoomRequest");
                            request.requested = true;
                            request.relay_id = pkt_for_join.advertisement.relay_id;
                            request.endpoint = pkt_for_join.advertisement.endpoint.clone();
                            request.room_id = pkt_for_join.advertisement.room.room_id;
                        });
                    } else if is_playing {
                        row.spawn((Text::new("游戏中"), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }, Node { flex_grow: 1.0, ..default() }));
                    } else {
                        row.spawn((Text::new("满员"), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }, Node { flex_grow: 1.0, ..default() }));
                    }
                });
            });
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// CreateRoomModal
// ═══════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct ModalRoomName;

#[derive(Component)]
pub struct ModalMapSize;

#[derive(Component)]
pub struct ModalPlayerCount;

#[derive(Component)]
pub struct ModalError;

#[derive(Resource)]
pub struct ModalState {
    pub room_name: String,
    pub map_id: String,
    pub max_players: u8,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            room_name: String::new(),
            map_id: "grassland_small".into(),
            max_players: 2,
        }
    }
}

/// Observe the "创建房间" button to open the modal.
fn open_create_room_modal(
    _trigger: On<Activate>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<LanLobbyModal>>,
) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");

    // Prevent multiple modals
    if existing.iter().next().is_some() {
        return;
    }

    let wrapper = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            LanLobbyModal,
        ))
        .id();

    let modal = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(30.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("创建房间"),
                TextFont { font: font.clone().into(), font_size: FontSize::Px(24.0), ..default() },
            ));

            // Room name
            parent.spawn((
                Text::new("房间名:"),
                TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() },
                Node { margin: UiRect::top(Val::Px(15.0)), ..default() },
            ));
            parent.spawn((
                EditableText {
                    visible_width: Some(15.),
                    allow_newlines: false,
                    ..default()
                },
                TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() },
                Node { padding: UiRect::all(Val::Px(8.0)), min_width: Val::Px(200.0), border: UiRect::all(Val::Px(1.0)), ..default() },
                BorderColor::all(Color::srgba(0.5, 0.5, 0.6, 1.0)),
                AutoFocus,
                ModalRoomName,
            ));

            // Map size (simple UI for MVP: click to cycle)
            parent.spawn((
                Text::new("地图:"),
                TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() },
                Node { margin: UiRect::top(Val::Px(15.0)), ..default() },
            ));
            parent.spawn((
                WidgetButton,
                Node { padding: UiRect::all(Val::Px(8.0)), min_width: Val::Px(200.0), ..default() },
                ButtonTheme::dark(),
                ModalMapSize,
            ))
            .with_child((Text::new("grassland_small"), TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() }));

            // Player count
            parent.spawn((
                Text::new("最大人数:"),
                TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() },
                Node { margin: UiRect::top(Val::Px(15.0)), ..default() },
            ));
            parent.spawn((
                WidgetButton,
                Node { padding: UiRect::all(Val::Px(8.0)), min_width: Val::Px(200.0), ..default() },
                ButtonTheme::dark(),
                ModalPlayerCount,
            ))
            .with_child((Text::new("2"), TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() }));

            // Error text (hidden by default)
            parent.spawn((
                Text::new(""),
                TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() },
                Node { margin: UiRect::top(Val::Px(10.0)), display: Display::None, ..default() },
                ModalError,
            ));

            // Buttons row
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(15.0),
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            })
            .with_children(|btn_row| {
                // Cancel
                btn_row.spawn((
                    WidgetButton,
                    Node { padding: UiRect::all(Val::Px(10.0)), border: UiRect::all(Val::Px(2.0)), ..default() },
                    ButtonTheme::dark(),
                    BorderColor::all(Color::srgba(0.5, 0.5, 0.6, 1.0)),
                ))
                .with_child((Text::new("取消"), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }))
                .observe(|_ev: On<Activate>, mut commands: Commands, q: Query<Entity, With<LanLobbyModal>>| {
                    for e in q.iter() { commands.entity(e).despawn(); }
                });

                // Create
                btn_row.spawn((
                    WidgetButton,
                    Node { padding: UiRect::all(Val::Px(10.0)), border: UiRect::all(Val::Px(2.0)), ..default() },
                    ButtonTheme::default(),
                    BorderColor::all(Color::srgba(0.2, 0.6, 0.2, 1.0)),
                ))
                .with_child((Text::new("创建房间"), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }))
                .observe(|_ev: On<Activate>,
                    mut request: ResMut<crate::CreateRoomRequest>,
                    modal_state: Option<Res<ModalState>>,
                    mut commands: Commands,
                    q: Query<Entity, With<LanLobbyModal>>,
                    editable_text_q: Query<&EditableText, With<ModalRoomName>>| {
                    let state = match modal_state {
                        Some(s) => s,
                        None => return,
                    };
                    request.requested = true;
                    // Read room name from EditableText input, fall back to ModalState
                    if let Some(editable) = editable_text_q.iter().next() {
                        let name = editable.value().to_string();
                        request.room_name = if name.is_empty() {
                            state.room_name.clone()
                        } else {
                            name
                        };
                    } else {
                        request.room_name = state.room_name.clone();
                    }
                    request.map_id = state.map_id.clone();
                    request.max_players = state.max_players;
                    // Close modal
                    for e in q.iter() { commands.entity(e).despawn(); }
                });
            });
        })
        .id();

    commands.entity(wrapper).add_child(modal);
}

pub fn cleanup_lan_lobby(mut commands: Commands, q: Query<Entity, With<LanLobbyUI>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}
