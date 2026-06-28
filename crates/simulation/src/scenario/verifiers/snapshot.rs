use std::collections::HashMap;

use bevy_ecs::world::World;

use crate::events::SimulationEvents;
use crate::golden_test::hash_world_state;

use super::super::verifier::{Verifier, VerifyError};

/// Verifies that the world state hash matches an expected value.
pub struct SnapshotVerifier {
    pub expected: u64,
}

impl SnapshotVerifier {
    pub fn hash(expected: u64) -> Self {
        Self { expected }
    }
}

impl Verifier for SnapshotVerifier {
    fn name(&self) -> &'static str {
        "SnapshotVerifier"
    }

    fn verify(
        &self,
        world: &mut World,
        _events: &HashMap<u32, SimulationEvents>,
    ) -> Result<(), VerifyError> {
        let actual = hash_world_state(world);
        if actual != self.expected {
            Err(VerifyError::HashMismatch {
                expected: self.expected,
                actual,
                source: self.name(),
            })
        } else {
            Ok(())
        }
    }
}
