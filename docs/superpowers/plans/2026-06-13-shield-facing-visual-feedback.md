# 盾牌 & 朝向 & 掉落盾牌 视觉反馈 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add visual feedback for shields, facing direction, and dropped shields in the render_view layer.

**Architecture:** All changes are in `render_view` crate only. `debug_shape.rs` handles world-space Gizmo rendering (facing line, shield shape, dropped shields). `unit_info_bar.rs` handles overhead HP/EXP bar layout (shield HP bar). No simulation layer changes.

**Tech Stack:** Rust, Bevy 0.18, bevy_prototype_lyon 0.16, Gizmos, SimulationWorld ECS queries

---

## File Map

### Files to Modify
| File | Changes |
|------|---------|
| `render_view/src/debug_shape.rs` | Add facing line + shield shape to soldier rendering, add `draw_dropped_shields_system` |
| `render_view/src/unit_info_bar.rs` | Add shield HP bar above existing HP bar, extend `UnitBarInfo` and `BarParts` |
| `render_view/src/lib.rs` | Register `draw_dropped_shields_system` in plugin |

### Files NOT Modified
- `simulation/` — no changes, existing components already expose all needed data
- `bevy_adapter/` — no changes
- `presentation/` — no changes

---

## Task 1: Facing Direction Line

**Files:**
- Modify: `render_view/src/debug_shape.rs`

### Step 1.1: Add facing line to soldier rendering

In the "Draw soldiers" block of `draw_debug_shapes_system`, after drawing the soldier circle, add a facing direction line:

```rust
// Inside the soldier iteration loop, after gizmos.circle_2d(...):

// Facing direction line
if let Some(facing) = world.get::<simulation::types::FacingDirection>(entity_ref) {
    let angle_deg = facing.angle.to_float();
    let angle_rad = angle_deg * std::f32::consts::PI / 180.0;
    let line_len = r * 1.5;
    let dir = Vec2::new(angle_rad.cos(), angle_rad.sin());
    let p = Vec2::new(pos.0.x.to_float(), pos.0.y.to_float());
    let line_color = match faction.0 {
        simulation::types::Faction::Player => Color::srgb(0.5, 0.7, 1.0),
        simulation::types::Faction::Enemy => Color::srgb(1.0, 0.5, 0.5),
        simulation::types::Faction::Neutral => Color::srgb(0.7, 0.7, 0.7),
    };
    gizmos.line_2d(p, p + dir * line_len, line_color);
}
```

NOTE: The current soldier query uses `(&LogicalPosition, &FactionComponent, &SoldierTypeComponent)`. To access `FacingDirection`, you need to either:
- Add it to the query tuple: `(&LogicalPosition, &FactionComponent, &SoldierTypeComponent, &FacingDirection)`
- OR use `world.get::<FacingDirection>(entity)` after getting the entity

The cleaner approach is to add `FacingDirection` to the query. But since the current query doesn't have an `Entity` handle, use a separate query or add `Entity` to the existing query.

**Recommended approach:** Add `Entity` and `Option<&FacingDirection>` to the soldier query:

```rust
let mut query = world.query::<(Entity, &LogicalPosition, &FactionComponent, &SoldierTypeComponent, Option<&simulation::types::FacingDirection>)>();
for (entity, pos, faction, stype, facing) in query.iter(world) {
    // ... existing circle rendering ...

    // Facing direction line
    if let Some(facing) = facing {
        let angle_deg = facing.angle.to_float();
        let angle_rad = angle_deg * std::f32::consts::PI / 180.0;
        let line_len = r * 1.5;
        let dir = Vec2::new(angle_rad.cos(), angle_rad.sin());
        let p = Vec2::new(pos.0.x.to_float(), pos.0.y.to_float());
        let line_color = match faction.0 {
            simulation::types::Faction::Player => Color::srgb(0.5, 0.7, 1.0),
            simulation::types::Faction::Enemy => Color::srgb(1.0, 0.5, 0.5),
            simulation::types::Faction::Neutral => Color::srgb(0.7, 0.7, 0.7),
        };
        gizmos.line_2d(p, p + dir * line_len, line_color);
    }
}
```

### Step 1.2: Verify

Run: `cargo build -p render_view`
Expected: compiles cleanly.

### Step 1.3: Commit

```bash
git add crates/render_view/src/debug_shape.rs
git commit -m "feat(render): add facing direction line to soldier debug shapes"
```

---

## Task 2: Shield Visual on Soldiers

**Files:**
- Modify: `render_view/src/debug_shape.rs`

### Step 2.1: Add shield rectangle to soldier rendering

In the same soldier iteration loop, after the facing line, add shield visual:

