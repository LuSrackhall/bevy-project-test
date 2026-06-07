use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;
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
        Query<(Entity, &Soldier, &Transform)>,
    )>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    // Collect enemy data: (Entity, SoldierType, Faction, position)
    let enemies: Vec<(Entity, SoldierType, Faction, Vec2)> = param_set.p1().iter()
        .map(|(e, s, t)| (e, s.soldier_type, s.faction, t.translation.xy()))
        .collect();

    for (entity, mut soldier, transform) in param_set.p0().iter_mut() {
        if soldier.soldier_type != SoldierType::Archer { continue; }
        soldier.attack_timer.tick(time.delta());
        if !soldier.attack_timer.just_finished() { continue; }

        let range = soldier_attack_range(SoldierType::Archer);
        let archer_pos = transform.translation.xy();
        let my_faction = soldier.faction;

        // Collect all enemies in attack range, sorted by distance
        let mut targets_in_range: Vec<(Entity, SoldierType, f32)> = enemies.iter()
            .filter(|(_, _, faction, _)| *faction != my_faction)
            .filter(|(_, _, _, pos)| archer_pos.distance(*pos) <= range)
            .map(|(e, st, _, pos)| (*e, *st, archer_pos.distance(*pos)))
            .collect();

        targets_in_range.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        if targets_in_range.is_empty() { continue; }

        let attack = soldier.attack + soldier.level as f32 * LEVEL_ATTACK_GAIN;

        let multi_chance = archer_multi_shot_chance(soldier.level);
        let target_count = if rng.gen::<f32>() < multi_chance {
            rng.gen_range(2..=5)
        } else {
            1
        };
        let actual_count = target_count.min(targets_in_range.len());

        for i in 0..actual_count {
            let (target_entity, target_stype, _) = targets_in_range[i];
            let mut damage = attack;
            if target_stype == SoldierType::Infantry { damage *= 0.9; }

            let arrow_color = match my_faction {
                Faction::Player => Color::srgb(0.3, 0.5, 0.9),
                Faction::Enemy => Color::srgb(0.9, 0.3, 0.3),
                Faction::Neutral => Color::srgb(0.6, 0.6, 0.6),
            };
            let arrow_shape = ShapeBuilder::with(&shapes::Circle { radius: 3.0, center: Vec2::ZERO })
                .fill(Fill::color(arrow_color))
                .build();

            commands.spawn((
                Arrow { target: target_entity, damage, speed: 400.0, from_faction: my_faction, shooter: Some(entity) },
                Transform::from_xyz(archer_pos.x, archer_pos.y, 4.0),
                arrow_shape,
            ));
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
    mut soldier_query: Query<(Entity, &mut Soldier, &Transform, Option<&InfantryShield>, Option<&SlowDebuff>)>,
) {
    // Collect death events to process after dropping target borrows
    let mut pending_deaths: Vec<(Entity, Option<Entity>, SoldierType, Faction, Option<Entity>)> = Vec::new();

    for (arrow_entity, arrow, arrow_transform) in arrow_query.iter() {
        if let Ok((_target_entity, mut target_soldier, target_transform, shield, slow_debuff)) = soldier_query.get_mut(arrow.target) {
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
                    let existing_stacks = slow_debuff.map(|s| s.stacks).unwrap_or(0);
                    let new_stacks = (existing_stacks + 1).min(MAX_SLOW_STACKS);
                    commands.entity(arrow.target).insert(SlowDebuff { stacks: new_stacks, timer: Timer::from_seconds(ARCHER_SLOW_DURATION, TimerMode::Once) });
                }
                if target_soldier.health <= 0.0 {
                    pending_deaths.push((arrow.target, arrow.shooter, target_soldier.soldier_type, target_soldier.faction, target_soldier.city_origin));
                }
                commands.entity(arrow_entity).despawn();
            }
        }
    }

    // Process deaths (after dropping all target borrows)
    for (target_entity, shooter, stype, faction, origin) in pending_deaths {
        // Grant XP to archer
        if let Some(shooter_entity) = shooter {
            if let Ok((_, mut killer, _, _, _)) = soldier_query.get_mut(shooter_entity) {
                killer.exp += EXP_PER_KILL;
                while killer.exp >= EXP_TO_LEVEL {
                    killer.exp -= EXP_TO_LEVEL;
                    killer.level += 1;
                    killer.max_health += LEVEL_HP_GAIN;
                    killer.health = killer.max_health;
                    killer.attack += LEVEL_ATTACK_GAIN;
                }
            }
        }
        commands.trigger(SoldierDiedEvent { entity: target_entity, killer: shooter, soldier_type: stype, faction, city_origin: origin });
        commands.entity(target_entity).despawn();
    }
}

