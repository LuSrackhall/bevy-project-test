use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button as WidgetButton};

#[derive(Component)]
pub struct PauseUI;

pub fn setup_pause(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center,
        align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), PauseUI,
    ))
    .with_children(|parent| {
        parent.spawn((Text::new("游戏暂停"), TextFont { font: font.clone(), font_size: 36.0, ..default() }));
        for (label, btn) in [("继续", PauseBtn::Resume), ("重新开始", PauseBtn::Restart), ("主菜单", PauseBtn::Menu)] {
            parent.spawn((WidgetButton, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), ..default() }, btn))
                .with_child((Text::new(label), TextFont { font: font.clone(), font_size: 24.0, ..default() }))
                .observe(move |_ev: On<Activate>, q: Query<&PauseBtn>, mut next: ResMut<NextState<crate::GameState>>| {
                    if let Ok(pause_btn) = q.get(_ev.entity) {
                        match pause_btn {
                            PauseBtn::Resume => next.set(crate::GameState::Playing),
                            PauseBtn::Restart => next.set(crate::GameState::Playing),
                            PauseBtn::Menu => next.set(crate::GameState::MainMenu),
                        }
                    }
                });
        }
    });
}

pub fn cleanup_pause(mut commands: Commands, query: Query<Entity, With<PauseUI>>) {
    for e in query.iter() { commands.entity(e).despawn(); }
}

#[derive(Component)]
pub(crate) enum PauseBtn { Resume, Restart, Menu }
