## Why

`overlap_resolution_system` 使用全局固定的 `min_separation: 8` 作为所有兵种的碰撞半径。不同兵种应有不同的碰撞体积（骑兵 > 步兵 > 弓兵），且碰撞体积应可通过 `content/units.ron` 外置配置。

## What Changes

- `SoldierUnitConfig` 新增 `collision_radius: u32` 字段（含 serde default）
- `content/units.ron` 各兵种添加 `collision_radius` 值
- `overlap_resolution_system`: 查询 `SoldierTypeComponent`，用 `radius_A + radius_B` 替代全局 `min_separation`
- 移除 `OverlapResolutionConfig.min_separation`（不再需要全局值）

## Capabilities

### New Capabilities

无。此为现有 `overlap-resolution` 的增强。

### Modified Capabilities

- `overlap-resolution`: 碰撞半径从全局固定值改为每兵种可配置

## Impact

- `crates/simulation/src/soldier/config.rs`: `SoldierUnitConfig` + 字段
- `content/units.ron`: 4 个兵种各 + 字段
- `crates/simulation/src/soldier/mod.rs`: `overlap_resolution_system` 查询类型组件
- `crates/simulation/src/combat/config.rs`: `OverlapResolutionConfig` 移除 `min_separation`
- `content/combat.ron`: 移除 `min_separation`
