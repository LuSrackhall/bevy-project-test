use crate::camera::MainCamera;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy_adapter::input::ForceMoveNext;
use bevy_adapter::tick::{CommandSink, SimulationWorld};
use simulation::command::*;
use simulation::soldier::*;
use simulation::types::*;

/// Returns true if the mouse cursor is currently over any UI element.
/// Uses HoverMap from Bevy's picking system, which is automatically maintained.
/// Transparent containers have Pickable::IGNORE so they don't appear in HoverMap.
fn is_cursor_over_ui(hover_map: &HoverMap, nodes: &Query<&Node>) -> bool {
    hover_map
        .get(&PointerId::Mouse)
        .is_some_and(|h| h.keys().any(|e| nodes.get(*e).is_ok()))
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

impl SelectionState {
    pub fn clear(&mut self) {
        self.selected_unit_ids.clear();
        self.selected_city = None;
        self.drag_start = None;
        self.drag_current = None;
        self.is_dragging = false;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Circle,
    Rect,
}

#[derive(Component)]
pub struct Waypoint;

const DRAG_THRESHOLD: f32 = 10.0;

// ══════════ Screen-to-world helper ══════════

fn screen_to_world(
    cursor: Vec2,
    _window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

// ══════════ Click selection ══════════

#[allow(clippy::too_many_arguments)]
pub fn selection_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    hover_map: Res<HoverMap>,
    nodes: Query<&Node>,
    sim_world: bevy::ecs::system::NonSend<SimulationWorld>,
    mut selection: ResMut<SelectionState>,
) {
    let lid = crate::local_player_id(&*sim_world);
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    // Skip if any UI element is being pressed (prevents clearing selection on UI clicks)
    if is_cursor_over_ui(&hover_map, &nodes) {
        return;
    }
    let Ok(window) = q_windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_t)) = camera_query.single() else {
        return;
    };
    let Some(world_pos) = screen_to_world(cursor, window, camera, cam_t) else {
        return;
    };

    let world = sim_world.world_ref();
    let index = world.get_resource::<simulation::unit_index::UnitIdEntityIndex>();

    // Priority 1: click a friendly city
    let mut hit_city: Option<UnitId> = None;
    {
        let mut q = sim_world.query::<(
            &LogicalPosition,
            &FactionComponent,
            &UnitIdComponent,
            &CityRadius,
            &CityMarker,
        )>();
        for (pos, fac, id, radius, _) in q.iter(world) {
            if fac.0 != FactionId(lid) {
                continue;
            }
            let dx = pos.0.x.to_float() - world_pos.x;
            let dy = pos.0.y.to_float() - world_pos.y;
            if (dx * dx + dy * dy) < (radius.0 as f32).powi(2) {
                hit_city = Some(id.0);
                break;
            }
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
        let mut q = sim_world.query::<(
            &LogicalPosition,
            &FactionComponent,
            &UnitIdComponent,
            &SoldierMarker,
        )>();
        for (pos, fac, id, _) in q.iter(world) {
            if fac.0 != FactionId(lid) {
                continue;
            }
            let dx = pos.0.x.to_float() - world_pos.x;
            let dy = pos.0.y.to_float() - world_pos.y;
            if (dx * dx + dy * dy) < 144.0 {
                hit = Some(id.0);
                break;
            }
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
    hover_map: Res<HoverMap>,
    nodes: Query<&Node>,
    sim_world: bevy::ecs::system::NonSend<SimulationWorld>,
    mut selection: ResMut<SelectionState>,
) {
    let lid = crate::local_player_id(&*sim_world);
    let Ok(window) = q_windows.single() else {
        return;
    };
    let Ok((camera, cam_t)) = camera_query.single() else {
        return;
    };

    // Get cursor position; if outside window and not dragging, skip
    let cursor = window.cursor_position();

    // Don't start drag when clicking on UI elements, but once dragging
    // is in progress (past threshold), continue tracking regardless of UI overlap.
    // This matches the window-escape fix pattern: active drags must not freeze.
    if let Some(cursor_pos) = cursor {
        if selection.drag_start.is_some() && selection.is_dragging {
            // Active drag: always update position, ignore UI overlap
            if mouse.pressed(MouseButton::Left) {
                if let Some(world_pos) = screen_to_world(cursor_pos, window, camera, cam_t) {
                    selection.drag_current = Some(world_pos);
                }
            }
        } else if !is_cursor_over_ui(&hover_map, &nodes) {
            // No active drag: check UI guard as before
            if mouse.pressed(MouseButton::Left) {
                let Some(world_pos) = screen_to_world(cursor_pos, window, camera, cam_t) else {
                    return;
                };
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
        }
    }

    // Complete drag on mouse release (use !pressed for cross-window detection)
    if !mouse.pressed(MouseButton::Left) && selection.is_dragging {
        if let (Some(start), Some(end)) = (selection.drag_start, selection.drag_current) {
            let world = sim_world.world_ref();
            selection.selected_city = None;
            let mut q = sim_world.query::<(
                &LogicalPosition,
                &FactionComponent,
                &UnitIdComponent,
                &SoldierMarker,
            )>();
            let new_sel: Vec<UnitId> = q
                .iter(world)
                .filter(|(pos, fac, _, _)| {
                    if fac.0 != FactionId(lid) {
                        return false;
                    }
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
    sim_world: bevy::ecs::system::NonSend<SimulationWorld>,
    mut selection: ResMut<SelectionState>,
) {
    let lid = crate::local_player_id(&*sim_world);
    if keyboard.just_pressed(KeyCode::KeyA)
        && (keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            || keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]))
    {
        let world = sim_world.world_ref();
        let mut q = sim_world.query::<(&FactionComponent, &UnitIdComponent, &SoldierMarker)>();
        selection.selected_city = None;
        selection.selected_unit_ids = q
            .iter(world)
            .filter(|(fac, _, _)| fac.0 == FactionId(lid))
            .map(|(_, id, _)| id.0)
            .collect();
    }

    // Esc handled by ui::handle_pause_input (deselect then pause)
}

// ══════════ Selection visual ══════════

pub fn selection_visual_system(
    mut gizmos: Gizmos,
    selection: Res<SelectionState>,
    sim_world: bevy::ecs::system::NonSend<SimulationWorld>,
) {
    let lid = crate::local_player_id(&*sim_world);
    let world = sim_world.world_ref();

    // O(1) per lookup using UnitIdEntityIndex (rebuilt each tick in run_tick)
    for &uid in &selection.selected_unit_ids {
        let entity = world
            .get_resource::<simulation::unit_index::UnitIdEntityIndex>()
            .and_then(|idx| idx.get(uid));
        if let Some(entity) = entity {
            if let Some(pos) = world.get::<LogicalPosition>(entity) {
                let p = Vec2::new(pos.0.x.to_float(), pos.0.y.to_float());
                gizmos.circle_2d(p, 10.0, Color::srgb(0.2, 1.0, 0.2));
            }
        }
    }
}

// ══════════ Drag visual ══════════

pub fn drag_visual_system(
    mut gizmos: Gizmos,
    selection: Res<SelectionState>,
    _cam_query: Query<&Projection, With<crate::camera::MainCamera>>,
) {
    if !selection.is_dragging {
        return;
    }
    let Some(start) = selection.drag_start else {
        return;
    };
    let Some(end) = selection.drag_current else {
        return;
    };

    let color = Color::srgba(0.2, 1.0, 0.2, 0.5);

    match selection.selection_mode {
        SelectionMode::Rect => {
            let center = (start + end) / 2.0;
            let size = (end - start).abs();
            gizmos.rect_2d(center, size, color);
        }
        SelectionMode::Circle => {
            let radius = start.distance(end);
            gizmos.circle_2d(start, radius, color);
        }
    }
}

// ══════════ Command issue (right-click) ══════════

#[allow(clippy::too_many_arguments)]
pub fn command_issue_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    hover_map: Res<HoverMap>,
    nodes: Query<&Node>,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
    mut cmd_buf: ResMut<CommandBuffer>,
    selection: ResMut<SelectionState>,
    tick_clock: Res<bevy_adapter::tick::TickClock>,
    force_next: Option<ResMut<ForceMoveNext>>,
) {
    let lid = crate::local_player_id(&*sim_world);
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    if selection.selected_unit_ids.is_empty() {
        return;
    }
    // Skip if any UI element is being pressed (prevents issuing commands when clicking UI)
    if is_cursor_over_ui(&hover_map, &nodes) {
        return;
    }

    let Ok(window) = q_windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_t)) = camera_query.single() else {
        return;
    };
    let Some(world_pos) = screen_to_world(cursor, window, camera, cam_t) else {
        return;
    };

    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let force = shift || force_next.as_ref().is_some_and(|f| f.active);
    if let Some(mut f) = force_next {
        f.active = false;
    }

    let world = sim_world.world_ref();
    let next_tick = tick_clock.current_tick + 1;

    // Priority 1: enemy soldier
    let mut hit_enemy: Option<UnitId> = None;
    {
        let mut q = sim_world.query::<(&LogicalPosition, &FactionComponent, &UnitIdComponent)>();
        for (pos, fac, id) in q.iter(world) {
            if fac.0 != FactionId(lid) {
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
            cmd_buf.push(GameCommand {
                tick: next_tick,
                player_id: lid,
                action: Action::Attack { unit: uid, target },
            });
        }
        return;
    }

    // Priority 2: enemy/neutral city
    let mut hit_city: Option<UnitId> = None;
    {
        let mut q = sim_world.query::<(
            &LogicalPosition,
            &FactionComponent,
            &CityRadius,
            &UnitIdComponent,
        )>();
        for (pos, fac, radius, id) in q.iter(world) {
            if fac.0 != FactionId(lid) {
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
            cmd_buf.push(GameCommand {
                tick: next_tick,
                player_id: lid,
                action: Action::Attack { unit: uid, target },
            });
        }
        return;
    }

    // Priority 3: friendly city
    let mut hit_friendly: Option<UnitId> = None;
    {
        let mut q = sim_world.query::<(
            &LogicalPosition,
            &FactionComponent,
            &CityRadius,
            &UnitIdComponent,
        )>();
        for (pos, fac, radius, id) in q.iter(world) {
            if fac.0 == FactionId(lid) {
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
            cmd_buf.push(GameCommand {
                tick: next_tick,
                player_id: lid,
                action: Action::ReturnToCity {
                    unit: uid,
                    city: target,
                },
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
        cmd_buf.push(GameCommand {
            tick: next_tick,
            player_id: lid,
            action,
        });
    }
}

// ══════════ Waypoint cleanup ══════════

pub fn waypoint_cleanup_system(
    mut commands: Commands,
    waypoint_query: Query<(Entity, &Transform), With<Waypoint>>,
    sim_world: bevy::ecs::system::NonSend<SimulationWorld>,
) {
    let world = sim_world.world_ref();
    for (wp_entity, wp_transform) in waypoint_query.iter() {
        let _wp_pos = wp_transform.translation.xy();
        let mut q = sim_world.query::<(&Movement,)>();
        let has_targeter = q.iter(world).any(|(mov,)| mov.target.is_some());
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
    sim_world: bevy::ecs::system::NonSend<SimulationWorld>,
) {
    let lid = crate::local_player_id(&*sim_world);
    if !keyboard.just_pressed(KeyCode::KeyS) {
        return;
    }
    if selection.selected_unit_ids.is_empty() {
        return;
    }
    // Don't trigger when seek panel input is active
    if seek_state.input_active {
        return;
    }
    // Don't trigger when Ctrl/Cmd is held (that would be Ctrl+S etc.)
    if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        || keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
    {
        return;
    }

    let next_tick = tick_clock.current_tick + 1;
    let seek_range: u32 = 30; // default selection seek range per design D4

    cmd_buf.push(GameCommand {
        tick: next_tick,
        player_id: lid,
        action: Action::SetSeekStance {
            scope: SeekScope::All, // scope is irrelevant when unit_ids is set
            seek_range,
            unit_ids: selection.selected_unit_ids.clone(),
        },
    });
}
