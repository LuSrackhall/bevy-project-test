## ADDED Requirements

### Requirement: PickingInteraction 替代 Interaction

系统 SHALL 使用 `PickingInteraction` 替代 `Interaction` 作为全局交互检测源。

#### Scenario: 悬停检测使用 PickingInteraction

- **WHEN** 玩家鼠标悬停在兵种按钮上
- **THEN** `update_bottom_panel` SHALL 通过 `PickingInteraction::Hovered` 检测到悬停状态，并显示对应兵种百科信息

#### Scenario: 穿透保护使用 PickingInteraction

- **WHEN** 玩家点击 UI 按钮
- **THEN** `is_any_ui_pressed` SHALL 通过 `Query<&PickingInteraction>` 检测到 `PickingInteraction::Pressed` 状态，并阻止游戏世界的点击处理

#### Scenario: PickingInteraction 自动维护

- **WHEN** UI 节点被 Picking 系统检测到
- **THEN** `PickingInteraction` 组件 SHALL 自动插入和更新，无需手动管理

### Requirement: 消除 Interaction 依赖

Phase 1.5 完成后，代码库 SHALL NOT 包含任何对 `Interaction` 组件的查询或引用。

#### Scenario: 无 Interaction 查询

- **WHEN** Phase 1.5 迁移完成
- **THEN** `selection.rs` 和 `hud.rs` 中 SHALL NOT 存在 `Query<&Interaction>` 或 `Changed<Interaction>` 的使用

#### Scenario: 功能不受影响

- **WHEN** Phase 1.5 迁移完成
- **THEN** 所有现有功能（选中、穿透保护、悬停百科）SHALL 保持不变
