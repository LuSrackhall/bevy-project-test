## 1. 弹出层可见性改造

- [x] 1.1 修改 `setup_hud` 中弹出层初始样式：移除 `display: Display::None`，改为 `left: Val::Px(-9999.0)`，保持 `Display::Flex`
- [x] 1.2 修改 `seek_panel_dropdown_system` 中弹出层可见性控制：从 `node.display` 切换改为 `node.left` 偏移切换

## 2. 移除 Changed<Interaction> 依赖

- [x] 2.1 修改 `seek_panel_dropdown_system`：`dropdown_btn` 和 `option_btns` 查询移除 `Changed<Interaction>` 过滤器
- [x] 2.2 修改 `seek_panel_input_system`：`input_btn` 查询移除 `Changed<Interaction>` 过滤器
- [x] 2.3 修改 `seek_panel_issue_system`：`issue_btn` 查询移除 `Changed<Interaction>` 过滤器

## 3. 防抖与防重入

- [x] 3.1 所有交互系统中按钮点击检测配合 `mouse.just_pressed(MouseButton::Left)` 做防抖
- [x] 3.2 `seek_panel_input_system` 中输入框进入编辑态使用 `mouse.just_pressed` + `!state.editing` 双重防护

## 4. 验证

- [x] 4.1 `cargo build` 编译通过
- [x] 4.2 `cargo test -p simulation` 全部通过
