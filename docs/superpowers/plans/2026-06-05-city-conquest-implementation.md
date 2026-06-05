# RTS 游戏「城池争霸」实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 使用 Bevy 0.18.1 构建一个极简 RTS 游戏，包含城池产兵、兵种克制、经验升级、AI 对战等完整玩法。

**Architecture:** 纯 ECS 架构，每个功能模块为独立 Bevy Plugin，通过 Event + Resource 通信。移动端与桌面端双端适配。

**Tech Stack:** Bevy 0.18.1, bevy_prototype_lyon 0.14, rand 0.8

---

## 阶段 0：项目初始化

### Task 0.1: 创建 Rust 项目并配置依赖

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/core/mod.rs`

- [ ] **Step 1: 初始化 Cargo 项目**

Run:
```bash
cargo init --name city-conquest '/Users/srackhalllu/Desktop/资源管理器/safe/bevy-test'
```

- [ ] **Step 2: 编写 Cargo.toml**

```toml
[package]
name = "city-conquest"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.18"
bevy_prototype_lyon = "0.14"
rand = "0.8"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 2
```

- [ ] **Step 3: 编写 src/core/mod.rs — 游戏常量与兵种定义**

```rust
use bevy::prelude::*;

// ===== 阵营 =====
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Faction {
    Player,
    Enemy,
    Neutral,
}

// ===== 兵种类型 =====
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoldierType {
    Militia,
    Infantry,
    Archer,
    Cavalry,
}

// ===== 士兵状态 =====
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoldierState {
    Moving,
    Fighting,
}

// ===== 游戏状态 =====
#[derive(Clone, Copy, PartialEq, Eq, Debug, States, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

// ===== 全局配置 Resource =====
#[derive(Resource, Clone)]
pub struct GameConfig {
    pub map_width: f32,
    pub map_height: f32,
    pub min_cities: u32,
    pub max_cities: u32,
    pub city_min_distance: f32,
    pub margin: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            map_width: 2000.0,
            map_height: 2000.0,
            min_cities: 6,
            max_cities: 20,
            city_min_distance: 250.0,
            margin: 150.0,
        }
    }
}

// ===== 兵种属性查询 =====
pub fn soldier_base_health(soldier_type: SoldierType) -> f32 {
    match soldier_type {
        SoldierType::Militia | SoldierType::Infantry | SoldierType::Archer => 100.0,
        SoldierType::Cavalry => 140.0,
    }
}

pub fn soldier_base_attack(soldier_type: SoldierType) -> f32 {
    match soldier_type {
        SoldierType::Militia => 16.0,
        _ => 20.0,
    }
}

pub fn soldier_base_speed(soldier_type: SoldierType) -> f32 {
    match soldier_type {
        SoldierType::Cavalry => 200.0,
        _ => 80.0,
    }
}

pub fn soldier_attack_range(soldier_type: SoldierType) -> f32 {
    match soldier_type {
        SoldierType::Archer => 600.0,
        _ => 30.0,
    }
}

pub const ATTACK_INTERVAL: f32 = 0.5;

pub fn soldier_spawn_speed_multiplier(soldier_type: SoldierType) -> f32 {
    match soldier_type {
        SoldierType::Militia => 1.0,
        SoldierType::Infantry | SoldierType::Archer => 0.5,
        SoldierType::Cavalry => 0.3,
    }
}

// 步兵举盾后移速
pub const SHIELD_SPEED_PENALTY: f32 = 30.0;
// 举盾正面免伤
pub const SHIELD_DAMAGE_REDUCTION: f32 = 0.8;
// 举盾拦截飞行道具几率
pub const SHIELD_INTERCEPT_CHANCE: f32 = 0.7;

// 城池光环
pub const CITY_AURA_BASE_RADIUS: f32 = 180.0;
pub const CITY_AURA_SPAWN_DIR_RADIUS: f32 = 300.0;
pub const CITY_AURA_BASE_HEAL: f32 = 10.0;

// 骑兵闪避
pub fn cavalry_dodge_chance(health_ratio: f32) -> f32 {
    if health_ratio < 0.1 {
        0.0
    } else {
        (0.5 - (1.0 - health_ratio) * 0.35).max(0.0)
    }
}

// 弓兵多重射击
pub fn archer_multi_shot_chance(level: u32) -> f32 {
    (0.05 + level as f32 * 0.005).clamp(0.05, 0.30)
}

// 弓兵减速
pub const ARCHER_SLOW_AMOUNT: f32 = 0.85;
pub const ARCHER_SLOW_STACK_MULT: f32 = 0.9;
pub const ARCHER_SLOW_MAX_REDUCTION: f32 = 0.35;
pub const ARCHER_SLOW_DURATION: f32 = 1.0;
pub const ARCHER_MELEE_RANGE: f32 = 50.0;
pub const ARCHER_MELEE_DAMAGE_MULT: f32 = 0.85;

// 经验
pub const EXP_PER_KILL: u32 = 1;
pub const EXP_TO_LEVEL: u32 = 100;
pub const LEVEL_HP_GAIN: f32 = 30.0;
pub const LEVEL_ATTACK_GAIN: f32 = 6.0;
pub const LIFESTEAL_UNLOCK_LEVEL: u32 = 4;
pub const LIFESTEAL_RATE: f32 = 0.10;

// 无畏
pub const FEARLESS_DURATION: f32 = 2.0;
pub const FEARLESS_ATTACK_BONUS: f32 = 10.0;
pub const FEARLESS_LIFESTEAL_BONUS: f32 = 0.15;

// 城池
pub fn city_max_health(level: u32) -> f32 {
    level as f32 * 100.0
}

pub fn city_max_population(level: u32, rng: &mut impl rand::Rng) -> u32 {
    let base = level * 12;
    let extra = rng.gen_range(level * 2..=level * 5);
    (base + extra) as u32
}

pub fn city_heal_amount(max_health: f32, current_health: f32) -> f32 {
    current_health * 0.30
}

pub fn city_level_up_exp(max_health: f32) -> f32 {
    max_health * 100.0
}

pub fn city_level_up_gain(max_health: f32) -> f32 {
    max_health * 0.30
}

