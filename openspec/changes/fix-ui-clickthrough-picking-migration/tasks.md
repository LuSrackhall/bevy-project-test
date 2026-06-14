## 1. Phase 1b: 穿透修复（Interaction::Pressed 替代 UiFocusBlocker）

- [x] 1.1 在 `selection_click_system` 中添加 `Res<HoverMap>` 参数（初版，后改为 Interaction）
- [x] 1.2 将 `selection_click_system` 中的 `ui_blocker.blocked` 检查替换为 `Interaction::Pressed` 检查
- [x] 1.3 在 `command_issue_system` 中添加同样的 `Interaction::Pressed` 检查
- [x] 1.4 删除 `UiFocusBlocker` 资源定义和 `reset_ui_focus_blocker` 系统（hud.rs）
- [x] 1.5 删除 `UiFocusBlocker` 资源注册和 reset 系统注册（mod.rs）
- [x] 1.6 删除 `seek_panel_dropdown_system`、`seek_panel_input_system`、`seek_panel_issue_system` 中的 `blocker` 参数
- [x] 1.7 透明容器添加 `Pickable::IGNORE`（预防性措施）
- [x] 1.8 验证：左键可正常选中游戏单位
- [x] 1.9 验证：选中单位后点击 UI 按钮，选中状态保持
- [x] 1.10 验证：右键点击 UI 按钮，不触发游戏命令
- [x] 1.11 验证：点击游戏世界空白处，选中状态正常清除

## 2. Phase 1a: Observer 机制验证

- [x] 2.1 创建 `crates/render_view/src/ui/observer.rs` 模块
- [x] 2.2 实现 `menu_press_observer` 函数，监听 `Pointer<Press>`，查询 `MenuButton` 组件
- [x] 2.3 在 `UiPlugin::build` 中注册全局 Observer
- [x] 2.4 验证 Observer 触发（用户确认控制台出现日志）
- [x] 2.5 清理诊断代码，保留最终 Observer
- [ ] 2.6 更新提案，记录 Phase 1a 实证结论

### Phase 1a 实证结论

- ✅ Observer 机制可行（可监听 Pointer 事件）
- ✅ Observer 可访问 ECS 资源（`ResMut`、`Query`）
- ✅ 事件冒泡正常工作（Press 事件在 4 个实体上触发）
- ⚠️ `Pointer<Click>` 在 UI 按钮上不可靠（Press 和 Release 之间有微小移动时 Click 不生成）
- ✅ `Pointer<Press>` 可靠替代方案
- 📌 Phase 2 应使用 `Pointer<Press>` 而非 `Pointer<Click>` 作为按钮交互基础

## 3. 清理与文档

- [x] 3.1 编译验证：`cargo build` 无错误
- [x] 3.2 运行验证：`cargo run` 游戏正常运行（用户已验证通过）
- [x] 3.3 更新 `ui/CLAUDE.md` 中的架构准则（已完成）
