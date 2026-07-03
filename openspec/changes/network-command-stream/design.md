## Context

当前项目已完成 simulation-command-pipeline 架构固化（v0.3.3）。Simulation 是确定性状态机（15000 ticks 验证），所有外部输入通过 GameCommand 驱动，CommandSource trait 封装 Live/Replay 差异。宪法强制执行零感知（§1.2.7）、Pipeline 固化（§2.5.4）、Scheduler 域盲（§2.5.5）。

架构上的联机基础已就位：CommandSource = 可插拔输入源，Replay = command stream，ReplayRecorder = Driver side-effect sink。Network 层只需要实现一个新的 CommandSource 变体。

选用 Relay-backed deterministic lockstep（方案 A）。详见 brainstorm-spec.md（第 2 节架构决策）。

## Goals / Non-Goals

**Goals:**
- 支持 2-8 名玩家联机对战
- 新增 NetworkCommandSource，复用 CommandSource trait，不修改 Simulation
- Relay Server 做命令收集 + tick barrier + 广播 + command log 缓存
- 所有模式可录制（Live / Network / Replay 共用 ReplayRecorder）
- 断线重连 = replay-based recovery（seed + full command log）
- 输入延迟默认 3 ticks @20Hz，可配置

**Non-Goals:**
- Client-Server 权威模型不做
- Pure P2P 不做
- Snapshot-based reconnect 不做
- 自定义二进制协议不做（用 serde bincode）
- 预测回滚不做
- 大厅/匹配/账户不做

## Decisions

### D1: CommandSource trait 扩展（三处最小改动）

```rust
pub trait CommandSource {
    fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand>;
    fn total_ticks(&self) -> Option<u32>;
    fn is_tick_ready(&self, tick: u32) -> bool { true }   // 新增
    fn should_record(&self) -> bool { true }               // 新增
}
```

- Live/Replay 保持默认实现，不改变已有行为
- Network 唯一 override is_tick_ready()（基于 relay batch 到达）
- 用 should_record() 替代 driver.rs 中的 is_live 类型检查

### D2: 单 canonical source 严格锁步（方案 A）

```
cmd_buf (staging) → PlayerTickFrame → relay
Relay: collect → barrier → broadcast CommandBatch (all players)
NetworkCommandSource.commands_for_tick() = relay_buffer.get(tick)
Driver → simulation::run_tick_default()
```

- 不做 merge，不做 local execution path
- relay echo 所有玩家（含发送者自身）
- cmd_buf 在 Network 模式下生命周期终点 = 上行到 relay

### D3: 数据结构分层

- `TickCommands` = simulation artifact（纯命令批次，无网络元数据）。用于 replay、reconnect、seek。
- `BroadcastFrame` = transport envelope（含 game_id、ruleset_version、payload: TickCommands）
- `PlayerTickFrame` = upstream（game_id、tick、player_id、commands、player_sid）

### D4: Input Delay 公式

`D >= R / T_tick + J`，向上取整（D=默认 3）。输入延迟偏移只发生在 NetworkCommandSource 内部。timeout 基准 = relay wall clock first_arrival[tick]。

### D5: Relay Server 角色

仅做收集 + barrier + 日志。不做 simulation、不做排序、不解析 Action semantic。

## Risks / Trade-offs

- **[强中心时间依赖]** relay wall clock 是唯一时间裁决器 → Phase 1 可接受。未来多 relay 需要 redesign
- **[cmd_buf 跨模式语义不同]** Live = execution source，Network = staging only → 通过文档约束 + runtime assert 控制
- **[TickCommands 含 relay policy]** replay 录的是 finalized batch，不是 raw input → 文档已显式说明，防止未来误用
- **[单 relay 单点]** 当前 relay 是单进程 → 作为最小可行发布，Phase 1 不做高可用

## Migration Plan

1. CommandSource trait 扩展：加 is_tick_ready()、should_record() 默认实现 → 零改动测试
2. driver.rs 修改：将 is_live 替换为 should_record() → 回归测试全部通过
3. 新增 network.rs：NetworkCommandSource + relay 通信协议
4. Cargo.toml 添加 tokio + bincode
5. 更新 command-pipeline-guide.md
6. e2e 测试：本地起 relay → 两个客户端联机 → replay 验证

## Open Questions

- relay 网络库选型：tokio (tokio::net) vs async-std
- relay 二进制部署形态：bevy 内置线程 vs 独立进程
