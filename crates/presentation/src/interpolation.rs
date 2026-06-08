use bevy::prelude::*;
use bevy_adapter::binding::{PresentationPosition, InterpolationData};

/// Global interpolation factor [0, 1) — how far we are from the last tick toward the next.
#[derive(Resource, Default)]
pub struct RenderInterpolationAlpha(pub f32);

/// Compute alpha from TickClock accumulator.
pub fn compute_alpha_system(
    tick_clock: Res<bevy_adapter::tick::TickClock>,
    mut alpha: ResMut<RenderInterpolationAlpha>,
) {
    alpha.0 = (tick_clock.accumulator / tick_clock.tick_duration).clamp(0.0, 1.0);
}

/// Each frame: interpolate positions and write PresentationPosition.
pub fn interpolate_positions_system(
    alpha: Res<RenderInterpolationAlpha>,
    mut render_query: Query<(&mut PresentationPosition, &InterpolationData)>,
) {
    let t = alpha.0;
    for (mut pres_pos, interp) in render_query.iter_mut() {
        if interp.is_new {
            pres_pos.0 = interp.current_logical_pos;
        } else {
            pres_pos.0 = interp.previous_logical_pos.lerp(interp.current_logical_pos, t);
        }
    }
}
