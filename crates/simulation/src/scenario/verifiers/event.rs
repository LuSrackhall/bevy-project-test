use std::collections::HashMap;

use bevy_ecs::world::World;

use crate::events::{SimulationEvents, UnitSpawned, CityCaptured, DamageDealt, SoldierLeveledUp, UnitDestroyed};

use super::super::verifier::{Verifier, VerifyError};

type EventCheck = Box<dyn Fn(&SimulationEvents) -> Option<String>>;

/// Verifies events at specific ticks using builder-pattern checks.
pub struct EventVerifier {
    checks: Vec<(u32, EventCheck)>,
}

impl EventVerifier {
    pub fn new() -> Self {
        Self { checks: vec![] }
    }

    /// Expect a predicate on spawned events at a given tick.
    pub fn expect_spawned_at(
        mut self,
        tick: u32,
        pred: impl Fn(&[UnitSpawned]) -> bool + 'static,
    ) -> Self {
        self.checks.push((
            tick,
            Box::new(move |evs| {
                if pred(&evs.spawned) {
                    None
                } else {
                    Some(format!(
                        "spawned check failed: got {} events",
                        evs.spawned.len()
                    ))
                }
            }),
        ));
        self
    }

    /// Expect a predicate on captured events at a given tick.
    pub fn expect_captured_at(
        mut self,
        tick: u32,
        pred: impl Fn(&[CityCaptured]) -> bool + 'static,
    ) -> Self {
        self.checks.push((
            tick,
            Box::new(move |evs| {
                if pred(&evs.captured) {
                    None
                } else {
                    Some(format!(
                        "captured check failed: got {} events",
                        evs.captured.len()
                    ))
                }
            }),
        ));
        self
    }

    /// Expect a predicate on damage events at a given tick.
    pub fn expect_damage_at(
        mut self,
        tick: u32,
        pred: impl Fn(&[DamageDealt]) -> bool + 'static,
    ) -> Self {
        self.checks.push((
            tick,
            Box::new(move |evs| {
                if pred(&evs.damage) {
                    None
                } else {
                    Some(format!(
                        "damage check failed: got {} events",
                        evs.damage.len()
                    ))
                }
            }),
        ));
        self
    }

    /// Expect a predicate on destroyed events at a given tick.
    pub fn expect_destroyed_at(
        mut self,
        tick: u32,
        pred: impl Fn(&[UnitDestroyed]) -> bool + 'static,
    ) -> Self {
        self.checks.push((
            tick,
            Box::new(move |evs| {
                if pred(&evs.destroyed) {
                    None
                } else {
                    Some(format!(
                        "destroyed check failed: got {} events",
                        evs.destroyed.len()
                    ))
                }
            }),
        ));
        self
    }

    /// Expect a predicate on level-up events at a given tick.
    pub fn expect_leveled_up_at(
        mut self,
        tick: u32,
        pred: impl Fn(&[SoldierLeveledUp]) -> bool + 'static,
    ) -> Self {
        self.checks.push((
            tick,
            Box::new(move |evs| {
                if pred(&evs.leveled_up) {
                    None
                } else {
                    Some(format!(
                        "leveled_up check failed: got {} events",
                        evs.leveled_up.len()
                    ))
                }
            }),
        ));
        self
    }
}

impl Verifier for EventVerifier {
    fn name(&self) -> &'static str {
        "EventVerifier"
    }

    fn verify(
        &self,
        _world: &mut World,
        events: &HashMap<u32, SimulationEvents>,
    ) -> Result<(), VerifyError> {
        let mut errors = vec![];
        for (tick, check) in &self.checks {
            let evs = events.get(tick).unwrap_or_else(|| {
                panic!("EventVerifier: no events collected for tick {tick}")
            });
            if let Some(detail) = check(evs) {
                errors.push(VerifyError::EventMismatch {
                    tick: *tick,
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
