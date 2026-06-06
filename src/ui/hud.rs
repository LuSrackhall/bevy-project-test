use bevy::prelude::*;
use bevy::text::TextFont;

use crate::core::{GameState, SoldierType};
use crate::city::{City, CitySelectedEvent};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(|_trigger: On<CitySelectedEvent>, mut panel_query: Query<&mut Node, With<BottomPanel>>| {
            for mut node in panel_query.iter_mut() {
                node.display = Display::Flex;
            }
        });
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
           .add_systems(OnExit(GameState::Playing), cleanup_hud)
           .add_systems(Update, soldier_type_button_system.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
pub struct TopBar;

#[derive(Component)]
pub struct BottomPanel;

#[derive(Component)]
struct CityInfoText;

#[derive(Component)]
pub struct SoldierTypeButton(pub SoldierType);

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("fonts/Arial Unicode.ttf");
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        HudRoot,
    ))
    .with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            TopBar,
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("🏰 0/0"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
            parent.spawn((Text::new("👥 0"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
            parent.spawn((Text::new("⏱️ 0:00"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
            parent.spawn((Button, Node { padding: UiRect::all(Val::Px(5.0)), ..default() })).with_child((Text::new("⏸️"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
        });

        parent.spawn(Node { flex_grow: 1.0, ..default() });

        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(160.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            BottomPanel,
        ))
        .with_children(|parent| {
            parent.spawn(((Text::new("城池 Lv.?"), CityInfoText), TextFont { font: font_handle.clone(), font_size: 18.0, ..default() }));
            parent.spawn(Node { flex_direction: FlexDirection::Row, ..default() })
            .with_children(|parent| {
                for (st, label) in [(SoldierType::Militia, "民兵"), (SoldierType::Infantry, "步兵"), (SoldierType::Archer, "弓兵"), (SoldierType::Cavalry, "骑兵")] {
                    parent.spawn((Button, Node { padding: UiRect::all(Val::Px(10.0)), margin: UiRect::all(Val::Px(5.0)), ..default() }, SoldierTypeButton(st))).with_child((Text::new(label), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
                }
            });
        });

        parent.spawn((
            Node { width: Val::Percent(100.0), height: Val::Px(50.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            parent.spawn(Button::default()).with_child((Text::new("⭕框选"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
            parent.spawn(Button::default()).with_child((Text::new("⬜框选"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
            parent.spawn(Button::default()).with_child((Text::new("🛡️举盾"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
        });
    });
}

fn cleanup_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in query.iter() { commands.entity(entity).despawn(); }
}

fn soldier_type_button_system(
    mut _interaction_query: Query<(&SoldierTypeButton, &Interaction), Changed<Interaction>>,
    mut _city_query: Query<&mut City>,
) {
    // 兵种切换 — 后续完善
}