```rust
// After facing line code, still inside the soldier loop:

// Shield visual — small rectangle to the right of the soldier
if let Some(shield) = world.get::<simulation::soldier::ShieldItem>(entity_ref) {
    if shield.hp > 0 {
        let shield_offset = r + 3.0; // collision_radius + 3px
        let shield_pos = p + Vec2::new(shield_offset, 0.0);
        let shield_color = match faction.0 {
            simulation::types::Faction::Player => Color::srgb(0.4, 0.6, 1.0),
            simulation::types::Faction::Enemy => Color::srgb(1.0, 0.4, 0.4),
            simulation::types::Faction::Neutral => Color::srgb(0.6, 0.6, 0.6),
        };
        // Draw shield as a small rectangle outline (4x5)
        let hw = 2.0; // half width
        let hh = 2.5; // half height
        let corners = [
            shield_pos + Vec2::new(-hw, -hh),
            shield_pos + Vec2::new(hw, -hh),
            shield_pos + Vec2::new(hw, hh),
            shield_pos + Vec2::new(-hw, hh),
        ];
        gizmos.line_2d(corners[0], corners[1], shield_color);
        gizmos.line_2d(corners[1], corners[2], shield_color);
        gizmos.line_2d(corners[2], corners[3], shield_color);
        gizmos.line_2d(corners[3], corners[0], shield_color);
    }
}
```

NOTE: `ShieldItem` is in `simulation::types` (or `simulation::soldier`). Check the correct import path.

### Step 2.2: Verify

Run: `cargo build -p render_view`
Expected: compiles cleanly.

### Step 2.3: Commit

```bash
git add crates/render_view/src/debug_shape.rs
git commit -m "feat(render): add shield visual rectangle next to soldiers"
```

---

## Task 3: Dropped Shield Rendering

**Files:**
- Modify: `render_view/src/debug_shape.rs`
- Modify: `render_view/src/lib.rs`

### Step 3.1: Add `draw_dropped_shields_system` to debug_shape.rs

Add a new public system function at the end of the file:

```rust
/// Render dropped shields on the ground as gray rectangles.
pub fn draw_dropped_shields_system(
    mut gizmos: Gizmos,
    mut sim_world: bevy::ecs::system::NonSendMut<SimulationWorld>,
) {
    let world = &mut sim_world.0;
    let mut query = world.query::<(&simulation::types::DroppedShield,)>();
    for (dropped,) in query.iter(world) {
        let p = Vec2::new(dropped.position.x.to_float(), dropped.position.y.to_float());
        let color = Color::srgb(0.6, 0.6, 0.6);

        // 6x8 rectangle outline
        let hw = 3.0;
        let hh = 4.0;
        let corners = [
            p + Vec2::new(-hw, -hh),
            p + Vec2::new(hw, -hh),
            p + Vec2::new(hw, hh),
            p + Vec2::new(-hw, hh),
        ];
        gizmos.line_2d(corners[0], corners[1], color);
        gizmos.line_2d(corners[1], corners[2], color);
        gizmos.line_2d(corners[2], corners[3], color);
        gizmos.line_2d(corners[3], corners[0], color);
    }
}
```

### Step 3.2: Register in lib.rs

Add to the `Update` system set in `RenderViewPlugin::build`:

```rust
crate::debug_shape::draw_dropped_shields_system,
```

### Step 3.3: Verify

Run: `cargo build -p render_view`
Expected: compiles cleanly.

### Step 3.4: Commit

```bash
git add crates/render_view/src/debug_shape.rs crates/render_view/src/lib.rs
git commit -m "feat(render): add dropped shield rendering on ground"
```

---

## Task 4: Shield HP Bar

**Files:**
- Modify: `render_view/src/unit_info_bar.rs`

### Step 4.1: Add shield data to `UnitBarInfo`

Add `shield_hp: u32` and `shield_max: u32` fields:

```rust
struct UnitBarInfo {
    unit_id: simulation::types::UnitId,
    world_pos: Vec2,
    hp_cur: u32,
    hp_max: u32,
    shield_hp: u32,   // NEW
    shield_max: u32,  // NEW
    level: u32,
    exp: u32,
}
```

### Step 4.2: Add shield bar components

Add new component markers:

```rust
#[derive(Component)]
pub(crate) struct ShieldFill;

#[derive(Component)]
struct ShieldNumText;
```

### Step 4.3: Add shield fields to `BarParts`

```rust
#[derive(Clone)]
pub(crate) struct BarParts {
    root: Entity,
    lvl_text: Entity,
    shield_fill: Entity,   // NEW (Entity::PLACEHOLDER if no shield)
    hp_fill: Entity,
    exp_fill: Entity,
    shield_num: Entity,    // NEW (Entity::PLACEHOLDER if no shield)
    hp_num: Entity,
    exp_num: Entity,
}
```

### Step 4.4: Add layout constants

```rust
const SHIELD_BAR_W: f32 = 40.0;
const SHIELD_BAR_H: f32 = 4.0;
const SHIELD_BG: Color = Color::srgb(0.3, 0.3, 0.3);
const SHIELD_FILL: Color = Color::srgb(0.9, 0.9, 0.9);
```

### Step 4.5: Collect shield data in `unit_info_bar_system`

In the soldier collection block, also collect shield info:

