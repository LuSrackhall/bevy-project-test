use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseScroll;

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

pub fn camera_drag_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
    mut last_pos: Local<Option<Vec2>>,
    q_windows: Query<&Window>,
) {
    let Ok(window) = q_windows.single() else { return };
    let cursor = window.cursor_position();
    if mouse.pressed(MouseButton::Middle) || mouse.pressed(MouseButton::Right) {
        if let Some(cursor) = cursor {
            if let Some(last) = *last_pos {
                let delta = cursor - last;
                for mut transform in query.iter_mut() {
                    transform.translation.x -= delta.x;
                    transform.translation.y += delta.y;
                }
            }
            *last_pos = Some(cursor);
        }
    } else {
        *last_pos = cursor;
    }
}

pub fn camera_zoom_system(
    mouse_wheel: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut Projection, With<MainCamera>>,
) {
    for mut proj in query.iter_mut() {
        if let Projection::Orthographic(ref mut ortho) = *proj {
            ortho.scale = (ortho.scale - mouse_wheel.delta.y * 0.01).clamp(0.2, 3.0);
        }
    }
}
