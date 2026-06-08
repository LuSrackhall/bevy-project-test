use bevy::prelude::*;
use crate::camera::MainCamera;
use simulation::types::*;

// Basic HUD placeholder — full UI migration from src/ui/* in later tasks.

#[derive(Component)]
pub struct HudRoot;

pub fn setup_basic_hud(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        HudRoot,
    ));
}
