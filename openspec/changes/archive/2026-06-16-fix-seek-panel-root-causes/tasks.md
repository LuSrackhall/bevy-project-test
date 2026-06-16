## 1. 弹出层修复

- [x] 1.1 `setup_hud` 中弹出层改回 `display: Display::None`，`left: Val::Px(0.0)`
- [x] 1.2 `seek_panel_dropdown_system` 中弹出层可见性从 `node.left` 切换改回 `node.display` 切换

## 2. Text 实体 ID 修复

- [x] 2.1 `setup_hud` 中 `seek_scope_text` 改为存储 Text 子实体 ID（在 `with_children` 闭包内捕获）
- [x] 2.2 `setup_hud` 中 `seek_range_text` 改为存储 Text 子实体 ID
- [x] 2.3 `setup_hud` 中 `toast_text` 验证是否存储正确（已在顶层 spawn，无需修改）

## 3. 选择系统 UI 守卫

- [x] 3.1 `selection_click_system` 增加 `Query<&Interaction>` 参数，检测光标下是否有 UI 元素，有则跳过

## 4. 编辑态保护

- [x] 4.1 `seek_panel_mode_system` 在 `state.editing == true` 时跳过模式切换

## 5. 系统排序

- [x] 5.1 通过 UI 交互守卫（task 3.1）解决，无需额外排序约束

## 6. 验证

- [x] 6.1 `cargo build` 编译通过
- [x] 6.2 `cargo test -p simulation` 全部通过
