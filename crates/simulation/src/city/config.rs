//! City configuration — loaded from content/cities.ron

use bevy_ecs::prelude::Resource;
use serde::Deserialize;

/// Aura healing configuration (sub-object).
#[derive(Clone, Debug, Deserialize)]
pub struct AuraConfig {
    pub base_radius: u32,
    pub spawn_dir_radius: u32,
    pub base_heal: u32,
    pub per_level_heal: u32,
}

/// Global city parameters.
#[derive(Clone, Debug, Deserialize, Resource)]
pub struct CityGlobalConfig {
    pub level_hp_multiplier: u32,
    pub base_population_per_level: u32,
    pub visual_radius_base: f32,
    pub visual_radius_per_level: f32,
    pub heal_ratio: f32,
    pub level_up_cost_multiplier: f32,
    pub level_up_gain_ratio: f32,
    pub capture_hp_ratio: f32,
    pub aura: AuraConfig,
}

impl CityGlobalConfig {
    /// Load from a RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, String> {
        ron::from_str(ron_str)
            .map_err(|e| format!("Failed to parse cities.ron: {}", e))
    }
}
