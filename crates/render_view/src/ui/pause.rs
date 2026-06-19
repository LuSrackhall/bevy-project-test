use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button as WidgetButton};
use bevy::picking::hover::Hovered;
use crate::ui::hud::ButtonTheme;

#[derive(Component)]
pub struct PauseUI;

/// Spawn pause menu UI (initially hidden). Called from setup_hud.
pub fn spawn_pause_menu(commands: &mut Commands, font: &Handle<Font>) {
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center,
        align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), PauseUI,
        Visibility::Hidden,
    ))
    .with_children(|parent| {
        parent.spawn((Text::new("游戏暂停"), TextFont { font: font.clone().into(), font_size: FontSize::Px(36.0), ..default() }));
        for (label, btn) in [("继续", PauseBtn::Resume), ("重新开始", PauseBtn::Restart), ("主菜单", PauseBtn::Menu)] {
            parent.spawn((WidgetButton, Node { margin: UiRect::all(Val::Px(10.0)), padding: UiRect::all(Val::Px(20.0)), border: UiRect::all(Val::Px(2.0)), ..default() }, btn, ButtonTheme::default(), Hovered::default(), BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 1.0))))
                .with_child((Text::new(label), TextFont { font: font.clone().into(), font_size: FontSize::Px(24.0), ..default() }))
                .observe(move |_ev: On<Activate>, q: Query<&PauseBtn>, mut paused: ResMut<bevy_adapter::Paused>, mut needs_reset: ResMut<crate::NeedsGameReset>, mut next: ResMut<NextState<crate::GameState>>| {
                    if let Ok(pause_btn) = q.get(_ev.entity) {
                        match pause_btn {
                            PauseBtn::Resume => { paused.0 = false; }
                            PauseBtn::Restart => {
                                paused.0 = false;
                                *needs_reset = crate::NeedsGameReset::SameSize;
                                next.set(crate::GameState::MainMenu);
                            }
                            PauseBtn::Menu => {
                                paused.0 = false;
                                next.set(crate::GameState::MainMenu);
                            }
                        }
                    }
                });
        }
    });
}

#[derive(Component)]
pub(crate) enum PauseBtn { Resume, Restart, Menu }
