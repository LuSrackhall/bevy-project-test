use crate::ui::hud::ButtonTheme;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button as WidgetButton};
use simulation::map::MapSize;

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component, Clone, Copy)]
pub enum MapSizeBtn {
    Small,
    Medium,
    Large,
    Huge,
}

impl MapSizeBtn {
    fn label(&self) -> &str {
        match self {
            MapSizeBtn::Small => "小 (2000)",
            MapSizeBtn::Medium => "中 (3500)",
            MapSizeBtn::Large => "大 (5000)",
            MapSizeBtn::Huge => "巨大 (8000)",
        }
    }

    fn map_size(&self) -> MapSize {
        match self {
            MapSizeBtn::Small => MapSize::Small,
            MapSizeBtn::Medium => MapSize::Medium,
            MapSizeBtn::Large => MapSize::Large,
            MapSizeBtn::Huge => MapSize::Huge,
        }
    }
}

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
        parent.spawn((Text::new("城池争霸"), TextFont { font: font.clone().into(), font_size: FontSize::Px(48.0), ..default() }));
        parent.spawn((Text::new("选择地图大小"), TextFont { font: font.clone().into(), font_size: FontSize::Px(20.0), ..default() },
            Node { margin: UiRect::top(Val::Px(20.0)), ..default() }));
        // Map size buttons row
        parent.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0),
            margin: UiRect::top(Val::Px(10.0)), ..default() })
            .with_children(|row| {
                for btn in [MapSizeBtn::Small, MapSizeBtn::Medium, MapSizeBtn::Large, MapSizeBtn::Huge] {
                    row.spawn((WidgetButton, Node { padding: UiRect::all(Val::Px(15.0)), border: UiRect::all(Val::Px(2.0)), ..default() },
                        btn, ButtonTheme::default(), Hovered::default(),
                        BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                        .with_child((Text::new(btn.label()), TextFont { font: font.clone().into(), font_size: FontSize::Px(18.0), ..default() }))
                        .observe(|_ev: On<Activate>, q: Query<&MapSizeBtn>, mut next: ResMut<NextState<crate::GameState>>, mut needs_reset: ResMut<crate::NeedsGameReset>| {
                            if let Ok(btn) = q.get(_ev.entity) {
                                *needs_reset = crate::NeedsGameReset::NewGame(btn.map_size());
                                next.set(crate::GameState::Playing);
                            }
                        });
                }
            });
    });
}

pub fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
