## ADDED Requirements

### Requirement: HoverMap 替代 Interaction 作为穿透保护

系统 SHALL 使用 `HoverMap` 资源替代 `Interaction` 组件作为 UI 穿透保护检测源。

#### Scenario: 穿透保护使用 HoverMap

- **WHEN** 玩家点击 UI 按钮
- **THEN** `is_cursor_over_ui` SHALL 通过 `HoverMap` 检测到光标在 UI 上，并阻止游戏世界的点击处理

#### Scenario: 光标在游戏世界时不阻止

- **WHEN** 玩家点击游戏世界空白处（无 UI 覆盖）
- **THEN** `is_cursor_over_ui` SHALL 返回 false，游戏世界正常处理点击

#### Scenario: 透明容器不影响检测

- **WHEN** 光标位于透明容器上方（有 `Pickable::IGNORE`）
- **THEN** 透明容器 SHALL NOT 出现在 `HoverMap` 中，不干扰穿透检测

### Requirement: 悬停检测使用 Hovered 组件

系统 SHALL 使用 `Hovered` 组件（由 Picking 系统自动维护）替代 `Interaction::Hovered` 作为悬停检测源。

#### Scenario: 兵种按钮悬停显示百科

- **WHEN** 玩家鼠标悬停在兵种按钮上
- **THEN** `HoveredSoldierType` 资源 SHALL 通过 `Pointer<Over>` Observer 更新，并显示对应兵种百科信息

### Requirement: 消除 Interaction 依赖

代码库 SHALL NOT 包含任何对 `Interaction` 组件的查询或引用。

#### Scenario: 无 Interaction 查询

- **WHEN** 迁移完成
- **THEN** `selection.rs` 和 `hud.rs` 中 SHALL NOT 存在 `Query<&Interaction>` 或 `Changed<Interaction>` 的使用
