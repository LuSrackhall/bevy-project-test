use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;
use bevy_adapter::tick::SimulationWorld;
use simulation::soldier::*;
use crate::selection::SelectionState;
use crate::camera::MainCamera;
use std::collections::HashMap;

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
struct LvlText;

#[derive(Component)]
struct HpNumText;

#[derive(Component)]
struct ExpNumText;

#[derive(Component)]
pub(crate) struct ShieldFill;

#[derive(Component)]
struct ShieldNumText;

// ══════════ Display Mode ══════════

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InfoBarMode {
    Always,
    Selected,
    Smart,
    Classic,
}

impl InfoBarMode {
    fn next(self) -> Self {
        match self {
            InfoBarMode::Always => InfoBarMode::Selected,
            InfoBarMode::Selected => InfoBarMode::Smart,
            InfoBarMode::Smart => InfoBarMode::Classic,
            InfoBarMode::Classic => InfoBarMode::Always,
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
        Self { mode: InfoBarMode::Classic }
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

const SHIELD_BAR_W: f32 = 40.0;
const SHIELD_BAR_H: f32 = 4.0;
const SHIELD_BG: Color = Color::srgb(0.3, 0.3, 0.3);
const SHIELD_FILL: Color = Color::srgb(0.9, 0.9, 0.9);

// ══════════ Collected unit info ══════════

struct UnitBarInfo {
    unit_id: simulation::types::UnitId,
    world_pos: Vec2,
    hp_cur: u32,
    hp_max: u32,
    level: u32,
    exp: u32,
    shield_hp: u32,
    shield_max: u32,
}

// ══════════ Tracked bar child entity references ══════════

#[derive(Clone)]
pub(crate) struct BarParts {
    root: Entity,
    lvl_text: Entity,
    shield_fill: Entity,
    hp_fill: Entity,
    exp_fill: Entity,
    shield_num: Entity,
    hp_num: Entity,
    exp_num: Entity,
}

// ══════════ Main System ══════════

pub(crate) fn unit_info_bar_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut font_cache: Local<Option<Handle<Font>>>,
    settings: Res<UnitInfoBarSettings>,
    selection: Res<SelectionState>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
    mut bar_parts: Local<HashMap<simulation::types::UnitId, BarParts>>,
    mut root_xform_vis: Query<(&mut Transform, &mut Visibility), (With<BarRoot>, Without<HpFill>, Without<ExpFill>, Without<ShieldFill>)>,
    mut text_q: Query<&mut Text2d>,
    mut shield_fill_q: Query<(&mut Sprite, &mut Transform), (With<ShieldFill>, Without<HpFill>, Without<ExpFill>)>,
    mut hp_fill_q: Query<(&mut Sprite, &mut Transform), (With<HpFill>, Without<ExpFill>, Without<ShieldFill>)>,
    mut exp_fill_q: Query<(&mut Sprite, &mut Transform), (With<ExpFill>, Without<HpFill>, Without<ShieldFill>)>,
    q_windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if font_cache.is_none() {
        *font_cache = Some(asset_server.load("fonts/Arial Unicode.ttf"));
    }
    let font = font_cache.as_ref().unwrap().clone();

    let world = &mut sim_world.0;

    // ── Collect all units ──
    let mut units: Vec<UnitBarInfo> = Vec::new();

    {
        let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &Health, &Level)>();
        for (entity, id, pos, hp, lvl) in q.iter(world) {
            let (shield_hp, shield_max) = if let Some(shield) = world.get::<simulation::types::ShieldItem>(entity) {
                (shield.hp, shield.max_hp)
            } else {
                (0, 0)
            };
            units.push(UnitBarInfo {
                unit_id: id.0,
                world_pos: Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()),
                hp_cur: hp.current,
                hp_max: hp.max,
                level: lvl.level,
                exp: lvl.exp,
                shield_hp,
                shield_max,
            });
        }
    }

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
                shield_hp: 0,
                shield_max: 0,
            });
        }
    }

    // ── Selection set ──
    let selected: std::collections::HashSet<simulation::types::UnitId> =
        selection.selected_unit_ids.iter().copied().collect();
    let sel_city = selection.selected_city;

    // ── Hover detection ──
    let mut hovered_ids: std::collections::HashSet<simulation::types::UnitId> =
        std::collections::HashSet::new();
    if settings.mode == InfoBarMode::Classic || settings.mode == InfoBarMode::Selected {
        if let (Ok(window), Ok((camera, cam_t))) = (q_windows.single(), q_camera.single()) {
            if let Some(cursor) = window.cursor_position() {
                if let Some(world_pos) = camera.viewport_to_world_2d(cam_t, cursor).ok() {
                    // Collect city radii for hover threshold
                    let city_radii: std::collections::HashMap<simulation::types::UnitId, f32> = {
                        let mut q = world.query::<(&UnitIdComponent, &CityRadius)>();
                        q.iter(world).map(|(id, r)| (id.0, r.0 as f32)).collect()
                    };

                    for info in &units {
                        let dx = info.world_pos.x - world_pos.x;
                        let dy = info.world_pos.y - world_pos.y;
                        let dist_sq = dx * dx + dy * dy;
                        let threshold = city_radii
                            .get(&info.unit_id)
                            .map(|r| r * r)
                            .unwrap_or(144.0); // 12px radius for soldiers
                        if dist_sq < threshold {
                            hovered_ids.insert(info.unit_id);
                        }
                    }
                }
            }
        }
    }

    let live_ids: std::collections::HashSet<simulation::types::UnitId> =
        units.iter().map(|u| u.unit_id).collect();

    // ── Clean up bars for destroyed units ──
    let dead_ids: Vec<simulation::types::UnitId> = bar_parts
        .keys()
        .filter(|uid| !live_ids.contains(uid))
        .copied()
        .collect();
    for uid in dead_ids {
        if let Some(parts) = bar_parts.remove(&uid) {
            commands.entity(parts.root).despawn();
        }
    }

    // ── Process each unit ──
    for info in &units {
        let is_selected = selected.contains(&info.unit_id) || sel_city == Some(info.unit_id);
        let is_hovered = hovered_ids.contains(&info.unit_id);
        let should_show = match settings.mode {
            InfoBarMode::Always => true,
            InfoBarMode::Selected => is_selected || is_hovered,
            InfoBarMode::Smart => is_selected || info.hp_cur < info.hp_max || info.exp > 0 || info.shield_hp < info.shield_max,
            InfoBarMode::Classic => is_selected || is_hovered,
        };

        if let Some(parts) = bar_parts.get_mut(&info.unit_id) {
            update_bar(
                parts, info, should_show,
                &mut root_xform_vis, &mut text_q,
                &mut shield_fill_q, &mut hp_fill_q, &mut exp_fill_q,
            );
        } else {
            let parts = create_bar(&mut commands, info, should_show, &font);
            bar_parts.insert(info.unit_id, parts);
        }
    }
}

