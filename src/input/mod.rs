use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;

use crate::core::*;
use crate::camera::MainCamera;
use crate::city::City;
use crate::soldier::Soldier;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
           .init_resource::<ForceMoveNext>()
           .add_systems(Update, selection_click_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, drag_select_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, command_issue_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, selection_visual_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, selection_shortcut_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, drag_visual_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, waypoint_cleanup_system.run_if(in_state(GameState::Playing)));
    }
}

// ---- Resources ----

/// One-shot toggle for force-move on next command (mobile toolbar button)
#[derive(Resource, Default)]
pub struct ForceMoveNext {
    pub active: bool,
}

#[derive(Resource)]
pub struct SelectionState {
    pub selected_soldiers: Vec<Entity>,
    pub selected_city: Option<Entity>,
    pub drag_start: Option<Vec2>,       // world coords
    pub drag_current: Option<Vec2>,     // world coords
    pub is_dragging: bool,
    pub selection_mode: SelectionMode,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_soldiers: Vec::new(),
            selected_city: None,
            drag_start: None,
            drag_current: None,
            is_dragging: false,
            selection_mode: SelectionMode::Rect,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Circle,
    Rect,
}

// ---- Components ----

#[derive(Component)]
pub struct SelectionIndicator;

/// Waypoint entity for move commands
#[derive(Component)]
pub struct Waypoint;

const DRAG_THRESHOLD: f32 = 10.0; // min drag distance in pixels (screen space)

// ---- Systems ----

/// Screen-to-world coordinate conversion helper
fn screen_to_world(
    cursor: Vec2,
    _window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

/// Click on soldiers to select them
fn selection_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    soldier_query: Query<(Entity, &Transform, &Soldier)>,
    mut selection: ResMut<SelectionState>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }

    let Ok(window) = q_windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Some(world_pos) = screen_to_world(cursor, window, camera, camera_transform) else { return };

    // Check if a friendly soldier was clicked
    let hit_soldier: Option<Entity> = soldier_query.iter()
        .filter(|(_, t, s)| s.faction == Faction::Player && t.translation.xy().distance(world_pos) <= 10.0)
        .map(|(e, _, _)| e)
        .next();

    if let Some(entity) = hit_soldier {
        // Ctrl/Cmd+click: add to selection. Otherwise: replace selection
        let ctrl_held = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            || keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);

        if ctrl_held {
            if !selection.selected_soldiers.contains(&entity) {
                selection.selected_soldiers.push(entity);
            }
        } else {
            selection.selected_soldiers = vec![entity];
        }
        selection.drag_start = Some(world_pos);
    } else {
        // Clicked empty space — deselect
        selection.selected_soldiers.clear();
        selection.selected_city = None;
        selection.drag_start = None;
        selection.is_dragging = false;
    }
}

/// Handle drag selection (box or circle)
fn drag_select_system(
    mouse: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    soldier_query: Query<(Entity, &Transform, &Soldier)>,
    mut selection: ResMut<SelectionState>,
) {
    let Ok(window) = q_windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };

    if mouse.pressed(MouseButton::Left) {
        let Some(world_pos) = screen_to_world(cursor, window, camera, camera_transform) else { return };

        selection.drag_current = Some(world_pos);

        // Check if drag distance exceeds threshold in world space
        if let Some(start) = selection.drag_start {
            if start.distance(world_pos) > DRAG_THRESHOLD {
                selection.is_dragging = true;
            }
        }

        // Set drag start on first frame of press
        if selection.drag_start.is_none() {
            selection.drag_start = Some(world_pos);
        }
    }

    if mouse.just_released(MouseButton::Left) {
        if selection.is_dragging {
            // Perform selection based on drag rect
            if let (Some(start), Some(end)) = (selection.drag_start, selection.drag_current) {
                let new_selection: Vec<Entity> = soldier_query.iter()
                    .filter(|(_, t, s)| {
                        if s.faction != Faction::Player { return false; }
                        let pos = t.translation.xy();
                        match selection.selection_mode {
                            SelectionMode::Rect => {
                                let min = start.min(end);
                                let max = start.max(end);
                                pos.x >= min.x && pos.x <= max.x && pos.y >= min.y && pos.y <= max.y
                            }
                            SelectionMode::Circle => {
                                let center = start;
                                let radius = start.distance(end);
                                pos.distance(center) <= radius
                            }
                        }
                    })
                    .map(|(e, _, _)| e)
                    .collect();
                selection.selected_soldiers = new_selection;
            }
        }

        selection.is_dragging = false;
        selection.drag_start = None;
        selection.drag_current = None;
    }
}

