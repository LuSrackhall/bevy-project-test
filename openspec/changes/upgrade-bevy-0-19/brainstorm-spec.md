# Bevy 0.18 → 0.19 迁移方案

## Context

项目 `city-conquest` 是一款工业级 RTS 游戏，采用分层架构：`simulation`（纯仿真）→ `bevy_adapter`（适配）→ `presentation`（插值）→ `render_view`（视觉/UI）。

当前状态：
- Bevy 版本：0.18（锁定 0.18.1）
- `bevy_ecs` 版本：0.18（simulation 层单独依赖）
- `bevy_prototype_lyon`：0.16（第三方 2D 图形库）
- 启用的实验性 feature：`experimental_bevy_ui_widgets`、`bevy_input_focus`

Bevy 0.19 于 2026-06-18 在 GitHub 发布（crates.io 同步发布），引入多项破坏性变更。本次迁移目标是将整个项目一次性升级到 Bevy 0.19.0。

## Goals / Non-Goals

**Goals:**
- 将所有 Bevy 依赖升级到 0.19.0
- 移除 `bevy_prototype_lyon`，改用 Bevy 原生 Sprite + Gizmos
- 修复所有已知破坏性 API 变更
- 修复迁移过程中发现的运行时行为变更
- 修复原有 bug（HUD 时间显示）
- 确保项目可编译、可运行，所有 UI 功能正常

**Non-Goals:**
- 不引入 0.19 新特性（如 BSN 宏、不可变资源等）
- 不重构架构或优化现有逻辑
- 不修改仿真层业务逻辑
- 不处理 `bevy_prototype_lyon` 未来可能的兼容版本

## Decisions

### D1: 移除 `bevy_prototype_lyon`，用原生 API 替代

**决定：** 完全移除该第三方依赖。

**替代方案：**

| 当前用法 | 文件 | 替代方式 |
|---------|------|---------|
| 选中单位圆环指示器 (`Circle + Stroke`) | `render_view/src/selection.rs` | `Gizmos::circle_2d` |
| 框选拖拽矩形 (`Rectangle + Stroke`) | `render_view/src/selection.rs` | `Gizmos::rect_2d` / `Gizmos::circle_2d` |
| 血条/经验条/护盾条矩形背景 (`Rectangle + fill`) | `render_view/src/unit_info_bar.rs` | `Sprite { color, custom_size }` |

改用 `Gizmos` 后，`selection_visual_system` 和 `drag_visual_system` 不再需要实体生命周期管理（每帧自动清除）。移除了 `SelectionIndicator` 组件。

### D2: Feature flag 重命名

**决定：** 更新 feature flags。经验证，所有 widget 插件（`ButtonPlugin`、`MenuPlugin`、`PopoverPlugin`、`InputDispatchPlugin`、`TabNavigationPlugin`）均已含于 `DefaultPlugins`，无需手动注册。

| 变更 | 操作 |
|------|------|
| `experimental_bevy_ui_widgets` → `bevy_ui_widgets` | `render_view/Cargo.toml` feature 改名 |
| 手动插件注册 | 从 `render_view/src/lib.rs` 移除（已含于 DefaultPlugins）|

### D3: TextFont API 变更

**决定：** 更新所有 `TextFont` 使用以匹配新 API。

```rust
// 0.18
TextFont { font: asset_server.load("xxx.ttf"), font_size: 10.0, ..default() }

// 0.19
TextFont { font: asset_server.load("xxx.ttf").into(), font_size: FontSize::Px(10.0), ..default() }
```

涉及 5 个文件共约 40 处 `TextFont` 构造。

### D4: NonSend API 重命名

**决定：** `insert_non_send_resource` → `insert_non_send`。涉及 `src/main.rs`。

### D5: MenuAction::Close 移除

**决定：** Bevy 0.19 移除了 `MenuAction::Close`，只保留 `MenuAction::CloseAll`。将 `hud.rs` 中的 `MenuAction::Close | MenuAction::CloseAll` 改为 `MenuAction::CloseAll`。

### D6: MenuButton 无法接收点击事件（迁移行为变更）

**决定：** `MenuButton`（来自 `bevy::ui_widgets`）在 Bevy 0.19 中存在 picking 问题，无法接收 `Pointer<Press>` 事件，导致 scope 下拉菜单完全无法使用。

