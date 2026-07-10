use crate::ui::hud::ButtonTheme;
use crate::AutoRecordReplay;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button as WidgetButton};
use simulation::map::MapSize;

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub struct AutoRecordToggle;

#[derive(Component)]
pub struct ReplayFileList;

#[derive(Component)]
pub struct ReplayFileEntry(pub String); // stores file path

#[derive(Component)]
pub struct NetworkRelayAddrInput(pub String);

#[derive(Component)]
pub struct NetworkPlayerCount(pub u8);

#[derive(Component)]
pub struct NetworkPlayerId(pub u8);

#[derive(Component, Clone, Copy)]
pub enum MapSizeBtn {
    Small,
    Medium,
    Large,
    Huge,
}

impl MapSizeBtn {
    fn label(&self) -> &str {
        match self {
            MapSizeBtn::Small => "小 (2000)",
            MapSizeBtn::Medium => "中 (3500)",
            MapSizeBtn::Large => "大 (5000)",
            MapSizeBtn::Huge => "巨大 (8000)",
        }
    }

    fn map_size(&self) -> MapSize {
        match self {
            MapSizeBtn::Small => MapSize::Small,
            MapSizeBtn::Medium => MapSize::Medium,
            MapSizeBtn::Large => MapSize::Large,
            MapSizeBtn::Huge => MapSize::Huge,
        }
    }
}

