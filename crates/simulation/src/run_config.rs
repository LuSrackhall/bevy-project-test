/// Simulation runtime configuration.
///
/// Belongs to the simulation initialization layer (like `seed`, `map_size`),
/// not the Tick-level command pipeline. Controls subsystem availability
/// without participating in GameCommand flow (constitution §2.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunConfig {
    /// Whether to execute the AI decision phase. Disabled in harness
    /// scenarios that need to isolate player commands.
    pub enable_ai: bool,
}

impl RunConfig {
    /// AI drives every non-human faction. Single-player and replay.
    pub const fn ai_enabled() -> Self {
        Self { enable_ai: true }
    }

    /// Every faction is human-driven. Network lockstep: if one client
    /// synthesised AI commands the peers would diverge, so the network session
    /// must state this explicitly rather than infer it from the transport.
    pub const fn ai_disabled() -> Self {
        Self { enable_ai: false }
    }
}

impl Default for RunConfig {
    fn default() -> Self {
        Self::ai_enabled()
    }
}