**解决方案：** 将 scope 下拉菜单从 `MenuButton` + `MenuEvent` 改为 `WidgetButton` + `Activate`。`WidgetButton` 已在项目中其他按钮（城池兵种切换、暂停菜单等）中验证可用。

**影响范围：** 仅 `render_view/src/ui/hud.rs` 中的 scope 下拉菜单。其他使用 `WidgetButton` 的按钮不受影响。

### D7: Pickable::IGNORE 行为变更

**决定：** Bevy 0.19 中 `Pickable::IGNORE` 的传播行为变更——父节点的 `Pickable::IGNORE` 会阻断子节点的点击事件。

移除了以下容器上的 `Pickable::IGNORE`：
- `BottomZone`（底部区域容器）
- 左侧 30% 面板（城池详情面板的父容器）
- `SeekPanelRoot`（索引面板容器）

保留 `Pickable::IGNORE` 的位置：
- `HudRoot`（根容器，需让点击穿透到游戏世界）
- spacer（纯占位节点）
- 右侧 70% 面板（纯文字内容，无交互元素）

### D8: HUD 时间显示修复（原有 bug）

**决定：** `update_top_bar` 原本使用 `Time::elapsed()`（墙钟时间）显示游戏时间，导致：
- 暂停后恢复时间跳跃（墙钟时间在暂停期间继续累加）
- 重启/重置后时间不归零（`Time` 资源是引擎级的，不随游戏重置）

改为读取 `TickClock.current_tick * tick_duration`，与仿真 tick 同步。

### D9: ICU4X CJK 分段日志刷屏

**决定：** Bevy 0.19 的文本系统（Parley）在处理 CJK 字符时，`icu_segmenter` 会尝试加载日语分段字典数据。数据缺失时通过 `icu_provider` 的 fallback `log` 模块输出 `eprintln!`，绕过 LogPlugin 过滤。

**解决方案：** 在根 `Cargo.toml` 中添加 `icu_provider = { version = "2", features = ["logging"] }`，让错误通过 `log` 通道输出。同时在 `LogPlugin` 中设置 `filter: "warn,icu_provider=error"` 过滤该警告。

### D10: Scope 下拉菜单点击外部关闭

**决定：** 使用 `WidgetButton` 替代 `MenuButton` 后，失去了 `MenuPopup` 的内置焦点关闭机制。新增 `scope_popup_close_system` 系统，通过 `SeekPanelState.open_scope_popup` 追踪弹出状态，检测鼠标左键按下时关闭。增加了 `Hovered` 检测，避免在点击菜单项时被系统抢先关闭。

### D11: 版本号更新

| 文件 | 变更 |
|------|------|
| `Cargo.toml`（根） | `bevy = "0.19"`，添加 `icu_provider` |
| `crates/simulation/Cargo.toml` | `bevy_ecs = "0.19"` |
| `crates/bevy_adapter/Cargo.toml` | `bevy = "0.19"` |
| `crates/presentation/Cargo.toml` | `bevy = "0.19"` |
| `crates/render_view/Cargo.toml` | `bevy = "0.19"`，feature 改名，移除 `bevy_prototype_lyon` |

## Risks / Trade-offs

### R1: MenuButton picking 问题 [已发生]

`MenuButton` 在 Bevy 0.19 中无法接收点击事件。已通过改用 `WidgetButton` 解决。这是 Bevy 0.19 的 bug，未来版本可能修复。届时可考虑切回 `MenuButton` 以获得内置的菜单焦点管理。

### R2: ICU4X CJK 分段数据缺失 [已缓解]

Parley 的 ICU4X 数据不包含完整的日语分段字典。通过启用 `icu_provider` 的 `logging` feature 并设置 LogPlugin filter 过滤。根本修复需等 Parley/ICU4X 更新。

### R3: Text 系统重写（Cosmic Text → Parley）[已验证]

TextFont API 变更已全部适配，所有界面文字正常显示。

### R4: Pickable::IGNORE 行为变更 [已修复]

父节点的 `Pickable::IGNORE` 在 0.19 中阻断子节点事件。已从交互容器上移除。

### R5: Resources as Components [已检查]

simulation 层未使用 `World::clear_entities`，不受影响。
