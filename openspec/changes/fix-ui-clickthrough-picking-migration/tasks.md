## 1. Phase 1b: 穿透修复（HoverMap 替代 UiFocusBlocker）

- [x] 1.1 在 `selection_click_system` 中添加 `Res<HoverMap>` 参数，临时添加日志验证 HoverMap 行为
- [x] 1.2 将 `selection_click_system` 中的 `ui_blocker.blocked` 检查替换为 `HoverMap` 非空检查
- [x] 1.3 在 `command_issue_system` 中添加同样的 `HoverMap` 非空检查
- [x] 1.4 删除 `UiFocusBlocker` 资源定义和 `reset_ui_focus_blocker` 系统（hud.rs）
- [x] 1.5 删除 `UiFocusBlocker` 资源注册和 reset 系统注册（mod.rs）
- [x] 1.6 删除 `seek_panel_dropdown_system`、`seek_panel_input_system`、`seek_panel_issue_system` 中的 `blocker` 参数
- [x] 1.7 透明容器（根节点、spacer、底部区域、左右面板容器）添加 `Pickable::IGNORE`
- [ ] 1.8 验证：选中单位后点击 UI 按钮，选中状态保持
- [ ] 1.9 验证：右键点击 UI 按钮，不触发游戏命令
- [ ] 1.10 验证：点击游戏世界空白处，选中状态正常清除
- [ ] 1.11 验证：左键可正常选中游戏单位

## 2. Phase 1a: Observer 机制验证

- [ ] 2.1 创建 `crates/render_view/src/ui/observer.rs` 模块
- [ ] 2.2 实现 `menu_click_observer` 函数，监听 `Pointer<Click>`，查询 `MenuButton` 组件
- [ ] 2.3 在 `UiPlugin::build` 中注册全局 Observer
- [ ] 2.4 在 Observer 中添加 `info!` 日志，验证点击时触发
- [ ] 2.5 临时禁用 `menu_button_system`，验证 Observer 独立完成状态切换
- [ ] 2.6 恢复 `menu_button_system`，验证两者并行运行无冲突

## 3. 清理与文档

- [x] 3.1 编译验证：`cargo build` 无错误
- [ ] 3.2 运行验证：`cargo run` 游戏正常运行
- [ ] 3.3 更新 `ui/CLAUDE.md` 中的架构准则（如需要）