pub fn setup_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    auto_record: Res<AutoRecordReplay>,
) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    let record_label = if auto_record.0 {
        "自动录制: 开"
    } else {
        "自动录制: 关"
    };
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, ..default() },
        MainMenuUI,
    ))
    .with_children(|parent| {
        parent.spawn((Text::new("城池争霸"), TextFont { font: font.clone().into(), font_size: FontSize::Px(48.0), ..default() }));
        parent.spawn((Text::new("选择地图大小"), TextFont { font: font.clone().into(), font_size: FontSize::Px(20.0), ..default() },
            Node { margin: UiRect::top(Val::Px(20.0)), ..default() }));
        // Map size buttons row
        parent.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0),
            margin: UiRect::top(Val::Px(10.0)), ..default() })
            .with_children(|row| {
                for btn in [MapSizeBtn::Small, MapSizeBtn::Medium, MapSizeBtn::Large, MapSizeBtn::Huge] {
                    row.spawn((WidgetButton, Node { padding: UiRect::all(Val::Px(15.0)), border: UiRect::all(Val::Px(2.0)), ..default() },
                        btn, ButtonTheme::default(), Hovered::default(),
                        BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                        .with_child((Text::new(btn.label()), TextFont { font: font.clone().into(), font_size: FontSize::Px(18.0), ..default() }))
                        .observe(|_ev: On<Activate>, q: Query<&MapSizeBtn>, mut next: ResMut<NextState<crate::GameState>>, mut needs_reset: ResMut<crate::NeedsGameReset>| {
                            if let Ok(btn) = q.get(_ev.entity) {
                                *needs_reset = crate::NeedsGameReset::NewGame(btn.map_size());
                                next.set(crate::GameState::Playing);
                            }
                        });
                }
            });
        // Settings row
        parent.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0),
            margin: UiRect::top(Val::Px(30.0)), ..default() })
            .with_children(|row| {
                row.spawn((WidgetButton, Node { padding: UiRect::all(Val::Px(10.0)), border: UiRect::all(Val::Px(2.0)), ..default() },
                    AutoRecordToggle, ButtonTheme::dark(), Hovered::default(),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                    .with_child((Text::new(record_label), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }))
                    .observe(|_ev: On<Activate>, mut auto_record: ResMut<AutoRecordReplay>, _q: Query<&mut Text, With<AutoRecordToggle>>| {
                        auto_record.0 = !auto_record.0;
                        // Text update would need a separate system - for now just toggle the resource
                    });
            });
        // Network section
        parent.spawn((Text::new("联机模式"), TextFont { font: font.clone().into(), font_size: FontSize::Px(20.0), ..default() },
            Node { margin: UiRect::top(Val::Px(20.0)), ..default() }));
        parent.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0),
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(10.0)), ..default() })
            .with_children(|row| {
                row.spawn((Text::new("Relay 地址:"), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }));
                // Text input for relay address — store in a Resource
                row.spawn((WidgetButton, Node { padding: UiRect::all(Val::Px(10.0)), border: UiRect::all(Val::Px(2.0)), min_width: Val::Px(200.0), ..default() },
                    NetworkRelayAddrInput("127.0.0.1:9876".to_string()), ButtonTheme::dark(), Hovered::default(),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                    .with_child((Text::new("127.0.0.1:9876"), TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() }));
                row.spawn((Text::new("玩家:"), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }));
                row.spawn((WidgetButton, Node { padding: UiRect::all(Val::Px(10.0)), border: UiRect::all(Val::Px(2.0)), min_width: Val::Px(40.0), ..default() },
                    NetworkPlayerCount(2u8), ButtonTheme::dark(), Hovered::default(),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                    .with_child((Text::new("2"), TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() }))
                    .observe(|_ev: On<Activate>,
                        mut count_q: Query<(&mut NetworkPlayerCount, &Children)>,
                        mut id_q: Query<(&mut NetworkPlayerId, &Children), Without<NetworkPlayerCount>>,
                        mut text_q: Query<&mut Text>,
                    | {
                        let Ok((mut count, children)) = count_q.get_mut(_ev.entity) else { return };
                        count.0 = if count.0 >= 4 { 2 } else { count.0 + 1 };
                        if let Some(child) = children.iter().next().copied() {
                            if let Ok(mut text) = text_q.get_mut(child) {
                                text.0 = count.0.to_string();
                            }
                        }
                        for (mut id, id_children) in id_q.iter_mut() {
                            if id.0 >= count.0 {
                                id.0 = if count.0 > 0 { count.0 - 1 } else { 0 };
                                if let Some(child) = id_children.iter().next().copied() {
                                    if let Ok(mut text) = text_q.get_mut(child) {
                                        text.0 = id.0.to_string();
                                    }
                                }
                            }
                        }
                    });
                row.spawn((Text::new("  ID:"), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }));
                row.spawn((WidgetButton, Node { padding: UiRect::all(Val::Px(10.0)), border: UiRect::all(Val::Px(2.0)), min_width: Val::Px(30.0), ..default() },
                    NetworkPlayerId(0u8), ButtonTheme::dark(), Hovered::default(),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                    .with_child((Text::new("0"), TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() }))
                    .observe(|_ev: On<Activate>,
                        mut q: Query<(&mut NetworkPlayerId, &Children)>,
                        count_q: Query<&NetworkPlayerCount>,
                        mut text_q: Query<&mut Text>,
                    | {
                        let Ok((mut id, children)) = q.get_mut(_ev.entity) else { return };
                        let count = count_q.iter().next().map_or(2, |c| c.0);
                        id.0 = if id.0 >= count - 1 { 0 } else { id.0 + 1 };
                        if let Some(child) = children.iter().next().copied() {
                            if let Ok(mut text) = text_q.get_mut(child) {
                                text.0 = id.0.to_string();
                            }
                        }
                    });
            });
        parent.spawn(Node { flex_direction: FlexDirection::Row, margin: UiRect::top(Val::Px(10.0)), ..default() })
            .with_children(|row| {
                row.spawn((WidgetButton, Node { padding: UiRect::all(Val::Px(10.0)), border: UiRect::all(Val::Px(2.0)), ..default() },
                    ButtonTheme::default(), Hovered::default(),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                    .with_child((Text::new("开始联机"), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }))
                    .observe(|_ev: On<Activate>,
                        addr_q: Query<&NetworkRelayAddrInput>,
                        count_q: Query<&NetworkPlayerCount>,
                        id_q: Query<&NetworkPlayerId>,
                        mut next: ResMut<NextState<crate::GameState>>,
                        mut needs_reset: ResMut<crate::NeedsGameReset>| {
                        let addr = addr_q.iter().next().map_or("127.0.0.1:9876".to_string(), |a| a.0.clone());
                        let count = count_q.iter().next().map_or(2, |c| c.0);
                        let id = id_q.iter().next().map_or(0, |i| i.0);
                        *needs_reset = crate::NeedsGameReset::Network {
                            relay_addr: addr,
                            player_count: count,
                            player_id: id,
                        };
                        next.set(crate::GameState::Lobby);
                    });
            });
        // Replay section
        let replays = list_replay_files();
        if !replays.is_empty() {
            parent.spawn((Text::new("回放录像"), TextFont { font: font.clone().into(), font_size: FontSize::Px(20.0), ..default() },
                Node { margin: UiRect::top(Val::Px(20.0)), ..default() }));
            parent.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(5.0),
                margin: UiRect::top(Val::Px(10.0)), max_height: Val::Px(200.0), ..default() },
                ReplayFileList,
            )).with_children(|list| {
                for path in replays {
                    let label = std::path::Path::new(&path)
                        .file_stem().and_then(|s| s.to_str()).unwrap_or(&path).to_string();
                    list.spawn((WidgetButton, Node { padding: UiRect::all(Val::Px(8.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                        ReplayFileEntry(path.clone()), ButtonTheme::dark(), Hovered::default(),
                        BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 1.0))))
                        .with_child((Text::new(label), TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() }))
                        .observe(|_ev: On<Activate>, q: Query<&ReplayFileEntry>,
                            mut next: ResMut<NextState<crate::GameState>>,
                            mut needs_reset: ResMut<crate::NeedsGameReset>| {
                            if let Ok(entry) = q.get(_ev.entity) {
                                match load_replay_file(&entry.0) {
                                    Ok(replay) => {
                                        *needs_reset = crate::NeedsGameReset::Replay(replay);
                                        next.set(crate::GameState::Playing);
                                    }
                                    Err(e) => {
                                        bevy::log::warn!("Failed to load replay: {}", e);
                                    }
                                }
                            }
                        });
                }
            });
        }
    });
}

pub fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}

/// List .ron files in the replays/ directory.
fn list_replay_files() -> Vec<String> {
    let dir = std::path::Path::new("replays");
    if !dir.is_dir() {
        return vec![];
    }
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "ron")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    files.sort();
    files.reverse(); // newest first
    files
}

/// Load and validate a replay file from disk.
fn load_replay_file(path: &str) -> Result<simulation::replay::ReplayFile, String> {
    let ron_str = std::fs::read_to_string(path).map_err(|e| format!("Cannot read file: {}", e))?;
    simulation::replay::ReplayFile::from_ron(&ron_str)
}
