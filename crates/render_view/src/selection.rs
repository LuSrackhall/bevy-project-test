use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;
use bevy_adapter::tick::SimulationWorld;
use bevy_adapter::mapper::UnitIdMapper;
use bevy_adapter::input::ForceMoveNext;
use simulation::types::*;
use simulation::soldier::*;
use simulation::command::*;
use crate::camera::MainCamera;

/// Returns true if any UI element is currently being pressed (clicked).
/// This prevents game-world click processing when interacting with UI buttons.
fn is_any_ui_pressed(pressed: &Query<&Pressed>) -> bool {
    !pressed.is_empty()
}

// ══════════ Resources ══════════

#[derive(Resource)]
pub struct SelectionState {
    pub selected_unit_ids: Vec<UnitId>,
    pub selected_city: Option<UnitId>,
    pub drag_start: Option<Vec2>,
    pub drag_current: Option<Vec2>,
    pub is_dragging: bool,
    pub selection_mode: SelectionMode,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_unit_ids: Vec::new(),
            selected_city: None,
            drag_start: None,
            drag_current: None,
            is_dragging: false,
            selection_mode: SelectionMode::Rect,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode { Circle, Rect }

#[derive(Component)]
pub struct SelectionIndicator;

#[derive(Component)]
pub struct Waypoint;

const DRAG_THRESHOLD: f32 = 10.0;

// ══════════ Screen-to-world helper ══════════

fn screen_to_world(
    cursor: Vec2, _window: &Window, camera: &Camera, camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

// ══════════ Click selection ══════════

pub fn selection_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    pressed: Query<&Pressed>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
    mut selection: ResMut<SelectionState>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }
    // Skip if any UI element is being pressed (prevents clearing selection on UI clicks)
    if is_any_ui_pressed(&pressed) { return; }
    let Ok(window) = q_windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, cam_t)) = camera_query.single() else { return };
    let Some(world_pos) = screen_to_world(cursor, window, camera, cam_t) else { return };

    let world = &mut sim_world.0;

    // Priority 1: click a friendly city
    let mut hit_city: Option<UnitId> = None;
    {
        let mut q = world.query::<(&LogicalPosition, &FactionComponent, &UnitIdComponent, &CityRadius, &CityMarker)>();
        for (pos, fac, id, radius, _) in q.iter(world) {
            if fac.0 != Faction::Player { continue; }
            let dx = pos.0.x.to_float() - world_pos.x;
            let dy = pos.0.y.to_float() - world_pos.y;
            if (dx*dx+dy*dy) < (radius.0 as f32).powi(2) { hit_city = Some(id.0); break; }
        }
    }
    if let Some(cid) = hit_city {
        selection.selected_city = Some(cid);
        selection.selected_unit_ids.clear();
        selection.drag_start = None;
        selection.is_dragging = false;
        return;
    }

    // Priority 2: click a friendly soldier
    let mut hit: Option<UnitId> = None;
    {
        let mut q = world.query::<(&LogicalPosition, &FactionComponent, &UnitIdComponent, &SoldierMarker)>();
        for (pos, fac, id, _) in q.iter(world) {
            if fac.0 != Faction::Player { continue; }
            let dx = pos.0.x.to_float() - world_pos.x;
            let dy = pos.0.y.to_float() - world_pos.y;
            if (dx*dx+dy*dy) < 144.0 { hit = Some(id.0); break; }
        }
    }
    if let Some(uid) = hit {
        selection.selected_city = None;
        let ctrl = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            || keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);
        if ctrl {
            if !selection.selected_unit_ids.contains(&uid) {
                selection.selected_unit_ids.push(uid);
            }
        } else {
            selection.selected_unit_ids = vec![uid];
        }
        selection.drag_start = Some(world_pos);
    } else {
        selection.selected_city = None;
        selection.selected_unit_ids.clear();
        selection.drag_start = None;
        selection.is_dragging = false;
    }
}

// ══════════ Drag selection ══════════

pub fn drag_select_system(
    mouse: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    pressed: Query<&Pressed>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
    mut selection: ResMut<SelectionState>,
) {
    let Ok(window) = q_windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, cam_t)) = camera_query.single() else { return };

    // Don't start drag when clicking on UI elements
    if is_any_ui_pressed(&pressed) {
        return;
    }

    if mouse.pressed(MouseButton::Left) {
        let Some(world_pos) = screen_to_world(cursor, window, camera, cam_t) else { return };
        selection.drag_current = Some(world_pos);
        if let Some(start) = selection.drag_start {
            if start.distance(world_pos) > DRAG_THRESHOLD {
                selection.is_dragging = true;
            }
        }
        if selection.drag_start.is_none() {
            selection.drag_start = Some(world_pos);
        }
    }

    if mouse.just_released(MouseButton::Left) && selection.is_dragging {
        if let (Some(start), Some(end)) = (selection.drag_start, selection.drag_current) {
            let world = &mut sim_world.0;
            selection.selected_city = None;
            let mut query = world.query::<(&LogicalPosition, &FactionComponent, &UnitIdComponent, &SoldierMarker)>();
            let new_sel: Vec<UnitId> = query.iter(world)
                .filter(|(pos, fac, _, _)| {
                    if fac.0 != Faction::Player { return false; }
                    let p = Vec2::new(pos.0.x.to_float(), pos.0.y.to_float());
                    match selection.selection_mode {
                        SelectionMode::Rect => {
                            let min = start.min(end);
                            let max = start.max(end);
                            p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
                        }
                        SelectionMode::Circle => {
                            let center = start;
                            let radius = start.distance(end);
                            p.distance(center) <= radius
                        }
                    }
                })
                .map(|(_, _, id, _)| id.0)
                .collect();
            selection.selected_unit_ids = new_sel;
        }
        selection.is_dragging = false;
        selection.drag_start = None;
        selection.drag_current = None;
    }
}

