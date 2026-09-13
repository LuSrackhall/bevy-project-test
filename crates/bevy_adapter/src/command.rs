//! Single entry point for turning UI intent into a scheduled `GameCommand`.
//!
//! Why this exists instead of every call site building a `GameCommand`: the
//! target tick is a **mode-dependent** value. Live mode consumes a command on
//! the very next tick (`+1`); Network mode must land it inside the relay uplink
//! window, so it is offset by `input_delay`. A call site that hardcodes
//! `current_tick + 1` still works in single-player (the command just waits for
//! that tick) but in network mode falls outside
//! [`network_flush_system`](crate::transport::network_flush_system)'s
//! `[current + 1, current + input_delay]` window, is never uplinked, and is
//! silently dropped — the classic "works solo, does nothing online" defect.
//!
//! See `docs/engineering/command-pipeline-guide.md` for the full pipeline.

use crate::driver::SimulationDriver;
use simulation::command::{Action, CommandBuffer, GameCommand};

/// The tick a command issued *now* should target, for this driver's mode.
///
/// Live/Replay → the next tick. Network → `current + input_delay`, the far edge
/// of the uplink window, so the frame cannot already be late for the relay.
pub fn target_tick(driver: &SimulationDriver, current_tick: u32) -> u32 {
    current_tick + driver.command_delay()
}

/// Schedule `action` for `player_id` on the correct tick for the driver's
/// current mode. Returns the tick the command was scheduled for.
///
/// This is the only sanctioned way for `render_view` to enqueue a command; see
/// the module docs for why the offset must not be inlined at call sites.
pub fn enqueue(
    cmd_buf: &mut CommandBuffer,
    driver: &SimulationDriver,
    current_tick: u32,
    player_id: u8,
    action: Action,
) -> u32 {
    let tick = target_tick(driver, current_tick);
    cmd_buf.push(GameCommand {
        tick,
        player_id,
        action,
    });
    tick
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::CommandSource;

    #[test]
    fn live_targets_the_next_tick() {
        assert_eq!(target_tick(&SimulationDriver::new_live(), 10), 11);
    }

    #[test]
    fn network_targets_the_far_edge_of_the_input_delay_window() {
        // Mirror what a session actually installs: `bootstrap::wire()` swaps in
        // `NetworkCommandSource::new(game_id, player_id, 3)`. On its own,
        // `new_network()` leaves the default delay of 0 and is only a placeholder
        // state — with 0 the window [current + 1, current + 0] would be empty and
        // no command could ever be uplinked.
        let mut driver = SimulationDriver::new_network();
        let CommandSource::Network(ns) = &mut driver.source else {
            panic!("new_network must produce a NetworkCommandSource");
        };
        ns.input_delay = 3;

        // The far edge of network_flush_system's window [current+1, current+delay]:
        // a larger offset would never be uplinked, smaller ones risk arriving late.
        assert_eq!(target_tick(&driver, 10), 13);
    }
}