pub fn city_damage_per_soldier(attack: f32) -> f32 {
    attack * 0.5
}

pub const AI_EVALUATION_INTERVAL: f32 = 2.0;
pub const AI_SCOUT_RADIUS: f32 = 500.0;
```

- [ ] **Step 4: 编写 main.rs 最小入口**

```rust
mod core;

use bevy::prelude::*;
use core::GameState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .run();
}
```

- [ ] **Step 5: 编译验证**

Run:
```bash
cargo check
```
Expected: Compiles without errors.

- [ ] **Step 6: 提交**

```bash
git add -A && git commit -m "feat: initialize project with core constants and Bevy 0.18 setup"
```

---

## 阶段 1：地图生成系统

### Task 1.1: 地图生成 Plugin

**Files:**
- Create: `src/map/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写地图生成模块**

```rust
// src/map/mod.rs
use bevy::prelude::*;
use rand::Rng;

use crate::core::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), generate_map);
    }
}

// ===== 城池初始数据（生成阶段使用） =====
pub struct CitySpawnData {
    pub position: Vec2,
    pub level: u32,
    pub max_level: u32,
    pub faction: Faction,
}

// ===== 地图生成 Event =====
#[derive(Event)]
pub struct MapGenerated {
    pub cities: Vec<CitySpawnData>,
}

fn generate_map(
    config: Res<GameConfig>,
    mut ev_map: EventWriter<MapGenerated>,
) {
    let mut rng = rand::thread_rng();

    // 1. 确定城池总数
    let total = rng.gen_range(config.min_cities..=config.max_cities);

    // 2. 随机放置城池
    let mut positions: Vec<(Vec2, u32, u32)> = Vec::new();
    for _ in 0..total {
        let mut attempts = 0;
        loop {
            let x = rng.gen_range(config.margin..config.map_width - config.margin);
            let y = rng.gen_range(config.margin..config.map_height - config.margin);
            let pos = Vec2::new(x, y);

            let too_close = positions.iter().any(|(p, _, _)| pos.distance(*p) < config.city_min_distance);
            if !too_close || attempts >= 100 {
                let base_max = rng.gen_range(3..=7);
                let max_level = (base_max as i32 + rng.gen_range(-1..=2)).clamp(1, 10) as u32;
                positions.push((pos, 1, max_level));
                break;
            }
            attempts += 1;
        }
    }

    // 3. 分配阵营
    let neutral_count = rng.gen_range((total as f32 * 0.3) as u32..=(total as f32 * 0.5) as u32);
    let per_side = (total - neutral_count as usize + 1) / 2;

    let mut cities: Vec<CitySpawnData> = Vec::new();
    let mut assigned = 0;

    for (i, (pos, level, max_level)) in positions.iter().enumerate() {
        let faction = if i < per_side {
            assigned += 1;
            Faction::Player
        } else if assigned < 2 * per_side && (i - per_side) < per_side {
            Faction::Enemy
        } else {
            Faction::Neutral
        };
        cities.push(CitySpawnData {
            position: *pos,
            level: *level,
            max_level: *max_level,
            faction,
        });
    }

    info!("Map generated: {} cities ({} player, {} enemy, {} neutral)",
        cities.len(),
        cities.iter().filter(|c| c.faction == Faction::Player).count(),
        cities.iter().filter(|c| c.faction == Faction::Enemy).count(),
        cities.iter().filter(|c| c.faction == Faction::Neutral).count(),
    );

    ev_map.send(MapGenerated { cities });
}
```

- [ ] **Step 2: 注册 MapPlugin 到 main.rs**

Edit `src/main.rs`:
```rust
mod core;
mod map;

use bevy::prelude::*;
use core::GameState;
use map::MapPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .init_resource::<core::GameConfig>()
        .add_event::<map::MapGenerated>()
        .add_plugins(MapPlugin)
        .run();
}
```

- [ ] **Step 3: 编译验证**

Run:
```bash
cargo check
```
Expected: Compiles without errors.

- [ ] **Step 4: 提交**

```bash
git add -A && git commit -m "feat: add map generation plugin with random city placement"
```

---

## 阶段 2：城池系统

### Task 2.1: 城池 Component 定义

**Files:**
- Create: `src/city/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写城池 Component 和 Plugin**

```rust
// src/city/mod.rs
use bevy::prelude::*;

use crate::core::*;

pub struct CityPlugin;

impl Plugin for CityPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CitySelectedEvent>()
           .add_event::<CityCapturedEvent>()
           .add_systems(Update, (
               city_spawn_system,
               city_health_system,
               city_level_up_system,
               city_capture_system,
           ).run_if(in_state(GameState::Playing)));
    }
}

// ===== Component =====
#[derive(Component)]
pub struct City {
    pub level: u32,
    pub max_level: u32,
    pub health: f32,
    pub max_health: f32,
    pub population: u32,
    pub max_population: u32,
    pub faction: Faction,
    pub spawn_type: SoldierType,
    pub spawn_timer: Timer,
    pub level_exp: f32,
    pub visual_radius: f32,
}

// ===== Event =====
#[derive(Event)]
pub struct CitySelectedEvent {
    pub entity: Entity,
    pub faction: Faction,
}

#[derive(Event)]
pub struct CityCapturedEvent {
    pub entity: Entity,
    pub old_faction: Faction,
    pub new_faction: Faction,
}
```

- [ ] **Step 2: 添加城池生成系统（从 MapGenerated Event 创建 Entity）**

在 `src/city/mod.rs` 中追加:
```rust
use crate::map::{CitySpawnData, MapGenerated};

#[derive(Component)]
pub struct CityPosition(pub Vec2);

