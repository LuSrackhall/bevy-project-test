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
    pub aggression_range: u32,
    pub attack_interval_ticks: u32,
    pub spawn_speed_mult: f32,
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
