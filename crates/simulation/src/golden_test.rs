//! Golden determinism tests — verify that the simulation produces identical
//! results given the same seed and command sequence.

use bevy_ecs::world::World;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::soldier::*;
use crate::types::*;

/// Compute a deterministic hash of the entire simulation world state.
/// Extracts fields from all entities (sorted by UnitId) and hashes them.
pub fn hash_world_state(world: &mut World) -> u64 {
    let mut hasher = DefaultHasher::new();

    // Collect and sort entities by UnitId
    let mut entities: Vec<UnitId> = {
        let mut q = world.query::<&UnitIdComponent>();
        q.iter(world).map(|id| id.0).collect()
    };
    entities.sort();

    (entities.len() as u64).hash(&mut hasher);

    for uid in &entities {
        let Some(entity) = find_entity_by_unit_id(world, *uid) else {
            continue;
        };
        let em = world.entity(entity);

        uid.0.hash(&mut hasher);

        // Position
        if let Some(p) = em.get::<LogicalPosition>() {
            p.0.x.0.hash(&mut hasher);
            p.0.y.0.hash(&mut hasher);
        }
        // Health
        if let Some(h) = em.get::<Health>() {
            h.current.hash(&mut hasher);
            h.max.hash(&mut hasher);
        }
        // Attack
        if let Some(a) = em.get::<Attack>() {
            a.damage.hash(&mut hasher);
            a.range.hash(&mut hasher);
            a.cooldown_remaining.hash(&mut hasher);
        }
        // Movement
        if let Some(m) = em.get::<Movement>() {
            m.speed.hash(&mut hasher);
            m.target.map(|t| t.0).hash(&mut hasher);
            m.force_move.hash(&mut hasher);
        }
        // Faction
        if let Some(f) = em.get::<FactionComponent>() {
            (f.0 as u8).hash(&mut hasher);
        }
        // SoldierType
        if let Some(st) = em.get::<SoldierTypeComponent>() {
            (st.0 as u8).hash(&mut hasher);
        }
        // Level
        if let Some(l) = em.get::<Level>() {
            l.level.hash(&mut hasher);
            l.exp.hash(&mut hasher);
        }
        // CityComponent
        if let Some(c) = em.get::<CityComponent>() {
            c.level.hash(&mut hasher);
            c.health_current.hash(&mut hasher);
            c.health_max.hash(&mut hasher);
            c.population.hash(&mut hasher);
            c.spawn_cooldown.hash(&mut hasher);
            c.level_exp.hash(&mut hasher);
        }
        // ShieldItem
        if let Some(s) = em.get::<ShieldItem>() {
            s.hp.hash(&mut hasher);
            s.max_hp.hash(&mut hasher);
        }
    }

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::*;
    use crate::init_simulation_world;
    use crate::map;
    use crate::run_tick;

    #[test]
    fn test_golden_empty_map_no_commands() {
        let mut world1 = init_simulation_world(42);
        map::generate_map(&mut world1, map::MapSize::Small);
        let mut world2 = init_simulation_world(42);
        map::generate_map(&mut world2, map::MapSize::Small);

        for tick in 1..=1000 {
            run_tick(&mut world1, tick);
            run_tick(&mut world2, tick);
        }

        let hash1 = hash_world_state(&mut world1);
        let hash2 = hash_world_state(&mut world2);
        assert_eq!(
            hash1, hash2,
            "Same seed should produce identical world state after 1000 ticks"
        );
    }

    #[test]
    fn test_golden_different_seeds_different_state() {
        let mut world1 = init_simulation_world(42);
        map::generate_map(&mut world1, map::MapSize::Small);
        let mut world2 = init_simulation_world(99);
        map::generate_map(&mut world2, map::MapSize::Small);

        for tick in 1..=100 {
            run_tick(&mut world1, tick);
            run_tick(&mut world2, tick);
        }

        let hash1 = hash_world_state(&mut world1);
        let hash2 = hash_world_state(&mut world2);
        assert_ne!(
            hash1, hash2,
            "Different seeds should produce different world state"
        );
    }

    #[test]
    fn test_golden_with_commands() {
        let mut world1 = init_simulation_world(42);
        map::generate_map(&mut world1, map::MapSize::Small);
        let mut world2 = init_simulation_world(42);
        map::generate_map(&mut world2, map::MapSize::Small);

        for tick in 1..=500 {
            if tick == 10 {
                let mut q = world1.query::<(&UnitIdComponent, &FactionComponent, &SoldierMarker)>();
                if let Some((id, _fac, _)) =
                    q.iter(&world1).find(|(_, f, _)| f.0 == Faction::Player)
                {
                    let uid = id.0;
                    let target = FixedVec2::new(Fixed::from_int(200), Fixed::from_int(200));
                    world1.resource_mut::<CommandBuffer>().push(GameCommand {
                        tick: 11,
                        player_id: 0,
                        action: Action::MoveTo { unit: uid, target },
                    });
                    world2.resource_mut::<CommandBuffer>().push(GameCommand {
                        tick: 11,
                        player_id: 0,
                        action: Action::MoveTo { unit: uid, target },
                    });
                }
            }
            run_tick(&mut world1, tick);
            run_tick(&mut world2, tick);
        }

        let hash1 = hash_world_state(&mut world1);
        let hash2 = hash_world_state(&mut world2);
        assert_eq!(
            hash1, hash2,
            "Same seed + same commands → identical state after 500 ticks"
        );
    }

    #[test]
    fn test_golden_determinism_across_runs() {
        fn run_sim() -> u64 {
            let mut world = init_simulation_world(12345);
            map::generate_map(&mut world, map::MapSize::Medium);
            for tick in 1..=2000 {
                run_tick(&mut world, tick);
            }
            hash_world_state(&mut world)
        }

        let hash1 = run_sim();
        let hash2 = run_sim();
        assert_eq!(
            hash1, hash2,
            "Simulation must be deterministic across multiple runs"
        );
    }
}
