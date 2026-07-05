use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use simulation::types::*;
use simulation::soldier::*;
use simulation::soldier::config::SoldierConfig;
use simulation::combat::config::CombatGlobalConfig;

fn create_world_mixed(count: usize, seed: u64) -> bevy_ecs::world::World {
    let mut world = simulation::init_simulation_world(seed);
    let soldier_config = world.resource::<SoldierConfig>().clone();
    let combat_config = world.resource::<CombatGlobalConfig>().clone();

    for i in 0..count {
        let uid = world.resource_mut::<IdGenerator>().next_id();
        let x = (i as i32 % 50) * 20;
        let y = (i as i32 / 50) * 20;
        let faction = if i % 2 == 0 { FactionId(0) } else { FactionId(1) };
        let stype = if i % 3 == 0 { SoldierType::Archer } else { SoldierType::Infantry };
        let cfg = soldier_config.get(stype);
        let shield_hp = combat_config.shield.initial_hp;
        let e = world.spawn((
            UnitIdComponent(uid),
            SoldierMarker,
            LogicalPosition(FixedVec2::new(Fixed::from_int(x), Fixed::from_int(y))),
            Movement { speed: cfg.speed, target: None, command_target: None, waypoint: None, force_move: false },
            SeekStance { active: true, seek_range: 60 },
            Health { current: cfg.health, max: cfg.health },
            Attack { damage: cfg.attack, range: cfg.attack_range, interval_ticks: cfg.attack_interval_ticks, cooldown_remaining: 0 },
            FactionComponent(faction),
            SoldierTypeComponent(stype),
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

fn compact_all(world: &mut World) {
    let mut q = world.query::<(Entity, &FactionComponent, &UnitIdComponent, &mut LogicalPosition)>();
    let mut i = 0;
    for (_, _, _, mut pos) in q.iter_mut(world) {
        let fx = i % 20;
        let fy = (i / 20) % 50;
        let faction_offset = if i % 2 == 0 { 0 } else { 30 };
        pos.0 = FixedVec2::new(Fixed::from_int(faction_offset + fx * 5), Fixed::from_int(fy * 5));
        i += 1;
    }
}

fn bench_phase_engagement_compact(c: &mut Criterion) {
    c.bench_function("phase/compact/combat_engagement/1000", |b| {
        let mut world = create_world_mixed(1000, 42);
        compact_all(&mut world);
        b.iter(|| { black_box(simulation::combat::combat_engagement_system(&mut world)); });
    });
}

fn bench_phase_melee_compact(c: &mut Criterion) {
    c.bench_function("phase/compact/melee_attack/1000", |b| {
        let mut world = create_world_mixed(1000, 42);
        compact_all(&mut world);
        b.iter(|| { black_box(simulation::combat::melee_attack_system(&mut world, 1)); });
    });
}

fn bench_phase_overlap_compact(c: &mut Criterion) {
    c.bench_function("phase/compact/overlap/1000", |b| {
        let mut world = create_world_mixed(1000, 42);
        compact_all(&mut world);
        b.iter(|| { black_box(simulation::soldier::overlap_resolution_system(&mut world)); });
    });
}

fn bench_phase_archer_compact(c: &mut Criterion) {
    c.bench_function("phase/compact/archer_attack/1000", |b| {
        let mut world = create_world_mixed(1000, 42);
        compact_all(&mut world);
        b.iter(|| { black_box(simulation::combat::archer_attack_system(&mut world)); });
    });
}

fn bench_phase_arrow_compact(c: &mut Criterion) {
    c.bench_function("phase/compact/arrow_movement/1000", |b| {
        let mut world = create_world_mixed(1000, 42);
        compact_all(&mut world);
        simulation::combat::archer_attack_system(&mut world);
        b.iter(|| { black_box(simulation::combat::arrow_movement_system(&mut world, 1)); });
    });
}

fn bench_phase_engagement_compact_1500(c: &mut Criterion) {
    c.bench_function("phase/compact/combat_engagement/1500", |b| {
        let mut world = create_world_mixed(1500, 42);
        compact_all(&mut world);
        b.iter(|| { black_box(simulation::combat::combat_engagement_system(&mut world)); });
    });
}

fn bench_phase_engagement_compact_3000(c: &mut Criterion) {
    c.bench_function("phase/compact/combat_engagement/3000", |b| {
        let mut world = create_world_mixed(3000, 42);
        compact_all(&mut world);
        b.iter(|| { black_box(simulation::combat::combat_engagement_system(&mut world)); });
    });
}

fn bench_phase_melee_compact_1500(c: &mut Criterion) {
    c.bench_function("phase/compact/melee_attack/1500", |b| {
        let mut world = create_world_mixed(1500, 42);
        compact_all(&mut world);
        b.iter(|| { black_box(simulation::combat::melee_attack_system(&mut world, 1)); });
    });
}

criterion_group!(
    benches,
    bench_phase_engagement_compact,
    bench_phase_engagement_compact_1500,
    bench_phase_engagement_compact_3000,
    bench_phase_melee_compact,
    bench_phase_melee_compact_1500,
    bench_phase_overlap_compact,
    bench_phase_archer_compact,
    bench_phase_arrow_compact,
);
criterion_main!(benches);
