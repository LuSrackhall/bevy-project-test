# bevy-adapter-crate

## Purpose

TBD
## Requirements

### Requirement: MapBounds 资源
bevy_adapter SHALL 提供 `MapBounds { width: f32, height: f32 }` 资源。在 `reset_game_system` 完成后从 `MapGenConfig` 创建。

#### Scenario: MapBounds 初始化
- **WHEN** 地图生成完成，MapGenConfig 为 5000x5000
- **THEN** `MapBounds { width: 5000.0, height: 5000.0 }` 被插入 Bevy world

### Requirement: UnitIdMapper 双向映射

bevy_adapter SHALL 维护 `UnitIdMapper` 资源，包含 `unit_to_entity: HashMap<UnitId, Entity>` 和 `entity_to_unit: HashMap<Entity, UnitId>` 两个 O(1) 双向映射。映射 SHALL 在实体创建时插入、实体销毁时移除。

#### Scenario: 新实体注册

- **WHEN** simulation 层发出 `UnitSpawned` 事件，且 bevy_adapter 在 Bevy 世界中创建对应实体
- **THEN** `UnitIdMapper` 中同时包含 `unit_to_entity` 和 `entity_to_unit` 两个方向的条目

#### Scenario: 实体销毁注销

- **WHEN** simulation 层发出 `UnitDestroyed` 事件，且 bevy_adapter 销毁 Bevy 世界中的对应实体
- **THEN** `UnitIdMapper` 中不再包含该 UnitId 或 Entity 的条目

#### Scenario: O(1) 查找

- **WHEN** 通过 `UnitId` 查找对应的 Bevy `Entity`
- **THEN** `mapper.unit_to_entity.get(&unit_id)` 在 O(1) 时间内返回 `Some(Entity)`

### Requirement: Tick 调度驱动
bevy_adapter SHALL 提供 `TickClock` 资源、`GameActive(bool)` 资源和 `Paused(bool)` 资源。`tick_driver` SHALL 仅在 `GameActive == true` 且 `Paused == false` 时运行。`GameActive` 由 render_view 在进入/退出 Playing 时设置。`Paused` 由 render_view 的暂停系统设置。每帧累加 `time.delta_secs()`，当 `accumulator >= tick_duration` 时执行一次完整 Tick。

#### Scenario: 固定频率触发
- **WHEN** `tick_duration = 50ms`，而帧时间累积达到 100ms，且 `GameState::Playing` 且未暂停
- **THEN** `tick_driver` 连续执行 2 次完整 Tick，`accumulator` 剩余 < 50ms

#### Scenario: 主菜单时不 tick
- **WHEN** `GameState::MainMenu`
- **THEN** `tick_driver` 不运行，`TickClock` 不变化

#### Scenario: 暂停时不 tick
- **WHEN** `GameState::Playing` 且 `Paused.0 == true`
- **THEN** `tick_driver` 不运行，`TickClock` 不变化

### Requirement: 输入→命令翻译

bevy_adapter SHALL 提供系统将 Bevy 鼠标/键盘/HUD 按钮事件翻译为 `GameCommand` 并推入 `CommandBuffer`。翻译系统 SHALL 读取 `render_view` 暴露的 `SelectionState` 来获取当前选中的 `UnitId` 列表。

#### Scenario: 右键空地→移动命令

- **WHEN** 玩家右键点击空地，且当前选中了 3 个 UnitId
- **THEN** 翻译系统向 `CommandBuffer` 推入 3 条 `Action::MoveTo` 命令（每条对应一个选中单位），标记为下一个未完成的 Tick

#### Scenario: 右键敌方单位→攻击命令

- **WHEN** 玩家右键点击一个敌方士兵
- **THEN** 翻译系统对每个选中的 UnitId 推入一条 `Action::Attack` 命令，目标为被点击的敌方 UnitId

#### Scenario: 右键友方城池→回城命令

- **WHEN** 玩家右键点击一个友方城池
- **THEN** 翻译系统对每个选中的 UnitId 推入一条 `Action::ReturnToCity` 命令

#### Scenario: 举盾按钮→SetShield 命令

- **WHEN** 玩家按下 HUD 举盾按钮
- **THEN** 翻译系统对每个选中的步兵 UnitId 推入一条 `Action::SetShield` 命令

#### Scenario: Shift+右键→强制移动命令

- **WHEN** 玩家按住 Shift 并右键点击空地
- **THEN** 翻译系统对每个选中的 UnitId 推入 `Action::ForceMove` 命令（而非 `MoveTo`），单位移动中途不自动接战

### Requirement: 实体生灭同步
bevy_adapter SHALL 通过 `sync_entities_system` 监听 simulation 层事件，在 Bevy 世界中同步创建/销毁对应实体。`sync_entities_system` SHALL 仅在 `GameState::Playing` 且 `Paused.0 == false` 时运行。`backfill_entities_system` SHALL 由 `reset_game_system` 调用（从 `Startup` 移除），在 `OnEnter(Playing)` 时执行。

#### Scenario: 新实体同步
- **WHEN** `GameState::Playing` 且未暂停，simulation 层产出新实体
- **THEN** `sync_entities_system` 创建对应 Bevy 实体并注册到 `UnitIdMapper`

#### Scenario: 暂停时不同步
- **WHEN** `GameState::Playing` 且 `Paused.0 == true`
- **THEN** `sync_entities_system` 不运行

### Requirement: Tick 命令快照

bevy_adapter SHALL 在每个 Tick 执行前，从 `CommandBuffer` 中提取标记为当前 `current_tick` 的所有命令，组装为不可变快照传递给 `simulation::run_tick`。缺少命令的 player SHALL 被注入 `NoOp` 命令。

#### Scenario: 命令快照打包

- **WHEN** `CommandBuffer` 中包含 5 条 `tick: 3` 的命令和 2 条 `tick: 4` 的命令，当前 `current_tick == 3`
- **THEN** 仅 5 条 `tick: 3` 的命令被传递给 `simulation::run_tick`，2 条 `tick: 4` 的命令保留在缓冲中

#### Scenario: No-Op 补齐保证时序

- **WHEN** `current_tick == 3`，但 `CommandBuffer` 中没有 `player_id: 0` 的命令
- **THEN** 系统自动注入 `GameCommand { tick: 3, player_id: 0, action: NoOp }`，保证 player 0 在 Tick 3 有确定性的执行路径

### Requirement: UnitIdMapper 清理方法
`UnitIdMapper` SHALL 提供 `clear()` 方法，清空 `unit_to_entity` 和 `entity_to_unit` 两个映射表。

#### Scenario: 清空映射
- **WHEN** `reset_game_system` 调用 `mapper.clear()`
- **THEN** `unit_to_entity` 和 `entity_to_unit` 均为空 `HashMap`

