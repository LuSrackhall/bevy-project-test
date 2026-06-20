## 1. 移除 bevy_prototype_lyon

- [x] 1.1 从 `Cargo.toml`（根）和 `crates/render_view/Cargo.toml` 中移除 `bevy_prototype_lyon` 依赖
- [x] 1.2 重写 `render_view/src/selection.rs` 中的 `selection_visual_system`：选中圆环指示器改用 `Gizmos::circle_2d`，移除 `SelectionIndicator` 组件的实体管理逻辑
- [x] 1.3 重写 `render_view/src/selection.rs` 中的 `drag_visual_system`：拖拽框改用 `Gizmos::rect_2d` / `Gizmos::circle_2d`，移除实体管理逻辑
- [x] 1.4 重写 `render_view/src/unit_info_bar.rs` 中的 `create_bar`：血条/经验条/护盾条背景矩形改用 `Sprite { color, custom_size }`（与现有填充部分实现方式一致）
- [x] 1.5 从 `src/main.rs` 中移除 `ShapePlugin` 的 `add_plugins` 注册
- [x] 1.6 从 `render_view/src/selection.rs` 和 `render_view/src/unit_info_bar.rs` 中移除 `bevy_prototype_lyon` 相关 import

## 2. 版本号与 Feature Flag 更新

- [x] 2.1 更新 `Cargo.toml`（根）：`bevy = "0.19"`
- [x] 2.2 更新 `crates/simulation/Cargo.toml`：`bevy_ecs = "0.19"`
- [x] 2.3 更新 `crates/bevy_adapter/Cargo.toml`：`bevy = "0.19"`
- [x] 2.4 更新 `crates/presentation/Cargo.toml`：`bevy = "0.19"`
- [x] 2.5 更新 `crates/render_view/Cargo.toml`：`bevy = "0.19"`，feature `experimental_bevy_ui_widgets` 改为 `bevy_ui_widgets`

## 3. TextFont API 适配

- [x] 3.1 更新 `render_view/src/unit_info_bar.rs` 中所有 `TextFont` 构造：`font` 字段改为 `.into()`，`font_size` 改为 `FontSize::Px(...)`
- [x] 3.2 更新 `render_view/src/ui/hud.rs` 中所有 `TextFont` 构造
- [x] 3.3 更新 `render_view/src/ui/menu.rs` 中所有 `TextFont` 构造
- [x] 3.4 更新 `render_view/src/ui/pause.rs` 中所有 `TextFont` 构造
- [x] 3.5 更新 `render_view/src/ui/gameover.rs` 中所有 `TextFont` 构造

## 4. NonSend API 适配

- [x] 4.1 更新 `src/main.rs` 中 `insert_non_send_resource` 为 `insert_non_send`

## 5. UI Widget 插件注册调整

- [x] 5.1 从 `render_view/src/lib.rs` 中移除手动注册的 `ButtonPlugin`、`MenuPlugin`、`PopoverPlugin`、`InputDispatchPlugin`、`TabNavigationPlugin`（确认已含于 `DefaultPlugins`）
- [x] 5.2 如 `DefaultPlugins` 未自动包含上述插件，则保留手动注册

## 6. 编译修复

- [x] 6.1 执行 `cargo build`，修复所有编译错误（Observer 生命周期、Command trait、Node 新字段等）
- [x] 6.2 检查 `simulation` 层是否使用了 `World::clear_entities`，如有则改为 `World::clear_all` 或添加显式资源保留逻辑

## 7. 功能验证

- [ ] 7.1 运行程序，验证主菜单正常显示和交互
- [ ] 7.2 进入游戏，验证 HUD（顶部栏、底部面板、工具栏）正常显示
- [ ] 7.3 验证选择系统：点选、框选、选中指示器、拖拽框视觉
- [ ] 7.4 验证血条/经验条/护盾条正常显示和更新
- [ ] 7.5 验证暂停菜单和结算画面
- [ ] 7.6 验证文字显示正常（无空白、字体大小正确）

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/upgrade-bevy-0-19`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.
