## Context

将 city-conquest 项目从 Bevy 0.18.1 迁移到 Bevy 0.19.0。项目采用四层架构（simulation → bevy_adapter → presentation → render_view），本次迁移涉及全部五个 Cargo.toml 和约 25 个 Rust 源文件。同时移除第三方依赖 `bevy_prototype_lyon`，用原生 API 替代。

## Goals / Non-Goals

**Goals:**
- 所有 crate 编译通过，程序可正常运行
- UI 系统（HUD、菜单、暂停、结算）功能正常
- 选择系统（点选、框选、指示器）功能正常
- 血条/经验条/护盾条显示正常
- Scope 下拉菜单功能正常（弹出、选择、关闭）

**Non-Goals:**
- 不利用 0.19 新特性重构代码
- 不优化仿真逻辑或渲染性能
- 不处理 `bevy_prototype_lyon` 的未来兼容版本

## Decisions

### 移除 bevy_prototype_lyon 策略

`bevy_prototype_lyon` 在项目中仅用于绘制基础矩形和圆，完全可用原生 API 替代：

- **矩形背景**（unit_info_bar.rs 中的血条背景）→ `Sprite { color, custom_size }`：与现有血条填充部分实现方式一致
- **选中圆环**（selection.rs）→ `Gizmos::circle_2d`：天然适配每帧重绘模式
- **拖拽选择框**（selection.rs）→ `Gizmos::rect_2d` / `Gizmos::circle_2d`：同上

移除了 `SelectionIndicator` 组件和相关实体管理逻辑，简化了 `selection_visual_system` 和 `drag_visual_system`。

### 版本升级策略

采用一次性全部升级 + 编译驱动修复的方式。理由：Bevy 0.19 的破坏性变更主要集中在 render_view 层，分层迁移无法独立验证该层。

### Feature flag 迁移

经验证，`ButtonPlugin`、`MenuPlugin`、`PopoverPlugin`、`InputDispatchPlugin`、`TabNavigationPlugin` 均已含于 `DefaultPlugins`。尝试手动注册会导致 panic（"plugin was already added"）。因此移除所有手动插件注册。

### MenuButton 替代方案

`MenuButton` 在 Bevy 0.19 中存在 picking 问题：尽管 `Button` 组件提供了 `FocusPolicy::Block` 和 `Interaction`，`MenuButton` 实体仍无法接收 `Pointer<Press>` 事件。根因未完全确定，可能与 `ActivateOnPress` 的事件处理链路有关。

采用 `WidgetButton` + `Activate` 替代，这是项目中已验证的按钮交互模式。代价是失去了 `MenuPopup` 的内置焦点关闭机制，需要自行实现 `scope_popup_close_system`。

### Pickable::IGNORE 调整

Bevy 0.19 中 `Pickable::IGNORE` 的行为变更导致父节点阻断子节点事件。从包含交互元素的容器上移除 `Pickable::IGNORE`，保留仅在不需要交互穿透的节点上（HudRoot、spacer、纯文字面板）。

### ICU4X 日志处理

`icu_segmenter` 的 CJK 分段数据缺失导致 `eprintln!` 刷屏。根因是 `icu_provider` 的 `logging` feature 未启用，错误通过 fallback `eprintln!` 模块输出，绕过了 LogPlugin。启用 `logging` feature 后，错误通过 `log` 通道输出，可被 LogPlugin 的 filter 过滤。

### HUD 时间显示修复

`update_top_bar` 使用 `Time::elapsed()` 是原有 bug，非迁移引入。改为 `TickClock.current_tick * tick_duration`，与仿真 tick 同步，修复暂停后时间跳跃和重启不归零。

## Risks / Trade-offs

- **MenuButton picking 问题** [已发生] → 用 WidgetButton 替代，未来 Bevy 版本修复后可切回
- **ICU4X CJK 数据缺失** [已缓解] → 通过 LogPlugin filter 过滤，根本修复需等上游更新
- **Text 系统重写** [已验证] → TextFont API 全部适配，文字正常显示
- **Pickable::IGNORE 行为变更** [已修复] → 从交互容器上移除
- **Resources as Components** [已检查] → simulation 层未使用 `World::clear_entities`
