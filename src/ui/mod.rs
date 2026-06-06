pub mod menu;
pub mod hud;
pub mod pause;

use bevy::prelude::*;
use menu::MainMenuPlugin;
use hud::HudPlugin;
use pause::PauseMenuPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MainMenuPlugin)
           .add_plugins(HudPlugin)
           .add_plugins(PauseMenuPlugin);
    }
}