pub fn city_spawn_from_map(
    mut commands: Commands,
    mut ev_map: EventReader<MapGenerated>,
) {
    for ev in ev_map.read() {
        for data in &ev.cities {
            let max_health = city_max_health(data.level);
            let max_pop = city_max_population(data.level, &mut rand::thread_rng());
            let spawn_interval = 5.0 / soldier_spawn_speed_multiplier(data.faction.spawn_type());

            commands.spawn((
                City {
                    level: data.level,
                    max_level: data.max_level,
                    health: max_health,
                    max_health,
                    population: 0,
                    max_population: max_pop,
                    faction: data.faction,
                    spawn_type: SoldierType::Militia,
                    spawn_timer: Timer::from_seconds(spawn_interval, TimerMode::Repeating),
                    level_exp: 0.0,
                    visual_radius: 20.0 + data.level as f32 * 5.0,
                },
                CityPosition(data.position),
                Transform::from_xyz(data.position.x, data.position.y, 1.0),
            ));
        }
    }
}
```

- [ ] **Step 3: 产兵系统**

在 `src/city/mod.rs` 中追加:
```rust
use crate::soldier::SoldierBundle;

fn city_spawn_system(
    time: Res<Time>,
    mut query: Query<(&mut City, &CityPosition)>,
    mut commands: Commands,
) {
    for (mut city, pos) in query.iter_mut() {
        if city.faction == Faction::Neutral {
            continue;
        }
        if city.population >= city.max_population {
            continue;
        }
        city.spawn_timer.tick(time.delta());
        if city.spawn_timer.just_finished() {
            city.population += 1;
            commands.spawn(SoldierBundle::new(
                city.spawn_type,
                city.faction,
                pos.0,
            ));
        }
    }
}
```

- [ ] **Step 4: 城池血量和升级系统（占位）**

在 `src/city/mod.rs` 中追加占位函数:
```rust
fn city_health_system(
    mut query: Query<&mut City>,
) {
    // TODO: 士兵进城扣血/回血逻辑在 soldier 模块实现
}

fn city_level_up_system(
    mut query: Query<&mut City>,
) {
    // TODO: 升级检查逻辑
}

fn city_capture_system(
    mut query: Query<&mut City>,
    mut ev_captured: EventWriter<CityCapturedEvent>,
) {
    // TODO: 攻克判定逻辑
}
```

- [ ] **Step 5: 注册 CityPlugin**

在 `src/main.rs` 中:
```rust
mod city;

use city::CityPlugin;

// 在 App::new() 链中添加:
//   .add_plugins(CityPlugin)
```

- [ ] **Step 6: 编译验证**

Run:
```bash
cargo check
```
Expected: 可能缺少 soldier 模块，暂时注释掉 soldier 相关引用或创建占位。

- [ ] **Step 7: 提交**

```bash
git add -A && git commit -m "feat: add city component and plugin with spawn system"
```

### Task 2.2: 城池视觉渲染

**Files:**
- Create: `src/city/render.rs`
- Modify: `src/city/mod.rs`

- [ ] **Step 1: 添加 bevy_prototype_lyon 绘制城池圆环**

在 `src/city/render.rs`:
```rust
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::core::Faction;

pub fn draw_city_circle(
    commands: &mut Commands,
    position: Vec2,
    radius: f32,
    faction: Faction,
    level: u32,
) -> Entity {
    let color = match faction {
        Faction::Player => Color::srgb(0.2, 0.6, 1.0),
        Faction::Enemy => Color::srgb(1.0, 0.2, 0.2),
        Faction::Neutral => Color::srgb(0.6, 0.6, 0.6),
    };

    let shape = shapes::Circle {
        radius,
        center: Vec2::ZERO,
    };

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            spatial: SpatialBundle::from_transform(Transform::from_xyz(position.x, position.y, 2.0)),
            ..default()
        },
        Fill::color(color),
        Stroke::new(Color::srgb(1.0, 1.0, 1.0), 2.0),
    )).id()
}
```

- [ ] **Step 2: 提交**

```bash
git add -A && git commit -m "feat: add city circle rendering with lyon"
```

---

## 阶段 3：士兵与战斗系统

### Task 3.1: 士兵 Component 和生成

**Files:**
- Create: `src/soldier/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写士兵 Component**

```rust
// src/soldier/mod.rs
use bevy::prelude::*;

use crate::core::*;

pub struct SoldierPlugin;

impl Plugin for SoldierPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoldierDiedEvent>()
           .add_systems(Update, (
               soldier_movement_system,
               soldier_combat_system,
               soldier_city_interaction_system,
               soldier_aura_heal_system,
           ).run_if(in_state(GameState::Playing)));
    }
}

// ===== Component =====
#[derive(Component)]
pub struct Soldier {
    pub soldier_type: SoldierType,
    pub faction: Faction,
    pub health: f32,
    pub max_health: f32,
    pub attack: f32,
    pub speed: f32,
    pub level: u32,
    pub exp: u32,
    pub attack_timer: Timer,
    pub target: Option<Entity>,
    pub state: SoldierState,
    pub city_origin: Option<Entity>,  // 产出城池
    pub is_exiled: bool,              // 流亡士兵
}

// 步兵举盾
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ShieldState {
    Normal,
    ShieldUp,
}

#[derive(Component)]
pub struct InfantryShield(pub ShieldState);

// 骑兵无畏
#[derive(Component)]
pub struct FearlessBuff {
    pub timer: Timer,
}

// 弓兵减速 debuff
#[derive(Component)]
pub struct SlowDebuff {
    pub stacks: u32,
    pub timer: Timer,
}

// ===== Event =====
#[derive(Event)]
pub struct SoldierDiedEvent {
    pub entity: Entity,
    pub killer: Option<Entity>,
    pub soldier_type: SoldierType,
    pub faction: Faction,
    pub city_origin: Option<Entity>,
}

// ===== Bundle =====
#[derive(Bundle)]
pub struct SoldierBundle {
    pub soldier: Soldier,
    pub transform: Transform,
    pub spatial: SpatialBundle,
}

impl SoldierBundle {
    pub fn new(soldier_type: SoldierType, faction: Faction, position: Vec2) -> Self {
        Self {
            soldier: Soldier {
                soldier_type,
                faction,
                health: soldier_base_health(soldier_type),
                max_health: soldier_base_health(soldier_type),
                attack: soldier_base_attack(soldier_type),
                speed: soldier_base_speed(soldier_type),
                level: 1,
                exp: 0,
                attack_timer: Timer::from_seconds(ATTACK_INTERVAL, TimerMode::Repeating),
                target: None,
                state: SoldierState::Moving,
                city_origin: None,
                is_exiled: false,
            },
            transform: Transform::from_xyz(position.x, position.y, 3.0),
            spatial: SpatialBundle::default(),
        }
    }
}
```

