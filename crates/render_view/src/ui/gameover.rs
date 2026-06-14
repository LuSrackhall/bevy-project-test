use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button as WidgetButton};

#[derive(Component)]
pub struct GameOverUI;

pub fn setup_gameover(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center,
        align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)), GameOverUI,
    ))
    .with_children(|parent| {
        parent.spawn((Text::new("游戏结束"), TextFont { font: font.clone(), font_size: 48.0, ..default() }));
        for (label, btn) in [("再来一局", EndBtn::Restart), ("主菜单", EndBtn::Menu)] {
            parent.spawn((WidgetButton, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), ..default() }, btn))
                .with_child((Text::new(label), TextFont { font: font.clone(), font_size: 24.0, ..default() }))
                .observe(move |_ev: On<Activate>, q: Query<&EndBtn>, mut next: ResMut<NextState<crate::GameState>>| {
                    if let Ok(end_btn) = q.get(_ev.entity) {
                        match end_btn {
                            EndBtn::Restart => next.set(crate::GameState::Playing),
                            EndBtn::Menu => next.set(crate::GameState::MainMenu),
                        }
                    }
                });
        }
    });
}

pub fn cleanup_gameover(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for e in query.iter() { commands.entity(e).despawn(); }
}

#[derive(Component)]
pub(crate) enum EndBtn { Restart, Menu }
