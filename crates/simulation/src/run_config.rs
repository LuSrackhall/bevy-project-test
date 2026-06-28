/// Simulation runtime configuration.
///
/// Belongs to the simulation initialization layer (like `seed`, `map_size`),
/// not the Tick-level command pipeline. Controls subsystem availability
/// without participating in GameCommand flow (constitution §2.5).
pub struct RunConfig {
    /// Whether to execute the AI decision phase. Disabled in harness
    /// scenarios that need to isolate player commands.
    pub enable_ai: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self { enable_ai: true }
    }
}
