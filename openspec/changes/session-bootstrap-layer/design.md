## Context

v0.4.0 已实现联机基础设施（Relay-backed lockstep），但入口仅支持单人/回放模式。需要新增"联机"入口和统一初始化管道。

当前入口流：`main_menu → reset_game_system → Playing`。需要新增 Network 路径，将 UI 输入映射为 `CommandSource::Network`。

## Goals / Non-Goals

**Goals:**
- 主菜单增加联机入口（输入 relay 地址 + 玩家数量）
- GameIntent → SessionConfig → SessionArtifacts → wire 管道
- Network 模式下连接 relay、完成握手、启动 tick loop
- 统一所有模式的初始化路径

**Non-Goals:**
- 不做匹配/大厅/房间系统
- 不修改 relay 协议（v0.4.0 冻结）
- 不修改 Simulation 层

## Decisions

### D1: UI → Intent → Config → Artifacts → Wire
UI 产生 GameIntent（render_view）。resolve_intent() 纯转换为 SessionConfig。dispatch 调度 initializer 产生 SessionArtifacts（enum）。wire() 将 artifacts 写入 Driver/World/Resources。

### D2: BootstrapPhase 替代 ECS Resource
`SimulationDriver.bootstrap_phase: BootstrapPhase { Init, Wired, Active }` 管理启动状态。不依赖 ECS resource timing（宪法 §2.5.5）。

### D3: prepare → validate → commit
Bootstrap 分三阶段：prepare（I/O，不修改系统）→ validate（完成性检查）→ commit（固定顺序写入，driver.source 最后）。

### D4: Module Initializers
single / replay / network 各为模块函数，返回 mode 特定的 bootstrap facts。wire() 统一构造 CommandSource 并注册资源。

## Risks / Trade-offs

- **[handshake 同步阻塞]** → UI 短暂冻结，bootstrap 完成后消失。属于一次性初始化，不做异步。
- **[dispatch 扩展压力]** → 3 模式 match 当前最优；≥5-6 模式时演进为 registry。
- **[InitCtx 膨胀]** → 约 8 个字段，超 12 个或出现自然分组时拆分。
- **[无宪法级不变量]** → P1-P10 均为 ADR 级约束，非全局宪法。
