//! Replay player UI — control bar shown during replay playback.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::ui_widgets::{Activate, Button as WidgetButton};
use bevy::picking::hover::Hovered;
use crate::ui::hud::ButtonTheme;
use bevy_adapter::replay::{GameMode, ReplayController};

#[derive(Component)]
pub struct ReplayPlayerUI;

#[derive(Component)]
pub struct ReplayTickText;

#[derive(Component)]
pub struct ReplaySpeedBtn(pub u32);

#[derive(Component)]
pub struct ReplayPauseBtnText;

#[derive(Component)]
pub struct ReplayProgressFill;

#[derive(Component)]
pub struct ReplayProgressBg;

/// Setup replay player UI when entering Playing state in Replay mode.
pub fn setup_replay_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    commands.spawn((Node {
        width: Val::Percent(100.0), height: Val::Px(44.0),
        position_type: PositionType::Absolute,
        bottom: Val::Px(0.0), left: Val::Px(0.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(10.0),
        padding: UiRect::horizontal(Val::Px(16.0)),
        ..default()
    }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)), ReplayPlayerUI))
    .with_children(|bar| {
        // Pause/Play button
        bar.spawn((WidgetButton, Node { padding: UiRect::horizontal(Val::Px(8.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
            ButtonTheme::dark(), Hovered::default(),
            BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
            .with_children(|btn| {
                btn.spawn((Text::new("⏸"), TextFont { font: font.clone().into(), font_size: FontSize::Px(18.0), ..default() },
                    ReplayPauseBtnText));
            })
            .observe(|_ev: On<Activate>, mut ctrl: Option<ResMut<ReplayController>>| {
                if let Some(ref mut c) = ctrl { c.is_paused = !c.is_paused; }
            });

        // Speed buttons
        for speed in [1u32, 2, 4] {
            let label = format!("{}x", speed);
            bar.spawn((WidgetButton, Node { padding: UiRect::horizontal(Val::Px(8.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                ReplaySpeedBtn(speed), ButtonTheme::dark(), Hovered::default(),
                BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                .with_child((Text::new(label), TextFont { font: font.clone().into(), font_size: FontSize::Px(14.0), ..default() }))
                .observe(move |_ev: On<Activate>, mut ctrl: Option<ResMut<ReplayController>>| {
                    if let Some(ref mut c) = ctrl { c.speed_multiplier = speed; }
                });
        }

        // Progress bar: click/drag handled by progress_bar_seek_system
        bar.spawn((Node {
            width: Val::Px(300.0), height: Val::Px(14.0),
            border: UiRect::all(Val::Px(1.0)), ..default()
        }, BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0)),
          BorderColor::all(Color::srgba(0.4, 0.4, 0.4, 1.0)),
          ReplayProgressBg, Pickable::default()))
            .with_children(|bg| {
                bg.spawn((Node {
                    width: Val::Percent(0.0), height: Val::Percent(100.0),
                    ..default()
                }, BackgroundColor(Color::srgba(0.3, 0.6, 0.9, 1.0)),
                  ReplayProgressFill));
            });

        // Tick counter
        bar.spawn((Text::new("T 0 / 0"), TextFont { font: font.clone().into(), font_size: FontSize::Px(13.0), ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)), ReplayTickText));
    });
}

/// Progress bar click/drag to seek.
/// The bar is 300px wide, centered in the bottom control bar.
/// We compute its screen position from window width.
pub fn progress_bar_seek_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut drag_state: Local<bool>,
    mut ctrl: Option<ResMut<ReplayController>>,
) {
    let Some(ref mut ctrl) = ctrl else { return };
    let total = ctrl.replay.total_ticks;
    if total == 0 { return; }

    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };

    // Bar is 300px wide, centered horizontally, at bottom of screen (44px bar)
    let bar_w = 300.0f32;
    let win_w = window.width();
    let win_h = window.height();
    let bar_left = (win_w - bar_w) / 2.0;
    let bar_right = bar_left + bar_w;
    let bar_top = win_h - 44.0; // control bar is 44px at bottom
    let bar_bottom = win_h;

    let inside = cursor.x >= bar_left && cursor.x <= bar_right
              && cursor.y >= bar_top && cursor.y <= bar_bottom;

    if mouse.just_pressed(MouseButton::Left) && inside {
        *drag_state = true;
    }
    if mouse.just_released(MouseButton::Left) {
        *drag_state = false;
    }

    if *drag_state && bar_w > 0.0 {
        let pct = ((cursor.x - bar_left) / bar_w).clamp(0.0, 1.0);
        let target_tick = (pct * total as f32) as u32;
        ctrl.seek_target = Some(target_tick.min(total));
    }
}

/// Update replay player UI each frame.
pub fn update_replay_player(
    game_mode: Res<GameMode>,
    ctrl: Option<Res<ReplayController>>,
    mut tick_text: Query<&mut Text, With<ReplayTickText>>,
    mut pause_text: Query<&mut Text, (With<ReplayPauseBtnText>, Without<ReplayTickText>)>,
    mut progress_fill: Query<&mut Node, With<ReplayProgressFill>>,
) {
    let GameMode::Replay = *game_mode else { return };
    let Some(ref ctrl) = ctrl else { return };

    let total = ctrl.replay.total_ticks.max(1);
    let current = ctrl.current_tick;

    for mut text in tick_text.iter_mut() {
        **text = format!("T {} / {}", current, total);
    }

    let icon = if ctrl.is_paused { "▶" } else { "⏸" };
    for mut text in pause_text.iter_mut() {
        **text = icon.to_string();
    }

    let pct = (current as f32 / total as f32 * 100.0).min(100.0);
    for mut node in progress_fill.iter_mut() {
        node.width = Val::Percent(pct);
    }
}

/// Cleanup replay player UI.
pub fn cleanup_replay_player(
    mut commands: Commands,
    query: Query<Entity, With<ReplayPlayerUI>>,
) {
    for e in query.iter() { commands.entity(e).despawn(); }
}

/// Condition: only show replay player when in Replay mode.
pub fn in_replay_mode(game_mode: Res<GameMode>) -> bool {
    *game_mode == GameMode::Replay
}
