//! Soldier configuration — loaded from content/units.ron

use bevy_ecs::prelude::Resource;
use serde::Deserialize;
use std::collections::HashMap;
use crate::types::SoldierType;

/// Per-unit-type configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct SoldierUnitConfig {
    pub health: u32,
    pub attack: u32,
    pub speed: u32,
    pub attack_range: u32,
    pub attack_interval_ticks: u32,
    pub spawn_speed_mult: f32,
    // Archer-specific fields (serde default for non-archer types)
    #[serde(default = "default_arrow_speed")]
    pub arrow_speed: u32,
    #[serde(default = "default_range_base")]
    pub attack_range_base: u32,
    #[serde(default = "default_range_max")]
    pub attack_range_max: u32,
    #[serde(default = "default_range_max_level")]
    pub attack_range_max_level: u32,
    #[serde(default = "default_overshoot_base")]
    pub overshoot_base: u32,
    #[serde(default = "default_overshoot_per_level")]
    pub overshoot_per_level: u32,
    #[serde(default = "default_pierce_base")]
    pub pierce_base_chance: f32,
    #[serde(default = "default_pierce_per_level")]
    pub pierce_per_level: f32,
    #[serde(default = "default_pierce_unlock")]
    pub pierce_unlock_level: u32,
    #[serde(default = "default_collision_radius")]
    pub collision_radius: u32,
}

fn default_arrow_speed() -> u32 { 20 }
fn default_range_base() -> u32 { 380 }
fn default_range_max() -> u32 { 600 }
fn default_range_max_level() -> u32 { 4 }
fn default_overshoot_base() -> u32 { 20 }
fn default_overshoot_per_level() -> u32 { 20 }
fn default_pierce_base() -> f32 { 0.05 }
fn default_pierce_per_level() -> f32 { 0.02 }
fn default_pierce_unlock() -> u32 { 2 }
fn default_collision_radius() -> u32 { 6 }

impl SoldierUnitConfig {
    /// Compute attack range for archers based on level.
    pub fn compute_attack_range(&self, level: u32) -> u32 {
        let lvl = level.min(self.attack_range_max_level);
        let base = self.attack_range_base as u32;
        let range = self.attack_range_max - self.attack_range_base;
        let steps = self.attack_range_max_level - 1;
        if steps == 0 { return base; }
        base + (lvl - 1) * range / steps
    }

    /// Compute overshoot distance based on level.
    pub fn compute_overshoot(&self, level: u32) -> u32 {
        self.overshoot_base + level.saturating_sub(1) * self.overshoot_per_level
    }

    /// Compute flight ticks for an arrow based on level.
    pub fn compute_flight_ticks(&self, level: u32) -> u32 {
        let total_distance = self.compute_attack_range(level) + self.compute_overshoot(level);
        if self.arrow_speed == 0 { return 10; }
        (total_distance + self.arrow_speed - 1) / self.arrow_speed // ceil division
    }

    /// Max flight ticks (used for precomputation).
    pub fn max_flight_ticks(&self) -> u32 {
        let max_dist = self.attack_range_max + self.overshoot_base + (self.attack_range_max_level - 1) * self.overshoot_per_level;
        if self.arrow_speed == 0 { return 34; }
        (max_dist + self.arrow_speed - 1) / self.arrow_speed
    }

    /// Pierce chance for given level.
    pub fn compute_pierce_chance(&self, level: u32) -> f32 {
        if level < self.pierce_unlock_level { return 0.0; }
        (self.pierce_base_chance + (level - self.pierce_unlock_level) as f32 * self.pierce_per_level).max(0.0)
    }
}

/// Full soldier configuration, indexed by soldier type.
#[derive(Clone, Debug, Resource)]
pub struct SoldierConfig {
    pub units: HashMap<SoldierType, SoldierUnitConfig>,
}

impl SoldierConfig {
    /// Load from a RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, String> {
        let raw: HashMap<String, SoldierUnitConfig> = ron::from_str(ron_str)
            .map_err(|e| format!("Failed to parse units.ron: {}", e))?;

        let mut units = HashMap::new();
        for (key, config) in raw {
            let stype = match key.as_str() {
                "militia" => SoldierType::Militia,
                "infantry" => SoldierType::Infantry,
                "archer" => SoldierType::Archer,
                "cavalry" => SoldierType::Cavalry,
                other => return Err(format!("Unknown soldier type in config: '{}'", other)),
            };
            units.insert(stype, config);
        }
        Ok(SoldierConfig { units })
    }

    /// Get config for a soldier type (panics if missing — should be loaded at startup).
    pub fn get(&self, stype: SoldierType) -> &SoldierUnitConfig {
        self.units.get(&stype)
            .unwrap_or_else(|| panic!("Soldier config missing for {:?}", stype))
    }
}