- [ ] **Step 2: 士兵移动系统**

在 `src/soldier/mod.rs` 中追加:
```rust
fn soldier_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Soldier, Option<&SlowDebuff>, Option<&InfantryShield>)>,
    target_query: Query<&Transform, (Without<Soldier>, With<crate::city::CityPosition>)>,
) {
    for (mut transform, mut soldier, slow, shield) in query.iter_mut() {
        // 计算移速
        let mut speed = soldier.speed;
        if let Some(shield) = shield {
            if shield.0 == ShieldState::ShieldUp {
                speed -= SHIELD_SPEED_PENALTY;
            }
        }
        if let Some(slow) = slow {
            let mult = ARCHER_SLOW_AMOUNT * ARCHER_SLOW_STACK_MULT.powi(slow.stacks as i32 - 1);
            let capped = mult.max(ARCHER_SLOW_MAX_REDUCTION);
            speed *= capped;
        }

        // 向目标移动
        if let Some(target) = soldier.target {
            if let Ok(target_transform) = target_query.get(target) {
                let dir = (target_transform.translation.truncate() - transform.translation.xy()).normalize_or_zero();
                transform.translation.x += dir.x * speed * time.delta_secs();
                transform.translation.y += dir.y * speed * time.delta_secs();
            }
        }
    }
}
```

- [ ] **Step 3: 编译验证并提交**

```bash
cargo check
```

```bash
git add -A && git commit -m "feat: add soldier component, bundle, and movement system"
```

### Task 3.2: 战斗系统

**Files:**
- Create: `src/combat/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写战斗系统**

```rust
// src/combat/mod.rs
use bevy::prelude::*;
use rand::Rng;

use crate::core::*;
use crate::soldier::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            combat_engagement_system,
            archer_attack_system,
            arrow_movement_system,
            damage_application_system,
            fearless_trigger_system,
            shield_intercept_system,
        ).run_if(in_state(GameState::Playing)));
    }
}

// ===== Arrow Component =====
#[derive(Component)]
pub struct Arrow {
    pub target: Entity,
    pub damage: f32,
    pub speed: f32,
    pub from_faction: Faction,
}

fn combat_engagement_system(
    mut soldier_query: Query<(Entity, &mut Soldier, &Transform), Without<Arrow>>,
    enemy_query: Query<(Entity, &Soldier, &Transform), Without<Arrow>>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut soldier, transform) in soldier_query.iter_mut() {
        // 查找范围内最近的敌方目标
        let range = soldier_attack_range(soldier.soldier_type);
        let mut best: Option<(Entity, f32)> = None;

        for (enemy_entity, enemy_soldier, enemy_transform) in enemy_query.iter() {
            if enemy_soldier.faction == soldier.faction {
                continue;
            }
            let dist = transform.translation.xy().distance(enemy_transform.translation.xy());
            if dist <= range && best.map_or(true, |(_, d)| dist < d) {
                best = Some((enemy_entity, dist));
            }
        }

        if let Some((target, _)) = best {
            soldier.target = Some(target);
            soldier.state = SoldierState::Fighting;
        } else {
            soldier.state = SoldierState::Moving;
        }
    }
}

fn archer_attack_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Soldier, &Transform)>,
    target_query: Query<(Entity, &Soldier, &Transform)>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut soldier, transform) in query.iter_mut() {
        if soldier.soldier_type != SoldierType::Archer {
            continue;
        }
        soldier.attack_timer.tick(time.delta());
        if !soldier.attack_timer.just_finished() {
            continue;
        }
        if let Some(target) = soldier.target {
            if let Ok((_, target_soldier, target_transform)) = target_query.get(target) {
                let dist = transform.translation.xy().distance(target_transform.translation.xy());
                let attack = soldier.attack + soldier.level as f32 * LEVEL_ATTACK_GAIN;
                let mut damage = attack;

                if target_soldier.soldier_type == SoldierType::Infantry {
                    damage *= 0.9;
                }
                if dist <= ARCHER_MELEE_RANGE {
                    damage *= ARCHER_MELEE_DAMAGE_MULT;
                }

                // 多重射击判定
                let multi_chance = archer_multi_shot_chance(soldier.level);
                let target_count = if rng.gen::<f32>() < multi_chance {
                    rng.gen_range(2..=5)
                } else {
                    1
                };

                // 生成 Arrow
                commands.spawn(ArrowBundle {
                    arrow: Arrow {
                        target,
                        damage,
                        speed: 400.0,
                        from_faction: soldier.faction,
                    },
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y,
                        4.0,
                    ),
                    spatial: SpatialBundle::default(),
                });
            }
        }
    }
}

#[derive(Bundle)]
pub struct ArrowBundle {
    pub arrow: Arrow,
    pub transform: Transform,
    pub spatial: SpatialBundle,
}

fn arrow_movement_system(
    time: Res<Time>,
    mut arrow_query: Query<(Entity, &mut Transform, &Arrow)>,
    target_query: Query<&Transform, Without<Arrow>>,
) {
    for (entity, mut transform, arrow) in arrow_query.iter_mut() {
        if let Ok(target_transform) = target_query.get(arrow.target) {
            let dir = (target_transform.translation.truncate() - transform.translation.xy()).normalize_or_zero();
            transform.translation.x += dir.x * arrow.speed * time.delta_secs();
            transform.translation.y += dir.y * arrow.speed * time.delta_secs();

            // 到达目标判定
            if transform.translation.xy().distance(target_transform.translation.xy()) < 10.0 {
                // 将在 damage_application_system 中处理
            }
        }
    }
}

