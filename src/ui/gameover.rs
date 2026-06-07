use bevy::prelude::*;
use bevy::text::TextFont;

use crate::core::{GameState, Faction};
use crate::game::GameStats;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), setup_game_over)
           .add_systems(OnExit(GameState::GameOver), cleanup_game_over)
           .add_systems(Update, game_over_button_system.run_if(in_state(GameState::GameOver)));
    }
}

#[derive(Component)]
struct GameOverUI;

#[derive(Component)]
enum GameOverButton {
    Restart,
    MainMenu,
}

fn setup_game_over(mut commands: Commands, asset_server: Res<AssetServer>, stats: Res<GameStats>, time: Res<Time>) {
    let font_handle = asset_server.load("fonts/Arial Unicode.ttf");

    let elapsed = time.elapsed().as_secs().saturating_sub(stats.start_time as u64);
    let minutes = elapsed / 60;
    let seconds = elapsed % 60;

    let (result_text, _color) = match stats.winner {
        Some(Faction::Player) => ("胜利!", Color::srgb(0.2, 0.8, 0.2)),
        Some(Faction::Enemy) => ("失败!", Color::srgb(0.9, 0.2, 0.2)),
        _ => ("游戏结束", Color::srgb(0.8, 0.8, 0.8)),
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        GameOverUI,
    ))
    .with_children(|parent| {
        parent.spawn((Text::new(result_text), TextFont { font: font_handle.clone(), font_size: 48.0, ..default() }));
        parent.spawn((Text::new(format!("游戏时间: {}:{:02}", minutes, seconds)), TextFont { font: font_handle.clone(), font_size: 24.0, ..default() }));
        parent.spawn((Text::new(format!("剩余城池: {}", stats.player_cities_remaining)), TextFont { font: font_handle.clone(), font_size: 24.0, ..default() }));
        parent.spawn((Text::new(format!("总击杀: {}", stats.total_kills)), TextFont { font: font_handle.clone(), font_size: 24.0, ..default() }));

        parent.spawn(Node { height: Val::Px(30.0), ..default() });

        parent.spawn((Button, GameOverButton::Restart, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), ..default() }))
            .with_child((Text::new("再来一局"), TextFont { font: font_handle.clone(), font_size: 24.0, ..default() }));

        parent.spawn((Button, GameOverButton::MainMenu, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), ..default() }))
            .with_child((Text::new("返回主菜单"), TextFont { font: font_handle.clone(), font_size: 24.0, ..default() }));
    });
}

fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for entity in query.iter() { commands.entity(entity).despawn(); }
}

fn game_over_button_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&GameOverButton, &Interaction), Changed<Interaction>>,
) {
    for (button, interaction) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match button {
                GameOverButton::Restart => { next_state.set(GameState::Playing); }
                GameOverButton::MainMenu => { next_state.set(GameState::MainMenu); }
            }
        }
    }
}
