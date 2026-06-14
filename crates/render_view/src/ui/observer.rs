use bevy::prelude::*;
use bevy::picking::events::{Pointer, Click, Press, Release};
use super::menu::MenuButton;

/// 诊断: 监听所有 Pointer<Click> 事件
pub fn debug_pointer_click_observer(ev: On<Pointer<Click>>) {
    info!("[DEBUG] Pointer<Click> on entity {:?}", ev.entity);
}

/// 诊断: 监听所有 Pointer<Press> 事件
pub fn debug_pointer_press_observer(ev: On<Pointer<Press>>) {
    info!("[DEBUG] Pointer<Press> on entity {:?}", ev.entity);
}

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
