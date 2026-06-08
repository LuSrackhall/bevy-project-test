use bevy::prelude::*;
use bevy::text::TextFont;

use crate::core::*;
use crate::city::{City, CitySelectedEvent, CityDeselectedEvent};
use crate::input::SelectionState;
use crate::input::SelectionMode;
use crate::input::ForceMoveNext;
use crate::soldier::{InfantryShield, ShieldState};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudTextEntities>()
           .add_observer(|trigger: On<CitySelectedEvent>, mut hud_text: ResMut<HudTextEntities>, mut panel_query: Query<&mut Node, With<BottomPanel>>| {
               hud_text.selected_city = Some(trigger.entity);
               for mut node in panel_query.iter_mut() {
                   node.display = Display::Flex;
               }
           })
           .add_observer(|_trigger: On<CityDeselectedEvent>, mut hud_text: ResMut<HudTextEntities>, mut panel_query: Query<&mut Node, With<BottomPanel>>| {
               hud_text.selected_city = None;
               for mut node in panel_query.iter_mut() {
                   node.display = Display::None;
               }
           });
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
           .add_systems(OnExit(GameState::Playing), cleanup_hud)
           .add_systems(Update, update_top_bar_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, update_bottom_panel.run_if(in_state(GameState::Playing)))
           .add_systems(Update, pause_button_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, soldier_type_button_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, shield_toggle_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, selection_mode_toggle_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, force_move_button_system.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource, Default)]
