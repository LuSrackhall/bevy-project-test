use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;

use crate::core::*;
use crate::camera::MainCamera;
use crate::soldier::SoldierBundle;
use crate::map::MapGenerated;

pub struct CityPlugin;

impl Plugin for CityPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(|trigger: On<MapGenerated>, mut commands: Commands| {
            for data in &trigger.cities {
                let max_health = city_max_health(data.level);
                let max_pop = city_max_population(data.level, &mut rand::thread_rng());
                let spawn_interval = 3.0 / soldier_spawn_speed_multiplier(SoldierType::Militia);  // faster for testing
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
                        spawn_direction: Vec2::X,
                        last_attacker_faction: None,
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
           .add_systems(Update, city_capture_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, city_click_system.run_if(in_state(GameState::Playing)))
           .add_systems(Update, city_visual_update_system.run_if(in_state(GameState::Playing)));
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
    pub spawn_direction: Vec2,
    pub last_attacker_faction: Option<Faction>,
}

#[derive(Component)]
pub struct CityPosition(pub Vec2);

#[derive(Event)]
pub struct CitySelectedEvent {
    pub entity: Entity,
    pub faction: Faction,
}

/// Emitted when the player clicks empty space, used by UI to hide the bottom panel
#[derive(Event)]
pub struct CityDeselectedEvent;

#[derive(Event)]
pub struct CityCapturedEvent {
    pub entity: Entity,
    pub old_faction: Faction,
    pub new_faction: Faction,
}

fn city_spawn_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut City, &CityPosition)>,
    mut commands: Commands,
    mut first_run: Local<u32>,
) {
    *first_run += 1;
    if *first_run == 1 {
        let city_count = query.iter().count();
        warn!("city_spawn_system running! {} cities, delta={:.3}s", city_count, time.delta_secs());
        for (e, city, _) in query.iter() {
            warn!("  city {:?} faction={:?} pop={}/{} hp={:.0}/{:.0} timer={:.2}s/{:.2}s",
                e, city.faction, city.population, city.max_population,
                city.health, city.max_health,
                city.spawn_timer.elapsed_secs(), city.spawn_timer.duration().as_secs_f32());
        }
    }

    for (entity, mut city, pos) in query.iter_mut() {
        if city.faction == Faction::Neutral { continue; }
        if city.population >= city.max_population { continue; }
        city.spawn_timer.tick(time.delta());
        if city.spawn_timer.just_finished() {
            city.population += 1;
            // Spawn outside city visual_radius so soldier isn't immediately consumed
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            let dir = Vec2::new(angle.cos(), angle.sin());
            let spawn_pos = pos.0 + dir * (city.visual_radius + 20.0);
            warn!("Spawning {:?} at ({:.0}, {:.0})", city.spawn_type, spawn_pos.x, spawn_pos.y);
            SoldierBundle::spawn(&mut commands, city.spawn_type, city.faction, spawn_pos, Some(entity));
        }
    }
}

fn city_capture_system(
    mut query: Query<(Entity, &mut City)>,
    mut commands: Commands,
) {
    for (entity, mut city) in query.iter_mut() {
        if city.health > 0.0 { continue; }

        let old_faction = city.faction;

        let new_faction = match old_faction {
            Faction::Player => Faction::Enemy,
            Faction::Enemy => Faction::Player,
            Faction::Neutral => {
                city.last_attacker_faction.unwrap_or(Faction::Player)
            }
        };

        city.faction = new_faction;
        city.level = city.level.saturating_sub(1).max(1);
        city.max_health = city_max_health(city.level);
        city.health = city.max_health * 0.2;
        city.max_population = city_max_population(city.level, &mut rand::thread_rng());
        city.population = 0;
        city.level_exp = 0.0;
        city.visual_radius = 20.0 + city.level as f32 * 5.0;
        city.last_attacker_faction = None;

        commands.trigger(CityCapturedEvent { entity, old_faction, new_faction });
    }
}

fn city_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    city_query: Query<(Entity, &Transform, &City)>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }

    let Ok(window) = q_windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor) else { return };

    for (entity, transform, city) in city_query.iter() {
        let city_pos = transform.translation.xy();
        if world_pos.distance(city_pos) <= city.visual_radius && city.faction == Faction::Player {
            commands.trigger(CitySelectedEvent { entity, faction: city.faction });
            return;
        }
    }
    commands.trigger(CityDeselectedEvent);
}

fn city_visual_update_system(
    mut commands: Commands,
    query: Query<(Entity, &City), Changed<City>>,
) {
    for (entity, city) in query.iter() {
        let color = match city.faction {
            Faction::Player => Color::srgb(0.2, 0.6, 1.0),
            Faction::Enemy => Color::srgb(1.0, 0.2, 0.2),
            Faction::Neutral => Color::srgb(0.6, 0.6, 0.6),
        };
        let radius = 15.0 + city.level as f32 * 3.0;
        // Rebuild the city circle shape with updated color and radius
        let shape = ShapeBuilder::with(&shapes::Circle { radius, center: Vec2::ZERO })
            .fill(Fill::color(color))
            .build();
        commands.entity(entity).insert(shape);
    }
}
