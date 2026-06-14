use bevy::prelude::*;
use bevy::picking::events::{Pointer, Press};
use super::menu::MenuButton;

/// Phase 1a: Observer 机制验证
/// 全局监听 Pointer<Press>，检查是否命中菜单按钮。
/// 使用 Press 而非 Click，因为 Pointer<Click> 在 UI 按钮上不可靠
/// （Press 和 Release 之间有微小移动时 Click 不会生成）。
pub fn menu_press_observer(
    ev: On<Pointer<Press>>,
    btn_query: Query<&MenuButton>,
    mut next: ResMut<NextState<crate::GameState>>,
) {
    let Ok(_btn) = btn_query.get(ev.entity) else { return };
    info!("[Observer] Menu button pressed, switching to Playing state");
    next.set(crate::GameState::Playing);
}
