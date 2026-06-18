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

/// Middle-mouse drag with speed scaling by zoom level.
/// Clamps to map bounds after movement.
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

    // Get current scale
    let scale: f32 = proj_query.iter().next().map(|proj| {
        if let Projection::Orthographic(ref ortho) = proj { ortho.scale } else { 1.0 }
    }).unwrap_or(1.0);

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

    // Clamp to map bounds
    if let Some(bounds) = map_bounds.as_ref() {
        let padding_x = (bounds.width * 0.05).max(100.0);
        let padding_y = (bounds.height * 0.05).max(100.0);
        let min_x = -padding_x;
        let min_y = -padding_y;
        let max_x = bounds.width + padding_x;
        let max_y = bounds.height + padding_y;
        for mut transform in cam_query.iter_mut() {
            transform.translation.x = transform.translation.x.clamp(min_x, max_x);
            transform.translation.y = transform.translation.y.clamp(min_y, max_y);
        }
    }
}

/// Dynamic zoom range based on map size and window dimensions.
pub fn camera_zoom_system(
    mouse_wheel: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut Projection, With<MainCamera>>,
    q_windows: Query<&Window>,
    map_bounds: Option<Res<bevy_adapter::MapBounds>>,
) {
    for mut proj in query.iter_mut() {
        if let Projection::Orthographic(ref mut ortho) = *proj {
            ortho.scale -= mouse_wheel.delta.y * 0.01;

            // Dynamic zoom limits
            if let (Some(bounds), Ok(window)) = (map_bounds.as_ref(), q_windows.single()) {
                let max_scale = (bounds.width / window.width())
                    .max(bounds.height / window.height())
                    ;
                ortho.scale = ortho.scale.clamp(0.15, max_scale);
            } else {
                ortho.scale = ortho.scale.clamp(0.15, 3.0);
            }
        }
    }
}
