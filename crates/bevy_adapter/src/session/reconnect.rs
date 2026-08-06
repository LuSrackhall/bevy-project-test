//! Network reconnect recovery helpers.
//!
//! Scene B (process restart) rebuilds the world via `rebuild_world`, which MUST
//! match the live network initialization exactly (R1, specs/network-reconnect).

/// Rebuild a deterministic simulation world for the network reconnect path.
///
/// Uses `init_simulation_world_multi` + the session's player slots — the SAME
/// initialization as the live network path. `init_simulation_world` (2-slot) or
/// `run_tick_default` (AI on) are FORBIDDEN here: they diverge from the network
/// PlayerSlots/NoOp set and would desync.
///
/// The caller is responsible for `map::generate_map(&mut world, map_size)` with
/// the game's map size, then enabling `run_tick(enable_ai:false)` during replay.
pub fn rebuild_world(seed: u64, player_count: u8, player_id: u8) -> simulation::World {
    let slots = simulation::types::PlayerSlots::multi_player(player_count, player_id);
    simulation::init_simulation_world_multi(seed, slots)
}