// ══════════ Keyboard shortcuts ══════════

pub fn selection_shortcut_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
    mut selection: ResMut<SelectionState>,
) {
    if keyboard.just_pressed(KeyCode::KeyA)
        && (keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            || keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]))
    {
        let world = &mut sim_world.0;
        let mut query = world.query::<(&FactionComponent, &UnitIdComponent, &SoldierMarker)>();
        selection.selected_city = None;
        selection.selected_unit_ids = query.iter(world)
            .filter(|(fac, _, _)| fac.0 == Faction::Player)
            .map(|(_, id, _)| id.0)
            .collect();
    }

    // Esc handled by ui::handle_pause_input (deselect then pause)
}

// ══════════ Selection visual ══════════

pub fn selection_visual_system(
    mut commands: Commands,
    selection: Res<SelectionState>,
    indicator_query: Query<Entity, With<SelectionIndicator>>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    for e in indicator_query.iter() { commands.entity(e).despawn(); }

    let world = &mut sim_world.0;
    let mut query = world.query::<(&UnitIdComponent, &LogicalPosition)>();
    for &uid in &selection.selected_unit_ids {
        for (id_comp, pos) in query.iter(world) {
            if id_comp.0 == uid {
                let p = Vec2::new(pos.0.x.to_float(), pos.0.y.to_float());
                let indicator = ShapeBuilder::with(&shapes::Circle { radius: 10.0, center: Vec2::ZERO })
                    .stroke(Stroke::new(Color::srgb(0.2, 1.0, 0.2), 2.0))
                    .build();
                commands.spawn((indicator, SelectionIndicator, Transform::from_xyz(p.x, p.y, 5.0)));
                break;
            }
        }
    }
}

// ══════════ Drag visual ══════════

pub fn drag_visual_system(
    selection: Res<SelectionState>,
    mut commands: Commands,
    mut previous: Local<Option<Entity>>,
) {
    if let Some(e) = *previous {
        commands.entity(e).despawn();
        *previous = None;
    }
    if !selection.is_dragging { return; }
    let Some(start) = selection.drag_start else { return };
    let Some(end) = selection.drag_current else { return };

    let visual = match selection.selection_mode {
        SelectionMode::Rect => {
            let size = (end - start).abs();
            ShapeBuilder::with(&shapes::Rectangle {
                extents: Vec2::new(size.x, size.y),
                origin: shapes::RectangleOrigin::Center,
                radii: None,
            })
            .stroke(Stroke::new(Color::srgba(0.2, 1.0, 0.2, 0.5), 1.5))
            .build()
        }
        SelectionMode::Circle => {
            let radius = start.distance(end);
            ShapeBuilder::with(&shapes::Circle { radius, center: Vec2::ZERO })
                .stroke(Stroke::new(Color::srgba(0.2, 1.0, 0.2, 0.5), 1.5))
                .build()
        }
    };

    let e = match selection.selection_mode {
        SelectionMode::Rect => {
            let center = (start + end) / 2.0;
            commands.spawn((visual, Transform::from_xyz(center.x, center.y, 10.0))).id()
        }
        SelectionMode::Circle => {
            commands.spawn((visual, Transform::from_xyz(start.x, start.y, 10.0))).id()
        }
    };
    *previous = Some(e);
}

// ══════════ Command issue (right-click) ══════════

