//! Map generation configuration — loaded from content/map.ron

use bevy_ecs::prelude::Resource;
use serde::Deserialize;

/// Map generation parameters.
#[derive(Clone, Debug, Deserialize, Resource)]
pub struct MapGenConfig {
    pub width: u32,
    pub height: u32,
    pub min_cities: u32,
    pub max_cities: u32,
    pub city_min_distance: u32,
    pub margin: u32,
    /// [min, max] ratio of neutral cities
    pub neutral_city_ratio: [f32; 2],
    /// [min, max] range for city max level
    pub city_level_range: [u32; 2],
}

impl MapGenConfig {
    /// Load from a RON string.
    pub fn from_ron(ron_str: &str) -> Result<Self, String> {
        ron::from_str(ron_str)
            .map_err(|e| format!("Failed to parse map.ron: {}", e))
    }
}
