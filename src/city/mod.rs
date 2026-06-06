use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;

use crate::core::*;
use crate::soldier::SoldierBundle;
use crate::map::MapGenerated;

pub struct CityPlugin;

impl Plugin for CityPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(|trigger: On<MapGenerated>, mut commands: Commands| {
            for data in &trigger.cities {
                let max_health = city_max_health(data.level);
                let max_pop = city_max_population(data.level, &mut rand::thread_rng());
                let spawn_interval = 5.0 / soldier_spawn_speed_multiplier(SoldierType::Militia);
                let color = match data.faction {
                    Faction::Player => Color::srgb(0.2, 0.6, 1.0),
                    Faction::Enemy => Color::srgb(1.0, 0.2, 0.2),
                    Faction::Neutral => Color::srgb(0.6, 0.6, 0.6),
                };
                let radius = 15.0 + data.level as f32 * 3.0;
                let shape = ShapeBuilder::with(&shapes::Circle { radius, center: Vec2::ZERO })
                    .fill(Fill::color(color))
                    .build();
                commands.spawn((
                    City {
                        level: data.level,
                        max_level: data.max_level,
                        health: max_health,
                        max_health,
                        population: 0,
                        max_population: max_pop,
                        faction: data.faction,
                        spawn_type: SoldierType::Militia,
                        spawn_timer: Timer::from_seconds(spawn_interval, TimerMode::Repeating),
                        level_exp: 0.0,
                        visual_radius: 20.0 + data.level as f32 * 5.0,
                    },
                    CityPosition(data.position),
                    Transform::from_xyz(data.position.x, data.position.y, 1.0),
                    shape,
                ));
            }
        });
        app.add_observer(|trigger: On<CityCapturedEvent>, mut soldier_query: Query<&mut crate::soldier::Soldier>| {
            let ev = &trigger;
            for mut soldier in soldier_query.iter_mut() {
                if soldier.faction == ev.old_faction && soldier.city_origin == Some(ev.entity) {
                    soldier.is_exiled = true;
                }
            }
        });
        app.add_systems(Update, city_spawn_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, city_capture_system.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct City {
    pub level: u32,
    pub max_level: u32,
    pub health: f32,
    pub max_health: f32,
    pub population: u32,
    pub max_population: u32,
    pub faction: Faction,
    pub spawn_type: SoldierType,
    pub spawn_timer: Timer,
    pub level_exp: f32,
    pub visual_radius: f32,
}

#[derive(Component)]
pub struct CityPosition(pub Vec2);

#[derive(Event)]
pub struct CitySelectedEvent {
    pub entity: Entity,
    pub faction: Faction,
}

#[derive(Event)]
pub struct CityCapturedEvent {
    pub entity: Entity,
    pub old_faction: Faction,
    pub new_faction: Faction,
}

fn city_spawn_system(
    time: Res<Time>,
    mut query: Query<(&mut City, &CityPosition)>,
    mut commands: Commands,
) {
    for (mut city, pos) in query.iter_mut() {
        if city.faction == Faction::Neutral {
            continue;
        }
        if city.population >= city.max_population {
            continue;
        }
        city.spawn_timer.tick(time.delta());
        if city.spawn_timer.just_finished() {
            city.population += 1;
            let bundle = SoldierBundle::new(
                city.spawn_type,
                city.faction,
                pos.0,
            );
            commands.spawn((bundle.soldier, bundle.transform, bundle.shape));
        }
    }
}

fn city_capture_system(
    mut query: Query<(Entity, &mut City)>,
    mut commands: Commands,
) {
    for (entity, mut city) in query.iter_mut() {
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
            commands.trigger(CityCapturedEvent { entity, old_faction, new_faction });
        }
    }
}
