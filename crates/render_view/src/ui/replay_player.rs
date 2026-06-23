//! Replay player UI — control bar shown during replay playback.

use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button as WidgetButton};
use bevy::picking::hover::Hovered;
use crate::ui::hud::ButtonTheme;
use bevy_adapter::replay::{GameMode, ReplayController, ReplayStatus};
use bevy_adapter::tick::{SimulationWorld, TickClock, PendingEvents};

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

/// Ticks per second at 20Hz
const TICKS_PER_SEC: u32 = 20;
/// Skip interval: 10 seconds
const SKIP_TICKS: u32 = TICKS_PER_SEC * 10;
/// Ticks per frame during async seek (fast but doesn't freeze UI)

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
        column_gap: Val::Px(6.0),
        padding: UiRect::horizontal(Val::Px(12.0)),
        ..default()
    }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)), ReplayPlayerUI))
    .with_children(|bar| {
        // Skip backward 10s (async multi-frame seek)
        bar.spawn((WidgetButton, Node { padding: UiRect::horizontal(Val::Px(6.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
            ButtonTheme::dark(), Hovered::default(),
            BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
            .with_child((Text::new("<< 10s"), TextFont { font: font.clone().into(), font_size: FontSize::Px(12.0), ..default() }))
            .observe(|_ev: On<Activate>, mut ctrl: Option<ResMut<ReplayController>>| {
                if let Some(ref mut c) = ctrl {
                    let target = c.current_tick.saturating_sub(SKIP_TICKS).max(1);
                    if target < c.current_tick {
                        // Backward seek: reinit world, enter async seek mode
                        c.seek_target = Some(target);
                        c.async_seek = true;
                        c.is_paused = false;
                    }
                }
            });

        // Pause/Play button
        bar.spawn((WidgetButton, Node { padding: UiRect::horizontal(Val::Px(8.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
            ButtonTheme::dark(), Hovered::default(),
            BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
            .with_children(|btn| {
                btn.spawn((Text::new("||"), TextFont { font: font.clone().into(), font_size: FontSize::Px(18.0), ..default() },
                    ReplayPauseBtnText));
            })
            .observe(|_ev: On<Activate>, mut ctrl: Option<ResMut<ReplayController>>| {
                if let Some(ref mut c) = ctrl { c.is_paused = !c.is_paused; }
            });

        // Skip forward 10s
        bar.spawn((WidgetButton, Node { padding: UiRect::horizontal(Val::Px(6.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
            ButtonTheme::dark(), Hovered::default(),
            BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
            .with_child((Text::new("10s >>"), TextFont { font: font.clone().into(), font_size: FontSize::Px(12.0), ..default() }))
            .observe(|_ev: On<Activate>, mut ctrl: Option<ResMut<ReplayController>>| {
                if let Some(ref mut c) = ctrl {
                    let target = (c.current_tick + SKIP_TICKS).min(c.replay.total_ticks);
                    c.seek_target = Some(target);
                    c.async_seek = true;
                    c.is_paused = false;
                }
            });

        // Speed buttons: 1x, 2x, 4x, 8x, 16x
        for speed in [1u32, 2, 4, 8, 16] {
            let label = format!("{}x", speed);
            bar.spawn((WidgetButton, Node { padding: UiRect::horizontal(Val::Px(6.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                ReplaySpeedBtn(speed), ButtonTheme::dark(), Hovered::default(),
                BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                .with_child((Text::new(label), TextFont { font: font.clone().into(), font_size: FontSize::Px(13.0), ..default() }))
                .observe(move |_ev: On<Activate>, mut ctrl: Option<ResMut<ReplayController>>| {
                    if let Some(ref mut c) = ctrl { c.speed_multiplier = speed; }
                });
        }

        // Progress bar (visual only)
        bar.spawn((Node {
            width: Val::Px(250.0), height: Val::Px(8.0),
            border: UiRect::all(Val::Px(1.0)), ..default()
        }, BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0)),
          BorderColor::all(Color::srgba(0.4, 0.4, 0.4, 1.0))))
            .with_children(|bg| {
                bg.spawn((Node {
                    width: Val::Percent(0.0), height: Val::Percent(100.0),
                    ..default()
                }, BackgroundColor(Color::srgba(0.3, 0.6, 0.9, 1.0)),
                  ReplayProgressFill));
            });

        // Time display
        bar.spawn((Text::new("0:00 / 0:00"), TextFont { font: font.clone().into(), font_size: FontSize::Px(12.0), ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)), ReplayTickText));
    });
}

/// Async seek system: handles backward/forward seek across multiple frames.
/// When async_seek is true, processes SEEK_TICKS_PER_FRAME ticks per frame
/// until reaching seek_target, then resumes normal playback.
pub fn replay_seek_system(
    mut ctrl: Option<ResMut<ReplayController>>,
    mut status: ResMut<ReplayStatus>,
    mut sim_world: NonSendMut<SimulationWorld>,
    mut tick_clock: ResMut<TickClock>,
    mut pending: ResMut<PendingEvents>,
) {
    let Some(ref mut ctrl) = ctrl else { return };
    if !ctrl.async_seek {
        status.is_seeking = false;
        return;
    }
    let Some(target) = ctrl.seek_target else {
        ctrl.async_seek = false;
        status.is_seeking = false;
        return;
    };
    status.is_seeking = true;

    // If target is behind current position, reinitialize world
    if target < ctrl.current_tick {
        let seed = ctrl.replay.seed;
        let map_size = ctrl.replay.map_size;
        let mut world = simulation::init_simulation_world(seed);
        simulation::map::generate_map(&mut world, map_size);
        sim_world.0 = world;
        ctrl.current_tick = 0;
        tick_clock.current_tick = 0;
        pending.events.clear();
    }

    // Fixed batch: process up to 5000 ticks per frame
    let end = (ctrl.current_tick + 5000).min(target);
    while ctrl.current_tick < end {
        ctrl.current_tick += 1;
        let cmds = ctrl.replay.commands_for_tick(ctrl.current_tick).to_vec();
        {
            let mut sim_cmds = sim_world.0.resource_mut::<simulation::command::CommandBuffer>();
            for cmd in cmds { sim_cmds.0.push(cmd); }
        }
        simulation::run_tick(&mut sim_world.0, ctrl.current_tick);
    }

    tick_clock.current_tick = ctrl.current_tick;

    // Check if seek is complete
    if ctrl.current_tick >= target {
        ctrl.seek_target = None;
        ctrl.async_seek = false;
        status.is_seeking = false;
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

    let cur_sec = current / TICKS_PER_SEC;
    let tot_sec = total / TICKS_PER_SEC;
    for mut text in tick_text.iter_mut() {
        **text = format!("{}:{:02} / {}:{:02}", cur_sec / 60, cur_sec % 60, tot_sec / 60, tot_sec % 60);
    }

    let icon = if ctrl.is_paused { ">" } else { "||" };
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
