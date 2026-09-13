//! Guards the world-construction entry points against drifting apart.
//!
//! Two functions build a simulation world, reached by different modes:
//!
//! * solo play → [`simulation::init_simulation_world`]
//! * network play, including the reconnect rebuild →
//!   [`bevy_adapter::session::reconnect::rebuild_world`]
//!
//! These used to maintain two hand-copied resource lists. A resource added to
//! one and not the other gives that mode state the other lacks, and the symptom
//! is a missing-resource panic or a quiet behavioural difference — never a
//! compile error. `crates/simulation/src/lib.rs` now forwards the solo entry
//! point to the shared implementation; these tests keep it that way.
//!
//! The test that previously claimed to cover this
//! (`test_reconnect_rebuild_matches_live_network_path`) compared the same
//! function against itself and could not fail.

use std::any::TypeId;
use std::collections::BTreeMap;

use bevy_adapter::session::reconnect::rebuild_world;
use simulation::types::{Controller, PlayerSlots};
use simulation::World;

/// Resource type name → type id, so a failure can name the differing types.
fn resource_types(world: &World) -> BTreeMap<String, TypeId> {
    world
        .iter_resources()
        .filter_map(|(info, _)| info.type_id().map(|id| (info.name().to_string(), id)))
        .collect()
}

#[test]
fn solo_and_network_construction_install_the_same_resource_set() {
    let solo = simulation::init_simulation_world(42);
    let network = rebuild_world(42, 4, 0);

    let solo_types = resource_types(&solo);
    let network_types = resource_types(&network);

    let only_solo: Vec<&String> = solo_types
        .keys()
        .filter(|k| !network_types.contains_key(*k))
        .collect();
    let only_network: Vec<&String> = network_types
        .keys()
        .filter(|k| !solo_types.contains_key(*k))
        .collect();

    assert!(
        only_solo.is_empty() && only_network.is_empty(),
        "solo and network world construction diverged:\n  \
         only in solo: {only_solo:?}\n  \
         only in network: {only_network:?}\n  \
         A resource installed by only one entry point gives that mode state the other never gets."
    );
}

#[test]
fn the_entry_points_differ_only_in_player_slots() {
    // Same resources, deliberately different rosters: solo is 1 human + 1 AI,
    // network is N humans with the AI off. If this stops holding, the two modes
    // are no longer running the same game.
    let solo = simulation::init_simulation_world(42);
    let network = rebuild_world(42, 4, 0);

    let solo_slots = solo.resource::<PlayerSlots>();
    assert_eq!(solo_slots.slots.len(), 2, "solo is 1 human + 1 AI");
    assert!(matches!(
        solo_slots.slots[0].controller,
        Controller::HumanLocal
    ));
    assert!(matches!(solo_slots.slots[1].controller, Controller::AI(_)));

    let network_slots = network.resource::<PlayerSlots>();
    assert_eq!(network_slots.slots.len(), 4);
    for (i, slot) in network_slots.slots.iter().enumerate() {
        let expect_local = i == 0;
        assert_eq!(
            matches!(slot.controller, Controller::HumanLocal),
            expect_local,
            "slot {i} should be {} in a network game, got {:?}",
            if expect_local {
                "the local human"
            } else {
                "a remote human"
            },
            slot.controller
        );
    }
}