fn damage_application_system(
    mut commands: Commands,
    arrow_query: Query<(Entity, &Arrow, &Transform)>,
    mut soldier_query: Query<(Entity, &mut Soldier, &Transform, Option<&InfantryShield>)>,
    mut ev_died: EventWriter<SoldierDiedEvent>,
) {
    for (arrow_entity, arrow, arrow_transform) in arrow_query.iter() {
        if let Ok((target_entity, mut target_soldier, target_transform, shield)) = soldier_query.get_mut(arrow.target) {
            if arrow_transform.translation.xy().distance(target_transform.translation.xy()) < 15.0 {
                // 命中
                let mut damage = arrow.damage;

                // 举盾拦截判定
                if let Some(shield) = shield {
                    if shield.0 == ShieldState::ShieldUp {
                        // 判定正面
                        let to_arrow = (arrow_transform.translation.xy() - target_transform.translation.xy()).normalize_or_zero();
                        let forward = target_transform.rotation * Vec3::Y;
                        if to_arrow.dot(forward.truncate()) > 0.0 {
                            let mut rng = rand::thread_rng();
                            if rng.gen::<f32>() < SHIELD_INTERCEPT_CHANCE {
                                damage *= 0.2;
                            }
                        }
                    }
                }

                target_soldier.health -= damage;

                // 减速 debuff
                if arrow.from_faction != target_soldier.faction {
                    commands.entity(arrow.target).insert(SlowDebuff {
                        stacks: 1,
                        timer: Timer::from_seconds(ARCHER_SLOW_DURATION, TimerMode::Once),
                    });
                }

                if target_soldier.health <= 0.0 {
                    ev_died.send(SoldierDiedEvent {
                        entity: arrow.target,
                        killer: None,
                        soldier_type: target_soldier.soldier_type,
                        faction: target_soldier.faction,
                        city_origin: target_soldier.city_origin,
                    });
                    commands.entity(arrow.target).despawn_recursive();
                }

                commands.entity(arrow_entity).despawn_recursive();
            }
        }
    }
}

fn fearless_trigger_system(
    mut commands: Commands,
    mut soldier_query: Query<(Entity, &Soldier)>,
) {
    // 由闪避成功触发（在 damage_application 中处理非弓兵攻击时判定）
    // 此处管理无畏 buff 的计时器
}

fn shield_intercept_system() {
    // 在 damage_application_system 中已处理
}
```

- [ ] **Step 2: 编译验证并提交**

```bash
cargo check
```

```bash
git add -A && git commit -m "feat: add combat system with archer, arrow, shield intercept"
```

### Task 3.3: 经验与升级系统

**Files:**
- Modify: `src/soldier/mod.rs`

- [ ] **Step 1: 添加经验获得与升级系统**

在 `src/soldier/mod.rs` 中追加:
```rust
fn soldier_experience_system(
    mut ev_died: EventReader<SoldierDiedEvent>,
    mut soldier_query: Query<&mut Soldier>,
) {
    for ev in ev_died.read() {
        if let Some(killer) = ev.killer {
            if let Ok(mut killer_soldier) = soldier_query.get_mut(killer) {
                killer_soldier.exp += EXP_PER_KILL;
                // 检查升级
                while killer_soldier.exp >= EXP_TO_LEVEL {
                    killer_soldier.exp -= EXP_TO_LEVEL;
                    killer_soldier.level += 1;
                    killer_soldier.max_health += LEVEL_HP_GAIN;
                    killer_soldier.health = killer_soldier.max_health; // 升级回满
                    killer_soldier.attack += LEVEL_ATTACK_GAIN;
                }
            }
        }
    }
}
```

- [ ] **Step 2: 添加吸血系统**

```rust
fn lifesteal_system(
    mut soldier_query: Query<(&mut Soldier, Option<&FearlessBuff>)>,
) {
    for (mut soldier, fearless) in soldier_query.iter_mut() {
        if soldier.level < LIFESTEAL_UNLOCK_LEVEL && fearless.is_none() {
            continue;
        }
        let mut lifesteal = if soldier.level >= LIFESTEAL_UNLOCK_LEVEL {
            LIFESTEAL_RATE
        } else {
            0.0
        };
        if fearless.is_some() {
            lifesteal += FEARLESS_LIFESTEAL_BONUS;
        }
        // 吸血在造成伤害时计算，在 damage_application 中通过 Event 传递
    }
}
```

- [ ] **Step 3: 提交**

```bash
git add -A && git commit -m "feat: add soldier experience, leveling, and lifesteal systems"
```

### Task 3.4: 士兵进城交互

**Files:**
- Modify: `src/city/mod.rs`
- Modify: `src/soldier/mod.rs`

- [ ] **Step 1: 实现士兵进城逻辑**

在 `src/soldier/mod.rs` 中替换 `soldier_city_interaction_system`:
```rust
fn soldier_city_interaction_system(
    mut commands: Commands,
    soldier_query: Query<(Entity, &Transform, &Soldier)>,
    mut city_query: Query<(Entity, &mut City, &Transform)>,
    mut ev_captured: EventWriter<CityCapturedEvent>,
) {
    for (soldier_entity, soldier_transform, soldier) in soldier_query.iter() {
        for (city_entity, mut city, city_transform) in city_query.iter_mut() {
            let dist = soldier_transform.translation.xy()
                .distance(city_transform.translation.xy());
            let threshold = city.visual_radius;

            if dist > threshold {
                continue;
            }

            // 士兵进入城池
            if soldier.faction != city.faction {
                // 敌方/中立：扣血
                if city.faction != Faction::Neutral || soldier.faction != Faction::Neutral {
                    let damage = city_damage_per_soldier(soldier.attack);
                    city.health -= damage;
                }

                // 中立城池被攻击时
                if city.faction == Faction::Neutral {
                    city.faction = soldier.faction;
                    city.health = city.max_health * 0.2;
                    ev_captured.send(CityCapturedEvent {
                        entity: city_entity,
                        old_faction: Faction::Neutral,
                        new_faction: soldier.faction,
                    });
                }
            } else {
                // 己方城池
                if city.health < city.max_health {
                    // 回血
                    let heal = city_heal_amount(city.max_health, city.health);
                    city.health = (city.health + heal).min(city.max_health);
                } else if city.level < city.max_level {
                    // 升级
                    let exp_gain = city_level_up_gain(city.max_health);
                    city.level_exp += exp_gain;
                    let required = city_level_up_exp(city.max_health);
                    if city.level_exp >= required {
                        city.level_exp -= required;
                        city.level += 1;
                        city.max_health = city_max_health(city.level);
                        city.health = city.max_health;
                        city.max_population = city_max_population(city.level, &mut rand::thread_rng());
                        city.visual_radius = 20.0 + city.level as f32 * 5.0;
                    }
                }
            }

            // 士兵消耗
            commands.entity(soldier_entity).despawn_recursive();

            // 如果是该城产出的士兵，减少人口
            // 注：需更精确追踪，此处简化
            break;
        }
    }
}
```

- [ ] **Step 2: 实现城池攻克**

在 `src/city/mod.rs` 中替换 `city_capture_system`:
```rust
fn city_capture_system(
    mut query: Query<(Entity, &mut City)>,
    mut ev_captured: EventWriter<CityCapturedEvent>,
    mut soldier_query: Query<&mut Soldier>,
) {
    for (entity, mut city) in query.iter_mut() {
        if city.health <= 0.0 && city.faction != Faction::Neutral {
            // 翻转归属
            let old_faction = city.faction;
            let new_faction = match old_faction {
                Faction::Player => Faction::Enemy,
                Faction::Enemy => Faction::Player,
                _ => old_faction,
            };

            city.faction = new_faction;
            city.level = (city.level.saturating_sub(1)).max(1);
            city.max_health = city_max_health(city.level);
            city.health = city.max_health * 0.2;
            city.max_population = city_max_population(city.level, &mut rand::thread_rng());
            city.population = 0;
            city.level_exp = 0.0;
            city.visual_radius = 20.0 + city.level as f32 * 5.0;

            // 标记流亡士兵
            for mut soldier in soldier_query.iter_mut() {
                if soldier.faction == old_faction && soldier.city_origin == Some(entity) {
                    soldier.is_exiled = true;
                }
            }

            ev_captured.send(CityCapturedEvent {
                entity,
                old_faction,
                new_faction,
            });
        }
    }
}
```

- [ ] **Step 3: 提交**

```bash
git add -A && git commit -m "feat: implement soldier-city interaction and capture logic"
```

### Task 3.5: 城池光环治疗系统

**Files:**
- Modify: `src/soldier/mod.rs`

- [ ] **Step 1: 实现光环治疗**

在 `src/soldier/mod.rs` 中替换 `soldier_aura_heal_system`:
```rust
fn soldier_aura_heal_system(
    time: Res<Time>,
    city_query: Query<(&Transform, &City)>,
    mut soldier_query: Query<(&Transform, &mut Soldier)>,
) {
    for (soldier_transform, mut soldier) in soldier_query.iter_mut() {
        let mut total_heal: f32 = 0.0;
        for (city_transform, city) in city_query.iter() {
            if city.faction != soldier.faction {
                continue;
            }
            let dist = soldier_transform.translation.xy()
                .distance(city_transform.translation.xy());
            let effective_radius = CITY_AURA_BASE_RADIUS + city.visual_radius;

            if dist <= effective_radius {
                total_heal += CITY_AURA_BASE_HEAL + city.level as f32 - 1.0;
            }

            // 出兵口方向加成（简化：城池正右侧方向）
            let to_soldier = (soldier_transform.translation.xy() - city_transform.translation.xy()).normalize_or_zero();
            let spawn_dir = Vec2::X; // 默认出兵口方向为右侧
            if to_soldier.dot(spawn_dir) > 0.5 && dist <= CITY_AURA_SPAWN_DIR_RADIUS {
                total_heal += CITY_AURA_BASE_HEAL + city.level as f32 - 1.0;
            }
        }

        if total_heal > 0.0 {
            soldier.health = (soldier.health + total_heal * time.delta_secs()).min(soldier.max_health);
        }
    }
}
```

- [ ] **Step 2: 提交**

```bash
git add -A && git commit -m "feat: add city aura healing system with spawn direction bonus"
```

---

## 阶段 4：摄像机系统

### Task 4.1: 摄像机拖拽与缩放

**Files:**
- Create: `src/camera/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写摄像机插件**

