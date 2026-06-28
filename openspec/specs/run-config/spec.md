# run-config Specification

## Purpose
TBD - created by archiving change add-scenario-test-harness. Update Purpose after archive.
## Requirements
### Requirement: RunConfig 仿真运行配置

RunConfig SHALL 定义 `enable_ai: bool` 字段，默认值为 true。RunConfig 属于仿真初始化参数（类似 seed、map_size），SHALL NOT 作为 Tick 级命令参与 CommandBuffer 流水线。

#### Scenario: 默认配置启用 AI

- **WHEN** 构造 RunConfig::default()
- **THEN** enable_ai 为 true

#### Scenario: 禁用 AI

- **WHEN** 构造 RunConfig { enable_ai: false }
- **THEN** enable_ai 为 false

### Requirement: run_tick 签名变更

run_tick SHALL 接受第三个参数 `config: &RunConfig`。SHALL 提供 `run_tick_default(world, tick)` 兼容包装，等价于 `run_tick(world, tick, &RunConfig::default())`。

#### Scenario: run_tick_with_config 禁用 AI

- **WHEN** 调用 run_tick(&mut world, tick, &RunConfig { enable_ai: false })
- **THEN** ai_decide 阶段不执行，AI 子系统不产生命令

#### Scenario: run_tick_default 行为不变

- **WHEN** 调用 run_tick_default(&mut world, tick)
- **THEN** 行为等价于 run_tick(&mut world, tick, &RunConfig::default())

### Requirement: RunConfig ADR

SHALL 创建 ADR 记录 RunConfig 的语义定位：决策内容、放弃方案、代价、修改条件。

#### Scenario: ADR 文件存在

- **WHEN** 检查 docs/adr/ 目录
- **THEN** 存在描述 RunConfig 语义定位的 ADR 文件