fn melee_attack_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Soldier, &Transform, Option<&FearlessBuff>)>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    // Phase 1: Tick attacker timers, collect those ready to attack
    let mut attacks: Vec<(Entity, Entity, f32, SoldierType, u32, bool, Vec2)> = Vec::new();
    for (entity, mut soldier, transform, fearless) in query.iter_mut() {
        if soldier.soldier_type == SoldierType::Archer { continue; }
        soldier.attack_timer.tick(time.delta());
        if !soldier.attack_timer.just_finished() { continue; }
        if soldier.state != SoldierState::Fighting { continue; }
        if let Some(target) = soldier.target {
            let attack = soldier.attack + soldier.level as f32 * LEVEL_ATTACK_GAIN
                + if fearless.is_some() { FEARLESS_ATTACK_BONUS } else { 0.0 };
            attacks.push((entity, target, attack, soldier.soldier_type, soldier.level, fearless.is_some(), transform.translation.xy()));
        }
    }

    // Phase 2: Process attacks
    for (attacker_entity, target_entity, attack_power, stype, level, has_fearless, attacker_pos) in attacks {
        // Read target data
        let (target_health, target_max_health, target_st, target_origin, target_faction, target_pos) = {
            if let Ok((_, target, target_transform, _)) = query.get(target_entity) {
                (target.health, target.max_health, target.soldier_type, target.city_origin, target.faction, target_transform.translation.xy())
            } else {
                continue;
            }
        };

        if attacker_pos.distance(target_pos) > soldier_attack_range(stype) { continue; }

        let mut damage = attack_power;

        // Cavalry dodge + Fearless trigger
        if target_st == SoldierType::Cavalry {
            let health_ratio = if target_max_health > 0.0 { target_health / target_max_health } else { 0.0 };
            if rng.gen::<f32>() < cavalry_dodge_chance(health_ratio) {
                damage = 0.0;
                commands.entity(target_entity).insert(FearlessBuff { timer: Timer::from_seconds(FEARLESS_DURATION, TimerMode::Once) });
            }
        }

        if damage > 0.0 {
            let target_died = if let Ok((_, mut target_soldier, _, _)) = query.get_mut(target_entity) {
                target_soldier.health -= damage;
                target_soldier.health <= 0.0
            } else {
                continue;
            };

            if target_died {
                // Grant XP to killer (inline to avoid observer query conflict)
                if let Ok((_, mut killer, _, _)) = query.get_mut(attacker_entity) {
                    killer.exp += EXP_PER_KILL;
                    while killer.exp >= EXP_TO_LEVEL {
                        killer.exp -= EXP_TO_LEVEL;
                        killer.level += 1;
                        killer.max_health += LEVEL_HP_GAIN;
                        killer.health = killer.max_health;
                        killer.attack += LEVEL_ATTACK_GAIN;
                    }
                    // Lifesteal on kill (if applicable)
                    let mut ls = if level >= LIFESTEAL_UNLOCK_LEVEL { LIFESTEAL_RATE } else { 0.0 };
                    if has_fearless { ls += FEARLESS_LIFESTEAL_BONUS; }
                    if ls > 0.0 {
                        killer.health = (killer.health + damage * ls).min(killer.max_health);
                    }
                }
                commands.trigger(SoldierDiedEvent { entity: target_entity, killer: Some(attacker_entity), soldier_type: target_st, faction: target_faction, city_origin: target_origin });
                commands.entity(target_entity).despawn();
            } else {
                // Lifesteal on hit
                let mut lifesteal_rate = if level >= LIFESTEAL_UNLOCK_LEVEL { LIFESTEAL_RATE } else { 0.0 };
                if has_fearless { lifesteal_rate += FEARLESS_LIFESTEAL_BONUS; }
                if lifesteal_rate > 0.0 {
                    if let Ok((_, mut attacker, _, _)) = query.get_mut(attacker_entity) {
                        attacker.health = (attacker.health + damage * lifesteal_rate).min(attacker.max_health);
                    }
                }
            }
        }
    }
}
