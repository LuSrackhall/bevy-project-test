use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy_adapter::tick::SimulationWorld;
use simulation::soldier::*;
use simulation::types::Faction;

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

/// Center camera on the first player city after map generation.
pub fn center_on_player_city(
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
    mut cam_query: Query<&mut Transform, With<MainCamera>>,
    mut centered: Local<bool>,
) {
    if *centered { return; }
    let world = &mut sim_world.0;
    let mut query = world.query::<(&LogicalPosition, &FactionComponent)>();
    for (pos, faction) in query.iter(world) {
        if faction.0 == Faction::Player {
            if let Some(mut cam) = cam_query.iter_mut().next() {
                cam.translation.x = pos.0.x.to_float();
                cam.translation.y = pos.0.y.to_float();
                *centered = true;
            }
            break;
        }
    }
}

/// Get the current ortho scale from the camera.
fn get_ortho_scale(q: &Query<&Projection, With<MainCamera>>) -> f32 {
    q.iter().next().and_then(|proj| {
        if let Projection::Orthographic(ref ortho) = proj { Some(ortho.scale) } else { None }
    }).unwrap_or(1.0)
}

/// Middle-mouse drag with speed scaling by zoom level.
/// Clamps to wall boundaries after movement.
pub fn camera_drag_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut cam_query: Query<&mut Transform, With<MainCamera>>,
    proj_query: Query<&Projection, With<MainCamera>>,
    mut last_pos: Local<Option<Vec2>>,
    q_windows: Query<&Window>,
    map_bounds: Option<Res<bevy_adapter::MapBounds>>,
) {
    let Ok(window) = q_windows.single() else { return };
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

    // Clamp to wall boundaries
    if let Some(bounds) = map_bounds.as_ref() {
        for mut transform in cam_query.iter_mut() {
            transform.translation.x = transform.translation.x.clamp(bounds.wall_min_x, bounds.wall_max_x);
            transform.translation.y = transform.translation.y.clamp(bounds.wall_min_y, bounds.wall_max_y);
        }
    }
}

/// Dynamic zoom range. Max = 6x map size. Min = 0.15.
pub fn camera_zoom_system(
    mouse_wheel: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut Projection, With<MainCamera>>,
    q_windows: Query<&Window>,
    map_bounds: Option<Res<bevy_adapter::MapBounds>>,
) {
    for mut proj in query.iter_mut() {
        if let Projection::Orthographic(ref mut ortho) = *proj {
            ortho.scale *= 1.0 - mouse_wheel.delta.y * 0.02;

            if let (Some(bounds), Ok(window)) = (map_bounds.as_ref(), q_windows.single()) {
                // Max zoom: 6x map size
                let map_dim = bounds.width.max(bounds.height);
                let max_scale = (map_dim * 6.0) / window.width().min(window.height());
                ortho.scale = ortho.scale.clamp(0.15, max_scale);
            } else {
                ortho.scale = ortho.scale.max(0.15);
            }
        }
    }
}
