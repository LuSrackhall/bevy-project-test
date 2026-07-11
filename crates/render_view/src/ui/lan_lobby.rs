use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button as WidgetButton};

use crate::ui::hud::ButtonTheme;

#[derive(Component)]
pub struct LanLobbyUI;

#[derive(Component)]
pub struct LanLobbyLayout;

pub fn setup_lan_lobby(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            LanLobbyUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("局域网大厅"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
            ));

            // Placeholder room list area — will be replaced by #7
            parent.spawn((
                Text::new("正在扫描局域网房间..."),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                Node {
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
                LanLobbyLayout,
            ));

            // Return button
            parent.spawn((
                WidgetButton,
                Node {
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
                ButtonTheme::dark(),
                BorderColor::all(Color::srgba(0.5, 0.5, 0.6, 1.0)),
                LanLobbyLayout,
            ))
            .with_child((
                Text::new("返回主菜单"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
            ))
            .observe(|_ev: On<Activate>, mut next: ResMut<NextState<crate::GameState>>| {
                next.set(crate::GameState::MainMenu);
            });
        });
}

pub fn cleanup_lan_lobby(mut commands: Commands, q: Query<Entity, With<LanLobbyUI>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}
