use bevy::prelude::*;
use simulation::types::*;
use simulation::soldier::*;
use simulation::command::*;
use bevy_adapter::tick::{SimulationWorld, TickClock};
use bevy_adapter::input::ForceMoveNext;
use crate::selection::SelectionState;

/// Resource tracking which city entity is selected for the HUD bottom panel.
#[derive(Resource, Default)]
pub struct SelectedCity(pub Option<Entity>);

/// Internal tracking of text entity IDs for updates.
#[derive(Resource, Default)]
pub(crate) struct HudTexts {
    cities_text: Option<Entity>,
    pop_text: Option<Entity>,
    time_text: Option<Entity>,
    city_info: Option<Entity>,
    hp_text: Option<Entity>,
    hp_fill: Option<Entity>,
    pop_detail: Option<Entity>,
    exp_text: Option<Entity>,
}

#[derive(Component)] struct HudRoot;
#[derive(Component)] pub struct BottomPanel;
#[derive(Component)] pub(crate) struct HpFill;
#[derive(Component)] pub struct SoldierTypeButton(pub SoldierType);

pub fn setup_hud(mut commands: Commands, mut hud_text: ResMut<HudTexts>, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, ..default() },
        HudRoot,
    ))
    .with_children(|parent| {
        // Top bar
        parent.spawn((Node { width: Val::Percent(100.0), height: Val::Px(36.0),
            flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center, padding: UiRect::horizontal(Val::Px(10.0)), ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            let c = parent.spawn((Text::new("城 0/0"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id();
            let p = parent.spawn((Text::new("兵 0"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id();
            let t = parent.spawn((Text::new("T 0:00"), TextFont { font: font.clone(), font_size: 14.0, ..default() })).id();
            hud_text.cities_text = Some(c);
            hud_text.pop_text = Some(p);
            hud_text.time_text = Some(t);
        });

        // Middle spacer
        parent.spawn(Node { flex_grow: 1.0, ..default() });

        // Bottom panel (hidden by default)
        parent.spawn((Node { width: Val::Percent(100.0), height: Val::Px(140.0),
            flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(8.0)),
            display: Display::None, ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            BottomPanel,
        ))
        .with_children(|parent| {
            let ci = parent.spawn((Text::new("[城池] Lv.?"), TextFont { font: font.clone(), font_size: 16.0, ..default() })).id();
            hud_text.city_info = Some(ci);

            let hp = parent.spawn((Text::new("HP ?/?"), TextFont { font: font.clone(), font_size: 13.0, ..default() })).id();
            hud_text.hp_text = Some(hp);

            // HP bar
            parent.spawn((Node { width: Val::Percent(100.0), height: Val::Px(10.0),
                margin: UiRect::vertical(Val::Px(3.0)), ..default() },
                BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 1.0)),
            ))
            .with_children(|parent| {
                let fill = parent.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                    BackgroundColor(Color::srgba(0.2, 0.8, 0.2, 1.0)),
                    HpFill,
                )).id();
                hud_text.hp_fill = Some(fill);
            });

            let pd = parent.spawn((Text::new("兵 ?/?"), TextFont { font: font.clone(), font_size: 13.0, ..default() })).id();
            let ex = parent.spawn((Text::new("经验 ?/?"), TextFont { font: font.clone(), font_size: 13.0, ..default() })).id();
            hud_text.pop_detail = Some(pd);
            hud_text.exp_text = Some(ex);

            // Soldier type buttons
            parent.spawn(Node { flex_direction: FlexDirection::Row, ..default() })
                .with_children(|parent| {
                    for (st, label) in [(SoldierType::Militia, "民兵"), (SoldierType::Infantry, "步兵"), (SoldierType::Archer, "弓兵"), (SoldierType::Cavalry, "骑兵")] {
                        parent.spawn((Button, Node { padding: UiRect::all(Val::Px(6.0)), margin: UiRect::all(Val::Px(3.0)), ..default() }, SoldierTypeButton(st)))
                            .with_child((Text::new(label), TextFont { font: font.clone(), font_size: 13.0, ..default() }));
                    }
                });
        });

        // Bottom toolbar (always visible)
        parent.spawn((Node { width: Val::Percent(100.0), height: Val::Px(40.0),
            flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            for (label, marker) in [("O框选", 0u8), ("[ ]框选", 1), ("盾", 2), ("[>]优先", 3)] {
                parent.spawn((Button, Node { padding: UiRect::all(Val::Px(6.0)), margin: UiRect::all(Val::Px(3.0)), ..default() }, ToolbarButton(marker)))
                    .with_child((Text::new(label), TextFont { font: font.clone(), font_size: 13.0, ..default() }));
            }
        });
    });

}

#[derive(Component)]
pub struct ToolbarButton(pub u8); // 0=circle, 1=rect, 2=shield, 3=force

/// Update top bar: cities, population, time
pub fn update_top_bar(
    mut text_query: Query<&mut Text>,
    hud_text: Res<HudTexts>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
    tick_clock: Res<TickClock>,
    time: Res<Time>,
) {
    let world = &mut sim_world.0;
    let mut city_query = world.query::<(&FactionComponent, &CityComponent)>();
    let player_cities: Vec<_> = city_query.iter(world).filter(|(f, _)| f.0 == Faction::Player).collect();
    let total: usize = city_query.iter(world).count();
    let pop: u32 = player_cities.iter().map(|(_, c)| c.population).sum();

    let elapsed = time.elapsed().as_secs();
    let mins = elapsed / 60;
    let secs = elapsed % 60;

    if let Some(e) = hud_text.cities_text {
        if let Ok(mut t) = text_query.get_mut(e) { t.0 = format!("城 {}/{}", player_cities.len(), total); }
    }
    if let Some(e) = hud_text.pop_text {
        if let Ok(mut t) = text_query.get_mut(e) { t.0 = format!("兵 {}", pop); }
    }
    if let Some(e) = hud_text.time_text {
        if let Ok(mut t) = text_query.get_mut(e) { t.0 = format!("T {}:{:02}", mins, secs); }
    }
}

/// Update bottom panel with selected city info.
/// Uses ParamSet for conflicting &mut Node queries (HpFill vs BottomPanel).
pub fn update_bottom_panel(
    mut text_query: Query<&mut Text>,
    mut node_params: ParamSet<(
        Query<(&mut Node, &mut BackgroundColor), With<HpFill>>,
        Query<&mut Node, With<BottomPanel>>,
    )>,
    mut panel_visible: Local<bool>,
    hud_text: Res<HudTexts>,
    selected_city: Res<SelectedCity>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    if selected_city.0.is_none() {
        if *panel_visible {
            for mut node in node_params.p1().iter_mut() { node.display = Display::None; }
            *panel_visible = false;
        }
        return;
    }

    if !*panel_visible {
        for mut node in node_params.p1().iter_mut() { node.display = Display::Flex; }
        *panel_visible = true;
    }

    let world = &mut sim_world.0;
    let Some(city_entity) = selected_city.0 else { return };
    let em = world.entity(city_entity);
    let Some(city) = em.get::<CityComponent>() else { return };

    let ratio = (city.health_current as f32 / city.health_max as f32).clamp(0.0, 1.0);

    if let Some(e) = hud_text.city_info {
        if let Ok(mut t) = text_query.get_mut(e) { t.0 = format!("[城池] Lv.{} (上限{})", city.level, city.max_level); }
    }
    if let Some(e) = hud_text.hp_text {
        if let Ok(mut t) = text_query.get_mut(e) { t.0 = format!("HP {}/{}", city.health_current, city.health_max); }
    }
    if let Some(e) = hud_text.hp_fill {
        if let Ok((mut node, mut bg)) = node_params.p0().get_mut(e) {
            node.width = Val::Percent(ratio * 100.0);
            bg.0 = if ratio > 0.5 { Color::srgba(0.2, 0.8, 0.2, 1.0) } else { Color::srgba(0.9, 0.2, 0.2, 1.0) };
        }
    }
    if let Some(e) = hud_text.pop_detail {
        if let Ok(mut t) = text_query.get_mut(e) { t.0 = format!("兵 {}/{}", city.population, city.max_population); }
    }
}

/// Handle soldier type button clicks
pub fn soldier_type_button_system(
    mut interaction_query: Query<(&SoldierTypeButton, &Interaction), Changed<Interaction>>,
    selected_city: Res<SelectedCity>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    for (btn, interaction) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed { continue; }
        if let Some(city_entity) = selected_city.0 {
            let world = &mut sim_world.0;
            if let Some(mut city) = world.entity_mut(city_entity).get_mut::<CityComponent>() {
                city.spawn_type = btn.0;
            }
        }
    }
}

/// Handle toolbar buttons
pub fn toolbar_button_system(
    mut interaction_query: Query<(&ToolbarButton, &Interaction), Changed<Interaction>>,
    mut selection: ResMut<SelectionState>,
    mut force_next: ResMut<ForceMoveNext>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    for (btn, interaction) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed { continue; }
        match btn.0 {
            0 => selection.selection_mode = crate::selection::SelectionMode::Circle,
            1 => selection.selection_mode = crate::selection::SelectionMode::Rect,
            2 => { /* shield toggle */ }
            3 => force_next.active = true,
            _ => {}
        }
    }
}

/// City click detection — selects a city for the HUD bottom panel.
/// Runs alongside the soldier selection system.
pub fn city_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::camera::MainCamera>>,
    mut selected_city: ResMut<SelectedCity>,
    mut selection: ResMut<SelectionState>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }
    let Ok(window) = q_windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, cam_t)) = camera_query.single() else { return };
    let Some(world_pos) = camera.viewport_to_world_2d(cam_t, cursor).ok() else { return };

    let world = &mut sim_world.0;
    let mut hit_city = None;
    {
        let mut query = world.query::<(Entity, &LogicalPosition, &CityRadius, &FactionComponent)>();
        for (e, pos, radius, fac) in query.iter(world) {
            if fac.0 != Faction::Player { continue; }
            let dx = pos.0.x.to_float() - world_pos.x;
            let dy = pos.0.y.to_float() - world_pos.y;
            if (dx * dx + dy * dy) < (radius.0 as f32).powi(2) {
                hit_city = Some(e); break;
            }
        }
    }

    if hit_city.is_some() {
        selected_city.0 = hit_city;
    } else {
        // Check if a soldier was clicked (handled by selection system, but we deselect city)
        let mut soldier_hit = false;
        {
            let mut query = world.query::<(&LogicalPosition, &FactionComponent)>();
            for (pos, fac) in query.iter(world) {
                if fac.0 == Faction::Player {
                    let dx = pos.0.x.to_float() - world_pos.x;
                    let dy = pos.0.y.to_float() - world_pos.y;
                    if (dx * dx + dy * dy) < 144.0 { soldier_hit = true; break; }
                }
            }
        }
        if !soldier_hit {
            selected_city.0 = None;
        }
    }
}
