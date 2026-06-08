use bevy::prelude::*;
use simulation::command::*;
use simulation::types::*;

/// One-shot toggle for force-move on next command.
#[derive(Resource, Default)]
pub struct ForceMoveNext {
    pub active: bool,
}

/// Translate Bevy input events into GameCommands.
/// This system runs every frame and pushes commands for the next tick.
pub fn input_translation_system(
    _mouse: Res<ButtonInput<MouseButton>>,
    _keyboard: Res<ButtonInput<KeyCode>>,
    _q_windows: Query<&Window>,
    _camera_query: Query<(&Camera, &GlobalTransform)>,
    mut _cmd_buf: ResMut<CommandBuffer>,
    _tick_clock: Res<crate::tick::TickClock>,
) {
    // The actual input translation will be implemented when render_view
    // exposes SelectionState. For now, this is a placeholder.
    //
    // Logic:
    // 1. Read mouse/keyboard events
    // 2. Convert screen coords to world coords
    // 3. Check hits (enemy soldiers, cities, ground)
    // 4. For each selected unit, generate appropriate Action
    // 5. Push commands to CommandBuffer with tick = current_tick + 1
}
