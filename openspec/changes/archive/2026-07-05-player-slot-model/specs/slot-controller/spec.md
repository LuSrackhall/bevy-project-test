## ADDED Requirements

### Requirement: SlotId 强类型

`SlotId` 是 `simulation::types` 中定义的独立强类型，标识游戏中的一个槽位编号。

```rust
pub struct SlotId(pub u8);
```

- `SlotId` 无 `Default` impl
- 实现 `Clone`、`Copy`、`PartialEq`、`Eq`、`Debug`、`Hash`、`Serialize`、`Deserialize`

#### Scenario: SlotId 构造

- **WHEN** 构造 `SlotId(0)`、`SlotId(1)`
- **THEN** 两者不相等

### Requirement: Controller 枚举

定义谁产生 Command。枚举位于 `simulation::types`。

```rust
pub enum Controller {
    HumanLocal,
    HumanRemote(PlayerId),
    AI(AiProfile),
    Agent(AgentId),
    Replay,
    Disabled,
}
```

- `Controller::HumanLocal` — 本地人类玩家，单人模式默认
- `Controller::HumanRemote(PlayerId)` — 远程人类玩家（联机预留）
- `Controller::AI(AiProfile)` — AI 控制
- `Controller::Agent(AgentId)` — LLM Agent 等未来扩展预留
- `Controller::Replay` — 回放数据源
- `Controller::Disabled` — 槽位关闭，不产生 Command

`Controller` 实现 `Clone`、`Debug`、`Serialize`、`Deserialize`。实现辅助方法 `is_active()` — `Disabled` 返回 false，其余返回 true。

#### Scenario: Controller active 判断

- **WHEN** `Controller::HumanLocal`
- **THEN** `is_active()` 返回 true

#### Scenario: Disabled 不产生命令

- **WHEN** `Controller::Disabled`
- **THEN** `is_active()` 返回 false，`collect_command_players` 排除此槽位

### Requirement: PlayerSlots 资源

`PlayerSlots` 是 `simulation::types` 中定义的 Resource，描述当前 Session 的所有槽位分配。

```rust
pub struct PlayerSlot {
    pub slot_id: SlotId,
    pub controller: Controller,
    pub faction: FactionId,
    pub team: TeamId,
}

pub struct PlayerSlots {
    pub slots: Vec<PlayerSlot>,
}
```

- `PlayerSlots` 实现 `Clone`、`Debug`、`Serialize`、`Deserialize`
- `PlayerSlots` 作为 Bevy ECS Resource 注入 Simulation 世界
- 单人模式默认初始化 2 个 slot：slot 0 → HumanLocal → FactionId(0) → TeamId(0)，slot 1 → AI → FactionId(1) → TeamId(1)

#### Scenario: 单人模式默认初始化

- **WHEN** `PlayerSlots::default()` 被调用
- **THEN** 返回包含 2 个 slot 的分配：HumanLocal(槽0, faction0, team0) + AI(槽1, faction1, team1)

#### Scenario: PlayerSlots 注入 Simulation

- **WHEN** SessionBootstrap 完成
- **THEN** Simulation world 中存在 `PlayerSlots` Resource