pub fn command_issue_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    pressed: Query<&Pressed>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
    selection: ResMut<SelectionState>,
    mut cmd_buf: ResMut<CommandBuffer>,
    tick_clock: Res<bevy_adapter::tick::TickClock>,
    force_next: Option<ResMut<ForceMoveNext>>,
) {
    if !mouse.just_pressed(MouseButton::Right) { return; }
    if selection.selected_unit_ids.is_empty() { return; }
    // Skip if any UI element is being pressed (prevents issuing commands when clicking UI)
    if is_any_ui_pressed(&pressed) { return; }

    let Ok(window) = q_windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, cam_t)) = camera_query.single() else { return };
    let Some(world_pos) = screen_to_world(cursor, window, camera, cam_t) else { return };

    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let force = shift || force_next.as_ref().map_or(false, |f| f.active);
    if let Some(mut f) = force_next { f.active = false; }

    let world = &mut sim_world.0;
    let next_tick = tick_clock.current_tick + 1;

    // Priority 1: enemy soldier
    let mut hit_enemy: Option<UnitId> = None;
    {
        let mut query = world.query::<(&LogicalPosition, &FactionComponent, &UnitIdComponent)>();
        for (pos, fac, id) in query.iter(world) {
            if fac.0 != Faction::Player {
                let dx = pos.0.x.to_float() - world_pos.x;
                let dy = pos.0.y.to_float() - world_pos.y;
                if (dx * dx + dy * dy) < 144.0 {
                    hit_enemy = Some(id.0);
                    break;
                }
            }
        }
    }
    if let Some(target) = hit_enemy {
        for &uid in &selection.selected_unit_ids {
            cmd_buf.push(GameCommand { tick: next_tick, player_id: 0,
                action: Action::Attack { unit: uid, target },
            });
        }
        return;
    }

    // Priority 2: enemy/neutral city
    let mut hit_city: Option<UnitId> = None;
    {
        let mut query = world.query::<(&LogicalPosition, &FactionComponent, &CityRadius, &UnitIdComponent)>();
        for (pos, fac, radius, id) in query.iter(world) {
            if fac.0 != Faction::Player {
                let dx = pos.0.x.to_float() - world_pos.x;
                let dy = pos.0.y.to_float() - world_pos.y;
                let r = radius.0 as f32;
                if (dx * dx + dy * dy) < (r * r) {
                    hit_city = Some(id.0);
                    break;
                }
            }
        }
    }
    if let Some(target) = hit_city {
        for &uid in &selection.selected_unit_ids {
            cmd_buf.push(GameCommand { tick: next_tick, player_id: 0,
                action: Action::Attack { unit: uid, target },
            });
        }
        return;
    }

    // Priority 3: friendly city
    let mut hit_friendly: Option<UnitId> = None;
    {
        let mut query = world.query::<(&LogicalPosition, &FactionComponent, &CityRadius, &UnitIdComponent)>();
        for (pos, fac, radius, id) in query.iter(world) {
            if fac.0 == Faction::Player {
                let dx = pos.0.x.to_float() - world_pos.x;
                let dy = pos.0.y.to_float() - world_pos.y;
                if (dx * dx + dy * dy) < (radius.0 as f32).powi(2) {
                    hit_friendly = Some(id.0);
                    break;
                }
            }
        }
    }
    if let Some(target) = hit_friendly {
        for &uid in &selection.selected_unit_ids {
            cmd_buf.push(GameCommand { tick: next_tick, player_id: 0,
                action: Action::ReturnToCity { unit: uid, city: target },
            });
        }
        return;
    }

    // Priority 4: ground move
    let target = FixedVec2::new(
        simulation::types::Fixed::from_float(world_pos.x),
        simulation::types::Fixed::from_float(world_pos.y),
    );
    for &uid in &selection.selected_unit_ids {
        let action = if force {
            Action::ForceMove { unit: uid, target }
        } else {
            Action::MoveTo { unit: uid, target }
        };
        cmd_buf.push(GameCommand { tick: next_tick, player_id: 0, action });
    }
}

// ══════════ Waypoint cleanup ══════════

pub fn waypoint_cleanup_system(
    mut commands: Commands,
    waypoint_query: Query<(Entity, &Transform), With<Waypoint>>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    let world = &mut sim_world.0;
    for (wp_entity, wp_transform) in waypoint_query.iter() {
        let wp_pos = wp_transform.translation.xy();
        let mut query = world.query::<(&Movement,)>();
        let has_targeter = query.iter(world).any(|(mov,)| {
            mov.target.is_some()
        });
        if !has_targeter {
            commands.entity(wp_entity).despawn();
        }
    }
}

// ══════════ Seek stance shortcut (S key) ══════════

pub fn seek_stance_shortcut_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    selection: Res<SelectionState>,
    seek_state: Res<crate::ui::hud::SeekPanelState>,
    mut cmd_buf: ResMut<CommandBuffer>,
    tick_clock: Res<bevy_adapter::tick::TickClock>,
    sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    if !keyboard.just_pressed(KeyCode::KeyS) { return; }
    if selection.selected_unit_ids.is_empty() { return; }
    // Don't trigger when seek panel input is active
    if seek_state.input_active { return; }
    // Don't trigger when Ctrl/Cmd is held (that would be Ctrl+S etc.)
    if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        || keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]) { return; }

    let world = &sim_world.0;
    let next_tick = tick_clock.current_tick + 1;
    let seek_range: u32 = 30; // default selection seek range per design D4

    cmd_buf.push(GameCommand {
        tick: next_tick,
        player_id: 0,
        action: Action::SetSeekStance {
            scope: SeekScope::All, // scope is irrelevant when unit_ids is set
            seek_range,
            unit_ids: selection.selected_unit_ids.clone(),
        },
    });
}
