## Why

Bevy 0.19 于 2026-06-17 发布，包含多项破坏性 API 变更。当前项目锁定在 Bevy 0.18.1，需要升级以获取最新修复和特性，并保持与 Bevy 生态的兼容性。同时移除不必要的第三方依赖 `bevy_prototype_lyon`，用原生 API 替代，减少外部依赖追赶负担。

## What Changes

- **BREAKING** Bevy 版本从 0.18 升级到 0.19，影响所有 crate
- **BREAKING** `bevy_ecs` 版本从 0.18 升级到 0.19（simulation 层）
- **BREAKING** 移除 `bevy_prototype_lyon` 第三方依赖，改用 Bevy 原生 `Sprite` + `Gizmos`
- **BREAKING** Feature flag 重命名：`experimental_bevy_ui_widgets` → `bevy_ui_widgets`
- **BREAKING** `TextFont` API 变更：`font` 字段类型改为 `FontSource`，`font_size` 改为 `FontSize` 枚举
- **BREAKING** NonSend API 重命名：`insert_non_send_resource` → `insert_non_send`
- **BREAKING** UI Widget 插件现包含于 `DefaultPlugins`，移除手动注册

## Capabilities

### New Capabilities

（无新增 capability）

### Modified Capabilities

- `simulation-crate`: bevy_ecs 依赖版本从 0.18 升级到 0.19
- `bevy-adapter-crate`: bevy 依赖版本升级，NonSend API 适配
- `presentation-crate`: bevy 依赖版本升级
- `render-view-crate`: bevy 依赖版本升级，UI Widget feature flag 改名，TextFont API 适配，移除 bevy_prototype_lyon

## Impact

- **依赖变更**：所有 5 个 Cargo.toml 版本号更新，移除 `bevy_prototype_lyon`
- **API 变更**：TextFont 构造方式全面改变，影响 render_view 中所有 UI 文件
- **图形替代**：selection.rs 和 unit_info_bar.rs 中的矢量图形改用 Sprite/Gizmos
- **插件注册**：src/main.rs 和 render_view/src/lib.rs 中的插件注册方式调整
- **风险点**：UI Widget 系统行为变更可能导致运行时交互异常，需全面功能验证
