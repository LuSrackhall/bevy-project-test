use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;

use crate::core::*;

pub struct SoldierPlugin;

impl Plugin for SoldierPlugin {
    fn build(&self, app: &mut App) {
        // Experience/population on kill handled inline in combat systems to avoid Query conflict
        app.add_observer(|trigger: On<SoldierDiedEvent>, mut city_query: Query<&mut City>| {
            // Decrease origin city population when soldier dies in combat
            if let Some(origin) = trigger.city_origin {
                if let Ok(mut city) = city_query.get_mut(origin) {
                    city.population = city.population.saturating_sub(1);
                }
            }
        });
        app.add_systems(Update, soldier_movement_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, soldier_city_interaction_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, soldier_aura_heal_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, slow_debuff_tick_system.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Soldier {
    pub soldier_type: SoldierType,
    pub faction: Faction,
    pub health: f32,
    pub max_health: f32,
    pub attack: f32,
    pub speed: f32,
    pub level: u32,
    pub exp: u32,
    pub attack_timer: Timer,
    pub target: Option<Entity>,
    pub state: SoldierState,
    pub city_origin: Option<Entity>,
    pub is_exiled: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ShieldState {
    Normal,
    ShieldUp,
}

#[derive(Component)]
pub struct InfantryShield(pub ShieldState);

#[derive(Component)]
pub struct FearlessBuff {
    pub timer: Timer,
}

#[derive(Component)]
pub struct SlowDebuff {
    pub stacks: u32,
    pub timer: Timer,
}

#[derive(Event)]
pub struct SoldierDiedEvent {
    pub entity: Entity,
    pub killer: Option<Entity>,
    pub soldier_type: SoldierType,
    pub faction: Faction,
    pub city_origin: Option<Entity>,
}

pub struct SoldierBundle;

impl SoldierBundle {
    pub fn spawn(commands: &mut Commands, soldier_type: SoldierType, faction: Faction, position: Vec2, city_origin: Option<Entity>) {
        let color = match faction {
            Faction::Player => Color::srgb(0.3, 0.5, 0.9),
            Faction::Enemy => Color::srgb(0.9, 0.3, 0.3),
            Faction::Neutral => Color::srgb(0.5, 0.5, 0.5),
        };
        // Build shape inline (exactly like cities do)
        let shape = ShapeBuilder::with(&shapes::Circle { radius: 12.0, center: Vec2::ZERO })
            .fill(Fill::color(color))
            .build();
        commands.spawn((
            Soldier {
                soldier_type,
                faction,
                health: soldier_base_health(soldier_type),
                max_health: soldier_base_health(soldier_type),
                attack: soldier_base_attack(soldier_type),
                speed: soldier_base_speed(soldier_type),
                level: 1,
                exp: 0,
                attack_timer: Timer::from_seconds(ATTACK_INTERVAL, TimerMode::Repeating),
                target: None,
                state: SoldierState::Moving,
                city_origin,
                is_exiled: false,
            },
            Transform::from_xyz(position.x, position.y, 3.0),
            shape,
        ));
    }
}

fn soldier_movement_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Soldier, Option<&SlowDebuff>, Option<&InfantryShield>)>,
    target_query: Query<&Transform, Without<Soldier>>,
) {
    for (_entity, mut transform, mut soldier, slow, shield) in query.iter_mut() {
        let mut speed = soldier.speed;
        if let Some(shield) = shield {
            if shield.0 == ShieldState::ShieldUp {
                speed -= SHIELD_SPEED_PENALTY;
            }
        }
        if let Some(slow) = slow {
            let mult = ARCHER_SLOW_AMOUNT * ARCHER_SLOW_STACK_MULT.powi(slow.stacks as i32 - 1);
            let capped = mult.max(ARCHER_SLOW_MAX_REDUCTION);
            speed *= capped;
        }

        if let Some(target) = soldier.target {
            if let Ok(target_transform) = target_query.get(target) {
                let target_pos = target_transform.translation.xy();
                let my_pos = transform.translation.xy();
                let dist = my_pos.distance(target_pos);

                // Check if reached target (waypoint or static point)
                if dist < 5.0 {
                    soldier.target = None;
                    soldier.state = SoldierState::Moving;
                    continue;
                }

                let dir = (target_pos - my_pos).normalize_or_zero();
                transform.translation.x += dir.x * speed * time.delta_secs();
                transform.translation.y += dir.y * speed * time.delta_secs();
            }
        }
    }
}

use crate::city::City; // Removed CityCapturedEvent — capture now handled by city_capture_system

fn soldier_city_interaction_system(
    mut commands: Commands,
    soldier_query: Query<(Entity, &Transform, &Soldier)>,
    mut city_query: Query<(Entity, &mut City, &Transform)>,
) {
    // Track origin cities that need population decrement
    let mut origin_decrements: Vec<Entity> = Vec::new();

    for (soldier_entity, soldier_transform, soldier) in soldier_query.iter() {
        for (city_entity, mut city, city_transform) in city_query.iter_mut() {
            let dist = soldier_transform.translation.xy()
                .distance(city_transform.translation.xy());
            let threshold = city.visual_radius;
            if dist > threshold { continue; }

            if soldier.faction != city.faction {
                // Enemy/neutral city: always auto-attack
                let damage = city_damage_per_soldier(soldier.attack);
                city.health -= damage;
                city.last_attacker_faction = Some(soldier.faction);
            } else {
                // Friendly city: only interact if explicitly commanded (soldier.target points to this city)
                if soldier.target == Some(city_entity) {
                    if city.health < city.max_health {
                        let heal = city_heal_amount(city.max_health, city.health);
                        city.health = (city.health + heal).min(city.max_health);
                    } else if city.level < city.max_level {
                        let exp_gain = city_level_up_gain(city.max_health);
                        city.level_exp += exp_gain;
                        let required = city_level_up_exp(city.max_health);
                        if city.level_exp >= required {
                            city.level_exp -= required;
                            city.level += 1;
                            city.max_health = city_max_health(city.level);
                            city.health = city.max_health;
                            city.max_population = city_max_population(city.level, &mut rand::thread_rng());
                            city.visual_radius = 20.0 + city.level as f32 * 5.0;
                        }
                    }
                } else {
                    // Soldier just passing through friendly city — skip
                    continue;
                }
            }

            // Decrease origin city population (soldier consumed by city entry)
            if let Some(origin) = soldier.city_origin {
                origin_decrements.push(origin);
            }

            // Soldier consumed — city capture is handled independently by city_capture_system
            commands.entity(soldier_entity).despawn();
            break;
        }
    }

    // Apply population decrements (after dropping city_query mutable borrow from iter_mut)
    for origin in origin_decrements {
        if let Ok((_, mut city, _)) = city_query.get_mut(origin) {
            city.population = city.population.saturating_sub(1);
        }
    }
}

fn soldier_aura_heal_system(
    time: Res<Time>,
    city_query: Query<(&Transform, &City)>,
    mut soldier_query: Query<(&Transform, &mut Soldier)>,
) {
    for (soldier_transform, mut soldier) in soldier_query.iter_mut() {
        let mut total_heal: f32 = 0.0;
        for (city_transform, city) in city_query.iter() {
            if city.faction != soldier.faction { continue; }
            let dist = soldier_transform.translation.xy()
                .distance(city_transform.translation.xy());
            let effective_radius = CITY_AURA_BASE_RADIUS + city.visual_radius;
            if dist <= effective_radius {
                total_heal += CITY_AURA_BASE_HEAL + city.level as f32 - 1.0;
            }
            let to_soldier = (soldier_transform.translation.xy() - city_transform.translation.xy()).normalize_or_zero();
            if to_soldier.dot(city.spawn_direction) > 0.5 && dist <= CITY_AURA_SPAWN_DIR_RADIUS {
                total_heal += CITY_AURA_BASE_HEAL + city.level as f32 - 1.0;
            }
        }
        if total_heal > 0.0 {
            soldier.health = (soldier.health + total_heal * time.delta_secs()).min(soldier.max_health);
        }
    }
}

fn slow_debuff_tick_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SlowDebuff)>,
) {
    for (entity, mut slow) in query.iter_mut() {
        slow.timer.tick(time.delta());
        if slow.timer.just_finished() {
            commands.entity(entity).remove::<SlowDebuff>();
        }
    }
}
