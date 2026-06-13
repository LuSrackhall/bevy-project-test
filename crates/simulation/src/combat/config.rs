//! Combat configuration — loaded from content/combat.ron

use bevy_ecs::prelude::Resource;
use serde::Deserialize;

/// Infantry shield mechanics.
#[derive(Clone, Debug, Deserialize)]
pub struct ShieldConfig {
    pub speed_penalty: u32,
    pub attack_speed_penalty: u32,
    pub passive_block_chance: f32,
    pub frontal_angle_deg: u32,
    pub initial_hp: u32,
    pub drop_survive_ticks: u32,
    pub disappear_animation_ticks: u32,
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

/// Post-tick overlap resolution — guarantees no soldiers overlap after each tick.
#[derive(Clone, Debug, Deserialize)]
pub struct OverlapResolutionConfig {
    pub max_iterations: u32,
}

/// Facing direction mechanics.
#[derive(Clone, Debug, Deserialize)]
pub struct FacingConfig {
    pub turn_rate_ticks_per_full_rotation: u32, // ticks for 360 degrees
}

/// Attack windup mechanics.
#[derive(Clone, Debug, Deserialize)]
pub struct AttackWindupConfig {
    pub windup_ticks: u32,          // 3 ticks = 0.15s for non-cavalry
    pub cavalry_no_windup: bool,    // true = cavalry attacks instantly
}

/// Top-level combat configuration.
#[derive(Clone, Debug, Deserialize, Resource)]
pub struct CombatGlobalConfig {
    pub city_damage_per_soldier_ratio: f32,
    pub arrow_building_damage_ratio: f32,
    pub facing: FacingConfig,
    pub attack_windup: AttackWindupConfig,
    pub shield: ShieldConfig,
    pub cavalry: CavalryConfig,
    pub archer_multi_shot: ArcherMultiShotConfig,
    pub slow_debuff: SlowDebuffConfig,
    pub level_up: LevelUpConfig,
    pub fearless: FearlessConfig,
    pub overlap_resolution: OverlapResolutionConfig,
}

impl CombatGlobalConfig {
    /// Load from a RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, String> {
        ron::from_str(ron_str)
            .map_err(|e| format!("Failed to parse combat.ron: {}", e))
    }
}
