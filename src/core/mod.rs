use bevy::prelude::*;

// ===== 阵营 =====
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Faction {
    Player,
    Enemy,
    Neutral,
}

impl Faction {
    pub fn spawn_type(&self) -> SoldierType {
        SoldierType::Militia
    }
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, States, Default)]
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

/// Range at which a soldier will seek out enemies to engage (larger than attack range for melee)
pub fn soldier_aggression_range(soldier_type: SoldierType) -> f32 {
    match soldier_type {
        SoldierType::Archer => 600.0,   // same as attack range
        SoldierType::Cavalry => 200.0,  // fast, wide awareness
        _ => 150.0,                      // Militia, Infantry
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
// 减速最大叠层上限（保证移速不低于原始速度 35%）
pub const MAX_SLOW_STACKS: u32 = 9;

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

pub fn city_heal_amount(_max_health: f32, current_health: f32) -> f32 {
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