/// Right-click to issue move/attack commands
fn command_issue_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    all_cities: Query<(Entity, &Transform, &City)>,
    selection: ResMut<SelectionState>,
    mut soldier_set: ParamSet<(
        Query<(Entity, &Transform, &Soldier)>,
        Query<(&Transform, &mut Soldier)>,
    )>,
    mut commands: Commands,
    force_next: Option<ResMut<ForceMoveNext>>,
) {
    if !mouse.just_pressed(MouseButton::Right) { return; }
    if selection.selected_soldiers.is_empty() { return; }

    let Ok(window) = q_windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Some(world_pos) = screen_to_world(cursor, window, camera, camera_transform) else { return };

    // Determine force-move: Shift held OR mobile button was pressed
    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let force = shift || force_next.as_ref().map_or(false, |f| f.active);
    if let Some(mut f) = force_next { f.active = false; }

    // Helper: apply order to all selected soldiers
    let apply_order = |target: Entity, soldier_set: &mut ParamSet<(
        Query<(Entity, &Transform, &Soldier)>,
        Query<(&Transform, &mut Soldier)>,
    )>, selection: &SelectionState, force: bool| {
        for &se in &selection.selected_soldiers {
            if let Ok((_, mut s)) = soldier_set.p1().get_mut(se) {
                s.command_target = Some(target);
                s.target = Some(target);
                s.force_move = force;
                s.state = SoldierState::Moving;
            }
        }
    };

    // Priority 1: enemy soldier
    let hit_enemy: Option<Entity> = soldier_set.p0().iter()
        .filter(|(_, t, s)| s.faction != Faction::Player && t.translation.xy().distance(world_pos) <= 12.0)
        .map(|(e, _, _)| e)
        .next();
    if let Some(target) = hit_enemy {
        apply_order(target, &mut soldier_set, &selection, force);
        return;
    }

    // Priority 2: enemy/neutral city
    let hit_city = all_cities.iter()
        .filter(|(_, t, c)| c.faction != Faction::Player && t.translation.xy().distance(world_pos) <= c.visual_radius)
        .map(|(e, _, _)| e)
        .next();
    if let Some(target) = hit_city {
        apply_order(target, &mut soldier_set, &selection, force);
        return;
    }

    // Priority 2.5: friendly city
    let hit_friendly = all_cities.iter()
        .filter(|(_, t, c)| c.faction == Faction::Player && t.translation.xy().distance(world_pos) <= c.visual_radius)
        .map(|(e, _, _)| e)
        .next();
    if let Some(target) = hit_friendly {
        apply_order(target, &mut soldier_set, &selection, force);
        return;
    }

    // Priority 3: empty ground → waypoint
    let waypoint = commands.spawn((
        Waypoint,
        Transform::from_xyz(world_pos.x, world_pos.y, 3.0),
    )).id();
    apply_order(waypoint, &mut soldier_set, &selection, force);
}

/// Maintain visual indicators on selected soldiers
fn selection_visual_system(
    mut commands: Commands,
    selection: Res<SelectionState>,
    indicator_query: Query<Entity, With<SelectionIndicator>>,
    soldier_query: Query<&Transform, With<Soldier>>,
) {
    // Remove old indicators
    for entity in indicator_query.iter() {
        commands.entity(entity).despawn();
    }

    // Add new indicators
    for &entity in &selection.selected_soldiers {
        if let Ok(transform) = soldier_query.get(entity) {
            let pos = transform.translation.xy();
            let indicator = ShapeBuilder::with(&shapes::Circle { radius: 10.0, center: Vec2::ZERO })
                .stroke(Stroke::new(Color::srgb(0.2, 1.0, 0.2), 2.0))
                .build();
            commands.spawn((
                indicator,
                SelectionIndicator,
                Transform::from_xyz(pos.x, pos.y, 5.0),
            ));
        }
    }
}

/// Keyboard shortcuts: Ctrl+A, Esc
fn selection_shortcut_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    soldier_query: Query<(Entity, &Soldier)>,
    mut selection: ResMut<SelectionState>,
) {
    // Ctrl+A: select all friendly soldiers
    if keyboard.just_pressed(KeyCode::KeyA)
        && (keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            || keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]))
    {
        selection.selected_soldiers = soldier_query.iter()
            .filter(|(_, s)| s.faction == Faction::Player)
            .map(|(e, _)| e)
            .collect();
    }

    // Esc handled by game pause system; deselection via empty-space click
}

/// Render drag selection visual (rectangle or circle)
fn drag_visual_system(
    selection: Res<SelectionState>,
    mut commands: Commands,
    mut previous: Local<Option<Entity>>,
) {
    // Despawn previous drag visual
    if let Some(entity) = *previous {
        commands.entity(entity).despawn();
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

    let visual_entity = match selection.selection_mode {
        SelectionMode::Rect => {
            let center = (start + end) / 2.0;
            commands.spawn((visual, Transform::from_xyz(center.x, center.y, 10.0))).id()
        }
        SelectionMode::Circle => {
            commands.spawn((visual, Transform::from_xyz(start.x, start.y, 10.0))).id()
        }
    };

    *previous = Some(visual_entity);
}

/// Despawn waypoints that no soldier is targeting
fn waypoint_cleanup_system(
    mut commands: Commands,
    waypoint_query: Query<(Entity, &Transform), With<Waypoint>>,
    soldier_query: Query<&Soldier>,
) {
    for (entity, _) in waypoint_query.iter() {
        let has_targeter = soldier_query.iter().any(|s| s.target == Some(entity));
        if !has_targeter {
            commands.entity(entity).despawn();
        }
    }
}
