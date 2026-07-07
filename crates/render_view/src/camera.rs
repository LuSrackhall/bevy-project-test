use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_adapter::tick::SimulationWorld;
use simulation::soldier::*;
use simulation::types::FactionId;

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

/// Center camera on the first player city after map generation.
/// Uses the LocalPlayerId stored in the simulation world (defaults to Player for single-player).
pub fn center_on_player_city(
    sim_world: bevy::ecs::system::NonSend<SimulationWorld>,
    mut cam_query: Query<&mut Transform, With<MainCamera>>,
    mut centered: Local<bool>,
) {
    if *centered {
        return;
    }
    let world = sim_world.world_ref();
    let local_id = crate::local_player_id(&sim_world);
    let target_faction = simulation::types::FactionId(local_id);
    let mut q = sim_world.query::<(&LogicalPosition, &FactionComponent)>();
    for (pos, faction) in q.iter(world) {
        if faction.0 == target_faction {
            if let Some(mut cam) = cam_query.iter_mut().next() {
                cam.translation.x = pos.0.x.to_float();
                cam.translation.y = pos.0.y.to_float();
                *centered = true;
            }
            break;
        }
    }
}

fn get_ortho_scale(q: &Query<&Projection, With<MainCamera>>) -> f32 {
    q.iter()
        .next()
        .and_then(|proj| {
            if let Projection::Orthographic(ref ortho) = proj {
                Some(ortho.scale)
            } else {
                None
            }
        })
        .unwrap_or(1.0)
}

/// Compute the camera viewport AABB in world space.
/// Returns (min_x, min_y, max_x, max_y).
pub(crate) fn viewport_aabb(
    cam_transform: &GlobalTransform,
    window: &Window,
    scale: f32,
) -> (f32, f32, f32, f32) {
    let half_w = window.width() * 0.5 * scale;
    let half_h = window.height() * 0.5 * scale;
    let pos = cam_transform.translation();
    let cx = pos.x;
    let cy = pos.y;
    (cx - half_w, cy - half_h, cx + half_w, cy + half_h)
}

/// Middle-mouse drag with speed scaling by zoom level.
pub fn camera_drag_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut cam_query: Query<&mut Transform, With<MainCamera>>,
    proj_query: Query<&Projection, With<MainCamera>>,
    mut last_pos: Local<Option<Vec2>>,
    q_windows: Query<&Window>,
    map_bounds: Option<Res<bevy_adapter::MapBounds>>,
) {
    let Ok(window) = q_windows.single() else {
        return;
    };
    let cursor = window.cursor_position();
    let scale = get_ortho_scale(&proj_query);

    if mouse.pressed(MouseButton::Middle) {
        if let Some(cursor) = cursor {
            if let Some(last) = *last_pos {
                let delta = cursor - last;
                for mut transform in cam_query.iter_mut() {
                    transform.translation.x -= delta.x * scale;
                    transform.translation.y += delta.y * scale;
                }
            }
            *last_pos = Some(cursor);
        }
    } else {
        *last_pos = cursor;
    }

    if let Some(bounds) = map_bounds.as_ref() {
        for mut transform in cam_query.iter_mut() {
            transform.translation.x = transform
                .translation
                .x
                .clamp(bounds.wall_min_x, bounds.wall_max_x);
            transform.translation.y = transform
                .translation
                .y
                .clamp(bounds.wall_min_y, bounds.wall_max_y);
        }
    }
}

/// Edge scrolling: mouse near screen edges auto-scrolls camera.
pub fn camera_edge_scroll_system(
    mut cam_query: Query<&mut Transform, With<MainCamera>>,
    proj_query: Query<&Projection, With<MainCamera>>,
    q_windows: Query<&Window>,
    map_bounds: Option<Res<bevy_adapter::MapBounds>>,
    time: Res<Time>,
) {
    let Ok(window) = q_windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let scale = get_ortho_scale(&proj_query);

    let edge_zone = 30.0;
    let base_speed = 800.0;
    let speed = base_speed * scale * time.delta_secs();

    let mut dx = 0.0f32;
    let mut dy = 0.0f32;

    if cursor.x < edge_zone {
        dx -= speed;
    }
    if cursor.x > window.width() - edge_zone {
        dx += speed;
    }
    if cursor.y < edge_zone {
        dy += speed;
    }
    if cursor.y > window.height() - edge_zone {
        dy -= speed;
    }

    if dx == 0.0 && dy == 0.0 {
        return;
    }

    for mut transform in cam_query.iter_mut() {
        transform.translation.x += dx;
        transform.translation.y += dy;
    }

    if let Some(bounds) = map_bounds.as_ref() {
        for mut transform in cam_query.iter_mut() {
            transform.translation.x = transform
                .translation
                .x
                .clamp(bounds.wall_min_x, bounds.wall_max_x);
            transform.translation.y = transform
                .translation
                .y
                .clamp(bounds.wall_min_y, bounds.wall_max_y);
        }
    }
}

/// Zoom toward cursor position. Max = 6x map size. Min = 0.15.
pub fn camera_zoom_system(
    mut scroll_events: MessageReader<MouseWheel>,
    mut cam_query: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
    q_windows: Query<&Window>,
    map_bounds: Option<Res<bevy_adapter::MapBounds>>,
) {
    let Ok(window) = q_windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    // Accumulate scroll delta, handling Line (mouse wheel) vs Pixel (trackpad)
    let mut delta = 0.0_f32;
    for ev in scroll_events.read() {
        delta += match ev.unit {
            MouseScrollUnit::Line => ev.y * 0.1,
            MouseScrollUnit::Pixel => ev.y * 0.003,
        };
    }
    if delta.abs() < f32::EPSILON {
        return;
    }

    for (mut transform, mut proj) in cam_query.iter_mut() {
        let Projection::Orthographic(ref mut ortho) = *proj else {
            continue;
        };

        let old_scale = ortho.scale;
        ortho.scale *= 1.0 - delta;

        // Clamp scale
        if let Some(bounds) = map_bounds.as_ref() {
            let map_dim = bounds.width.max(bounds.height);
            let max_scale = (map_dim * 6.0) / window.width().min(window.height());
            ortho.scale = ortho.scale.clamp(0.15, max_scale);
        } else {
            ortho.scale = ortho.scale.max(0.15);
        }

        // Zoom toward cursor: keep the world point under cursor fixed
        let new_scale = ortho.scale;
        if (new_scale - old_scale).abs() > f32::EPSILON {
            // Cursor position in window coords (origin = center)
            let cx = cursor.x - window.width() / 2.0;
            let cy = -(cursor.y - window.height() / 2.0);
            // Adjust camera so the world point under cursor stays fixed
            let factor = 1.0 - new_scale / old_scale;
            transform.translation.x += cx * factor * old_scale;
            transform.translation.y += cy * factor * old_scale;
        }

        // Clamp to wall boundaries
        if let Some(bounds) = map_bounds.as_ref() {
            transform.translation.x = transform
                .translation
                .x
                .clamp(bounds.wall_min_x, bounds.wall_max_x);
            transform.translation.y = transform
                .translation
                .y
                .clamp(bounds.wall_min_y, bounds.wall_max_y);
        }
    }
}
