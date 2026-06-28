use std::collections::HashMap;

use bevy_ecs::world::World;

use crate::events::SimulationEvents;

use super::super::verifier::{Verifier, VerifyError};

type InvariantCheck = Box<dyn Fn(&World) -> Option<String>>;

/// Verifies invariants on the world state after simulation completes.
pub struct InvariantVerifier {
    checks: Vec<InvariantCheck>,
}

impl InvariantVerifier {
    pub fn new() -> Self {
        Self { checks: vec![] }
    }

    /// Add an invariant check. The closure returns None on success,
    /// Some(detail) on failure.
    pub fn check(mut self, check: impl Fn(&World) -> Option<String> + 'static) -> Self {
        self.checks.push(Box::new(check));
        self
    }
}

impl Verifier for InvariantVerifier {
    fn name(&self) -> &'static str {
        "InvariantVerifier"
    }

    fn verify(
        &self,
        world: &mut World,
        _events: &HashMap<u32, SimulationEvents>,
    ) -> Result<(), VerifyError> {
        let mut errors = vec![];
        for check in &self.checks {
            if let Some(detail) = check(world) {
                errors.push(VerifyError::InvariantViolation {
                    detail,
                    source: self.name(),
                });
            }
        }
        if errors.is_empty() {
            Ok(())
        } else if errors.len() == 1 {
            Err(errors.into_iter().next().unwrap())
        } else {
            Err(VerifyError::Composite(errors))
        }
    }
}
