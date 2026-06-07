pub mod menu;
pub mod hud;
pub mod pause;
pub mod gameover;

use bevy::prelude::*;
use menu::MainMenuPlugin;
use hud::HudPlugin;
use pause::PauseMenuPlugin;
use gameover::GameOverPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MainMenuPlugin)
           .add_plugins(HudPlugin)
           .add_plugins(PauseMenuPlugin)
           .add_plugins(GameOverPlugin);
    }
}