// ══════════ Create ══════════

fn create_bar(
    commands: &mut Commands,
    info: &UnitBarInfo,
    visible: bool,
    font: &Handle<Font>,
) -> BarParts {
    let root_vis = if visible { Visibility::Inherited } else { Visibility::Hidden };
    let bar_pos = info.world_pos + Vec2::new(0.0, BAR_OFFSET_Y);
    let hp_ratio = info.hp_cur as f32 / info.hp_max.max(1) as f32;
    let exp_ratio = (info.exp as f32 / EXP_MAX as f32).min(1.0);

    let mut lvl_text_e = Entity::PLACEHOLDER;
    let mut shield_fill_e = Entity::PLACEHOLDER;
    let mut hp_fill_e = Entity::PLACEHOLDER;
    let mut exp_fill_e = Entity::PLACEHOLDER;
    let mut shield_num_e = Entity::PLACEHOLDER;
    let mut hp_num_e = Entity::PLACEHOLDER;
    let mut exp_num_e = Entity::PLACEHOLDER;

    let root = commands
        .spawn((
            Transform::from_xyz(bar_pos.x, bar_pos.y, 10.0),
            root_vis,
            UnitInfoBar(info.unit_id),
            BarRoot,
        ))
        .with_children(|parent| {
            // Level text
            lvl_text_e = parent.spawn((
                Text2d::new(format!("Lv.{}", info.level)),
                TextFont { font: font.clone(), font_size: 10.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(-20.0, 6.0, 0.02),
                Visibility::Inherited,
                LvlText,
            )).id();

            // Shield bar (only if unit has a shield)
            if info.shield_max > 0 {
                let shield_ratio = info.shield_hp as f32 / info.shield_max.max(1) as f32;
                let shield_w = SHIELD_BAR_W * shield_ratio;

                // Shield bg
                parent.spawn((
                    ShapeBuilder::with(&shapes::Rectangle {
                        extents: Vec2::new(SHIELD_BAR_W, SHIELD_BAR_H),
                        origin: shapes::RectangleOrigin::Center,
                        radii: None,
                    }).fill(SHIELD_BG).build(),
                    Transform::from_xyz(0.0, 8.0, 0.0),
                    Visibility::Inherited,
                ));

                // Shield fill
                shield_fill_e = parent.spawn((
                    Sprite {
                        color: SHIELD_FILL,
                        custom_size: Some(Vec2::new(shield_w, SHIELD_BAR_H)),
                        ..default()
                    },
                    Transform::from_xyz(-SHIELD_BAR_W / 2.0 + shield_w / 2.0, 8.0, 0.01),
                    Visibility::Inherited,
                    ShieldFill,
                )).id();

                // Shield numeric
                shield_num_e = parent.spawn((
                    Text2d::new(format!("{}/{}", info.shield_hp, info.shield_max)),
                    TextFont { font: font.clone(), font_size: 8.0, ..default() },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(22.0, 8.0, 0.02),
                    Visibility::Inherited,
                    ShieldNumText,
                )).id();
            }

            // HP bg
            parent.spawn((
                ShapeBuilder::with(&shapes::Rectangle {
                    extents: Vec2::new(HP_BAR_W, HP_BAR_H),
                    origin: shapes::RectangleOrigin::Center,
                    radii: None,
                }).fill(HP_BG).build(),
                Transform::from_xyz(0.0, 2.0, 0.0),
                Visibility::Inherited,
            ));

            // HP fill (Sprite for reliable rendering)
            let hp_w = HP_BAR_W * hp_ratio;
            hp_fill_e = parent.spawn((
                Sprite {
                    color: HP_FILL,
                    custom_size: Some(Vec2::new(hp_w, HP_BAR_H)),
                    ..default()
                },
                Transform::from_xyz(-HP_BAR_W / 2.0 + hp_w / 2.0, 2.0, 0.01),
                Visibility::Inherited,
                HpFill,
            )).id();

            // EXP bg
            parent.spawn((
                ShapeBuilder::with(&shapes::Rectangle {
                    extents: Vec2::new(EXP_BAR_W, EXP_BAR_H),
                    origin: shapes::RectangleOrigin::Center,
                    radii: None,
                }).fill(EXP_BG).build(),
                Transform::from_xyz(0.0, -3.0, 0.0),
                Visibility::Inherited,
            ));

            // EXP fill (Sprite for reliable rendering)
            let exp_w = EXP_BAR_W * exp_ratio;
            exp_fill_e = parent.spawn((
                Sprite {
                    color: EXP_FILL,
                    custom_size: Some(Vec2::new(exp_w, EXP_BAR_H)),
                    ..default()
                },
                Transform::from_xyz(-EXP_BAR_W / 2.0 + exp_w / 2.0, -3.0, 0.01),
                Visibility::Inherited,
                ExpFill,
            )).id();

            // HP numeric
            hp_num_e = parent.spawn((
                Text2d::new(format!("{}/{}", info.hp_cur, info.hp_max)),
                TextFont { font: font.clone(), font_size: 8.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(22.0, 2.0, 0.02),
                Visibility::Inherited,
                HpNumText,
            )).id();

            // EXP numeric
            exp_num_e = parent.spawn((
                Text2d::new(format!("{}/{}", info.exp, EXP_MAX)),
                TextFont { font: font.clone(), font_size: 8.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(22.0, -3.0, 0.02),
                Visibility::Inherited,
                ExpNumText,
            )).id();
        })
        .id();

    BarParts {
        root,
        lvl_text: lvl_text_e,
        shield_fill: shield_fill_e,
        hp_fill: hp_fill_e,
        exp_fill: exp_fill_e,
        shield_num: shield_num_e,
        hp_num: hp_num_e,
        exp_num: exp_num_e,
    }
}

// ══════════ Update ══════════

#[allow(clippy::too_many_arguments)]
fn update_bar(
    parts: &mut BarParts,
    info: &UnitBarInfo,
    should_show: bool,
    root_xform_vis: &mut Query<(&mut Transform, &mut Visibility), (With<BarRoot>, Without<HpFill>, Without<ExpFill>, Without<ShieldFill>)>,
    text_q: &mut Query<&mut Text2d>,
    shield_fill_q: &mut Query<(&mut Sprite, &mut Transform), (With<ShieldFill>, Without<HpFill>, Without<ExpFill>)>,
    hp_fill_q: &mut Query<(&mut Sprite, &mut Transform), (With<HpFill>, Without<ExpFill>, Without<ShieldFill>)>,
    exp_fill_q: &mut Query<(&mut Sprite, &mut Transform), (With<ExpFill>, Without<HpFill>, Without<ShieldFill>)>,
) {
    let bar_pos = info.world_pos + Vec2::new(0.0, BAR_OFFSET_Y);

    if let Ok((mut t, mut v)) = root_xform_vis.get_mut(parts.root) {
        t.translation.x = bar_pos.x;
        t.translation.y = bar_pos.y;
        *v = if should_show { Visibility::Inherited } else { Visibility::Hidden };
    }

    if !should_show {
        return;
    }

    // Update texts via stored entity IDs
    if let Ok(mut t) = text_q.get_mut(parts.lvl_text) {
        t.0 = format!("Lv.{}", info.level);
    }
    if let Ok(mut t) = text_q.get_mut(parts.hp_num) {
        t.0 = format!("{}/{}", info.hp_cur, info.hp_max);
    }
    if let Ok(mut t) = text_q.get_mut(parts.exp_num) {
        t.0 = format!("{}/{}", info.exp, EXP_MAX);
    }
    if info.shield_max > 0 {
        if let Ok(mut t) = text_q.get_mut(parts.shield_num) {
            t.0 = format!("{}/{}", info.shield_hp, info.shield_max);
        }
    }

    // Update fill bars: modify Sprite directly (no despawn/respawn)
    let hp_ratio = info.hp_cur as f32 / info.hp_max.max(1) as f32;
    let exp_ratio = (info.exp as f32 / EXP_MAX as f32).min(1.0);

    // Shield fill: update custom_size and position
    if info.shield_max > 0 {
        let shield_ratio = info.shield_hp as f32 / info.shield_max.max(1) as f32;
        let shield_w = SHIELD_BAR_W * shield_ratio;
        if let Ok((mut sprite, mut xform)) = shield_fill_q.get_mut(parts.shield_fill) {
            sprite.custom_size = Some(Vec2::new(shield_w, SHIELD_BAR_H));
            xform.translation.x = -SHIELD_BAR_W / 2.0 + shield_w / 2.0;
        }
    }

    // HP fill: update custom_size and position
    let hp_w = HP_BAR_W * hp_ratio;
    if let Ok((mut sprite, mut xform)) = hp_fill_q.get_mut(parts.hp_fill) {
        sprite.custom_size = Some(Vec2::new(hp_w, HP_BAR_H));
        xform.translation.x = -HP_BAR_W / 2.0 + hp_w / 2.0;
    }

    // EXP fill: update custom_size and position
    let exp_w = EXP_BAR_W * exp_ratio;
    if let Ok((mut sprite, mut xform)) = exp_fill_q.get_mut(parts.exp_fill) {
        sprite.custom_size = Some(Vec2::new(exp_w, EXP_BAR_H));
        xform.translation.x = -EXP_BAR_W / 2.0 + exp_w / 2.0;
    }
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
