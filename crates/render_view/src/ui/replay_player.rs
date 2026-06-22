//! Replay player UI — control bar shown during replay playback.

use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button as WidgetButton};
use bevy::picking::hover::Hovered;
use crate::ui::hud::ButtonTheme;
use bevy_adapter::replay::{GameMode, ReplayController, ReplayStatus};

#[derive(Component)]
pub struct ReplayPlayerUI;

#[derive(Component)]
pub struct ReplayTickText;

#[derive(Component)]
pub struct ReplaySpeedBtn(u32);

#[derive(Component)]
pub struct ReplayPauseBtn;

#[derive(Component)]
pub struct ReplayProgressBar;

/// Setup replay player UI when entering Playing state in Replay mode.
pub fn setup_replay_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    commands.spawn((Node {
        width: Val::Percent(100.0), height: Val::Px(40.0),
        position_type: PositionType::Absolute,
        bottom: Val::Px(0.0), left: Val::Px(0.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(8.0),
        padding: UiRect::horizontal(Val::Px(20.0)),
        ..default()
    }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), ReplayPlayerUI, Pickable::IGNORE))
    .with_children(|bar| {
        // Pause/Play button
        bar.spawn((WidgetButton, Node { padding: UiRect::all(Val::Px(6.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
            ReplayPauseBtn, ButtonTheme::dark(), Hovered::default(),
            BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
            .with_child((Text::new("⏸"), TextFont { font: font.clone().into(), font_size: FontSize::Px(16.0), ..default() }))
            .observe(|_ev: On<Activate>, mut ctrl: Option<ResMut<ReplayController>>| {
                if let Some(ref mut c) = ctrl { c.is_paused = !c.is_paused; }
            });

        // Speed buttons
        for speed in [1u32, 2, 4] {
            let label = format!("{}x", speed);
            bar.spawn((WidgetButton, Node { padding: UiRect::all(Val::Px(6.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                ReplaySpeedBtn(speed), ButtonTheme::dark(), Hovered::default(),
                BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                .with_child((Text::new(label), TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() }))
                .observe(move |_ev: On<Activate>, mut ctrl: Option<ResMut<ReplayController>>| {
                    if let Some(ref mut c) = ctrl { c.speed_multiplier = speed; }
                });
        }

        // Progress bar (visual only — click to seek to middle as placeholder)
        bar.spawn((Node {
            width: Val::Px(300.0), height: Val::Px(8.0),
            border: UiRect::all(Val::Px(1.0)), ..default()
        }, BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
          BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 1.0)),
          ReplayProgressBar, Pickable::default()));

        // Tick counter
        bar.spawn((Text::new("T 0 / 0"), TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)), ReplayTickText));
    });
}

/// Update replay player UI each frame.
pub fn update_replay_player(
    game_mode: Res<GameMode>,
    ctrl: Option<Res<ReplayController>>,
    status: Option<Res<ReplayStatus>>,
    mut tick_text: Query<&mut Text, With<ReplayTickText>>,
    mut pause_btn: Query<&mut Text, (With<ReplayPauseBtn>, Without<ReplayTickText>)>,
) {
    let GameMode::Replay = *game_mode else { return };
    let Some(ref ctrl) = ctrl else { return };

    // Update tick text
    let total = ctrl.replay.total_ticks;
    let current = ctrl.current_tick;
    for mut text in tick_text.iter_mut() {
        **text = format!("T {} / {}", current, total);
    }

    // Update pause button
    let label = if ctrl.is_paused { "▶" } else { "⏸" };
    for mut text in pause_btn.iter_mut() {
        **text = label.to_string();
    }
}

/// Cleanup replay player UI.
pub fn cleanup_replay_player(mut commands: Commands, query: Query<Entity, With<ReplayPlayerUI>>) {
    for e in query.iter() { commands.entity(e).despawn(); }
}

/// Condition: only show replay player when in Replay mode.
pub fn in_replay_mode(game_mode: Res<GameMode>) -> bool {
    *game_mode == GameMode::Replay
}

/// Condition: hide replay player when not in Replay mode.
pub fn not_in_replay_mode(game_mode: Res<GameMode>) -> bool {
    *game_mode != GameMode::Replay
}
