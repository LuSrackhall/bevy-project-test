use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;

use crate::core::*;

pub struct SoldierPlugin;

impl Plugin for SoldierPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(|trigger: On<SoldierDiedEvent>, mut soldier_query: Query<&mut Soldier>| {
            let ev = &trigger;
            if let Some(killer) = ev.killer {
                if let Ok(mut killer_soldier) = soldier_query.get_mut(killer) {
                    killer_soldier.exp += EXP_PER_KILL;
                    while killer_soldier.exp >= EXP_TO_LEVEL {
                        killer_soldier.exp -= EXP_TO_LEVEL;
                        killer_soldier.level += 1;
                        killer_soldier.max_health += LEVEL_HP_GAIN;
                        killer_soldier.health = killer_soldier.max_health;
                        killer_soldier.attack += LEVEL_ATTACK_GAIN;
                    }
                }
            }
        });
        app.add_systems(Update, soldier_movement_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, soldier_city_interaction_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, soldier_aura_heal_system.run_if(in_state(GameState::Playing)));
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

pub struct SoldierBundle {
    pub soldier: Soldier,
    pub transform: Transform,
    pub shape: Shape,
}

impl SoldierBundle {
    pub fn new(soldier_type: SoldierType, faction: Faction, position: Vec2) -> Self {
        let color = match faction {
            Faction::Player => Color::srgb(0.3, 0.5, 0.9),
            Faction::Enemy => Color::srgb(0.9, 0.3, 0.3),
            Faction::Neutral => Color::srgb(0.5, 0.5, 0.5),
        };
        Self {
            soldier: Soldier {
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
                city_origin: None,
                is_exiled: false,
            },
            transform: Transform::from_xyz(position.x, position.y, 3.0),
            shape: ShapeBuilder::with(&shapes::RegularPolygon { sides: 3, center: Vec2::ZERO, feature: shapes::RegularPolygonFeature::Radius(6.0) })
                .fill(Fill::color(color))
                .build(),
        }
    }
}

fn soldier_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Soldier, Option<&SlowDebuff>, Option<&InfantryShield>)>,
    target_query: Query<&Transform, (Without<Soldier>, With<crate::city::CityPosition>)>,
) {
    for (mut transform, soldier, slow, shield) in query.iter_mut() {
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
                let dir = (target_transform.translation.truncate() - transform.translation.xy()).normalize_or_zero();
                transform.translation.x += dir.x * speed * time.delta_secs();
                transform.translation.y += dir.y * speed * time.delta_secs();
            }
        }
    }
}

use crate::city::{City, CityCapturedEvent};

fn soldier_city_interaction_system(
    mut commands: Commands,
    soldier_query: Query<(Entity, &Transform, &Soldier)>,
    mut city_query: Query<(Entity, &mut City, &Transform)>,
) {
    for (soldier_entity, soldier_transform, soldier) in soldier_query.iter() {
        for (city_entity, mut city, city_transform) in city_query.iter_mut() {
            let dist = soldier_transform.translation.xy()
                .distance(city_transform.translation.xy());
            let threshold = city.visual_radius;
            if dist > threshold { continue; }

            if soldier.faction != city.faction {
                let damage = city_damage_per_soldier(soldier.attack);
                city.health -= damage;
            } else {
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
            }

            if city.health <= 0.0 && city.faction != Faction::Neutral {
                let old_faction = city.faction;
                let new_faction = match old_faction {
                    Faction::Player => Faction::Enemy,
                    Faction::Enemy => Faction::Player,
                    _ => old_faction,
                };
                city.faction = new_faction;
                city.level = city.level.saturating_sub(1).max(1);
                city.max_health = city_max_health(city.level);
                city.health = city.max_health * 0.2;
                city.max_population = city_max_population(city.level, &mut rand::thread_rng());
                city.population = 0;
                city.level_exp = 0.0;
                city.visual_radius = 20.0 + city.level as f32 * 5.0;
                commands.trigger(CityCapturedEvent { entity: city_entity, old_faction, new_faction });
            }

            commands.entity(soldier_entity).despawn();
            break;
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
            let spawn_dir = Vec2::X;
            if to_soldier.dot(spawn_dir) > 0.5 && dist <= CITY_AURA_SPAWN_DIR_RADIUS {
                total_heal += CITY_AURA_BASE_HEAL + city.level as f32 - 1.0;
            }
        }
        if total_heal > 0.0 {
            soldier.health = (soldier.health + total_heal * time.delta_secs()).min(soldier.max_health);
        }
    }
}
