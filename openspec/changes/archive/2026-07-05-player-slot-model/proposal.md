## Why

当前 `simulation` 层的 `Faction` 枚举将「谁控制」和「谁拥有」绑定在一起（`Faction::Player` = 人类 + 拥有单位，`Faction::Enemy` = AI + 拥有单位）。这导致无法支持多人 PvP、AI 无法分配给任意阵营、Faction 语义过载、Session 与 Simulation 耦合。本次变更将这些概念彻底解耦，建立「谁控制」与「谁拥有」的独立模型，为未来的联机对战、Agent 控制、灵活的房间系统奠定基础。

## What Changes

- **BREAKING**: `Faction` 枚举替换为 `FactionId(pub u8)` 强类型
- **BREAKING**: `FactionComponent(pub Faction)` 改为 `FactionComponent(pub FactionId)`
- 新增 `TeamId`、`SlotId` 强类型
- 新增 `Controller` 枚举：`HumanLocal` / `HumanRemote` / `AI` / `Agent` / `Replay` / `Disabled`
- 新增 `PlayerSlot` / `PlayerSlots` 资源（Session 层面的槽位分配）
- `collect_command_players()` 从扫描 FactionComponent 改为基于 `PlayerSlots`
- AI 不再硬编码 `Faction::Enemy`，改为基于 Slot 分配
- NoOp 注入从基于 Faction 枚举改为基于 `PlayerSlots`
- `RunConfig.enable_ai` 保留（语义从「禁用 AI tick」改为「禁止 AI Controller 生成命令」）
- 单人模式通过 1 × Human Slot + 1 × AI Slot 映射到 2-faction 体验，完全兼容

## Capabilities

### New Capabilities
- `faction-id`: `FactionId` 强类型定义与 `FactionComponent` 改造
- `slot-controller`: `SlotId`、`Controller`、`PlayerSlots` 类型定义与初始化
- `command-pipeline-slot`: `collect_command_players` 与 NoOp 注入改为基于 `PlayerSlots`
- `ai-slot-binding`: AI 决策层基于 Slot 分配，不再硬编码 `Faction::Enemy`

### Modified Capabilities
（无现有 spec 的 REQUIREMENTS 被修改——本次变更新增能力，不改现有 spec）

## Impact

| 系统 | 影响 |
|------|------|
| `crates/simulation/src/types.rs` | Faction 枚举移除，新增 FactionId/TeamId/SlotId/Controller/PlayerSlots |
| `crates/simulation/src/soldier/mod.rs` | `FactionComponent` 类型变更，`consume_commands` 的 SetSeekStance 筛选改用 `FactionId` |
| `crates/simulation/src/lib.rs` | `collect_command_players()` 重写；`run_tick()` 的 NoOp 注入逻辑变更；`RunConfig` 传入 `PlayerSlots` |
| `crates/simulation/src/ai/mod.rs` | AI 决策函数签名变更：接收 `PlayerSlots` 参数 |
| `crates/simulation/src/map/mod.rs` | 城市/单位创建时的 `Faction` 枚举改为 `FactionId` |
| `crates/simulation/src/combat/` | Faction 比较改为 FactionId 比较 |
| `crates/simulation/src/unit_index.rs` | 无影响 |
| `crates/bevy_adapter/src/driver.rs` | `simulation_driver_system` 传入 `PlayerSlots`（自动通过 Resource） |
| `crates/bevy_adapter/src/session/bootstrap.rs` | SessionBootstrap 初始化 `PlayerSlots`（使用默认 single_player，网络模式需后续补充） |
| `crates/render_view/src/selection.rs` | 筛选改用 `PlayerSlots`（当前通过 `LocalPlayerId` → `FactionId(lid)` 兼容路径） |
| `crates/render_view/src/camera.rs` | 居中逻辑改用 `PlayerSlots`（当前通过 `LocalPlayerId` 兼容路径） |
| `crates/render_view/src/ui/hud.rs` | HUD 展示逻辑改用 `PlayerSlots`（部分完成——top bar 用 FactionId，按钮 player_id 仍硬编码 `0`，需下个迭代修） |
| 测试 | 所有 Faction 枚举引用需更新为 FactionId（`Faction::Player` → `FactionId(0)`） |
| Replay | Controller 的序列化实现——HumanLocal/HumanRemote/AI 字段需 Serialize/Deserialize |
