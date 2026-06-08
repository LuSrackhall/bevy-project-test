pub mod types;
pub mod command;
pub mod events;
pub mod soldier;
pub mod city;
pub mod combat;
pub mod map;
pub mod ai;

pub use bevy_ecs::world::World;
pub use crate::events::SimulationEvents;
use crate::types::*;
use crate::command::*;
use crate::soldier::config::SoldierConfig;
use crate::city::config::CityGlobalConfig;
use crate::combat::config::CombatGlobalConfig;
use crate::map::config::MapGenConfig;

/// Initialize a new simulation world with all configs and resources.
pub fn init_simulation_world(seed: u64) -> World {
    let mut world = World::new();

    // Components are auto-registered by bevy_ecs when used in queries

    // Load configs
    let soldier_config = SoldierConfig::from_ron(include_str!("../../../content/units.ron"))
        .expect("Failed to parse units.ron");
    world.insert_resource(soldier_config);

    let city_config = CityGlobalConfig::from_ron(include_str!("../../../content/cities.ron"))
        .expect("Failed to parse cities.ron");
    world.insert_resource(city_config);

    let combat_config = CombatGlobalConfig::from_ron(include_str!("../../../content/combat.ron"))
        .expect("Failed to parse combat.ron");
    world.insert_resource(combat_config);

    let map_config = MapGenConfig::from_ron(include_str!("../../../content/map.ron"))
        .expect("Failed to parse map.ron");
    world.insert_resource(map_config);

    // Core resources
    world.insert_resource(DeterministicRng::new(seed));
    world.insert_resource(IdGenerator::new());
    world.insert_resource(CommandBuffer::default());
    world.insert_resource(SimulationEvents::new());

    world
}

/// Run one complete simulation tick. Returns events for this tick.
pub fn run_tick(world: &mut World, tick_number: u32) -> SimulationEvents {
    // Clear previous events
    {
        let mut events = world.resource_mut::<SimulationEvents>();
        *events = SimulationEvents::new();
    }

    // Phase 1: Consume commands
    soldier::consume_commands_system(world, tick_number);

    // Phase 2: Combat engagement (auto-targeting)
    combat::combat_engagement_system(world);

    // Phase 3: Soldier movement
    soldier::soldier_movement_system(world);

    // Phase 4: City spawn
    soldier::city_spawn_system(world);

    // Phase 5: City capture check
    soldier::city_capture_check_system(world);

    // Phase 6: City interaction (soldiers entering cities)
    soldier::city_interaction_system(world);

    // Phase 7: Aura heal
    soldier::aura_heal_system(world);

    // Phase 8: Melee attacks
    combat::melee_attack_system(world);

    // Phase 9: Archer attacks
    combat::archer_attack_system(world);

    // Phase 10: Arrow hits
    combat::arrow_hit_system(world);

    // Phase 11: Arrow expiration
    combat::arrow_expire_system(world);

    // Phase 12: Slow debuff ticks
    soldier::slow_debuff_tick_system(world);

    // Phase 13: Fearless buff ticks
    soldier::fearless_buff_tick_system(world);

    // Phase 14: Soldier level up
    soldier::soldier_level_up_system(world);

    // Phase 15: AI decision
    ai::ai_decide(world, tick_number);

    // Extract and return events
    let events = world.resource::<SimulationEvents>().clone();
    events
}
