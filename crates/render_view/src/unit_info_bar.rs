use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;
use bevy_adapter::tick::SimulationWorld;
use simulation::soldier::*;
use crate::selection::SelectionState;

// ══════════ Components ══════════

#[derive(Component, Clone, Debug)]
pub struct UnitInfoBar(pub simulation::types::UnitId);

#[derive(Component)]
pub(crate) struct BarRoot;

#[derive(Component)]
pub(crate) struct HpFill;

#[derive(Component)]
pub(crate) struct ExpFill;

#[derive(Component)]
pub(crate) struct LvlText;

#[derive(Component)]
pub(crate) struct HpNumText;

#[derive(Component)]
pub(crate) struct ExpNumText;

// ══════════ Display Mode ══════════

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InfoBarMode {
    Always,
    Selected,
    Smart,
}

impl InfoBarMode {
    fn next(self) -> Self {
        match self {
            InfoBarMode::Always => InfoBarMode::Selected,
            InfoBarMode::Selected => InfoBarMode::Smart,
            InfoBarMode::Smart => InfoBarMode::Always,
        }
    }
}

// ══════════ Settings Resource ══════════

#[derive(Resource)]
pub struct UnitInfoBarSettings {
    pub mode: InfoBarMode,
}

impl Default for UnitInfoBarSettings {
    fn default() -> Self {
        Self { mode: InfoBarMode::Smart }
    }
}

// ══════════ Layout Constants ══════════

const BAR_OFFSET_Y: f32 = 22.0;
const HP_BAR_W: f32 = 40.0;
const HP_BAR_H: f32 = 6.0;
const EXP_BAR_W: f32 = 40.0;
const EXP_BAR_H: f32 = 4.0;
const EXP_MAX: u32 = 100;

const HP_BG: Color = Color::srgb(0.8, 0.0, 0.0);
const HP_FILL: Color = Color::srgb(0.0, 0.8, 0.0);
const EXP_BG: Color = Color::srgb(0.133, 0.267, 0.667);
const EXP_FILL: Color = Color::srgb(0.667, 0.267, 1.0);

// ══════════ Collected unit info ══════════

struct UnitBarInfo {
    unit_id: simulation::types::UnitId,
    world_pos: Vec2,
    hp_cur: u32,
    hp_max: u32,
    level: u32,
    exp: u32,
}

// ══════════ Main System ══════════

pub fn unit_info_bar_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut font_cache: Local<Option<Handle<Font>>>,
    settings: Res<UnitInfoBarSettings>,
    selection: Res<SelectionState>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
    bar_query: Query<(Entity, &UnitInfoBar)>,
    hp_fill_query: Query<Entity, With<HpFill>>,
    exp_fill_query: Query<Entity, With<ExpFill>>,
    mut lvl_text_q: Query<(&mut Text2d, Entity), With<LvlText>>,
    mut hp_num_q: Query<(&mut Text2d, Entity), With<HpNumText>>,
    mut exp_num_q: Query<(&mut Text2d, Entity), With<ExpNumText>>,
    children_query: Query<&Children>,
    mut root_transforms: Query<&mut Transform, With<BarRoot>>,
    mut vis_query: Query<&mut Visibility, With<BarRoot>>,
) {
    if font_cache.is_none() {
        *font_cache = Some(asset_server.load("fonts/Arial Unicode.ttf"));
    }
    let font = font_cache.as_ref().unwrap().clone();

    let world = &mut sim_world.0;

    let mut units: Vec<UnitBarInfo> = Vec::new();

    // Soldiers
    {
        let mut q = world.query::<(&UnitIdComponent, &LogicalPosition, &Health, &Level)>();
        for (id, pos, hp, lvl) in q.iter(world) {
            units.push(UnitBarInfo {
                unit_id: id.0,
                world_pos: Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()),
                hp_cur: hp.current,
                hp_max: hp.max,
                level: lvl.level,
                exp: lvl.exp,
            });
        }
    }

    // Cities
    {
        let mut q = world.query::<(&UnitIdComponent, &LogicalPosition, &CityComponent)>();
        for (id, pos, city) in q.iter(world) {
            units.push(UnitBarInfo {
                unit_id: id.0,
                world_pos: Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()),
                hp_cur: city.health_current,
                hp_max: city.health_max,
                level: city.level,
                exp: city.level_exp as u32,
            });
        }
    }

    let selected: std::collections::HashSet<simulation::types::UnitId> =
        selection.selected_unit_ids.iter().copied().collect();
    let sel_city = selection.selected_city;

    let live_ids: std::collections::HashSet<simulation::types::UnitId> =
        units.iter().map(|u| u.unit_id).collect();

    let mut existing: std::collections::HashMap<simulation::types::UnitId, Entity> =
        std::collections::HashMap::new();
    for (e, bar) in bar_query.iter() {
        existing.insert(bar.0, e);
    }

    for info in &units {
        let is_selected = selected.contains(&info.unit_id) || sel_city == Some(info.unit_id);
        let should_show = match settings.mode {
            InfoBarMode::Always => true,
            InfoBarMode::Selected => is_selected,
            InfoBarMode::Smart => is_selected || info.hp_cur < info.hp_max || info.exp > 0,
        };

        if let Some(&bar_entity) = existing.get(&info.unit_id) {
            update_bar(
                &mut commands, bar_entity, info, should_show,
                &hp_fill_query, &exp_fill_query,
                &mut lvl_text_q, &mut hp_num_q, &mut exp_num_q,
                &children_query, &mut root_transforms, &mut vis_query,
            );
        } else {
            create_bar(&mut commands, info, should_show, &font);
        }
    }

    // Clean up bars for destroyed units
    for (unit_id, entity) in &existing {
        if !live_ids.contains(unit_id) {
            commands.entity(*entity).despawn();
        }
    }
}

