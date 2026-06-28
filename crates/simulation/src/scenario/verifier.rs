use bevy_ecs::world::World;
use std::collections::HashMap;
use std::fmt;

use crate::events::SimulationEvents;

/// Errors produced by verifiers during scenario validation.
#[derive(Debug)]
pub enum VerifyError {
    HashMismatch {
        expected: u64,
        actual: u64,
        source: &'static str,
    },
    EventMismatch {
        tick: u32,
        detail: String,
        source: &'static str,
    },
    InvariantViolation {
        detail: String,
        source: &'static str,
    },
    Composite(Vec<VerifyError>),
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerifyError::HashMismatch {
                expected,
                actual,
                source,
            } => write!(
                f,
                "[{source}] Hash mismatch: expected {expected:#018x}, got {actual:#018x}"
            ),
            VerifyError::EventMismatch { tick, detail, source } => {
                write!(f, "[{source}] Event mismatch at tick {tick}: {detail}")
            }
            VerifyError::InvariantViolation { detail, source } => {
                write!(f, "[{source}] Invariant violation: {detail}")
            }
            VerifyError::Composite(errors) => {
                writeln!(f, "Multiple verification errors ({}):", errors.len())?;
                for (i, err) in errors.iter().enumerate() {
                    writeln!(f, "  {}. {}", i + 1, err)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for VerifyError {}

/// Trait for scenario verification strategies.
///
/// Implementors MUST NOT modify World state — only read and assert.
/// Called inside `Scenario::run()` after all ticks complete.
pub trait Verifier {
    /// Human-readable identifier for error messages.
    fn name(&self) -> &'static str;

    /// Verify the simulation outcome. Return Ok(()) on success.
    fn verify(
        &self,
        world: &mut World,
        events: &HashMap<u32, SimulationEvents>,
    ) -> Result<(), VerifyError>;
}
