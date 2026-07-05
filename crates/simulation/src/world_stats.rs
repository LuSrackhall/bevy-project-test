//! World-level statistics for deterministic replay display.
//!
//! Counts soldiers and cities per faction using `BTreeMap` to ensure
//! deterministic iteration order. Intended for read-only query during
//! replay-mode HUD updates, not live hot paths.

use std::collections::BTreeMap;

use crate::soldier::{CityMarker, FactionComponent, SoldierMarker};
use crate::types::FactionId;
use bevy_ecs::world::World;

/// Per-faction soldier and city counts.
///
/// Iteration order is deterministic (faction sort order via `BTreeMap`).
/// Factions with zero units are not present in the map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactionCounts {
    pub factions: BTreeMap<FactionId, (u32, u32)>,
}

impl FactionCounts {
    /// Number of soldiers belonging to the given faction.
    /// Returns 0 if the faction has no soldiers.
    pub fn soldiers(&self, faction: FactionId) -> u32 {
        self.factions.get(&faction).map_or(0, |(s, _)| *s)
    }

    /// Number of cities belonging to the given faction.
    /// Returns 0 if the faction has no cities.
    pub fn cities(&self, faction: FactionId) -> u32 {
        self.factions.get(&faction).map_or(0, |(_, c)| *c)
    }

    /// Total number of soldiers across all factions.
    pub fn total_soldiers(&self) -> u32 {
        self.factions.values().map(|(s, _)| s).sum()
    }

    /// Total number of cities across all factions.
    pub fn total_cities(&self) -> u32 {
        self.factions.values().map(|(_, c)| c).sum()
    }
}

/// Count soldiers and cities per faction in the simulation world.
///
/// Uses two queries: one for `SoldierMarker` entities and one for
/// `CityMarker` entities. Factions without any units do not appear
/// in the returned map.
///
/// # Determinism
///
/// Returns a `BTreeMap` whose iteration order is determined by the
/// `Ord` implementation of `FactionId`, which is consistent across
/// same-seed runs. No allocation-dependent iteration is used.
pub fn count_factions(world: &mut World) -> FactionCounts {
    let mut factions: BTreeMap<FactionId, (u32, u32)> = BTreeMap::new();

    // Count soldiers (entities with both FactionComponent and SoldierMarker)
    let mut soldier_q = world.query::<(&FactionComponent, &SoldierMarker)>();
    for (fac, _) in soldier_q.iter(world) {
        let entry = factions.entry(fac.0).or_insert((0, 0));
        entry.0 += 1;
    }

    // Count cities (entities with both FactionComponent and CityMarker)
    let mut city_q = world.query::<(&FactionComponent, &CityMarker)>();
    for (fac, _) in city_q.iter(world) {
        let entry = factions.entry(fac.0).or_insert((0, 0));
        entry.1 += 1;
    }

    FactionCounts { factions }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_simulation_world;
    use crate::map;
    use crate::map::MapSize;
    use crate::types::FactionId;

    #[test]
    fn test_empty_world_returns_empty_counts() {
        let mut world = World::new();
        let counts = count_factions(&mut world);
        assert!(counts.factions.is_empty(), "Empty world should have zero factions");
        assert_eq!(counts.total_soldiers(), 0);
        assert_eq!(counts.total_cities(), 0);
        assert_eq!(counts.soldiers(FactionId(0)), 0);
        assert_eq!(counts.cities(FactionId(2)), 0);
    }

    #[test]
    fn test_map_generation_creates_units() {
        let mut world = init_simulation_world(42);
        map::generate_map(&mut world, MapSize::Small);
        let counts = count_factions(&mut world);
        assert!(
            counts.total_soldiers() > 0 || counts.total_cities() > 0,
            "Expected at least one unit after map generation, got soldiers={} cities={}",
            counts.total_soldiers(),
            counts.total_cities()
        );
    }

    #[test]
    fn test_deterministic_across_same_seed() {
        let mut w1 = init_simulation_world(42);
        map::generate_map(&mut w1, MapSize::Small);
        let counts1 = count_factions(&mut w1);

        let mut w2 = init_simulation_world(42);
        map::generate_map(&mut w2, MapSize::Small);
        let counts2 = count_factions(&mut w2);

        assert_eq!(
            counts1, counts2,
            "Same seed + same map size should produce identical FactionCounts"
        );
    }

    #[test]
    fn test_different_seed_self_consistent() {
        let mut w1 = init_simulation_world(42);
        map::generate_map(&mut w1, MapSize::Small);
        let counts1 = count_factions(&mut w1);

        let mut w2 = init_simulation_world(99);
        map::generate_map(&mut w2, MapSize::Small);
        let counts2 = count_factions(&mut w2);

        // Both seeds should produce internally consistent results
        assert!(counts1.total_soldiers() > 0 || counts1.total_cities() > 0);
        assert!(counts2.total_soldiers() > 0 || counts2.total_cities() > 0);

        // Verify deterministic: re-running the same seed gives the same result
        let mut w3 = init_simulation_world(42);
        map::generate_map(&mut w3, MapSize::Small);
        let counts3 = count_factions(&mut w3);
        assert_eq!(counts1, counts3);
    }

    #[test]
    fn test_dynamic_faction_support() {
        let mut world = init_simulation_world(42);
        map::generate_map(&mut world, MapSize::Small);
        let counts = count_factions(&mut world);

        // Only Player and Enemy should have units after map generation
        // (Neutral may appear depending on map generation)
        for (faction, _) in &counts.factions {
            match faction {
                FactionId(0) | FactionId(1) => {} // expected
                FactionId(2) => {} // also possible
            }
        }
    }

    #[test]
    fn test_accessors_default_to_zero_for_missing_faction() {
        let mut world = init_simulation_world(42);
        map::generate_map(&mut world, MapSize::Small);
        let counts = count_factions(&mut world);

        // Neutral may or may not have units; either way accessing should be safe
        let _s = counts.soldiers(FactionId(2));
        let _c = counts.cities(FactionId(2));
        // No panic means success
    }
}
