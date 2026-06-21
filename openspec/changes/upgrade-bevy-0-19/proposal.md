## Why

Bevy 0.19 于 2026-06-18 发布，包含多项破坏性 API 变更。当前项目锁定在 Bevy 0.18.1，需要升级以获取最新修复和特性，并保持与 Bevy 生态的兼容性。同时移除不必要的第三方依赖 `bevy_prototype_lyon`，用原生 API 替代，减少外部依赖追赶负担。

## What Changes

- **BREAKING** Bevy 版本从 0.18 升级到 0.19，影响所有 crate
- **BREAKING** `bevy_ecs` 版本从 0.18 升级到 0.19（simulation 层）
- **BREAKING** 移除 `bevy_prototype_lyon` 第三方依赖，改用 Bevy 原生 `Sprite` + `Gizmos`
- **BREAKING** Feature flag 重命名：`experimental_bevy_ui_widgets` → `bevy_ui_widgets`
- **BREAKING** `TextFont` API 变更：`font` 字段类型改为 `FontSource`（`.into()`），`font_size` 改为 `FontSize::Px()`
- **BREAKING** NonSend API 重命名：`insert_non_send_resource` → `insert_non_send`
- **BREAKING** `MenuAction::Close` 已移除，改为 `MenuAction::CloseAll`
- **BREAKING** `Pickable::IGNORE` 行为变更：父节点的 `Pickable::IGNORE` 阻断子节点点击事件，需从交互容器上移除
- **BREAKING** `MenuButton` 在 0.19 中无法接收点击事件，scope 下拉菜单改用 `WidgetButton` + `Activate` 实现
- **FIX** HUD 时间显示改用 `TickClock` 替代 `Time::elapsed()`，修复暂停后时间跳跃和重启不归零
- **FIX** 启用 `icu_provider` logging feature，解决 ICU4X CJK 分段日志刷屏
- **NEW** 新增 `scope_popup_close_system` 实现 scope 菜单点击外部关闭

## Capabilities

### New Capabilities

（无新增 capability）

### Modified Capabilities

- `simulation-crate`: bevy_ecs 依赖版本从 0.18 升级到 0.19
- `bevy-adapter-crate`: bevy 依赖版本升级，NonSend API 适配
- `presentation-crate`: bevy 依赖版本升级
- `render-view-crate`: bevy 依赖版本升级，UI Widget feature flag 改名，TextFont API 适配，移除 bevy_prototype_lyon，MenuButton 替代方案，Pickable 修复，ICU4X 过滤，HUD 时间修复

## Impact

- **依赖变更**：所有 5 个 Cargo.toml 版本号更新，移除 `bevy_prototype_lyon`，添加 `icu_provider`
- **API 变更**：TextFont 构造方式全面改变（约 40 处），MenuAction 枚举变更，NonSend API 重命名
- **图形替代**：selection.rs 和 unit_info_bar.rs 中的矢量图形改用 Sprite/Gizmos，移除 SelectionIndicator 组件
- **插件注册**：确认所有 widget 插件已含于 DefaultPlugins，移除手动注册
- **行为变更修复**：Pickable::IGNORE 移除、MenuButton 替代、scope 菜单关闭机制
- **原有 bug 修复**：HUD 时间显示改为基于 TickClock
