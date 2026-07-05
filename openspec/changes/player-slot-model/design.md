## Context

本次变更的核心目标在 brainstorm-spec.md 中已充分讨论。设计层面在「Decisions」和「Core Invariants」章节中已明确定义。本文档补充实现层面的技术决策和模块边界。

现状：`Faction` 枚举耦合了「谁控制」和「谁拥有」。本次变更加入 `PlayerSlots` 作为中间层解耦两者。

## Goals / Non-Goals

参见 brainstorm-spec.md「Goals / Non-Goals」章节。补充实现层面的约束：

**Goals：**
- 所有类型定义（FactionId / TeamId / SlotId / Controller / PlayerSlots）放在 `simulation` 层
- `PlayerSlots` 在 SessionBootstrap 时初始化，作为 Resource 注入 Simulation 世界
- `FactionComponent(FactionId)` 替换现有 `FactionComponent(Faction)`，编译期全覆盖
- AI 决策函数接收 `&PlayerSlots` 参数，不再访问 `Faction::Enemy` 枚举
- 单人模式默认 `PlayerSlots` 由 `bevy_adapter` 的 `BevyAdapterPlugin` 或 SessionBootstrap 初始化

**Non-Goals：**
- 不改变 map 模块的城市/单位生成策略（map 模块的 faction 分配保持现有逻辑，只改类型签名）
- 不新增 Lobby UI
- 不实现 Agent 行为

## Decisions

### D1：类型定义位置

```rust
// crates/simulation/src/types.rs
pub struct FactionId(pub u8);
pub struct TeamId(pub u8);
pub struct SlotId(pub u8);

pub enum Controller {
    HumanLocal,
    HumanRemote(PlayerId),
    AI(AiProfile),
    Agent(AgentId),
    Replay,
    Disabled,
}

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

所有类型集中在 `simulation` 层的 `types.rs`，保证上下游可见。`PlayerSlot` 用 `pub` 字段而非 getter/setter（简化操作，无封装增益）。

### D2：Controller 的序列化策略

直接 derive `Serialize/Deserialize`。`AiProfile` 和 `AgentId` 先定义为单元元组：

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct AiProfile(pub String); // 预留，当前用 DefaultAI

#[derive(Clone, Serialize, Deserialize)]
pub struct AgentId(pub u64);      // 预留
```

`HumanRemote` 的 `PlayerId` 复用已有 `simulation::types::PlayerId`。

### D3：PlayerSlots 初始化

SessionBootstrap 时创建。单人模式的默认值：

```rust
PlayerSlots {
    slots: vec![
        PlayerSlot {
            slot_id: SlotId(0),
            controller: Controller::HumanLocal,
            faction: FactionId(0),
            team: TeamId(0),
        },
        PlayerSlot {
            slot_id: SlotId(1),
            controller: Controller::AI(AiProfile::default()),
            faction: FactionId(1),
            team: TeamId(1),
        },
    ],
}
```

`bevy_adapter` 的 `SessionBootstrap` 负责构造并插入。`SimulationDriver` 的 `new_network()` 和 `new_live()` 方法不再需要修改——`PlayerSlots` 是独立的 Resource。

### D4：FactionComponent 改造范围

```rust
// Before
pub struct FactionComponent(pub Faction);

// After  
pub struct FactionComponent(pub FactionId);
```

需要全局替换的地方（git grep `Faction::` 的输出）：
- `simulation/src/soldier/mod.rs` — 消费命令、创建实体
- `simulation/src/combat/mod.rs` — 战斗判定（身份比较）
- `simulation/src/ai/mod.rs` — AI 决策筛选
- `simulation/src/map/mod.rs` — 城市/单位创建
- `simulation/src/lib.rs` — `collect_command_players`, `run_tick` 的 NoOp 注入
- `simulation/src/golden_test.rs` — 测试断言
- `simulation/src/world_stats.rs` — 统计
- `bevy_adapter/src/lifecycle.rs` — 实体同步
- `bevy_adapter/src/binding.rs` — 渲染绑定
- `render_view/src/selection.rs` — 指令选择
- `render_view/src/ui/hud.rs` — HUD 展示
- `render_view/src/camera.rs` — 摄像机居中
- `render_view/src/lib.rs` — 征战判定

策略：先定义新类型 → 改 simulation 层 → 编译通过 → 改 adapter/render 层 → 编译通过。（而非逐个文件手动替换）

### D5：AI 决策的 Slot 接入

```rust
pub fn ai_decide(world: &mut World, slots: &PlayerSlots, current_tick: u32) {
    if !current_tick.is_multiple_of(AI_TICK_INTERVAL) {
        return;
    }
    for slot in slots.iter().filter(|s| matches!(s.controller, Controller::AI(_))) {
        let faction = slot.faction;
        // 为该 faction 收集城市、生成命令
    }
}
```

驱动层（`bevy_adapter::driver::simulation_driver_system`）传入 `PlayerSlots`。`RunConfig.enable_ai` 控制是否执行 AI 循环。

### D6：collect_command_players 与 NoOp 注入

```rust
fn collect_command_players(slots: &PlayerSlots) -> Vec<u8> {
    slots.iter()
        .filter(|s| !matches!(s.controller, Controller::Disabled))
        .map(|s| s.faction.0)
        .collect()
}

fn inject_noop(commands: &mut Vec<GameCommand>, slots: &PlayerSlots, tick: u32) {
    let present: HashSet<u8> = commands.iter().map(|c| c.player_id).collect();
    for slot in slots.iter().filter(|s| s.controller.is_active()) {
        if !present.contains(&slot.faction.0) {
            commands.push(GameCommand { tick, player_id: slot.faction.0, action: Action::NoOp });
        }
    }
}
```

### D7：render_view 侧适配

`render_view/src/selection.rs` 和 `camera.rs` 中硬编码的 `Faction::Player` 改为读取 `PlayerSlots` 中本地玩家对应的 faction。本地玩家 identity 由 `LocalPlayerId`（或 `PlayerSlots` 中的 HumanLocal slot）确定。

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| `Faction` 枚举→`FactionId` 的大范围替换引入遗漏 | 编译强制覆盖——Faction 枚举删除后编译器报所有未改的引用 |
| `Faction::Player` 与 `Faction::Enemy` 的视觉区分丢失（颜色/标签） | 视觉区分由渲染层基于 `PlayerSlots` 的 `team` 字段决定，本次变更不改视觉，只保编译 |
| Replay 序列化兼容性 | `Controller` 直接 derive Serialize，HumanLocal 和 AI 的序列化格式明确 |
| FactionId(0) 与 FactionId 默认值混淆 | FactionId 无 Default impl，必须显式构造 |