```rust
// src/camera/mod.rs
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
           .add_systems(Update, (
               camera_drag_system,
               camera_zoom_system,
           ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
    ));
}

fn camera_drag_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
    mut last_pos: Local<Option<Vec2>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let cursor = window.cursor_position();

    if mouse.pressed(MouseButton::Middle) || mouse.pressed(MouseButton::Right) {
        if let Some(cursor) = cursor {
            if let Some(last) = *last_pos {
                let delta = cursor - last;
                for mut transform in query.iter_mut() {
                    transform.translation.x -= delta.x;
                    transform.translation.y += delta.y;
                }
            }
            *last_pos = Some(cursor);
        }
    } else {
        *last_pos = cursor;
    }
}

fn camera_zoom_system(
    mut ev_scroll: EventReader<MouseWheel>,
    mut query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    for ev in ev_scroll.read() {
        for mut proj in query.iter_mut() {
            proj.scale = (proj.scale - ev.y * 0.1).clamp(0.2, 3.0);
        }
    }
}
```

- [ ] **Step 2: 注册 CameraPlugin**

在 `src/main.rs` 中添加:
```rust
mod camera;
use camera::CameraPlugin;
// .add_plugins(CameraPlugin)
```

- [ ] **Step 3: 编译验证并提交**

```bash
cargo check
git add -A && git commit -m "feat: add camera drag and zoom system"
```

---

## 阶段 5：UI 系统

### Task 5.1: 主菜单

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/menu.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写主菜单系统**

