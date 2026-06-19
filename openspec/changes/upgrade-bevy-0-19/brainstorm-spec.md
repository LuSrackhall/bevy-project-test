# Bevy 0.18 → 0.19 迁移方案

## Context

项目 `city-conquest` 是一款工业级 RTS 游戏，采用分层架构：`simulation`（纯仿真）→ `bevy_adapter`（适配）→ `presentation`（插值）→ `render_view`（视觉/UI）。

当前状态：
- Bevy 版本：0.18（锁定 0.18.1）
- `bevy_ecs` 版本：0.18（simulation 层单独依赖）
- `bevy_prototype_lyon`：0.16（第三方 2D 图形库）
- 启用的实验性 feature：`experimental_bevy_ui_widgets`、`bevy_input_focus`

Bevy 0.19 于 2026-06-17 发布，引入多项破坏性变更。本次迁移目标是将整个项目一次性升级到 Bevy 0.19.0。

## Goals / Non-Goals

**Goals:**
- 将所有 Bevy 依赖升级到 0.19.0
- 移除 `bevy_prototype_lyon`，改用 Bevy 原生 Sprite + Gizmos
- 修复所有已知破坏性 API 变更
- 确保项目可编译、可运行

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
| 选中单位圆环指示器 (`Circle + Stroke`) | `render_view/src/selection.rs:266-269` | `Gizmos::circle_2d` |
| 框选拖拽矩形 (`Rectangle + Stroke`) | `render_view/src/selection.rs:297-314` | `Gizmos::rect_2d` / `Gizmos::circle_2d` |
| 血条/经验条/护盾条矩形背景 (`Rectangle + fill`) | `render_view/src/unit_info_bar.rs:311-376` | `Sprite { color, custom_size }` |

**注意：** `selection_visual_system` 和 `drag_visual_system` 当前每帧 despawn + respawn 实体。改用 `Gizmos` 后可直接在系统中绘制，无需实体生命周期管理。但 `Gizmos` 只在当前帧有效，不影响下一帧，天然适配每帧重绘的模式。

**涉及文件：**
- `Cargo.toml`（根）：移除 `bevy_prototype_lyon` 依赖
- `crates/render_view/Cargo.toml`：移除 `bevy_prototype_lyon` 依赖
- `src/main.rs`：移除 `ShapePlugin` 的 `add_plugins`
- `render_view/src/selection.rs`：重写选中指示器和拖拽视觉
- `render_view/src/unit_info_bar.rs`：矩形背景改用 Sprite

### D2: Feature flag 重命名

**决定：** 更新 feature flags，移除手动插件注册。

| 变更 | 操作 |
|------|------|
| `experimental_bevy_ui_widgets` → `bevy_ui_widgets` | `render_view/Cargo.toml` feature 改名 |
| `ButtonPlugin`, `MenuPlugin`, `PopoverPlugin` | 从 `render_view/src/lib.rs` 移除手动 add_plugins（已含于 DefaultPlugins）|
| `InputDispatchPlugin`, `TabNavigationPlugin` | 同上 |

**注意：** 需要验证 Bevy 0.19 的 `DefaultPlugins` 是否确实自动包含这些插件。如果项目未使用 `DefaultPlugins` 而是自选插件，需保留手动注册。

### D3: TextFont API 变更

**决定：** 更新所有 `TextFont` 使用以匹配新 API。

```rust
// 0.18
TextFont { font: asset_server.load("xxx.ttf"), font_size: 10.0, ..default() }

// 0.19
TextFont { font: asset_server.load("xxx.ttf").into(), font_size: FontSize::Px(10.0), ..default() }
```

**涉及文件：**
- `render_view/src/unit_info_bar.rs`：多处 TextFont 构造（Lv 文字、血条数值等）
- `render_view/src/ui/hud.rs`：HUD 各面板文字
- `render_view/src/ui/menu.rs`：菜单文字
- `render_view/src/ui/pause.rs`：暂停菜单文字
- `render_view/src/ui/gameover.rs`：游戏结束文字

### D4: NonSend API 重命名

**决定：** 更新 `insert_non_send_resource` → `insert_non_send`（兼容旧名仍可用但已 deprecated）。

**涉及文件：**
- `src/main.rs`：`insert_non_send_resource` 调用

### D5: Observer 生命周期事件重命名

**决定：** 检查项目中是否使用了 `on_replace` hook，如有则改为 `on_discard`。同时检查 `EntityComponentsTrigger` 的解构模式是否需要添加 `..`。

**涉及文件：**
- `bevy_adapter/src/lifecycle.rs`

### D6: Command trait 添加 `type Out`

**决定：** 如果项目中自定义了 `Command` 实现，添加 `type Out = ();`。

**涉及文件：** 检查所有 crate 中的 `impl Command for`。

### D7: 版本号更新

**Cargo.toml 变更清单：**

| 文件 | 变更 |
|------|------|
| `Cargo.toml`（根） | `bevy = "0.18"` → `"0.19"`, 移除 `bevy_prototype_lyon` |
| `crates/simulation/Cargo.toml` | `bevy_ecs = "0.18"` → `"0.19"` |
| `crates/bevy_adapter/Cargo.toml` | `bevy = "0.18"` → `"0.19"` |
| `crates/presentation/Cargo.toml` | `bevy = "0.18"` → `"0.19"` |
| `crates/render_view/Cargo.toml` | `bevy = "0.18"` → `"0.19"`, feature 改名, 移除 `bevy_prototype_lyon` |

## Risks / Trade-offs

### R1: UI Widget API 细微变更 [高]

**风险：** Bevy 0.19 中 `bevy::ui_widgets` 从实验性变为正式，API 可能有编译器无法捕获的行为变更（如 Observer 响应时序、Menu 事件传播方式）。

**缓解：** 迁移后全面测试 UI 交互：HUD 面板、菜单、暂停界面、游戏结束界面。

### R2: Text 系统重写（Cosmic Text → Parley）[中]

**风险：** TextFont 的 `font` 字段类型从 `Handle<Font>` 变为 `FontSource`。如果编译通过但运行时加载失败，会表现为文字消失。

**缓解：** 迁移后检查所有界面文字是否正常显示。

### R3: Resources as Components 行为变更 [低-中]

**风险：** `World::clear_entities` 在 0.19 中也会清空 Resources。如果仿真层使用了该方法，可能导致意外数据丢失。

**缓解：** 检查 simulation 层中所有 `World` 方法调用，确认无 `clear_entities` 使用。

### R4: `Node` 结构体新增 `direction` 字段 [低]

**风险：** `Node { ... }` 的结构体初始化可能因缺少 `direction` 字段而编译失败。

**缓解：** 编译器会直接报错，修复简单（添加 `..default()` 或显式设置）。

## 实施顺序

1. **D1** — 移除 `bevy_prototype_lyon`，用原生 API 替代
2. **D7** — 更新所有 Cargo.toml 版本号和 feature flags
3. **D3** — 修复 TextFont API 变更
4. **D4** — 修复 NonSend API
5. **D2** — 修复 UI Widget 插件注册
6. **D5** — 检查 Observer 生命周期
7. **D6** — 检查 Command trait
8. **编译修复** — `cargo build`，修复剩余编译错误
9. **运行验证** — 启动程序，验证 UI、选择框、血条、菜单等功能
