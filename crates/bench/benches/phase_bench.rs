use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simulation::types::*;
use simulation::soldier::*;
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
        world.spawn((
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
            FacingDirection { angle: Fixed::ZERO },
            AttackWindup { remaining_ticks: 0, target: None },
        ));
    }
    world
}

fn bench_phase_engagement_1000(c: &mut Criterion) {
    c.bench_function("phase/combat_engagement/1000", |b| {
        let mut world = create_world_with_soldiers(1000, 42);
        b.iter(|| {
            black_box(simulation::combat::combat_engagement_system(&mut world));
        });
    });
}

fn bench_phase_melee_1000(c: &mut Criterion) {
    c.bench_function("phase/melee_attack/1000", |b| {
        let mut world = create_world_with_soldiers(1000, 42);
        b.iter(|| {
            black_box(simulation::combat::melee_attack_system(&mut world, 1));
        });
    });
}

fn bench_phase_movement_1000(c: &mut Criterion) {
    c.bench_function("phase/soldier_movement/1000", |b| {
        let mut world = create_world_with_soldiers(1000, 42);
        b.iter(|| {
            black_box(simulation::soldier::soldier_movement_system(&mut world));
        });
    });
}

fn bench_phase_overlap_1000(c: &mut Criterion) {
    c.bench_function("phase/overlap_resolution/1000", |b| {
        let mut world = create_world_with_soldiers(1000, 42);
        b.iter(|| {
            black_box(simulation::soldier::overlap_resolution_system(&mut world));
        });
    });
}

criterion_group!(
    benches,
    bench_phase_engagement_1000,
    bench_phase_melee_1000,
    bench_phase_movement_1000,
    bench_phase_overlap_1000,
);
criterion_main!(benches);
