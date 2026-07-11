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
pub struct ReplayFileEntry(pub String);

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
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        MainMenuUI,
    ))
    .with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("城池争霸"),
            TextFont {
                font: font.clone().into(),
                font_size: FontSize::Px(48.0),
                ..default()
            },
        ));

        // Mode selection row
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            margin: UiRect::top(Val::Px(30.0)),
            ..default()
        })
        .with_children(|row| {
            // 单人模式
            row.spawn((
                WidgetButton,
                Node {
                    padding: UiRect::all(Val::Px(20.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    min_width: Val::Px(180.0),
                    ..default()
                },
                ButtonTheme::default(),
                Hovered::default(),
                BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0)),
            ))
            .with_child((
                Text::new("单人模式"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
            ));

            // 局域网模式
            row.spawn((
                WidgetButton,
                Node {
                    padding: UiRect::all(Val::Px(20.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    min_width: Val::Px(180.0),
                    ..default()
                },
                ButtonTheme::default(),
                Hovered::default(),
                BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0)),
            ))
            .with_child((
                Text::new("局域网模式"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
            ))
            .observe(|_ev: On<Activate>, mut next: ResMut<NextState<crate::GameState>>| {
                next.set(crate::GameState::LanLobby);
            });
        });

        // Map size section (单人模式用)
        parent.spawn((
            Text::new("选择地图大小"),
            TextFont {
                font: font.clone().into(),
                font_size: FontSize::Px(20.0),
                ..default()
            },
            Node {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
        ));
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            margin: UiRect::top(Val::Px(10.0)),
            ..default()
        })
        .with_children(|row| {
            for btn in [
                MapSizeBtn::Small,
                MapSizeBtn::Medium,
                MapSizeBtn::Large,
                MapSizeBtn::Huge,
            ] {
                row.spawn((
                    WidgetButton,
                    Node {
                        padding: UiRect::all(Val::Px(15.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    btn,
                    ButtonTheme::default(),
                    Hovered::default(),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0)),
                ))
                .with_child((
                    Text::new(btn.label()),
                    TextFont {
                        font: font.clone().into(),
                        font_size: FontSize::Px(18.0),
                        ..default()
                    },
                ))
                .observe(
                    |_ev: On<Activate>,
                     q: Query<&MapSizeBtn>,
                     mut next: ResMut<NextState<crate::GameState>>,
                     mut needs_reset: ResMut<crate::NeedsGameReset>| {
                        if let Ok(btn) = q.get(_ev.entity) {
                            *needs_reset = crate::NeedsGameReset::NewGame(btn.map_size());
                            next.set(crate::GameState::Playing);
                        }
                    },
                );
            }
        });

        // Settings row
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            margin: UiRect::top(Val::Px(30.0)),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                WidgetButton,
                Node {
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                AutoRecordToggle,
                ButtonTheme::dark(),
                Hovered::default(),
                BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0)),
            ))
            .with_child((
                Text::new(record_label),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
            ))
            .observe(
                |_ev: On<Activate>,
                 mut auto_record: ResMut<AutoRecordReplay>,
                 _q: Query<&mut Text, With<AutoRecordToggle>>| {
                    auto_record.0 = !auto_record.0;
                },
            );
        });

        // Replay section
        let replays = list_replay_files();
        if !replays.is_empty() {
            parent.spawn((
                Text::new("回放录像"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    max_height: Val::Px(200.0),
                    ..default()
                },
                ReplayFileList,
            ))
            .with_children(|list| {
                for path in replays {
                    let label = std::path::Path::new(&path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&path)
                        .to_string();
                    list.spawn((
                        WidgetButton,
                        Node {
                            padding: UiRect::all(Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        ReplayFileEntry(path.clone()),
                        ButtonTheme::dark(),
                        Hovered::default(),
                        BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 1.0)),
                    ))
                    .with_child((
                        Text::new(label),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                    ))
                    .observe(
                        |_ev: On<Activate>,
                         q: Query<&ReplayFileEntry>,
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
                        },
                    );
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
        .filter(|e| e.path().extension().map(|ext| ext == "ron").unwrap_or(false))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    files.sort();
    files.reverse();
    files
}

/// Load and validate a replay file from disk.
fn load_replay_file(path: &str) -> Result<simulation::replay::ReplayFile, String> {
    let ron_str =
        std::fs::read_to_string(path).map_err(|e| format!("Cannot read file: {}", e))?;
    simulation::replay::ReplayFile::from_ron(&ron_str)
}
