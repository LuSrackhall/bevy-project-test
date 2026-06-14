use bevy::prelude::*;
use bevy::picking::events::{Pointer, Click};
use super::menu::MenuButton;

/// Phase 1a: 验证 Observer 机制
/// 全局监听 Pointer<Click>，检查是否命中菜单按钮
pub fn menu_click_observer(
    ev: On<Pointer<Click>>,
    btn_query: Query<&MenuButton>,
    mut next: ResMut<NextState<crate::GameState>>,
) {
    let Ok(btn) = btn_query.get(ev.entity) else { return };
    info!("[Observer] Menu button clicked");
    match btn {
        MenuButton::SinglePlayer => {
            next.set(crate::GameState::Playing);
            info!("[Observer] Switching to Playing state");
        }
    }
}
