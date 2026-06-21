## MODIFIED Requirements

### Requirement: 仿真层依赖限制

simulation crate 的 `Cargo.toml` SHALL NOT 依赖 `bevy`（完整版）、`bevy_render`、`bevy_ui`、`bevy_window`、`bevy_input`、`bevy_audio`、`bevy_asset` 或任何图形/窗口/音频 crate。SHALL 仅依赖 `bevy_ecs` 核心子集（+ `serde`、`ron`、`rand`）。`bevy_ecs` 版本 SHALL 为 `0.19`。

#### Scenario: 编译时隔离验证

- **WHEN** 在 `simulation/src/` 中尝试 `use bevy::prelude::Transform`
- **THEN** 编译失败，因为 `bevy` 不在 `simulation` 的依赖中

#### Scenario: 独立运行测试

- **WHEN** 在 `crates/simulation/` 目录下执行 `cargo test`
- **THEN** SHALL 在无 Bevy 完整运行时的情况下成功编译并运行所有测试

#### Scenario: bevy_ecs 0.19 Resources as Components 兼容

- **WHEN** simulation crate 中的 `#[derive(Resource)]` 类型在 bevy_ecs 0.19 中使用
- **THEN** 资源作为组件存储在专用抽象实体上，不影响仿真层的 `world.resource::<T>()` 和 `world.resource_mut::<T>()` 读写语义