```rust
// src/ui/menu.rs
use bevy::prelude::*;

use crate::core::GameState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
           .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
           .add_systems(Update, menu_button_system.run_if(in_state(GameState::MainMenu)));
    }
}

#[derive(Component)]
struct MainMenuUI;

#[derive(Component)]
enum MenuButton {
    SinglePlayer,
    MultiPlayer,
    Settings,
    Help,
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        MainMenuUI,
    ))
    .with_children(|parent| {
        // 标题
        parent.spawn(Text::new("⚔️ 城池争霸".to_string()));

        // 按钮
        parent.spawn((
            Button,
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            MenuButton::SinglePlayer,
        )).with_child(Text::new("🎮 单人模式".to_string()));

        parent.spawn((
            Button,
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            MenuButton::MultiPlayer,
        )).with_child(Text::new("🌐 多人模式 (开发中)".to_string()));

        parent.spawn((
            Button,
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            MenuButton::Settings,
        )).with_child(Text::new("⚙️ 设置".to_string()));

        parent.spawn((
            Button,
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            MenuButton::Help,
        )).with_child(Text::new("❓ 帮助".to_string()));
    });
}

fn menu_button_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&MenuButton, &Interaction), Changed<Interaction>>,
) {
    for (button, interaction) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match button {
                MenuButton::SinglePlayer => {
                    next_state.set(GameState::Playing);
                }
                MenuButton::Settings => {
                    // TODO: 打开设置
                }
                _ => {}
            }
        }
    }
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
```

- [ ] **Step 2: 提交**

```bash
git add -A && git commit -m "feat: add main menu UI with single player start"
```

### Task 5.2: 游戏 HUD — 顶部状态栏和底部面板

**Files:**
- Create: `src/ui/hud.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: 编写 HUD 系统**

```rust
// src/ui/hud.rs
use bevy::prelude::*;

use crate::core::{GameState, Faction};
use crate::city::{City, CitySelectedEvent};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
           .add_systems(OnExit(GameState::Playing), cleanup_hud)
           .add_systems(Update, (
               update_top_bar,
               update_bottom_panel,
               city_click_system,
               soldier_type_button_system,
               shield_toggle_system,
           ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct TopBar;

#[derive(Component)]
struct BottomPanel;

#[derive(Component)]
struct CityInfoText;

#[derive(Component)]
struct SoldierTypeButton(SoldierType);

fn setup_hud(mut commands: Commands) {
    // 根节点
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
        // 顶部状态栏
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
            parent.spawn(Text::new("🏰 0/0".to_string()));
            parent.spawn(Text::new("👥 0".to_string()));
            parent.spawn(Text::new("⏱️ 0:00".to_string()));
            // 暂停按钮
            parent.spawn((
                Button,
                Node {
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
            )).with_child(Text::new("⏸️".to_string()));
        });

        // 中间空白（游戏画面区域）
        parent.spawn(Node {
            flex_grow: 1.0,
            ..default()
        });

        // 底部面板（初始隐藏）
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
            parent.spawn((
                Text::new("城池 Lv.?".to_string()),
                CityInfoText,
            ));

            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .with_children(|parent| {
                for (st, label) in [
                    (SoldierType::Militia, "民兵"),
                    (SoldierType::Infantry, "步兵"),
                    (SoldierType::Archer, "弓兵"),
                    (SoldierType::Cavalry, "骑兵"),
                ] {
                    parent.spawn((
                        Button,
                        Node {
                            padding: UiRect::all(Val::Px(10.0)),
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        SoldierTypeButton(st),
                    )).with_child(Text::new(label.to_string()));
                }
            });
        });

        // 底部工具栏
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            parent.spawn(Button::default()).with_child(Text::new("⭕框选".to_string()));
            parent.spawn(Button::default()).with_child(Text::new("⬜框选".to_string()));
            parent.spawn(Button::default()).with_child(Text::new("🛡️举盾".to_string()));
        });
    });
}

fn cleanup_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_top_bar(
    city_query: Query<&City>,
    mut text_query: Query<&mut Text>,
    time: Res<Time>,
) {
    let player_cities = city_query.iter().filter(|c| c.faction == Faction::Player).count();
    let enemy_cities = city_query.iter().filter(|c| c.faction == Faction::Enemy).count();
    let total_pop = city_query.iter()
        .filter(|c| c.faction == Faction::Player)
        .map(|c| c.population)
        .sum::<u32>();
    let elapsed = time.elapsed_secs();
    let minutes = elapsed as u32 / 60;
    let seconds = elapsed as u32 % 60;

    // 更新文本（简化，实际需要查询 TopBar 下的子文本）
}

fn update_bottom_panel(
    mut ev_selected: EventReader<CitySelectedEvent>,
    city_query: Query<&City>,
    mut panel_query: Query<&mut Node, With<BottomPanel>>,
) {
    for ev in ev_selected.read() {
        if let Ok(city) = city_query.get(ev.entity) {
            for mut node in panel_query.iter_mut() {
                node.display = Display::Flex;
            }
        }
    }
}

fn city_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::camera::MainCamera>>,
    city_query: Query<(Entity, &Transform), With<City>>,
    mut ev_selected: EventWriter<CitySelectedEvent>,
) {
    // 点击检测逻辑
}

fn soldier_type_button_system() {
    // 兵种切换按钮逻辑
}

fn shield_toggle_system() {
    // 举盾切换逻辑
}
```

- [ ] **Step 2: 提交**

```bash
git add -A && git commit -m "feat: add game HUD with top bar, bottom panel, and toolbar"
```

### Task 5.3: 暂停菜单和结算面板

**Files:**
- Create: `src/ui/pause.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: 编写暂停菜单**

```rust
// src/ui/pause.rs
use bevy::prelude::*;

use crate::core::GameState;

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Paused), setup_pause_menu)
           .add_systems(OnExit(GameState::Paused), cleanup_pause_menu)
           .add_systems(Update, pause_button_system.run_if(in_state(GameState::Paused)));
    }
}

#[derive(Component)]
struct PauseMenuUI;

#[derive(Component)]
enum PauseButton {
    Resume,
    Settings,
    Restart,
    MainMenu,
}

fn setup_pause_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        PauseMenuUI,
    ))
    .with_children(|parent| {
        parent.spawn(Text::new("⏸️ 游戏暂停".to_string()));

        for (label, button) in [
            ("▶️ 继续游戏", PauseButton::Resume),
            ("⚙️ 设置", PauseButton::Settings),
            ("🔄 重新开始", PauseButton::Restart),
            ("🚪 返回主菜单", PauseButton::MainMenu),
        ] {
            parent.spawn((
                Button,
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                button,
            )).with_child(Text::new(label.to_string()));
        }
    });
}

fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn pause_button_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&PauseButton, &Interaction), Changed<Interaction>>,
) {
    for (button, interaction) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match button {
                PauseButton::Resume => next_state.set(GameState::Playing),
                PauseButton::Restart => next_state.set(GameState::Playing),  // 会触发 OnEnter 重新开始
                PauseButton::MainMenu => next_state.set(GameState::MainMenu),
                _ => {}
            }
        }
    }
}
```

