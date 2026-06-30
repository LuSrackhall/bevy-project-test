use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use simulation::types::*;
use simulation::soldier::*;
use simulation::soldier::config::SoldierConfig;
use simulation::combat::config::CombatGlobalConfig;

fn create_world_with_soldiers(count: usize, seed: u64) -> bevy_ecs::world::World {
    let mut world = simulation::init_simulation_world(seed);
    let soldier_config = world.resource::<SoldierConfig>().clone();
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    for i in 0..count {
        let uid = world.resource_mut::<IdGenerator>().next_id();
        let x = (i as i32 % 50) * 20;
        let y = (i as i32 / 50) * 20;
        let faction = if i % 2 == 0 { Faction::Player } else { Faction::Enemy };
        let cfg = soldier_config.get(SoldierType::Infantry);
        let shield_hp = combat_config.shield.initial_hp;
        let e = world.spawn((
            UnitIdComponent(uid),
            SoldierMarker,
            LogicalPosition(FixedVec2::new(Fixed::from_int(x), Fixed::from_int(y))),
            Movement {
                speed: cfg.speed,
                target: None,
                command_target: None,
                waypoint: None,
                force_move: false,
            },
            SeekStance { active: true, seek_range: 60 },
            Health { current: cfg.health, max: cfg.health },
            Attack {
                damage: cfg.attack,
                range: cfg.attack_range,
                interval_ticks: cfg.attack_interval_ticks,
                cooldown_remaining: 0,
            },
            FactionComponent(faction),
            SoldierTypeComponent(SoldierType::Infantry),
            Level { level: 1, exp: 0 },
            ShieldComponent { state: ShieldState::Normal },
            ShieldItem { hp: shield_hp, max_hp: shield_hp },
            CityOrigin(UnitId(0)),
            SoldierStateComponent(SoldierState::Moving),
        )).id();
        world.entity_mut(e).insert(FacingDirection { angle: Fixed::ZERO });
        world.entity_mut(e).insert(AttackWindup { remaining_ticks: 0, target: None });
    }
    world
}

fn bench_tick_empty(c: &mut Criterion) {
    c.bench_function("tick/empty", |b| {
        let mut world = simulation::init_simulation_world(42);
        let config = simulation::run_config::RunConfig::default();
        let mut tick = 0u32;
        b.iter(|| {
            tick += 1;
            black_box(simulation::run_tick(&mut world, tick, &config));
        });
    });
}

fn bench_tick_100_idle(c: &mut Criterion) {
    c.bench_function("tick/100_idle", |b| {
        let mut world = create_world_with_soldiers(100, 42);
        let config = simulation::run_config::RunConfig { enable_ai: false };
        let mut tick = 0u32;
        b.iter(|| {
            tick += 1;
            black_box(simulation::run_tick(&mut world, tick, &config));
        });
    });
}

fn bench_tick_1000_idle(c: &mut Criterion) {
    c.bench_function("tick/1000_idle", |b| {
        let mut world = create_world_with_soldiers(1000, 42);
        let config = simulation::run_config::RunConfig { enable_ai: false };
        let mut tick = 0u32;
        b.iter(|| {
            tick += 1;
            black_box(simulation::run_tick(&mut world, tick, &config));
        });
    });
}

fn bench_tick_1000_combat(c: &mut Criterion) {
    c.bench_function("tick/1000_combat", |b| {
        let mut world = create_world_with_soldiers(1000, 42);
        // Position soldiers close together to trigger combat
        {
            let mut q = world.query::<(Entity, &UnitIdComponent, &mut LogicalPosition)>();
            let mut i = 0;
            for (_, _, mut pos) in q.iter_mut(&mut world) {
                let faction_offset = if i % 2 == 0 { 0 } else { 30 };
                pos.0 = FixedVec2::new(
                    Fixed::from_int(faction_offset + (i % 20) * 5),
                    Fixed::from_int((i / 20) * 5),
                );
                i += 1;
            }
        }
        let config = simulation::run_config::RunConfig { enable_ai: false };
        let mut tick = 0u32;
        b.iter(|| {
            tick += 1;
            black_box(simulation::run_tick(&mut world, tick, &config));
        });
    });
}

fn bench_tick_5000_idle(c: &mut Criterion) {
    c.bench_function("tick/5000_idle", |b| {
        let mut world = create_world_with_soldiers(5000, 42);
        let config = simulation::run_config::RunConfig { enable_ai: false };
        let mut tick = 0u32;
        b.iter(|| {
            tick += 1;
            black_box(simulation::run_tick(&mut world, tick, &config));
        });
    });
}

fn bench_tick_1500_combat(c: &mut Criterion) {
    c.bench_function("tick/1500_combat", |b| {
        let mut world = create_world_with_soldiers(1500, 42);
        {
            let mut q = world.query::<(Entity, &UnitIdComponent, &mut LogicalPosition)>();
            let mut i = 0;
            for (_, _, mut pos) in q.iter_mut(&mut world) {
                let faction_offset = if i % 2 == 0 { 0 } else { 30 };
                pos.0 = FixedVec2::new(
                    Fixed::from_int(faction_offset + (i % 20) * 5),
                    Fixed::from_int((i / 20) * 5),
                );
                i += 1;
            }
        }
        let config = simulation::run_config::RunConfig { enable_ai: false };
        let mut tick = 0u32;
        b.iter(|| {
            tick += 1;
            black_box(simulation::run_tick(&mut world, tick, &config));
        });
    });
}

fn bench_tick_3000_combat(c: &mut Criterion) {
    c.bench_function("tick/3000_combat", |b| {
        let mut world = create_world_with_soldiers(3000, 42);
        {
            let mut q = world.query::<(Entity, &UnitIdComponent, &mut LogicalPosition)>();
            let mut i = 0;
            for (_, _, mut pos) in q.iter_mut(&mut world) {
                let faction_offset = if i % 2 == 0 { 0 } else { 30 };
                pos.0 = FixedVec2::new(
                    Fixed::from_int(faction_offset + (i % 20) * 5),
                    Fixed::from_int((i / 20) * 5),
                );
                i += 1;
            }
        }
        let config = simulation::run_config::RunConfig { enable_ai: false };
        let mut tick = 0u32;
        b.iter(|| {
            tick += 1;
            black_box(simulation::run_tick(&mut world, tick, &config));
        });
    });
}

criterion_group!(
    benches,
    bench_tick_empty,
    bench_tick_100_idle,
    bench_tick_1000_idle,
    bench_tick_1000_combat,
    bench_tick_1500_combat,
    bench_tick_3000_combat,
    bench_tick_5000_idle,
);
criterion_main!(benches);
