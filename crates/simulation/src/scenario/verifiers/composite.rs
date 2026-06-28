use std::collections::HashMap;

use bevy_ecs::world::World;

use crate::events::SimulationEvents;

use super::super::verifier::{Verifier, VerifyError};

/// Combines multiple verifiers, collecting all errors.
/// Returns Ok only if ALL verifiers pass.
pub struct CompositeVerifier(pub Vec<Box<dyn Verifier>>);

impl Verifier for CompositeVerifier {
    fn name(&self) -> &'static str {
        "CompositeVerifier"
    }

    fn verify(
        &self,
        world: &mut World,
        events: &HashMap<u32, SimulationEvents>,
    ) -> Result<(), VerifyError> {
        let mut errors = vec![];
        for verifier in &self.0 {
            if let Err(e) = verifier.verify(world, events) {
                errors.push(e);
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
