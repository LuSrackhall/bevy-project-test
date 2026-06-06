use bevy::prelude::*;
use rand::Rng;

use crate::core::*;
use crate::soldier::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, combat_engagement_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, archer_attack_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, arrow_movement_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, arrow_damage_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, melee_attack_system.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Arrow {
    pub target: Entity,
    pub damage: f32,
    pub speed: f32,
    pub from_faction: Faction,
    pub shooter: Option<Entity>,
}

fn combat_engagement_system(
    mut param_set: ParamSet<(
        Query<(Entity, &mut Soldier, &Transform), Without<Arrow>>,
        Query<(Entity, &Soldier, &Transform), Without<Arrow>>,
    )>,
) {
    // Collect enemy data first from the immutable query
    let enemies: Vec<(Entity, Faction, Vec2)> = param_set.p1().iter()
        .map(|(e, s, t)| (e, s.faction, t.translation.xy()))
        .collect();
    // Then iterate soldiers from the mutable query
    for (_entity, mut soldier, transform) in param_set.p0().iter_mut() {
        let range = soldier_attack_range(soldier.soldier_type);
        let mut best: Option<(Entity, f32)> = None;
        for (enemy_entity, enemy_faction, enemy_pos) in &enemies {
            if *enemy_faction == soldier.faction { continue; }
            let dist = transform.translation.xy().distance(*enemy_pos);
            if dist <= range && best.as_ref().map_or(true, |(_, d)| dist < *d) {
                best = Some((*enemy_entity, dist));
            }
        }
        if let Some((target, _)) = best {
            soldier.target = Some(target);
            soldier.state = SoldierState::Fighting;
        } else {
            soldier.state = SoldierState::Moving;
        }
    }
}

fn archer_attack_system(
    time: Res<Time>,
    mut param_set: ParamSet<(
        Query<(Entity, &mut Soldier, &Transform)>,
        Query<(Entity, &Soldier)>,
    )>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    // Collect target soldier data before mutable iteration to avoid Query conflict
    let target_data: Vec<(Entity, SoldierType)> = param_set.p1().iter()
        .map(|(e, s)| (e, s.soldier_type))
        .collect();
    for (entity, mut soldier, transform) in param_set.p0().iter_mut() {
        if soldier.soldier_type != SoldierType::Archer { continue; }
        soldier.attack_timer.tick(time.delta());
        if !soldier.attack_timer.just_finished() { continue; }
        if let Some(target) = soldier.target {
            if let Some((_, target_stype)) = target_data.iter().find(|(e, _)| *e == target) {
                let attack = soldier.attack + soldier.level as f32 * LEVEL_ATTACK_GAIN;
                let mut damage = attack;
                if *target_stype == SoldierType::Infantry { damage *= 0.9; }
                let multi_chance = archer_multi_shot_chance(soldier.level);
                let target_count = if rng.gen::<f32>() < multi_chance { rng.gen_range(2..=5) } else { 1 };
                for _ in 0..target_count {
                    commands.spawn((
                        Arrow { target, damage, speed: 400.0, from_faction: soldier.faction, shooter: Some(entity) },
                        Transform::from_xyz(transform.translation.x, transform.translation.y, 4.0),
                    ));
                }
            }
        }
    }
}

fn arrow_movement_system(
    time: Res<Time>,
    mut arrow_query: Query<(Entity, &mut Transform, &Arrow)>,
    target_query: Query<&Transform, Without<Arrow>>,
) {
    for (_entity, mut transform, arrow) in arrow_query.iter_mut() {
        if let Ok(target_transform) = target_query.get(arrow.target) {
            let dir = (target_transform.translation.truncate() - transform.translation.xy()).normalize_or_zero();
            transform.translation.x += dir.x * arrow.speed * time.delta_secs();
            transform.translation.y += dir.y * arrow.speed * time.delta_secs();
        }
    }
}

