pub mod debug_shape;
pub mod camera;
pub mod selection;
pub mod ui;

use bevy::prelude::*;

pub struct RenderViewPlugin;

impl Plugin for RenderViewPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<crate::selection::SelectionState>()
            .add_systems(Startup, crate::camera::setup_camera)
            .add_systems(Update, (
                crate::debug_shape::draw_debug_shapes_system,
                crate::camera::camera_drag_system,
                crate::camera::camera_zoom_system,
                crate::camera::center_on_player_city,
            ));
    }
}
