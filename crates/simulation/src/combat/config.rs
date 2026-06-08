//! Combat configuration — loaded from content/combat.ron

use bevy_ecs::prelude::Resource;
use serde::Deserialize;

/// Infantry shield mechanics.
#[derive(Clone, Debug, Deserialize)]
pub struct ShieldConfig {
    pub speed_penalty: u32,
    pub damage_reduction: f32,
    pub intercept_chance: f32,
}

/// Cavalry dodge mechanics.
#[derive(Clone, Debug, Deserialize)]
pub struct CavalryConfig {
    pub dodge_max_chance: f32,
    pub dodge_decay_rate: f32,
}

/// Archer multi-shot mechanics.
#[derive(Clone, Debug, Deserialize)]
pub struct ArcherMultiShotConfig {
    pub base_chance: f32,
    pub per_level_bonus: f32,
    pub min_chance: f32,
    pub max_chance: f32,
}

/// Slow debuff applied by archer arrows.
#[derive(Clone, Debug, Deserialize)]
pub struct SlowDebuffConfig {
    pub base_amount: f32,
    pub stack_mult: f32,
    pub max_reduction: f32,
    pub duration_ticks: u32,
    pub max_stacks: u32,
}

/// Level-up and experience mechanics.
#[derive(Clone, Debug, Deserialize)]
pub struct LevelUpConfig {
    pub exp_per_kill: u32,
    pub exp_to_level: u32,
    pub hp_gain: u32,
    pub attack_gain: u32,
    pub lifesteal_unlock_level: u32,
    pub lifesteal_rate: f32,
}

/// Fearless buff (triggered on cavalry dodge).
#[derive(Clone, Debug, Deserialize)]
pub struct FearlessConfig {
    pub duration_ticks: u32,
    pub attack_bonus: u32,
    pub lifesteal_bonus: f32,
}

/// Top-level combat configuration.
#[derive(Clone, Debug, Deserialize, Resource)]
pub struct CombatGlobalConfig {
    pub city_damage_per_soldier_ratio: f32,
    pub archer_melee_range: u32,
    pub archer_melee_damage_mult: f32,
    pub shield: ShieldConfig,
    pub cavalry: CavalryConfig,
    pub archer_multi_shot: ArcherMultiShotConfig,
    pub slow_debuff: SlowDebuffConfig,
    pub level_up: LevelUpConfig,
    pub fearless: FearlessConfig,
}

impl CombatGlobalConfig {
    /// Load from a RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, String> {
        ron::from_str(ron_str)
            .map_err(|e| format!("Failed to parse combat.ron: {}", e))
    }
}
