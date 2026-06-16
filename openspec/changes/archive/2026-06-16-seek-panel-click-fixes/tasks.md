## 1. 输入框光标即时显示

- [x] 1.1 `seek_panel_input_system` 中，`input_active` 从 false 变为 true 时立即更新显示为"值▌"

## 2. 点击穿透防护

- [x] 2.1 新增 `UiFocusBlocker` 资源（`blocked: bool`），在 `seek_panel_*` 系统中当检测到 UI 交互时设为 true
- [x] 2.2 `selection_click_system` 检查 `UiFocusBlocker.blocked`，为 true 时跳过选择逻辑
- [x] 2.3 每帧开始时重置 `UiFocusBlocker.blocked = false`（`reset_ui_focus_blocker` 系统）

## 3. 验证

- [x] 3.1 `cargo build` 编译通过
- [x] 3.2 `cargo test -p simulation` 全部通过
