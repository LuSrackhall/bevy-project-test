use bevy::prelude::*;

use crate::core::GameState;

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Paused), setup_pause_menu)
           .add_systems(OnExit(GameState::Paused), cleanup_pause_menu)
           .add_systems(Update, pause_button_system.run_if(in_state(GameState::Paused)));
    }
}

#[derive(Component)]
struct PauseMenuUI;

#[derive(Component)]
enum PauseButton {
    Resume,
    Settings,
    Restart,
    MainMenu,
}

fn setup_pause_menu(mut commands: Commands) {
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        PauseMenuUI,
    ))
    .with_children(|parent| {
        parent.spawn(Text::new("⏸️ 游戏暂停"));
        for (label, button) in [("▶️ 继续游戏", PauseButton::Resume), ("⚙️ 设置", PauseButton::Settings), ("🔄 重新开始", PauseButton::Restart), ("🚪 返回主菜单", PauseButton::MainMenu)] {
            parent.spawn((Button, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), ..default() }, button)).with_child(Text::new(label));
        }
    });
}

fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuUI>>) {
    for entity in query.iter() { commands.entity(entity).despawn(); }
}

fn pause_button_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&PauseButton, &Interaction), Changed<Interaction>>,
) {
    for (button, interaction) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match button {
                PauseButton::Resume => next_state.set(GameState::Playing),
                PauseButton::Restart => next_state.set(GameState::Playing),
                PauseButton::MainMenu => next_state.set(GameState::MainMenu),
                _ => {}
            }
        }
    }
}
