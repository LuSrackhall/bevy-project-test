## ADDED Requirements

### Requirement: Observer 可监听 UI 按钮的 Pointer<Click> 事件

全局 Observer SHALL 能够通过 `app.add_observer()` 注册并监听 `Pointer<Click>` 事件，当 UI 按钮被点击时触发。

#### Scenario: 点击 UI 按钮触发 Observer

- **WHEN** 玩家点击主菜单的 SinglePlayer 按钮
- **THEN** 注册的全局 Observer SHALL 被触发，`ev.entity()` SHALL 返回被点击的按钮实体

### Requirement: Observer 可访问 ECS 系统参数

Observer 闭包 SHALL 能够访问标准的 ECS 系统参数，包括 `Query`、`ResMut`、`Res` 等。

#### Scenario: Observer 内读取按钮组件

- **WHEN** Observer 被触发
- **THEN** Observer 内 SHALL 能够通过 `Query<&MenuButton>` 读取被点击实体的 `MenuButton` 组件

#### Scenario: Observer 内修改游戏状态

- **WHEN** Observer 被触发且按钮为 SinglePlayer
- **THEN** Observer 内 SHALL 能够通过 `ResMut<NextState<GameState>>` 切换游戏状态到 `Playing`

### Requirement: Observer 与旧系统可并行运行

新增的 Observer 和现有的 `menu_button_system` SHALL 能够在同一帧内并行运行，不产生冲突。

#### Scenario: 并行运行不导致状态冲突

- **WHEN** Observer 和 `menu_button_system` 同时响应同一个按钮点击
- **THEN** 两者都设置 `GameState::Playing`，`NextState::set()` 的幂等性 SHALL 保证最终状态正确

### Requirement: Observer 验证不引入新依赖

Observer 验证 SHALL 仅使用 Bevy 0.18 已有的 `bevy_picking` 模块，不引入 `bevy_ui_widgets` 或其他 experimental feature。

#### Scenario: 无需启用 experimental feature

- **WHEN** Observer 验证代码编译运行
- **THEN** SHALL NOT 需要启用 `experimental_bevy_ui_widgets` feature gate
