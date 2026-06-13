pub mod types;
pub mod command;
pub mod events;
pub mod soldier;
pub mod city;
pub mod combat;
pub mod map;
pub mod ai;
pub mod facing;

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
    world.insert_resource(GlobalSeekDirective::default());
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

    // Phase 2.5: Facing direction turn
    facing::facing_turn_system(world);

    // Phase 3: Soldier movement
    soldier::soldier_movement_system(world);

    // Phase 4: City spawn
    soldier::city_spawn_system(world);

    // Phase 4.5: Overlap resolution (post-tick collision resolve)
    soldier::overlap_resolution_system(world);

    // Phase 5: City capture check
    soldier::city_capture_check_system(world);

    // Phase 6: City interaction (soldiers entering cities)
    soldier::city_interaction_system(world);

    // Phase 6.5: Shield pickup (soldiers picking up dropped shields)
    soldier::shield_pickup_system(world);

    // Phase 7: Aura heal
    soldier::aura_heal_system(world);

    // Phase 8: Melee attacks
    combat::melee_attack_system(world, tick_number);

    // Phase 8.5: Attack windup completion (non-cavalry delayed attacks)
    combat::attack_windup_system(world, tick_number);

    // Phase 9: Archer attacks (direction-based)
    combat::archer_attack_system(world);

    // Phase 10: Arrow movement (flight + collision + decay)
    combat::arrow_movement_system(world, tick_number);

    // Phase 11: Slow debuff ticks
    soldier::slow_debuff_tick_system(world);

    // Phase 12: Fearless buff ticks
    soldier::fearless_buff_tick_system(world);

    // Phase 13: Soldier level up
    soldier::soldier_level_up_system(world);

    // Phase 13.5: Shield decay (despawn expired dropped shields)
    soldier::shield_decay_system(world, tick_number);

    // Phase 14: AI decision
    ai::ai_decide(world, tick_number);

    // Extract and return events
    let events = world.resource::<SimulationEvents>().clone();
    events
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_init_simulation_world_parses_all_configs() {
        let world = init_simulation_world(42);
        // Verify all configs loaded
        assert!(world.get_resource::<SoldierConfig>().is_some());
        assert!(world.get_resource::<CityGlobalConfig>().is_some());
        assert!(world.get_resource::<CombatGlobalConfig>().is_some());
        assert!(world.get_resource::<MapGenConfig>().is_some());
        // Verify core resources
        assert!(world.get_resource::<DeterministicRng>().is_some());
        assert!(world.get_resource::<IdGenerator>().is_some());
        assert!(world.get_resource::<CommandBuffer>().is_some());
    }

    #[test]
    fn test_map_generation_creates_cities() {
        let mut world = init_simulation_world(42);
        map::generate_map(&mut world);
        // Verify cities were created
        let mut query = world.query::<(&soldier::CityComponent,)>();
        let count = query.iter(&mut world).count();
        assert!(count >= 6, "Expected at least 6 cities, got {}", count);
    }

    #[test]
    fn test_soldier_config_values() {
        let world = init_simulation_world(42);
        let config = world.resource::<SoldierConfig>();
        let militia = config.get(SoldierType::Militia);
        assert_eq!(militia.health, 100);
        assert_eq!(militia.attack, 16);
        assert_eq!(militia.speed, 80);
    }
}
