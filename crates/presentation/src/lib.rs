pub mod binding;
pub mod interpolation;

use bevy::prelude::*;

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::interpolation::RenderInterpolationAlpha>()
            .add_systems(
                Update,
                (
                    crate::interpolation::compute_alpha_system,
                    crate::interpolation::interpolate_positions_system,
                ),
            );
    }
}
