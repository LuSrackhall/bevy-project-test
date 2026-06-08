## Why

当前项目是一个单层 Bevy crate，所有仿真逻辑、渲染、输入处理、UI 混杂在一起，使用 `f32`/`Vec2` 做所有数值计算，系统直读鼠标键盘操作游戏状态。这在功能迭代中已暴露出严重问题：新增任何功能都需要在同一文件中同时处理逻辑、视觉和输入，代码耦合度极高，BUG 定位困难，且架构完全不符合 CLAUDE.md 宪法对 Lockstep/Replay/权威服务器的长期要求。必须在功能继续膨胀前完成架构性重构，以降低后续所有功能开发和修复的边际成本。

## What Changes

- **BREAKING**: 将单一 crate 拆分为 4 个独立 crate workspace（`simulation` → `bevy_adapter` → `presentation` → `render_view`） + `content/` 配置目录
- **BREAKING**: 所有仿真数值从 `f32`/`Vec2` 迁移到 `Fixed(i64)`/`FixedVec2` 定点数体系
- **BREAKING**: 所有实体标识从 Bevy `Entity` 迁移到 `UnitId` + `UnitIdMapper` 双向 O(1) 映射
- **BREAKING**: 所有玩家操作从系统内直读鼠标/键盘改为 `GameCommand` + `CommandBuffer` 命令管道驱动
- **BREAKING**: 仿真从帧率相关的 `time.delta_secs()` 改为固定 20Hz Tick 离散计算
- **BREAKING**: 所有平衡参数从 `core/mod.rs` 硬编码迁移到 `content/*.ron` 外置配置文件
- **BREAKING**: 随机数从 `rand::thread_rng()` 改为种子化 `DeterministicRng`，保证回放一致性
- 保持所有现有游戏功能：城池占领、士兵战斗（近战/弓兵/骑兵）、AI 决策、框选/点选、HUD/菜单/暂停/结算 UI、相机拖拽缩放
- 渲染层采用 DebugShape 阶段（Gizmos 几何体），确保后续可插拔替换正式美术资产
- 整个仿真 crate 可脱离 Bevy 主程序独立运行 `cargo test`

## Capabilities

### New Capabilities
- `simulation-crate`: 纯逻辑仿真 crate，包含定点数类型、UnitId、GameCommand/CommandBuffer、所有仿真组件与系统、确定性 PRNG、内容配置解析。独立于任何图形/窗口/输入依赖，可独立测试。
- `bevy-adapter-crate`: 引擎适配层，负责 UnitId↔Entity 映射、Tick 调度驱动、命令注入/消费、输入→命令翻译、实体生灭同步。
- `presentation-crate`: 桥接表现层，负责逻辑 Tick 与渲染帧之间的插值（InterpolationData）、渲染实体绑定（LogicEntityRef）、生灭监听。
- `render-view-crate`: 视觉与 UI 层，负责 DebugShape 渲染（Gizmos 几何体）、UI（HUD/菜单/暂停/结算）、相机、选择系统。可插拔替换正式美术资产。
- `content-config`: 外置数据配置文件（RON 格式），涵盖兵种属性、城池参数、战斗公式、地图生成参数。

### Modified Capabilities
<!-- 无现有 capability 需要修改，因为当前项目尚未建立 OpenSpec 规范体系 -->

## Impact

- **全部现有代码**：`src/` 下所有模块（`core/`, `map/`, `city/`, `soldier/`, `combat/`, `ai/`, `camera/`, `input/`, `ui/`, `game/`）将被迁移和拆分到新 crate 结构中
- **Cargo.toml**：从单一 package 变为 `[workspace]`，新增 4 个 crate 的 `Cargo.toml`，依赖关系严格单向
- **CLAUDE.md**：宪法已就位，本次重构正是为满足其要求
- **无外部依赖变更**：不引入新的第三方 crate（除 `ron` 用于配置解析、`serde` 用于序列化），现有 `bevy 0.18`、`bevy_prototype_lyon`、`rand` 保留但 `rand` 仅在上层或配置中使用，仿真层使用确定性 PRNG
