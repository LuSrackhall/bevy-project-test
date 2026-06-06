use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseScroll;

use crate::core::GameState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
           .add_systems(Update, camera_drag_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, camera_zoom_system.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn camera_drag_system(
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

fn camera_zoom_system(
    mouse_wheel: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut Projection, With<MainCamera>>,
) {
    for mut proj in query.iter_mut() {
        if let Projection::Orthographic(ref mut ortho) = *proj {
            ortho.scale = (ortho.scale - mouse_wheel.delta.y * 0.01).clamp(0.2, 3.0);
        }
    }
}
