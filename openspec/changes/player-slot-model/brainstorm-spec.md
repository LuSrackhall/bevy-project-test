## Context

当前 `simulation` 层将「谁控制」和「谁拥有」绑定在同一个 `Faction` 枚举中：

- `Faction::Player` = 人类玩家 + 拥有单位
- `Faction::Enemy` = AI 敌人 + 拥有单位
- `Faction::Neutral` = 无人控制

这导致以下限制：

1. **无法支持多人 PvP** — 第二个玩家无法控制第二阵营
2. **AI 与控制者绑定** — AI 硬编码 Enemy，无法将 AI 分配给任意阵营
3. **Faction 语义过载** — 一个枚举同时承载阵营身份、控制者身份、胜负阵营
4. **扩展困难** — 未来 Agent、Remote Player 等新控制者类型需要改 Faction 枚举
5. **Session 与 Simulation 耦合** — Lobby 层面的"谁坐哪个位置"与仿真层的"谁拥有哪个单位"混在一起

### 相关文件

| 文件 | 当前状态 |
|------|----------|
| `crates/simulation/src/types.rs` | `Faction` 枚举；`FactionComponent(pub Faction)` |
| `crates/simulation/src/lib.rs` | `collect_command_players()` 按枚举取值 |
| `crates/simulation/src/ai/mod.rs` | AI 硬编码 `Faction::Enemy` |
| `crates/simulation/src/map/mod.rs` | 按硬编码比例分配 Player/Enemy 城市 |
| `crates/bevy_adapter/src/driver.rs` | `RunConfig { enable_ai }` 一次性禁用全部 AI |
| `crates/render_view/src/selection.rs` | 硬编码 `Faction::Player` 过滤可选单位 |

## Goals / Non-Goals

### Goals（本变更负责）

- `FactionId`、`TeamId`、`SlotId` 强类型定义
- `Controller` 枚举（HumanLocal / HumanRemote / AI / Agent / Replay / Disabled）
- `PlayerSlot` / `PlayerSlots` 资源（描述当前 Session 的槽位分配）
- Slot → Controller → Faction 的映射结构
- `FactionComponent` 从枚举改为 `FactionId` 强类型
- `collect_command_players` 改为基于 `PlayerSlots`，不再扫描枚举
- AI 基于 Slot 工作 —— AI 分配给哪个 faction_id 就控制哪个 faction
- Command Pipeline 基于 Slot 工作 —— NoOp 按 Slot 注入
- 单人模式通过 1 × Human Slot + N × AI Slot 保持现有体验
- `RunConfig.enable_ai` 保留行为，但底层按 Slot 分配

### Non-Goals（本变更明确不负责）

- Lobby UI
- 房间系统 / Ready 机制
- 地图生成策略（均匀分布、固定出生点等——这是地图模块的职责）
- Spawn 分布策略
- 胜利条件（全灭、据点等——这是游戏规则的职责）
- Observer 模式
- Agent 行为实现（只预留枚举）
- LAN Discovery
- 局域网 / 互联网联机协议

## Decisions

### D1：Faction 改为纯 ID

```rust
pub struct FactionId(pub u8);
pub struct TeamId(pub u8);
pub struct SlotId(pub u8);

pub struct FactionComponent(pub FactionId);
```

`FactionId` 与 `TeamId` 是独立的强类型，编译器防止混用。单位只知道自己的 `FactionId`，不知道 Controller 是谁。

### D2：Controller 枚举

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

Agent 和 AiProfile 只留变体占位，具体实现在本变更之后。

### D3：PlayerSlots（Session 层面的槽位分配）

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

`PlayerSlots` 是 `Resource`，在 `SessionBootstrap` 期间初始化。Simulation 只读 `faction` 字段，不看 `controller`。

### D4：collect_command_players → 基于 Slot

```rust
fn collect_command_players(slots: &PlayerSlots) -> Vec<u8> {
    slots.iter()
        .filter(|s| !matches!(s.controller, Controller::Disabled))
        .map(|s| s.faction.0)
        .collect()
}
```

