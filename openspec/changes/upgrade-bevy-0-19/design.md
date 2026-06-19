## Context

将 city-conquest 项目从 Bevy 0.18.1 迁移到 Bevy 0.19.0。项目采用四层架构（simulation → bevy_adapter → presentation → render_view），本次迁移涉及全部五个 Cargo.toml 和约 25 个 Rust 源文件。同时移除第三方依赖 `bevy_prototype_lyon`，用原生 API 替代。

## Goals / Non-Goals

**Goals:**
- 所有 crate 编译通过，程序可正常运行
- UI 系统（HUD、菜单、暂停、结算）功能正常
- 选择系统（点选、框选、指示器）功能正常
- 血条/经验条/护盾条显示正常

**Non-Goals:**
- 不利用 0.19 新特性重构代码
- 不优化仿真逻辑或渲染性能
- 不处理 `bevy_prototype_lyon` 的未来兼容版本

## Decisions

### 移除 bevy_prototype_lyon 策略

`bevy_prototype_lyon` 在项目中仅用于绘制基础矩形和圆，完全可用原生 API 替代：

- **矩形背景**（unit_info_bar.rs 中的血条背景）→ `Sprite { color, custom_size }`：与现有血条填充部分实现方式一致，无需新增学习成本
- **选中圆环**（selection.rs）→ `Gizmos::circle_2d`：天然适配每帧重绘模式，无需管理实体生命周期
- **拖拽选择框**（selection.rs）→ `Gizmos::rect_2d` / `Gizmos::circle_2d`：同上

替代方案（保留依赖等官方更新）被否决：第三方依赖追赶 Bevy 版本通常需 1-3 个月，且当前用法过于简单，无保留必要。

### 版本升级策略

采用一次性全部升级 + 编译驱动修复的方式，而非分层渐进迁移。理由：Bevy 0.19 的破坏性变更主要集中在 render_view 层，分层迁移无法独立验证该层。

### Feature flag 迁移

Bevy 0.19 将 `experimental_bevy_ui_widgets` 纳入默认 features。`render_view/Cargo.toml` 需将 feature 名从 `experimental_bevy_ui_widgets` 改为 `bevy_ui_widgets`。同时 `ButtonPlugin`、`MenuPlugin`、`PopoverPlugin` 等插件已含于 `DefaultPlugins`，需从手动注册中移除，避免重复注册。

## Risks / Trade-offs

- **UI Widget 行为变更** [高] → 迁移后全面手动测试 HUD、菜单交互
- **Text 系统重写** [中] → 迁移后检查所有界面文字是否正常显示
- **Resources as Components** [低-中] → 检查仿真层是否使用了 `World::clear_entities`（会导致资源被意外清空）
- **Node 结构体新字段** [低] → 编译器直接报错，修复简单