```rust
// Existing soldier query:
let mut q = world.query::<(&UnitIdComponent, &LogicalPosition, &Health, &Level)>();
for (id, pos, hp, lvl) in q.iter(world) {
    // Check for shield
    let (shield_hp, shield_max) = /* get ShieldItem if present */;
    units.push(UnitBarInfo {
        unit_id: id.0,
        world_pos: Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()),
        hp_cur: hp.current,
        hp_max: hp.max,
        shield_hp,
        shield_max,
        level: lvl.level,
        exp: lvl.exp,
    });
}
```

To get shield data, you need to access the entity. Options:
1. Add `Entity` to the query, then `world.get::<ShieldItem>(entity)`
2. Build a separate HashMap of shield data

Recommended: Add `Entity` to the query and use `Option`:

```rust
let mut q = world.query::<(Entity, &UnitIdComponent, &LogicalPosition, &Health, &Level)>();
for (entity, id, pos, hp, lvl) in q.iter(world) {
    let (shield_hp, shield_max) = if let Some(shield) = world.get::<simulation::types::ShieldItem>(entity) {
        (shield.hp, shield.max_hp)
    } else {
        (0, 0)
    };
    // ...
}
```

For cities, set `shield_hp = 0, shield_max = 0`.

### Step 4.6: Add shield queries to system parameters

Add to `unit_info_bar_system` function signature:

```rust
mut shield_fill_q: Query<(&mut Sprite, &mut Transform), (With<ShieldFill>, Without<HpFill>, Without<ExpFill>)>,
mut shield_num_q: Query<&mut Text2d, With<ShieldNumText>>,
```

### Step 4.7: Modify `create_bar` for shield bar

Add shield bar creation ABOVE the HP bar (Y offset +6px):

```rust
// Shield bar — only if unit has a shield
let mut shield_fill_e = Entity::PLACEHOLDER;
let mut shield_num_e = Entity::PLACEHOLDER;

if info.shield_max > 0 {
    let shield_ratio = info.shield_hp as f32 / info.shield_max.max(1) as f32;

    // Shield bg
    parent.spawn((
        ShapeBuilder::with(&shapes::Rectangle {
            extents: Vec2::new(SHIELD_BAR_W, SHIELD_BAR_H),
            origin: shapes::RectangleOrigin::Center,
            radii: None,
        }).fill(SHIELD_BG).build(),
        Transform::from_xyz(0.0, 8.0, 0.0), // above HP bar
        Visibility::Inherited,
    ));

    // Shield fill
    let shield_w = SHIELD_BAR_W * shield_ratio;
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
```

### Step 4.8: Update `BarParts` construction

```rust
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
```

### Step 4.9: Modify `update_bar` for shield bar

Add shield bar update logic. The shield bar may not exist (Entity::PLACEHOLDER) for units without shields:

```rust
// Shield bar update — only if it exists
if info.shield_max > 0 {
    let shield_ratio = info.shield_hp as f32 / info.shield_max.max(1) as f32;
    let shield_w = SHIELD_BAR_W * shield_ratio;

    if let Ok((mut sprite, mut xform)) = shield_fill_q.get_mut(parts.shield_fill) {
        sprite.custom_size = Some(Vec2::new(shield_w, SHIELD_BAR_H));
        xform.translation.x = -SHIELD_BAR_W / 2.0 + shield_w / 2.0;
    }
    if let Ok(mut t) = shield_num_q.get_mut(parts.shield_num) {
        t.0 = format!("{}/{}", info.shield_hp, info.shield_max);
    }
}
```

### Step 4.10: Update `update_bar` signature

Add the shield query parameters:

```rust
fn update_bar(
    parts: &mut BarParts,
    info: &UnitBarInfo,
    should_show: bool,
    root_xform_vis: &mut Query<(&mut Transform, &mut Visibility), (With<BarRoot>, Without<HpFill>, Without<ExpFill>)>,
    text_q: &mut Query<&mut Text2d>,
    shield_fill_q: &mut Query<(&mut Sprite, &mut Transform), (With<ShieldFill>, Without<HpFill>, Without<ExpFill>)>,
    hp_fill_q: &mut Query<(&mut Sprite, &mut Transform), (With<HpFill>, Without<ExpFill>)>,
    exp_fill_q: &mut Query<(&mut Sprite, &mut Transform), (With<ExpFill>, Without<HpFill>)>,
)
```

### Step 4.11: Verify

Run: `cargo build -p render_view`
Expected: compiles cleanly.

### Step 4.12: Commit

```bash
git add crates/render_view/src/unit_info_bar.rs
git commit -m "feat(render): add white shield HP bar above HP bar"
```

---

## Task 5: Cleanup and Final Verification

**Files:**
- Modify: `render_view/src/unit_info_bar.rs` (if needed)

### Step 5.1: Run full build

Run: `cargo build`
Expected: entire workspace compiles cleanly.

### Step 5.2: Run all tests

Run: `cargo test -p simulation`
Expected: all 68 tests still pass.

### Step 5.3: Verify no reverse dependencies

Check that render_view does NOT export anything that simulation imports. The dependency should be one-way only.

### Step 5.4: Commit any final fixes

```bash
git add -A
git commit -m "fix(render): cleanup and final verification for visual feedback"
```
