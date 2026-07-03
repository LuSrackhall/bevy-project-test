## Why

当前项目已完成 simulation-command-pipeline 架构固化（v0.3.3），Simulation 已经是确定性状态机、CommandSource trait 封装了 Live/Replay 模式差异、Replay 系统验证了 15000 ticks 确定性。此时联机是结构性必然——不做联机后面所有 feature 都会成为"潜在非确定性债务"。

需要为游戏添加 2-8 名玩家的联机对战能力，以 Relay-backed deterministic lockstep 模式实现，不修改 Simulation 层。

## What Changes

- 在 `CommandSource` trait 中新增 `is_tick_ready()` + `should_record()` 方法（含默认实现）
- 新增 `NetworkCommandSource`：实现 CommandSource，从 relay 的 finalized batch 消费命令，不做 merge
- 新增 Relay Server：命令收集 + tick barrier + 广播 + command log 缓存，不做 simulation、不做排序决策
- 新增 `TickCommands` / `BroadcastFrame` / `PlayerTickFrame` 协议数据结构
- 新增 Reconnect 协议：replay-based recovery（seed + full command log）
- 新增输入延迟模型：默认 3 ticks @20Hz，可配置
- 修改 `ReplayRecorder` 条件：从 is_live 替换为 should_record()
- 修改 `crates/bevy_adapter/Cargo.toml`：添加 tokio + bincode 依赖
- **不修改** `simulation` crate（宪法 §1.2.7 零感知原则）
- **不引入**大厅、匹配、账户、observer 系统（Phase 2）

## Capabilities

### New Capabilities

- `network-command-source`: NetworkCommandSource 实现，从 relay finalized batch 消费命令，不做 merge
- `relay-server`: Relay Server 实现，命令收集 + tick barrier + 广播 + command log 缓存
- `network-reconnect`: 断线重连协议，replay-based recovery（seed + full command log）
- `input-delay-model`: 输入延迟模型与 config 默认 3 ticks

### Modified Capabilities

- (无) — 不修改 Simulation 层的行为

## Impact

- `crates/bevy_adapter/src/driver.rs`: CommandSource trait 加 is_tick_ready() + should_record()
- `crates/bevy_adapter/src/network.rs`: 新增 300-500 行，NetworkCommandSource + relay 通信
- `crates/bevy_adapter/Cargo.toml`: 新增 tokio + bincode 依赖
- `docs/engineering/command-pipeline-guide.md`: 补充 network 模式数据流
- 外部 crate 无感知：render_view、presentation、simulation 不修改
