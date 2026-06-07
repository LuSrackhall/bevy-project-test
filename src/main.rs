mod core;
mod map;
mod city;
mod soldier;
mod combat;
mod camera;
mod input;
mod ui;
mod ai;
mod game;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_prototype_lyon::prelude::*;

use core::*;
use map::MapPlugin;
use city::CityPlugin;
use soldier::SoldierPlugin;
use combat::CombatPlugin;
use camera::CameraPlugin;
use input::InputPlugin;
use ui::UiPlugin;
use ai::AiPlugin;
use game::GamePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "城池争霸".to_string(),
                    resolution: WindowResolution::new(1280, 720),
                    ..default()
                }),
                ..default()
            }),
            ShapePlugin,
        ))
        .init_state::<GameState>()
        .init_resource::<GameConfig>()
        .add_plugins(MapPlugin)
        .add_plugins(CityPlugin)
        .add_plugins(SoldierPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(AiPlugin)
        .add_plugins(GamePlugin)
        .run();
}
