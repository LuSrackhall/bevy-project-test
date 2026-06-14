use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button as WidgetButton};

#[derive(Component)]
pub struct MainMenuUI;

pub fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, ..default() },
        MainMenuUI,
    ))
    .with_children(|parent| {
        parent.spawn((Text::new("城池争霸"), TextFont { font: font.clone(), font_size: 48.0, ..default() }));
        parent.spawn((WidgetButton, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), ..default() }, MenuButton::SinglePlayer))
            .with_child((Text::new("单人模式"), TextFont { font: font.clone(), font_size: 24.0, ..default() }))
            .observe(|_ev: On<Activate>, mut next: ResMut<NextState<crate::GameState>>| {
                next.set(crate::GameState::Playing);
            });
    });
}

pub fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for e in query.iter() { commands.entity(e).despawn(); }
}

#[derive(Component)]
pub enum MenuButton { SinglePlayer }
