//! Replay player UI — control bar shown during replay playback.

use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button as WidgetButton};
use bevy::picking::hover::Hovered;
use crate::ui::hud::ButtonTheme;
use bevy_adapter::replay::{GameMode, ReplayController};

#[derive(Component)]
pub struct ReplayPlayerUI;

#[derive(Component)]
pub struct ReplayTickText;

#[derive(Component)]
pub struct ReplaySpeedBtn(u32);

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
        // Pause/Play button — text is on the same entity as the button
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

        // Progress bar: background + fill child
        bar.spawn((Node {
            width: Val::Px(300.0), height: Val::Px(10.0),
            border: UiRect::all(Val::Px(1.0)), ..default()
        }, BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0)),
          BorderColor::all(Color::srgba(0.4, 0.4, 0.4, 1.0)),
          WidgetButton, ReplayProgressBg, Pickable::default(), Hovered::default()))
            .with_children(|bg| {
                // Fill — width updated each frame
                bg.spawn((Node {
                    width: Val::Percent(0.0), height: Val::Percent(100.0),
                    ..default()
                }, BackgroundColor(Color::srgba(0.3, 0.6, 0.9, 1.0)),
                  ReplayProgressFill));
            })
            .observe(|ev: On<Activate>, q: Query<&ReplayProgressBg>, node_q: Query<&Node>,
                      mut ctrl: Option<ResMut<ReplayController>>| {
                // Click on progress bar to seek
                if q.get(ev.entity).is_ok() {
                    if let Some(ref mut c) = ctrl {
                        let total = c.replay.total_ticks;
                        // Simple: seek to a proportional position based on click
                        // For now use a simple approach: seek forward by 10% of remaining
                        let remaining = total.saturating_sub(c.current_tick);
                        let jump = (remaining / 10).max(1);
                        c.seek_target = Some((c.current_tick + jump).min(total));
                    }
                }
            });

        // Tick counter
        bar.spawn((Text::new("T 0 / 0"), TextFont { font: font.clone().into(), font_size: FontSize::Px(13.0), ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)), ReplayTickText));
    });
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

    // Update tick text
    for mut text in tick_text.iter_mut() {
        **text = format!("T {} / {}", current, total);
    }

    // Update pause/play icon
    let icon = if ctrl.is_paused { "▶" } else { "⏸" };
    for mut text in pause_text.iter_mut() {
        **text = icon.to_string();
    }

    // Update progress bar fill
    let pct = (current as f32 / total as f32 * 100.0).min(100.0);
    for mut node in progress_fill.iter_mut() {
        node.width = Val::Percent(pct);
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