struct HudTextEntities {
    cities_text: Option<Entity>,
    pop_text: Option<Entity>,
    time_text: Option<Entity>,
    city_info_text: Option<Entity>,
    hp_text: Option<Entity>,
    hp_bar_fill: Option<Entity>,
    pop_detail_text: Option<Entity>,
    exp_text: Option<Entity>,
    selected_city: Option<Entity>,
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
struct HpBarFill;

#[derive(Component)]
pub struct SoldierTypeButton(pub SoldierType);

#[derive(Component)]
struct ShieldButton;

#[derive(Component)]
struct ForceMoveButton;

#[derive(Component)]
struct PauseButton;

#[derive(Component)]
struct CircleSelectButton;

#[derive(Component)]
struct RectSelectButton;

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("fonts/Arial Unicode.ttf");
    let mut hud_text = HudTextEntities::default();

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
        // ---- Top Bar ----
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
            let cities_entity = parent.spawn((Text::new("城 0/0"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() })).id();
            let pop_entity = parent.spawn((Text::new("兵 0"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() })).id();
            let time_entity = parent.spawn((Text::new("T 0:00"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() })).id();
            parent.spawn((Button, PauseButton, Node { padding: UiRect::all(Val::Px(5.0)), ..default() }))
                .with_child((Text::new("[暂停]"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));

            hud_text.cities_text = Some(cities_entity);
            hud_text.pop_text = Some(pop_entity);
            hud_text.time_text = Some(time_entity);
        });

        // ---- Middle spacer ----
        parent.spawn(Node { flex_grow: 1.0, ..default() });

        // ---- Bottom Panel ----
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
            let info_entity = parent.spawn((
                Text::new("[城池] Lv. ?"),
                CityInfoText,
                TextFont { font: font_handle.clone(), font_size: 18.0, ..default() },
            )).id();
            hud_text.city_info_text = Some(info_entity);

            // HP text + bar
            let hp_text_entity = parent.spawn((Text::new("HP ?/?"), TextFont { font: font_handle.clone(), font_size: 14.0, ..default() })).id();
            hud_text.hp_text = Some(hp_text_entity);

            // HP bar container
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(14.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 1.0)),
            ))
            .with_children(|parent| {
                let fill_entity = parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.8, 0.2, 1.0)),
                    HpBarFill,
                )).id();
                hud_text.hp_bar_fill = Some(fill_entity);
            });

            // Population text (updated live)
            let pop_detail_entity = parent.spawn((Text::new("兵 ?/?"), TextFont { font: font_handle.clone(), font_size: 14.0, ..default() })).id();
            hud_text.pop_detail_text = Some(pop_detail_entity);

            // EXP text
            let exp_text_entity = parent.spawn((Text::new("经验 ?/?"), TextFont { font: font_handle.clone(), font_size: 14.0, ..default() })).id();
            hud_text.exp_text = Some(exp_text_entity);

            // Soldier type buttons
            parent.spawn(Node { flex_direction: FlexDirection::Row, ..default() })
            .with_children(|parent| {
                for (st, label) in [
                    (SoldierType::Militia, "民兵"),
                    (SoldierType::Infantry, "步兵"),
                    (SoldierType::Archer, "弓兵"),
                    (SoldierType::Cavalry, "骑兵"),
                ] {
                    parent.spawn((
                        Button,
                        Node { padding: UiRect::all(Val::Px(10.0)), margin: UiRect::all(Val::Px(5.0)), ..default() },
                        SoldierTypeButton(st),
                    )).with_child((Text::new(label), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
                }
            });
        });

        // ---- Bottom Toolbar ----
        parent.spawn((
            Node { width: Val::Percent(100.0), height: Val::Px(50.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            parent.spawn((Button, CircleSelectButton, Node { padding: UiRect::all(Val::Px(8.0)), margin: UiRect::all(Val::Px(4.0)), ..default() }))
                .with_child((Text::new("O框选"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
            parent.spawn((Button, RectSelectButton, Node { padding: UiRect::all(Val::Px(8.0)), margin: UiRect::all(Val::Px(4.0)), ..default() }))
                .with_child((Text::new("[ ]框选"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
            parent.spawn((Button, ShieldButton, Node { padding: UiRect::all(Val::Px(8.0)), margin: UiRect::all(Val::Px(4.0)), ..default() }))
                .with_child((Text::new("盾"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
            parent.spawn((Button, ForceMoveButton, Node { padding: UiRect::all(Val::Px(8.0)), margin: UiRect::all(Val::Px(4.0)), ..default() }))
                .with_child((Text::new("[>]优先"), TextFont { font: font_handle.clone(), font_size: 16.0, ..default() }));
        });
    });

    commands.insert_resource(hud_text);
}

fn cleanup_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in query.iter() { commands.entity(entity).despawn(); }
}

fn update_top_bar_system(
    city_query: Query<&City>,
    mut hud_text: ResMut<HudTextEntities>,
    mut text_query: Query<&mut Text>,
    time: Res<Time>,
) {
    let player_cities: Vec<_> = city_query.iter().filter(|c| c.faction == Faction::Player).collect();
    let total_cities = city_query.iter().count();
    let total_pop: u32 = player_cities.iter().map(|c| c.population).sum();

    let elapsed = time.elapsed().as_secs();
    let minutes = elapsed / 60;
    let seconds = elapsed % 60;

    if let Some(e) = hud_text.cities_text {
        if let Ok(mut t) = text_query.get_mut(e) { t.0 = format!("城 {}/{}", player_cities.len(), total_cities); }
    }
    if let Some(e) = hud_text.pop_text {
        if let Ok(mut t) = text_query.get_mut(e) { t.0 = format!("兵 {}", total_pop); }
    }
    if let Some(e) = hud_text.time_text {
        if let Ok(mut t) = text_query.get_mut(e) { t.0 = format!("T {}:{:02}", minutes, seconds); }
    }
}

fn update_bottom_panel(
    city_query: Query<&City>,
    hud_text: Res<HudTextEntities>,
    mut text_query: Query<&mut Text>,
    mut hp_query: Query<(&mut Node, &mut BackgroundColor), With<HpBarFill>>,
) {
    let Some(city_entity) = hud_text.selected_city else { return };
    let Ok(city) = city_query.get(city_entity) else { return };

    if let Some(e) = hud_text.city_info_text {
        if let Ok(mut t) = text_query.get_mut(e) {
            t.0 = format!("[城池] Lv.{} (上限 {})", city.level, city.max_level);
        }
    }

    let ratio = (city.health / city.max_health).clamp(0.0, 1.0);

    if let Some(e) = hud_text.hp_text {
        if let Ok(mut t) = text_query.get_mut(e) {
            t.0 = format!("HP {:.0}/{:.0}", city.health, city.max_health);
        }
    }
    if let Some(e) = hud_text.hp_bar_fill {
        if let Ok((mut node, mut bg)) = hp_query.get_mut(e) {
            node.width = Val::Percent(ratio * 100.0);
            bg.0 = if ratio > 0.5 {
                Color::srgba(0.2, 0.8, 0.2, 1.0)
            } else {
                Color::srgba(0.9, 0.2, 0.2, 1.0)
            };
        }
    }
    if let Some(e) = hud_text.pop_detail_text {
        if let Ok(mut t) = text_query.get_mut(e) {
            t.0 = format!("兵 {}/{}", city.population, city.max_population);
        }
    }
    if let Some(e) = hud_text.exp_text {
        if let Ok(mut t) = text_query.get_mut(e) {
            let required = crate::core::city_level_up_exp(city.max_health);
            t.0 = format!("经验 {:.0}/{:.0}", city.level_exp, required);
        }
    }
}

fn pause_button_system(
    mut interaction_query: Query<&Interaction, (With<PauseButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Paused);
        }
    }
}

fn soldier_type_button_system(
    mut interaction_query: Query<(&SoldierTypeButton, &Interaction), Changed<Interaction>>,
    mut city_query: Query<&mut City>,
    hud_text: Res<HudTextEntities>,
) {
    for (button, interaction) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if let Some(city_entity) = hud_text.selected_city {
                if let Ok(mut city) = city_query.get_mut(city_entity) {
                    city.spawn_type = button.0;
                }
            }
        }
    }
}

fn shield_toggle_system(
    mut interaction_query: Query<&Interaction, (With<ShieldButton>, Changed<Interaction>)>,
    selection: Res<SelectionState>,
    mut shield_query: Query<&mut InfantryShield>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed { continue; }

        // Collect shield states of selected infantry
        let mut has_normal = false;
        let mut has_shield = false;
        let mut infantry_entities: Vec<Entity> = Vec::new();

        for &entity in &selection.selected_soldiers {
            if let Ok(shield) = shield_query.get(entity) {
                infantry_entities.push(entity);
                match shield.0 {
                    ShieldState::Normal => has_normal = true,
                    ShieldState::ShieldUp => has_shield = true,
                }
            }
        }

        if infantry_entities.is_empty() { continue; }

        // Bulk toggle rule: if mixed, unify to ShieldUp first; if all ShieldUp, toggle to Normal
        let new_state = if has_normal && has_shield {
            ShieldState::ShieldUp
        } else if has_normal {
            ShieldState::ShieldUp
        } else {
            ShieldState::Normal
        };

        for entity in infantry_entities {
            if let Ok(mut shield) = shield_query.get_mut(entity) {
                shield.0 = new_state;
            }
        }
    }
}

fn selection_mode_toggle_system(
    mut interaction_circle: Query<&Interaction, (With<CircleSelectButton>, Changed<Interaction>)>,
    mut interaction_rect: Query<&Interaction, (With<RectSelectButton>, Changed<Interaction>)>,
    mut selection: ResMut<SelectionState>,
) {
    for interaction in interaction_circle.iter_mut() {
        if *interaction == Interaction::Pressed {
            selection.selection_mode = SelectionMode::Circle;
        }
    }
    for interaction in interaction_rect.iter_mut() {
        if *interaction == Interaction::Pressed {
            selection.selection_mode = SelectionMode::Rect;
        }
    }
}

fn force_move_button_system(
    mut interaction_query: Query<&Interaction, (With<ForceMoveButton>, Changed<Interaction>)>,
    mut force_next: ResMut<ForceMoveNext>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            force_next.active = true;
        }
    }
}