不再扫描世界中的 FactionComponent，完全基于 Slot 分配。

### D5：AI 基于 Slot

AI 不再硬编码 `Faction::Enemy`。AI tick 时读取 `PlayerSlots`，找到所有 `Controller::AI(_)` 的 slot，为每个 AI slot 对应的 faction 生成命令。

```rust
fn ai_decide(world: &mut World, slots: &PlayerSlots, current_tick: u32) {
    for slot in slots.iter().filter(|s| matches!(s.controller, Controller::AI(_))) {
        let faction_id = slot.faction;
        // 为这个 faction_id 生成命令
    }
}
```

### D6：单人模式兼容

单人模式初始化 `PlayerSlots` 映射到现有 2-faction 体验：

```rust
PlayerSlots {
    slots: vec![
        PlayerSlot { slot_id: SlotId(0), controller: Controller::HumanLocal, faction: FactionId(0), team: TeamId(0) },
        PlayerSlot { slot_id: SlotId(1), controller: Controller::AI(DefaultAI), faction: FactionId(1), team: TeamId(1) },
    ],
}
```

### D7：NoOp 注入基于 Slot

```rust
for slot in slots.iter().filter(|s| s.controller.is_active()) {
    if !present_players.contains(&slot.faction.0) {
        commands.push(GameCommand { tick, player_id: slot.faction.0, action: Action::NoOp });
    }
}
```

### D8：RunConfig.enable_ai 保留

`RunConfig { enable_ai: bool }` 从"是否跑 AI tick"改为"是否允许非空 AI Controller 生成命令"。逻辑上保持向后兼容。

## Core Invariants

> Simulation 只关心 Ownership（FactionId），Session 只关心 Control（Slot/Controller），两者通过映射关联，但彼此解耦。

### Invariant 1

> **Simulation 永远不知道 Human、Remote、AI、Agent。**

Simulation 只认识 `FactionId`。Controller 的语义在 Session 层完成。

### Invariant 2

> **Controller 只负责产生 Command。**

Controller 不拥有单位。单位只属于 `FactionId`。一个 Controller 产生的 Command 通过 Slot → Faction 映射路由到目标单位。

### Invariant 3

> **Faction 只负责 Ownership。**

Faction 不决定 Command 来源。一个 Faction 可以有多个 Controller，也可以没有。

### Invariant 4

> **Slot 是 Session 生命周期对象。**

Slot 可以变化（Lobby、Replay、Benchmark）。Faction 是 Simulation 生命周期对象。两者生命周期不同，不应混用。

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| 强类型改造涉及 simulation 全层 | 自动化替换，编译期验证 |
| `FactionId` ↔ `u8` 互操作可能导致边界错误 | 强类型封装，只在显式转换点允许 `into()` / `from()` |
| 现有单机模式 2 faction → 2 slot 的映射可能遗漏场景 | 单元测试覆盖：1v1 单机、2v2 Human+AI vs AI+AI |
| Controller 序列化增加成本（影响 replay） | Controller 只序列化必要字段，Agent 字段留空 |
| 触及三层（simulation / adapter / render_view） | 按层分批实现：类型 → pipeline → AI → render_view |

## Define Done

1. `FactionId`、`TeamId`、`SlotId`、`Controller`、`PlayerSlots` 类型已定义并编译
2. `FactionComponent` 改为 `FactionComponent(pub FactionId)`
3. `collect_command_players` 基于 `PlayerSlots`，不再扫描枚举
4. AI 基于 Slot 分配（不再硬编码 `Faction::Enemy`）
5. NoOp 按 Slot 注入
6. `RunConfig.enable_ai` 行为兼容
7. 单人模式（1 Human + 1 AI）现有体验不变
8. 所有现有测试通过
9. 不新增 Lobby UI、不新增地图生成策略、不新增胜利条件
