## 1. 移除 bevy_prototype_lyon

- [x] 1.1 从 `Cargo.toml`（根）和 `crates/render_view/Cargo.toml` 中移除 `bevy_prototype_lyon` 依赖
- [x] 1.2 重写 `render_view/src/selection.rs` 中的 `selection_visual_system`：选中圆环指示器改用 `Gizmos::circle_2d`，移除 `SelectionIndicator` 组件的实体管理逻辑
- [x] 1.3 重写 `render_view/src/selection.rs` 中的 `drag_visual_system`：拖拽框改用 `Gizmos::rect_2d` / `Gizmos::circle_2d`，移除实体管理逻辑
- [x] 1.4 重写 `render_view/src/unit_info_bar.rs` 中的 `create_bar`：血条/经验条/护盾条背景矩形改用 `Sprite { color, custom_size }`（与现有填充部分实现方式一致）
- [x] 1.5 从 `src/main.rs` 中移除 `ShapePlugin` 的 `add_plugins` 注册
- [x] 1.6 从 `render_view/src/selection.rs` 和 `render_view/src/unit_info_bar.rs` 中移除 `bevy_prototype_lyon` 相关 import

## 2. 版本号与 Feature Flag 更新

- [x] 2.1 更新 `Cargo.toml`（根）：`bevy = "0.19"`，添加 `icu_provider = { version = "2", features = ["logging"] }`
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

- [x] 5.1 确认所有 widget 插件均含于 `DefaultPlugins`，无需手动注册
- [x] 5.2 发现 `MenuButton` 在 Bevy 0.19 中存在 picking 问题，scope 下拉菜单改用 `WidgetButton` + `Activate` 实现

## 6. 编译修复

- [x] 6.1 修复 `MenuAction::Close` 已移除，改用 `MenuAction::CloseAll`
- [x] 6.2 检查 `simulation` 层未使用 `World::clear_entities`，无需修改

## 7. 额外 Bug 修复

- [x] 7.1 HUD 时间显示改用 `TickClock` 替代 `Time::elapsed()`，修复暂停后时间跳跃和重启不归零（原有 bug）
- [x] 7.2 移除 `BottomZone`、左侧面板、`SeekPanelRoot` 上的 `Pickable::IGNORE`，修复 Bevy 0.19 中事件传播阻断（迁移行为变更）
- [x] 7.3 启用 `icu_provider` 的 `logging` feature，让 ICU4X CJK 分段警告通过 log 通道输出并被 `LogPlugin` 过滤
- [x] 7.4 实现 scope 下拉菜单点击外部关闭功能（`scope_popup_close_system`）

## 8. 功能验证（用户手动测试）

- [x] 8.1 主菜单正常显示和交互
- [x] 8.2 HUD（顶部栏、底部面板、工具栏）正常显示
- [x] 8.3 选择系统：点选、框选、选中指示器、拖拽框视觉
- [x] 8.4 血条/经验条/护盾条正常显示和更新
- [x] 8.5 暂停菜单和结算画面
- [x] 8.6 文字显示正常（无空白、字体大小正确）
- [x] 8.7 Scope 下拉菜单：弹出、选择、关闭、点击外部关闭

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