// ══════════ Create ══════════

fn create_bar(commands: &mut Commands, info: &UnitBarInfo, visible: bool, font: &Handle<Font>) {
    let vis = if visible { Visibility::Inherited } else { Visibility::Hidden };
    let bar_pos = info.world_pos + Vec2::new(0.0, BAR_OFFSET_Y);
    let hp_ratio = info.hp_cur as f32 / info.hp_max.max(1) as f32;
    let exp_ratio = (info.exp as f32 / EXP_MAX as f32).min(1.0);

    commands
        .spawn((
            Transform::from_xyz(bar_pos.x, bar_pos.y, 10.0),
            vis,
            UnitInfoBar(info.unit_id),
            BarRoot,
        ))
        .with_children(|parent| {
            // Level text
            parent.spawn((
                Text2d::new(format!("Lv.{}", info.level)),
                TextFont { font: font.clone(), font_size: 10.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(-20.0, 6.0, 0.02),
                vis,
                LvlText,
            ));

            // HP bg
            parent.spawn((
                ShapeBuilder::with(&shapes::Rectangle {
                    extents: Vec2::new(HP_BAR_W, HP_BAR_H),
                    origin: shapes::RectangleOrigin::Center,
                    radii: None,
                }).fill(HP_BG).build(),
                Transform::from_xyz(0.0, 2.0, 0.0),
                vis,
            ));

            // HP fill
            let hp_w = HP_BAR_W * hp_ratio;
            parent.spawn((
                ShapeBuilder::with(&shapes::Rectangle {
                    extents: Vec2::new(hp_w, HP_BAR_H),
                    origin: shapes::RectangleOrigin::CustomCenter(Vec2::new(-HP_BAR_W / 2.0 + hp_w / 2.0, 0.0)),
                    radii: None,
                }).fill(HP_FILL).build(),
                Transform::from_xyz(0.0, 2.0, 0.01),
                vis,
                HpFill,
            ));

            // EXP bg
            parent.spawn((
                ShapeBuilder::with(&shapes::Rectangle {
                    extents: Vec2::new(EXP_BAR_W, EXP_BAR_H),
                    origin: shapes::RectangleOrigin::Center,
                    radii: None,
                }).fill(EXP_BG).build(),
                Transform::from_xyz(0.0, -3.0, 0.0),
                vis,
            ));

            // EXP fill
            let exp_w = EXP_BAR_W * exp_ratio;
            parent.spawn((
                ShapeBuilder::with(&shapes::Rectangle {
                    extents: Vec2::new(exp_w, EXP_BAR_H),
                    origin: shapes::RectangleOrigin::CustomCenter(Vec2::new(-EXP_BAR_W / 2.0 + exp_w / 2.0, 0.0)),
                    radii: None,
                }).fill(EXP_FILL).build(),
                Transform::from_xyz(0.0, -3.0, 0.01),
                vis,
                ExpFill,
            ));

            // HP numeric
            parent.spawn((
                Text2d::new(format!("{}/{}", info.hp_cur, info.hp_max)),
                TextFont { font: font.clone(), font_size: 8.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(22.0, 2.0, 0.02),
                vis,
                HpNumText,
            ));

            // EXP numeric
            parent.spawn((
                Text2d::new(format!("{}/{}", info.exp, EXP_MAX)),
                TextFont { font: font.clone(), font_size: 8.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(22.0, -3.0, 0.02),
                vis,
                ExpNumText,
            ));
        });
}

// ══════════ Update ══════════

#[allow(clippy::too_many_arguments)]
fn update_bar(
    commands: &mut Commands,
    bar_entity: Entity,
    info: &UnitBarInfo,
    should_show: bool,
    hp_fill_query: &Query<Entity, With<HpFill>>,
    exp_fill_query: &Query<Entity, With<ExpFill>>,
    lvl_text_q: &mut Query<(&mut Text2d, Entity), With<LvlText>>,
    hp_num_q: &mut Query<(&mut Text2d, Entity), With<HpNumText>>,
    exp_num_q: &mut Query<(&mut Text2d, Entity), With<ExpNumText>>,
    children_query: &Query<&Children>,
    root_transforms: &mut Query<&mut Transform, With<BarRoot>>,
    vis_query: &mut Query<&mut Visibility, With<BarRoot>>,
) {
    let bar_pos = info.world_pos + Vec2::new(0.0, BAR_OFFSET_Y);

    if let Ok(mut t) = root_transforms.get_mut(bar_entity) {
        t.translation.x = bar_pos.x;
        t.translation.y = bar_pos.y;
    }

    if let Ok(mut v) = vis_query.get_mut(bar_entity) {
        *v = if should_show { Visibility::Inherited } else { Visibility::Hidden };
    }

    if !should_show {
        return;
    }

    // Get bar children for matching
    let bar_kids: Vec<Entity> = children_query
        .get(bar_entity)
        .map(|c| c.to_vec())
        .unwrap_or_default();

    // Update texts
    let new_lvl = format!("Lv.{}", info.level);
    for (mut t, e) in lvl_text_q.iter_mut() {
        if bar_kids.iter().any(|k| *k == e) { t.0 = new_lvl.clone(); }
    }
    let new_hp = format!("{}/{}", info.hp_cur, info.hp_max);
    for (mut t, e) in hp_num_q.iter_mut() {
        if bar_kids.iter().any(|k| *k == e) { t.0 = new_hp.clone(); }
    }
    let new_exp = format!("{}/{}", info.exp, EXP_MAX);
    for (mut t, e) in exp_num_q.iter_mut() {
        if bar_kids.iter().any(|k| *k == e) { t.0 = new_exp.clone(); }
    }

    // Update fill bars (despawn old children, spawn new)
    let hp_ratio = info.hp_cur as f32 / info.hp_max.max(1) as f32;
    let exp_ratio = (info.exp as f32 / EXP_MAX as f32).min(1.0);

    let hp_kids: Vec<Entity> = hp_fill_query.iter()
        .filter(|e| bar_kids.iter().any(|k| *k == *e))
        .collect();
    for e in hp_kids {
        commands.entity(e).despawn();
    }

    let hp_w = HP_BAR_W * hp_ratio;
    commands.spawn((
        ShapeBuilder::with(&shapes::Rectangle {
            extents: Vec2::new(hp_w, HP_BAR_H),
            origin: shapes::RectangleOrigin::CustomCenter(Vec2::new(-HP_BAR_W / 2.0 + hp_w / 2.0, 0.0)),
            radii: None,
        }).fill(HP_FILL).build(),
        Transform::from_xyz(0.0, 2.0, 0.01),
        Visibility::Inherited,
        HpFill,
    )).set_parent_in_place(bar_entity);

    let exp_kids: Vec<Entity> = exp_fill_query.iter()
        .filter(|e| bar_kids.iter().any(|k| *k == *e))
        .collect();
    for e in exp_kids {
        commands.entity(e).despawn();
    }

    let exp_w = EXP_BAR_W * exp_ratio;
    commands.spawn((
        ShapeBuilder::with(&shapes::Rectangle {
            extents: Vec2::new(exp_w, EXP_BAR_H),
            origin: shapes::RectangleOrigin::CustomCenter(Vec2::new(-EXP_BAR_W / 2.0 + exp_w / 2.0, 0.0)),
            radii: None,
        }).fill(EXP_FILL).build(),
        Transform::from_xyz(0.0, -3.0, 0.01),
        Visibility::Inherited,
        ExpFill,
    )).set_parent_in_place(bar_entity);
}

// ══════════ Ctrl+H Mode Toggle ══════════

pub fn info_bar_mode_toggle_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<UnitInfoBarSettings>,
) {
    if keyboard.just_pressed(KeyCode::KeyH)
        && (keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            || keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]))
    {
        settings.mode = settings.mode.next();
        info!("Info bar mode: {:?}", settings.mode);
    }
}
