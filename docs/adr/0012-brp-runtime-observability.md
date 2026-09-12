# ADR 0012: 运行时可观测（BRP）与 cargo registry 迁移

## 状态

**Date**: 2026-09-12
**Status**: Accepted（实现于 P0.4）

## 背景

P0 的前四项把「静态验证」（测试 / 确定性 / 回放往返 / AI 自对弈）做成了机器门禁，
但 agent 仍无法观测**运行中的客户端**：仿真状态藏在 `SimulationWorld`（NonSend）里，
UI 是否正确只能靠人眼。Bevy 官方提供的运行时通道是 BRP（Bevy Remote Protocol，
JSON-RPC 2.0），其内置能力（`world.query`、`world.trigger_event`、生成 `Screenshot`
实体）是 Bevy 侧**唯一有官方支持的 UI 自动化路径**——官方
`examples/remote/integration_test.rs` 的流程即「找到按钮 → 注入点击 → 截图 → 断言」。

同时，启用 BRP 会引入 9 个新 crate，而 `~/.cargo/registry`（507M）位于系统盘，
与「把开发负载移出系统盘」的既定策略冲突。

## 决策

1. **BRP 以 feature 形式接入**（默认关闭）：
   - 根 `Cargo.toml`：`remote = ["bevy/bevy_remote", "bevy_adapter/remote"]`
   - `bevy_adapter`：`remote = ["bevy/bevy_remote", "dep:serde_json"]`
   - `src/main.rs`：`#[cfg(feature = "remote")]` 下挂
     `RemotePlugin` + `RemoteHttpPlugin`
2. **仿真状态用自定义方法暴露，而不是 `world.query`**：`simulation` 组件不能派生
   `Reflect`（§1.4 仅允许 bevy_ecs 白名单子集），因此新增
   `city_conquest/probe` 方法，由适配层 `bevy_adapter::remote` 桥接，返回
   `{tick, world_hash, total_soldiers, total_cities, factions[]}`。
   `world_hash` 与 `sim-cli` **同源**（`golden_test::hash_world_state`），可直接对比
   「运行中的客户端」与「无头回放」，从而判定分歧属于仿真层还是表现层。
3. **`~/.cargo/registry` 迁移到 SSD**：内容复制到 `/Volumes/SSD980/cargo/registry`
   （20795 文件 / 507M，逐项校验 0 差异）后删除原件并软链回 `~/.cargo/registry`，
   使新增依赖与后续所有 crate 下载不再落系统盘。

## 放弃的方案

- **给 simulation 组件派生 `Reflect`**：能让 BRP 内置查询直接可用，但会把
  `bevy_reflect` 引入仿真层，直接违反 §1.4 白名单与 §1.2 分层意图——为观测能力
  牺牲核心架构不划算。
- **自研零依赖 JSON 探针**：可省掉 9 个新依赖，但拿不到官方的
  `world.trigger_event` 事件注入与 `Screenshot` 实体，也就失去了
  「无人类 UI 验收」这条最有价值的路径。
- **把整个 `~/.cargo`（含 76M bin）搬到 SSD**：bin 中是 cargo/rustc 的 shim，
  留在本机可让「SSD 未挂载」时至少工具链本身仍可调用，影响面更小。

## 代价

- 默认构建不受影响；仅显式 `--features remote` 才编译 BRP，CI 增加一条 Linux
  步骤守护该 feature 的 clippy 与探针测试。
- 新增 9 个依赖（`bevy_remote` 与 `hyper/http` 栈），落在 SSD 的 registry 中。
- `~/.cargo/registry` 成为 SSD 上的软链：**SSD 未挂载时 cargo 无法解析依赖**，
  影响本机所有 Rust 项目。这是「不再增长系统盘」付出的可用性代价。
- BRP 默认监听 `127.0.0.1:15702` 且能改写 ECS 状态：仅供本地开发/调试，
  不得在发布构建中启用。

## 修改条件

1. 若 BRP 的维护成本（依赖栈、API 变更）超过其价值，或出现更轻的官方可观测方案，
   可整体移除该 feature。
2. 若要允许 agent 观测**发布构建**，必须先把 BRP 暴露面收敛为只读子集
   （移除 `world.insert_components` / `mutate_components` 等写方法）。
3. 若 SSD 不再是常驻卷，需把 `~/.cargo/registry` 的软链回退为本地目录。
