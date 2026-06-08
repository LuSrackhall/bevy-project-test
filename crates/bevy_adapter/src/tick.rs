use bevy::prelude::*;
use simulation::command::{CommandBuffer, GameCommand};
use simulation::SimulationEvents;

/// Drives the fixed-tick simulation loop.
#[derive(Resource)]
pub struct TickClock {
    pub current_tick: u32,
    pub tick_duration: f32,  // seconds (0.05 for 20Hz)
    pub accumulator: f32,
}

impl Default for TickClock {
    fn default() -> Self {
        Self { current_tick: 0, tick_duration: 0.05, accumulator: 0.0 }
    }
}

/// Wrapper to hold the simulation World in Bevy's ECS.
/// World is !Send, so we use a resource accessible only from the main thread.
#[derive(Resource)]
pub struct SimulationWorld(pub simulation::World);

/// Pending events from the last simulation tick, to be consumed by lifecycle systems.
#[derive(Resource, Default, Clone)]
pub struct PendingEvents {
    pub events: Vec<SimulationEvents>,
}

/// System that runs each frame and drives the simulation.
pub fn tick_driver_system(
    time: Res<Time>,
    mut tick_clock: ResMut<TickClock>,
    mut sim_world: NonSendMut<SimulationWorld>,
    mut cmd_buf: ResMut<CommandBuffer>,
    mut pending: ResMut<PendingEvents>,
) {
    tick_clock.accumulator += time.delta_secs();
    pending.events.clear();

    while tick_clock.accumulator >= tick_clock.tick_duration {
        tick_clock.accumulator -= tick_clock.tick_duration;
        tick_clock.current_tick += 1;

        // Copy Bevy-side commands into simulation world before tick
        let commands_for_tick: Vec<GameCommand> = cmd_buf.0
            .iter()
            .filter(|c| c.tick == tick_clock.current_tick)
            .cloned()
            .collect();
        {
            let mut sim_cmds = sim_world.0.resource_mut::<simulation::command::CommandBuffer>();
            for cmd in commands_for_tick {
                sim_cmds.0.push(cmd);
            }
        }

        // Remove consumed commands from Bevy buffer
        cmd_buf.0.retain(|c| c.tick > tick_clock.current_tick);

        let events = simulation::run_tick(&mut sim_world.0, tick_clock.current_tick);
        pending.events.push(events);
    }
}
