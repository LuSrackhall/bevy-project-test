#[cfg(test)]
mod tests {
    use crate::command::*;
    use crate::map;
    use crate::run_config::RunConfig;
    use crate::scenario::*;
    use crate::soldier::{FactionComponent, UnitIdComponent};
    use crate::types::*;

    #[test]
    fn test_snapshot_verifier_pass() {
        // First run to get the actual hash
        let mut world = crate::init_simulation_world(42);
        map::generate_map(&mut world, map::MapSize::Small);
        for tick in 1..=100 {
            crate::run_tick_default(&mut world, tick);
        }
        let expected_hash = crate::golden_test::hash_world_state(&mut world);

        // Now run with correct hash
        let result = Scenario {
            seed: 42,
            map_size: map::MapSize::Small,
            config: RunConfig::default(),
            commands: vec![],
            max_tick: 100,
            verifier: Box::new(SnapshotVerifier::hash(expected_hash)),
        }
        .run();

        assert!(result.is_ok(), "SnapshotVerifier should pass: {:?}", result.err());
    }

    #[test]
    fn test_snapshot_verifier_fail() {
        let result = Scenario {
            seed: 42,
            map_size: map::MapSize::Small,
            config: RunConfig::default(),
            commands: vec![],
            max_tick: 100,
            verifier: Box::new(SnapshotVerifier::hash(0xDEAD)),
        }
        .run();

        assert!(result.is_err(), "SnapshotVerifier should fail on wrong hash");
    }

    #[test]
    fn test_event_verifier_spawned_at_tick_1() {
        // Cities spawn soldiers immediately at tick 1
        let result = Scenario {
            seed: 42,
            map_size: map::MapSize::Small,
            config: RunConfig::default(),
            commands: vec![],
            max_tick: 5,
            verifier: Box::new(
                EventVerifier::new().expect_spawned_at(1, |s| !s.is_empty()),
            ),
        }
        .run();

        assert!(result.is_ok(), "EventVerifier should pass: {:?}", result.err());
    }

    #[test]
    fn test_event_verifier_fail() {
        let result = Scenario {
            seed: 42,
            map_size: map::MapSize::Small,
            config: RunConfig::default(),
            commands: vec![],
            max_tick: 2,
            verifier: Box::new(
                EventVerifier::new().expect_spawned_at(1, |s| s.len() > 100),
            ),
        }
        .run();

        assert!(result.is_err(), "EventVerifier should fail");
    }

    #[test]
    fn test_invariant_verifier_health() {
        let result = Scenario {
            seed: 42,
            map_size: map::MapSize::Small,
            config: RunConfig::default(),
            commands: vec![],
            max_tick: 100,
            verifier: Box::new(InvariantVerifier::new().check(|world| {
                let mut q = world.query::<&crate::soldier::Health>();
                for h in q.iter(world) {
                    if h.current > h.max {
                        return Some(format!("HP {} > max {}", h.current, h.max));
                    }
                }
                None
            })),
        }
        .run();

        assert!(result.is_ok(), "InvariantVerifier should pass: {:?}", result.err());
    }

    #[test]
    fn test_composite_verifier() {
        // Get expected hash first
        let mut world = crate::init_simulation_world(42);
        map::generate_map(&mut world, map::MapSize::Small);
        for tick in 1..=50 {
            crate::run_tick_default(&mut world, tick);
        }
        let expected_hash = crate::golden_test::hash_world_state(&mut world);

        let result = Scenario {
            seed: 42,
            map_size: map::MapSize::Small,
            config: RunConfig::default(),
            commands: vec![],
            max_tick: 50,
            verifier: Box::new(CompositeVerifier(vec![
                Box::new(SnapshotVerifier::hash(expected_hash)),
                Box::new(InvariantVerifier::new().check(|_| None)),
            ])),
        }
        .run();

        assert!(result.is_ok(), "CompositeVerifier should pass: {:?}", result.err());
    }

    #[test]
    fn test_scenario_with_commands() {
        // Inject a MoveTo command and verify the scenario runs without panic
        let mut world = crate::init_simulation_world(42);
        map::generate_map(&mut world, map::MapSize::Small);
        // Find a player unit
        let mut q = world.query::<(&UnitIdComponent, &FactionComponent)>();
        let uid = q
            .iter(&world)
            .find(|(_, f)| f.0 == FactionId(0))
            .map(|(id, _)| id.0)
            .expect("Should have a player unit");

        let target = FixedVec2::new(Fixed::from_int(200), Fixed::from_int(200));
        drop(q); // release world borrow
        drop(world); // we only needed the uid

        let result = Scenario {
            seed: 42,
            map_size: map::MapSize::Small,
            config: RunConfig::default(),
            commands: vec![GameCommand {
                tick: 5,
                player_id: 0,
                action: Action::MoveTo { unit: uid, target },
            }],
            max_tick: 100,
            verifier: Box::new(InvariantVerifier::new().check(|_| None)),
        }
        .run();

        assert!(result.is_ok(), "Scenario with commands should pass: {:?}", result.err());
    }

    #[test]
    fn test_scenario_no_ai() {
        let result = Scenario {
            seed: 42,
            map_size: map::MapSize::Small,
            config: RunConfig { enable_ai: false },
            commands: vec![],
            max_tick: 50,
            verifier: Box::new(InvariantVerifier::new().check(|_| None)),
        }
        .run();

        assert!(result.is_ok(), "Scenario with AI disabled should pass: {:?}", result.err());
    }
}