fn arrow_damage_system(
    mut commands: Commands,
    arrow_query: Query<(Entity, &Arrow, &Transform)>,
    mut soldier_query: Query<(Entity, &mut Soldier, &Transform, Option<&InfantryShield>)>,
) {
    for (arrow_entity, arrow, arrow_transform) in arrow_query.iter() {
        if let Ok((_target_entity, mut target_soldier, target_transform, shield)) = soldier_query.get_mut(arrow.target) {
            let dist = arrow_transform.translation.xy().distance(target_transform.translation.xy());
            if dist < 15.0 {
                let mut damage = arrow.damage;
                if dist <= ARCHER_MELEE_RANGE { damage *= ARCHER_MELEE_DAMAGE_MULT; }
                if let Some(shield) = shield {
                    if shield.0 == ShieldState::ShieldUp {
                        let to_arrow = (arrow_transform.translation.xy() - target_transform.translation.xy()).normalize_or_zero();
                        let forward = target_transform.rotation * Vec3::Y;
                        if to_arrow.dot(forward.truncate()) > 0.0 {
                            let mut rng = rand::thread_rng();
                            if rng.gen::<f32>() < SHIELD_INTERCEPT_CHANCE { damage *= 1.0 - SHIELD_DAMAGE_REDUCTION; }
                        }
                    }
                }
                target_soldier.health -= damage;
                if arrow.from_faction != target_soldier.faction {
                    commands.entity(arrow.target).insert(SlowDebuff { stacks: 1, timer: Timer::from_seconds(ARCHER_SLOW_DURATION, TimerMode::Once) });
                }
                if target_soldier.health <= 0.0 {
                    commands.trigger(SoldierDiedEvent { entity: arrow.target, killer: arrow.shooter, soldier_type: target_soldier.soldier_type, faction: target_soldier.faction, city_origin: target_soldier.city_origin });
                    commands.entity(arrow.target).despawn();
                }
                commands.entity(arrow_entity).despawn();
            }
        }
    }
}

fn melee_attack_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Soldier, &Transform)>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    let soldier_ids: Vec<Entity> = query.iter().filter(|(_, s, _)| s.soldier_type != SoldierType::Archer && s.state == SoldierState::Fighting).map(|(e, _, _)| e).collect();

    // Collect attack data to avoid double borrow
    let mut attacks: Vec<(Entity, Option<Entity>, f32, SoldierType, u32)> = Vec::new();
    for attacker_entity in &soldier_ids {
        if let Ok((_, attacker, _)) = query.get(*attacker_entity) {
            attacks.push((
                *attacker_entity,
                attacker.target,
                attacker.attack + attacker.level as f32 * LEVEL_ATTACK_GAIN,
                attacker.soldier_type,
                attacker.level,
            ));
        }
    }

    for (attacker_entity, target_opt, attack_power, stype, level) in attacks {
        if let Some(target_entity) = target_opt {
            // Collect target data first, drop the borrow
            let (target_health_before, target_max_health, target_soldier_type, target_city_origin, target_faction) = {
                if let Ok((_, target, _)) = query.get(target_entity) {
                    (target.health, target.max_health, target.soldier_type, target.city_origin, target.faction)
                } else {
                    continue;
                }
            };
            let dist = {
                let Ok((_, _, tgt_t)) = query.get(target_entity) else { continue };
                let Ok((_, _, atk_t)) = query.get(attacker_entity) else { continue };
                atk_t.translation.xy().distance(tgt_t.translation.xy())
            };
            if dist > soldier_attack_range(stype) { continue; }

            let mut damage = attack_power;
            if target_soldier_type == SoldierType::Cavalry {
                let health_ratio = target_health_before / target_max_health;
                let dodge = cavalry_dodge_chance(health_ratio);
                if rng.gen::<f32>() < dodge {
                    damage = 0.0;
                    commands.entity(target_entity).insert(FearlessBuff { timer: Timer::from_seconds(FEARLESS_DURATION, TimerMode::Once) });
                }
            }
            if damage > 0.0 {
                let target_died = {
                    let Ok((_, mut target_soldier, _)) = query.get_mut(target_entity) else { continue };
                    target_soldier.attack_timer.tick(time.delta());
                    target_soldier.health -= damage;
                    if target_soldier.health <= 0.0 {
                        commands.trigger(SoldierDiedEvent { entity: target_entity, killer: Some(attacker_entity), soldier_type: target_soldier_type, faction: target_faction, city_origin: target_city_origin });
                        commands.entity(target_entity).despawn();
                        true
                    } else {
                        false
                    }
                };
                if !target_died {
                    let lifesteal_rate = if level >= LIFESTEAL_UNLOCK_LEVEL { LIFESTEAL_RATE } else { 0.0 };
                    if lifesteal_rate > 0.0 {
                        if let Ok((_, mut attacker, _)) = query.get_mut(attacker_entity) {
                            attacker.health = (attacker.health + damage * lifesteal_rate).min(attacker.max_health);
                        }
                    }
                }
            }
        }
    }
}
