pub mod verifier;
pub mod verifiers;

use std::collections::{BTreeMap, HashMap};

use crate::command::{CommandBuffer, GameCommand};
use crate::events::SimulationEvents;
use crate::map;
use crate::run_config::RunConfig;

pub use verifier::{Verifier, VerifyError};
pub use verifiers::{CompositeVerifier, EventVerifier, InvariantVerifier, SnapshotVerifier};

/// A deterministic simulation scenario: seed + map + commands + verifier.
/// Run via `self.run()` to execute and validate in one step.
pub struct Scenario {
    pub seed: u64,
    pub map_size: map::MapSize,
    pub config: RunConfig,
    pub commands: Vec<GameCommand>,
    pub max_tick: u32,
    pub verifier: Box<dyn Verifier>,
}

/// Output of a completed scenario run. Does NOT contain World (§17).
pub struct ScenarioOutput {
    pub events_per_tick: HashMap<u32, SimulationEvents>,
}

impl Scenario {
    /// Execute the scenario and validate. Returns Err if verifier fails.
    pub fn run(self) -> Result<ScenarioOutput, VerifyError> {
        let mut world = crate::init_simulation_world(self.seed);
        map::generate_map(&mut world, self.map_size);

        // Group commands by tick
        let mut grouped: BTreeMap<u32, Vec<GameCommand>> = BTreeMap::new();
        for cmd in self.commands {
            grouped.entry(cmd.tick).or_default().push(cmd);
        }

        let mut events_per_tick = HashMap::new();

        for tick in 1..=self.max_tick {
            // Collect commands for this tick
            let mut cmds = grouped.remove(&tick).unwrap_or_default();

            // Sort by (player_id, action.sort_tag()) for determinism (§3.1)
            cmds.sort_by_key(|c| (c.player_id, c.action.sort_tag()));

            // Inject into CommandBuffer
            world.resource_mut::<CommandBuffer>().0.extend(cmds);

            // Execute tick
            let events = crate::run_tick(&mut world, tick, &self.config);
            events_per_tick.insert(tick, events);
        }

        // Verify
        self.verifier.verify(&mut world, &events_per_tick)?;

        Ok(ScenarioOutput { events_per_tick })
    }
}

#[cfg(test)]
mod scenario_test;