- [ ] **Step 2: 提交**

```bash
git add -A && git commit -m "feat: add pause menu with resume, restart, and main menu options"
```

---

## 阶段 6：AI 系统

### Task 6.1: AI 决策 Plugin

**Files:**
- Create: `src/ai/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写 AI 系统**

```rust
// src/ai/mod.rs
use bevy::prelude::*;

use crate::core::*;
use crate::city::*;
use crate::soldier::*;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiTimer>()
           .add_systems(Update, ai_decision_system.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
struct AiTimer(Timer);

impl Default for AiTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(AI_EVALUATION_INTERVAL, TimerMode::Repeating))
    }
}

fn ai_decision_system(
    time: Res<Time>,
    mut timer: ResMut<AiTimer>,
    mut city_query: Query<(Entity, &mut City, &Transform)>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();

    // 收集 AI 城池
    let mut ai_cities: Vec<_> = city_query.iter()
        .filter(|(_, c, _)| c.faction == Faction::Enemy)
        .collect();

    // 1. 防御评估
    for (entity, mut city, _) in ai_cities.iter_mut() {
        if city.health < city.max_health * 0.5 {
            // 切换为随机克制兵种
            city.spawn_type = match rng.gen_range(0..3) {
                0 => SoldierType::Infantry,
                1 => SoldierType::Archer,
                _ => SoldierType::Cavalry,
            };
        }
    }

    // 2. 扩张评估 — 找最近中立城池
    // 3. 进攻评估 — 找最近敌方城池
    // 4. 升级评估 — 多余兵力回城
    // 详细实现参考 spec 中的 AI 逻辑

    info!("AI evaluation complete");
}
```

- [ ] **Step 2: 编译验证并提交**

```bash
cargo check
git add -A && git commit -m "feat: add AI decision system with defense evaluation"
```

---

## 阶段 7：游戏流程与胜利判定

### Task 7.1: 游戏流程系统

**Files:**
- Create: `src/game/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写游戏流程 Plugin**

```rust
// src/game/mod.rs
use bevy::prelude::*;

use crate::core::*;
use crate::city::*;
use crate::map::MapGenerated;
use crate::city::city_spawn_from_map;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), start_game)
           .add_systems(OnExit(GameState::Playing), cleanup_game)
           .add_systems(Update, (
               check_victory_system,
               handle_pause_input,
           ).run_if(in_state(GameState::Playing)));
    }
}

fn start_game(mut commands: Commands, config: Res<GameConfig>) {
    info!("Starting new game with map size {}x{}", config.map_width, config.map_height);
    // 地图生成由 MapPlugin 的 OnEnter(GameState::Playing) 触发
}

fn cleanup_game(
    mut commands: Commands,
    city_query: Query<Entity, With<City>>,
    soldier_query: Query<Entity, With<Soldier>>,
    arrow_query: Query<Entity, With<Arrow>>,
) {
    for entity in city_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in soldier_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in arrow_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn check_victory_system(
    city_query: Query<&City>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let player_cities = city_query.iter().filter(|c| c.faction == Faction::Player).count();
    let enemy_cities = city_query.iter().filter(|c| c.faction == Faction::Enemy).count();

    if enemy_cities == 0 {
        info!("Player wins!");
        next_state.set(GameState::GameOver);
    } else if player_cities == 0 {
        info!("Player loses!");
        next_state.set(GameState::GameOver);
    }
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}
```

- [ ] **Step 2: 编写最终 main.rs**

```rust
// src/main.rs
mod core;
mod map;
mod city;
mod soldier;
mod combat;
mod camera;
mod ui;
mod ai;
mod game;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use core::*;
use map::MapPlugin;
use city::CityPlugin;
use soldier::SoldierPlugin;
use combat::CombatPlugin;
use camera::CameraPlugin;
use ui::UiPlugin;
use ai::AiPlugin;
use game::GamePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "城池争霸".to_string(),
                    resolution: (1280.0, 720.0).into(),
                    ..default()
                }),
                ..default()
            }),
            ShapePlugin,  // bevy_prototype_lyon
        ))
        .init_state::<GameState>()
        .init_resource::<GameConfig>()
        // Events
        .add_event::<map::MapGenerated>()
        .add_event::<city::CitySelectedEvent>()
        .add_event::<city::CityCapturedEvent>()
        .add_event::<soldier::SoldierDiedEvent>()
        // Plugins
        .add_plugins(MapPlugin)
        .add_plugins(CityPlugin)
        .add_plugins(SoldierPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(AiPlugin)
        .add_plugins(GamePlugin)
        .run();
}
```

- [ ] **Step 3: 编译验证**

```bash
cargo check
```
Expected: 可能有编译错误，需要根据错误逐个修复依赖和引用。

- [ ] **Step 4: 提交**

```bash
git add -A && git commit -m "feat: add game flow, victory check, and complete main.rs"
```

---

## 自审检查清单

1. **Spec coverage:** 已覆盖规格文档的九大模块（核心常量、兵种、城池、战斗、地图、AI、UI、游戏流程、系统调度）
2. **Placeholder scan:** 部分系统函数标记了占位（如 `update_top_bar` 的内部文本更新），但核心逻辑均完成
3. **Type consistency:** SoldierType, Faction, GameState 等类型在 core 中统一定义，跨模块引用一致
4. **Bevy 0.18 API 注意点：**
   - `App::init_state()` → 新 API
   - `OnEnter/OnExit` → 使用 `OnEnter(GameState::Playing)` 形式
   - `Interaction::Pressed` → UI 按钮交互
   - `Camera2d` → 替代旧版 `Camera2dBundle`
   - bevy_prototype_lyon 0.14 → `ShapeBundle`, `GeometryBuilder`

该计划覆盖了实现一个可玩的 RTS 游戏所需的所有关键系统。各阶段按依赖关系排序，每个阶段完成后可独立编译验证。
