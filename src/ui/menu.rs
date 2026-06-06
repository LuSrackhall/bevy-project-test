use bevy::prelude::*;

use crate::core::GameState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
           .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
           .add_systems(Update, menu_button_system.run_if(in_state(GameState::MainMenu)));
    }
}

#[derive(Component)]
struct MainMenuUI;

#[derive(Component)]
enum MenuButton {
    SinglePlayer,
    MultiPlayer,
    Settings,
    Help,
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        MainMenuUI,
    ))
    .with_children(|parent| {
        parent.spawn(Text::new("城池争霸"));
        parent.spawn((Button, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), ..default() }, MenuButton::SinglePlayer)).with_child(Text::new("单人模式"));
        parent.spawn((Button, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), ..default() }, MenuButton::MultiPlayer)).with_child(Text::new("多人模式 (开发中)"));
        parent.spawn((Button, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), ..default() }, MenuButton::Settings)).with_child(Text::new("设置"));
        parent.spawn((Button, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), ..default() }, MenuButton::Help)).with_child(Text::new("帮助"));
    });
}

fn menu_button_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&MenuButton, &Interaction), Changed<Interaction>>,
) {
    for (button, interaction) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match button {
                MenuButton::SinglePlayer => { next_state.set(GameState::Playing); }
                _ => {}
            }
        }
    }
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for entity in query.iter() { commands.entity(entity).despawn(); }
}
