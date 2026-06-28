pub mod ai;
pub mod city;
pub mod combat;
pub mod command;
pub mod events;
pub mod facing;
pub mod golden_test;
pub mod map;
pub mod replay;
pub mod run_config;
pub mod scenario;
pub mod soldier;
pub mod types;
pub mod unit_index;

use crate::city::config::CityGlobalConfig;
use crate::combat::config::CombatGlobalConfig;
use crate::command::*;
pub use crate::events::SimulationEvents;
use crate::replay::ReplayFile;
use crate::run_config::RunConfig;
use crate::soldier::config::SoldierConfig;
use crate::soldier::FactionComponent;
use crate::types::*;
pub use bevy_ecs::world::World;

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

    // Core resources
    world.insert_resource(DeterministicRng::new(seed));
    world.insert_resource(IdGenerator::new());
    world.insert_resource(CommandBuffer::default());
    world.insert_resource(GlobalSeekDirective::default());
    world.insert_resource(SimulationEvents::new());
    world.insert_resource(SimulationSeed(seed));

    world
}

/// Collect known command-producing players (Player=0, Enemy=1).
/// Neutral is excluded — it does not participate in the command pipeline.
fn collect_command_players(world: &mut World) -> Vec<u8> {
    let mut q = world.query::<&FactionComponent>();
    let mut players = Vec::new();
    for f in q.iter(world) {
        match f.0 {
            Faction::Player => { if !players.contains(&0) { players.push(0); } }
            Faction::Enemy => { if !players.contains(&1) { players.push(1); } }
            Faction::Neutral => {} // does not participate in command pipeline
        }
    }
    players
}

/// Run one complete simulation tick with explicit config.
/// Implements constitution §3.1 six-step Tick timing:
///   1. Command collection
///   2. No-Op injection for missing players
///   3. Command sorting by (player_id, sort_tag)
///   4. Command archiving (optional ReplayFile)
///   5. Deterministic simulation
///   6. State output
pub fn run_tick(world: &mut World, tick_number: u32, config: &RunConfig) -> SimulationEvents {
    // ── Step 1: Command collection ──
    let mut commands = world.resource_mut::<CommandBuffer>().take_for_tick(tick_number);

    // ── Step 2: No-Op injection for missing players ──
    let known_players = collect_command_players(world);
    let present_players: std::collections::HashSet<u8> =
        commands.iter().map(|c| c.player_id).collect();
    for player_id in known_players {
        if !present_players.contains(&player_id) {
            commands.push(GameCommand {
                tick: tick_number,
                player_id,
                action: Action::NoOp,
            });
        }
    }

    // ── Step 3: Command sorting ──
    commands.sort_by_key(|c| (c.player_id, c.action.sort_tag()));

    // ── Step 4: Command archiving (optional) ──
    if let Some(mut recorder) = world.get_resource_mut::<ReplayFile>() {
        recorder.record_tick(tick_number, commands.clone());
    }

    // ── Step 5: Deterministic simulation ──
    // Rebuild UnitId→Entity index for O(1) lookups this tick
    let unit_index = unit_index::UnitIdEntityIndex::rebuild(world);
    world.insert_resource(unit_index);

    // Clear previous events
    { *world.resource_mut::<SimulationEvents>() = SimulationEvents::new(); }

    // Phase 1: Consume pre-sorted commands
    soldier::consume_commands_system(world, commands);

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
    if config.enable_ai {
        ai::ai_decide(world, tick_number);
    }

    // ── Step 6: State output ──
    world.resource::<SimulationEvents>().clone()
}

/// Run one complete simulation tick with default config (AI enabled).
/// Convenience wrapper for backward compatibility.
pub fn run_tick_default(world: &mut World, tick_number: u32) -> SimulationEvents {
    run_tick(world, tick_number, &RunConfig::default())
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
        // MapGenConfig is now inserted by generate_map, not init_simulation_world
        // Verify core resources
        assert!(world.get_resource::<DeterministicRng>().is_some());
        assert!(world.get_resource::<IdGenerator>().is_some());
        assert!(world.get_resource::<CommandBuffer>().is_some());
    }

    #[test]
    fn test_map_generation_creates_cities() {
        let mut world = init_simulation_world(42);
        map::generate_map(&mut world, map::MapSize::Small);
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
