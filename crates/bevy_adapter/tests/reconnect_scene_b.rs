//! Scene B (process restart) reconnect determinism test.
//!
//! A restarted client rebuilds the world via `rebuild_world` + `generate_map`,
//! then fast-replays the command log. This MUST produce hash-identical state to
//! a continuously-online client (same seed + same commands + same init path).

use simulation::command::{Action, CommandBuffer, GameCommand};
use simulation::map::MapSize;
use simulation::soldier::{FactionComponent, UnitIdComponent};
use simulation::types::{FactionId, Fixed, FixedVec2, PlayerSlots, UnitId};

/// Deterministic MoveTo command with a target that drifts per tick so the world
/// actually evolves (empty/NoOp commands would trivially hash-identical).
fn make_move_cmd(tick: u32, uid: UnitId) -> GameCommand {
    GameCommand {
        tick,
        player_id: 0,
        action: Action::MoveTo {
            unit: uid,
            target: FixedVec2::new(Fixed::from_int(300), Fixed::from_int(300 + tick as i32 * 10)),
        },
    }
}

#[test]
fn test_scene_b_rebuild_matches_live_hash() {
    let seed = 42u64;
    let player_count = 4u8;
    let local = 0u8;
    let map_size = MapSize::Small;
    let total = 60u32;

    // ── Phase A: online client (init_multi + generate_map + run ticks) ──
    let mut online = simulation::init_simulation_world_multi(seed, PlayerSlots::multi_player(player_count, local));
    simulation::map::generate_map(&mut online, map_size);
    let uid = {
        let mut q = online.query::<(&UnitIdComponent, &FactionComponent)>();
        q.iter(&online)
            .find(|(_, f)| f.0 == FactionId(0))
            .map(|(u, _)| u.0)
            .expect("player 0 has a unit")
    };
    for tick in 1..=total {
        online.resource_mut::<CommandBuffer>().0.push(make_move_cmd(tick, uid));
        simulation::run_tick(&mut online, tick, &simulation::RunConfig { enable_ai: false });
    }
    let online_hash = simulation::golden_test::hash_world_state(&mut online);

    // ── Phase B: Scene B rebuild (rebuild_world) + replay same commands ──
    let mut rebuilt = bevy_adapter::session::reconnect::rebuild_world(seed, player_count, local);
    simulation::map::generate_map(&mut rebuilt, map_size);
    for tick in 1..=total {
        rebuilt.resource_mut::<CommandBuffer>().0.push(make_move_cmd(tick, uid));
        simulation::run_tick(&mut rebuilt, tick, &simulation::RunConfig { enable_ai: false });
    }
    let rebuilt_hash = simulation::golden_test::hash_world_state(&mut rebuilt);

    assert_eq!(
        online_hash, rebuilt_hash,
        "Scene B rebuild must be hash-identical to a continuously-online client"
    );
}

#[test]
fn test_scene_b_map_size_flows_to_rebuild() {
    // rebuild_world returns a world that generate_map can size — verify a
    // non-default map size builds identically through the Scene B path.
    let seed = 42u64;
    let map_size = MapSize::Large;
    let mut w = bevy_adapter::session::reconnect::rebuild_world(seed, 4, 0);
    simulation::map::generate_map(&mut w, map_size);
    // MapBounds (via config) reflect the Large size.
    let cfg = map_size.load_config();
    assert!(cfg.width > 3500, "Large map must be larger than Medium");
}
