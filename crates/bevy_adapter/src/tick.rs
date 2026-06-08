use bevy::prelude::*;
use simulation::command::CommandBuffer;

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

/// Tick driver system — runs every Bevy frame, executes simulation ticks as needed.
pub fn tick_driver(world: &mut World) {
    // We need split World access: read Bevy time, access simulation world
    // Use a system that operates on the Bevy World at the schedule level.
}

/// System that runs each frame and drives the simulation.
pub fn tick_driver_system(
    time: Res<Time>,
    mut tick_clock: ResMut<TickClock>,
    mut sim_world: NonSendMut<SimulationWorld>,
    mut cmd_buf: ResMut<CommandBuffer>,
) {
    tick_clock.accumulator += time.delta_secs();

    while tick_clock.accumulator >= tick_clock.tick_duration {
        tick_clock.accumulator -= tick_clock.tick_duration;
        tick_clock.current_tick += 1;

        let events = simulation::run_tick(&mut sim_world.0, tick_clock.current_tick);

        // Store events back for lifecycle systems to consume
        // (events are already in the simulation world — lifecycle reads them there)
        let _ = events;
    }
}
