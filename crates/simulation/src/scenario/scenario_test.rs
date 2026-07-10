use crate::command::*;
use crate::map;
use crate::run_config::RunConfig;
use crate::scenario::*;
use crate::soldier::{FactionComponent, SoldierMarker, UnitIdComponent};
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

// ═══════════════════════════════════════════════════════════════
// 多人命令管道测试（宪法 §3.1, §10.1）
// ═══════════════════════════════════════════════════════════════

/// Helper: 创建一个预跑世界，找到两个阵营的士兵单位 ID。
fn find_both_faction_unit_ids(
    seed: u64,
    map_size: map::MapSize,
    pre_ticks: u32,
) -> (UnitId, UnitId) {
    let mut world = crate::init_simulation_world(seed);
    map::generate_map(&mut world, map_size);
    for tick in 1..=pre_ticks {
        crate::run_tick_default(&mut world, tick);
    }
    let mut q = world.query::<(&UnitIdComponent, &FactionComponent, &SoldierMarker)>();
    let mut p0: Option<UnitId> = None;
    let mut p1: Option<UnitId> = None;
    for (uid, fac, _) in q.iter(&world) {
        if fac.0 == FactionId(0) && p0.is_none() {
            p0 = Some(uid.0);
        } else if fac.0 == FactionId(1) && p1.is_none() {
            p1 = Some(uid.0);
        }
        if p0.is_some() && p1.is_some() {
            break;
        }
    }
    drop(q);
    drop(world);
    (
        p0.expect("Player 0 has no soldier"),
        p1.expect("Player 1 has no soldier"),
    )
}

/// Helper: 检查士兵实体在给定 World 中是否有 Movement 组件。
/// Movement 被 consume_commands_system 中的 apply_movement 插入，
/// 在到达目标后由 soldier_movement_system 移除。
fn has_movement_component(world: &mut crate::World, uid: UnitId) -> bool {
    let mut q = world.query::<(&UnitIdComponent, &crate::soldier::Movement)>();
    q.iter(world).any(|(id, _)| id.0 == uid)
}

#[test]
fn test_two_players_commands_same_tick() {
    // §3.1 验证：同一 tick 多个玩家的命令都能被正确执行
    let (uid0, uid1) = find_both_faction_unit_ids(42, map::MapSize::Small, 30);
    let target = FixedVec2::new(Fixed::from_int(200), Fixed::from_int(200));

    let result = Scenario {
        seed: 42,
        map_size: map::MapSize::Small,
        config: RunConfig { enable_ai: false },
        commands: vec![
            GameCommand {
                tick: 5,
                player_id: 0,
                action: Action::MoveTo { unit: uid0, target },
            },
            GameCommand {
                tick: 5,
                player_id: 1,
                action: Action::MoveTo { unit: uid1, target },
            },
        ],
        // 用短 tick 数确保 Movement 组件尚未被消费
        max_tick: 10,
        verifier: Box::new(InvariantVerifier::new().check(move |world| {
            if !has_movement_component(world, uid0) {
                return Some("Player 0 command not executed".into());
            }
            if !has_movement_component(world, uid1) {
                return Some("Player 1 command not executed".into());
            }
            None
        })),
    }
    .run();

    assert!(
        result.is_ok(),
        "Two-player same-tick commands should both execute: {:?}",
        result.err()
    );
}

#[test]
fn test_missing_player_noop_injection() {
    // §3.1 Step 2 验证：只有一个玩家发命令时， NoOp 自动补齐
    let (uid0, _uid1) = find_both_faction_unit_ids(42, map::MapSize::Small, 30);
    let target = FixedVec2::new(Fixed::from_int(200), Fixed::from_int(200));

    let result = Scenario {
        seed: 42,
        map_size: map::MapSize::Small,
        config: RunConfig { enable_ai: false },
        commands: vec![
            GameCommand {
                tick: 5,
                player_id: 0,
                action: Action::MoveTo { unit: uid0, target },
            },
        ],
        max_tick: 10,
        verifier: Box::new(InvariantVerifier::new().check(move |world| {
            if !has_movement_component(world, uid0) {
                return Some("Single-player command should still execute".into());
            }
            None
        })),
    }
    .run();

    assert!(
        result.is_ok(),
        "Single-player command with NoOp injection should pass: {:?}",
        result.err()
    );
}

#[test]
fn test_commands_sorting_by_player_then_action() {
    // §3.1 Step 3 验证：逆序插入的命令依然按 (player_id, sort_tag) 排序执行
    let (uid0, uid1) = find_both_faction_unit_ids(42, map::MapSize::Small, 30);
    let target_a = FixedVec2::new(Fixed::from_int(200), Fixed::from_int(200));
    let target_b = FixedVec2::new(Fixed::from_int(-200), Fixed::from_int(-200));

    let result = Scenario {
        seed: 42,
        map_size: map::MapSize::Small,
        config: RunConfig { enable_ai: false },
        commands: vec![
            // 故意逆序插入
            GameCommand {
                tick: 5,
                player_id: 1,
                action: Action::MoveTo { unit: uid1, target: target_b },
            },
            GameCommand {
                tick: 5,
                player_id: 0,
                action: Action::MoveTo { unit: uid0, target: target_a },
            },
        ],
        max_tick: 10,
        verifier: Box::new(InvariantVerifier::new().check(move |world| {
            if !has_movement_component(world, uid0) {
                return Some("Player 0 command lost after sort".into());
            }
            if !has_movement_component(world, uid1) {
                return Some("Player 1 command lost after sort".into());
            }
            None
        })),
    }
    .run();

    assert!(
        result.is_ok(),
        "Commands sorted by (player_id, sort_tag) should all execute: {:?}",
        result.err()
    );
}

#[test]
fn test_many_commands_per_tick() {
    // 边界测试：同一 tick 大量命令不应丢失
    let (uid0, uid1) = find_both_faction_unit_ids(42, map::MapSize::Small, 30);
    let mut cmds = Vec::new();
    for i in 0..10 {
        let offset = Fixed::from_int((i as i32) * 10);
        cmds.push(GameCommand {
            tick: 5,
            player_id: 0,
            action: Action::MoveTo {
                unit: uid0,
                target: FixedVec2::new(Fixed::from_int(100) + offset, Fixed::from_int(100)),
            },
        });
        cmds.push(GameCommand {
            tick: 5,
            player_id: 1,
            action: Action::MoveTo {
                unit: uid1,
                target: FixedVec2::new(Fixed::from_int(-100) - offset, Fixed::from_int(-100)),
            },
        });
    }

    let result = Scenario {
        seed: 42,
        map_size: map::MapSize::Small,
        config: RunConfig { enable_ai: false },
        commands: cmds,
        max_tick: 10,
        verifier: Box::new(InvariantVerifier::new().check(move |world| {
            if !has_movement_component(world, uid0) {
                return Some("Player 0 command lost under load".into());
            }
            if !has_movement_component(world, uid1) {
                return Some("Player 1 command lost under load".into());
            }
            None
        })),
    }
    .run();

    assert!(
        result.is_ok(),
        "Many commands per tick should not be lost: {:?}",
        result.err()
    );
}
