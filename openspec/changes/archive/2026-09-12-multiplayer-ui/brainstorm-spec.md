## Context

当前联机主菜单中 `NetworkPlayerCount` 和 `NetworkPlayerId` 两个选择器按钮为静态显示（始终 "2" 和 "0"），用户无法通过界面选择 3-4 玩家配置。

后端（`simulation-multiplayer-slots` + `network-multiplayer-init`）已支持 2-4 玩家 PvP，但 UI 尚未对接。

此外，`开始联机`按钮的 Query 存在预存 Bug：它以 tuple query `Query<(&NetworkRelayAddrInput, &NetworkPlayerCount, &NetworkPlayerId)>` 尝试从**三个兄弟实体**上同时获取这三个组件，但 Bevy 的 Query 要求所有组件在**同一实体**上，因此始终返回 None，所有值回退到默认值。

## Goals / Non-Goals

**Goals:**
- `NetworkPlayerCount` 按钮点击可循环值：`2 → 3 → 4 → 2`
- `NetworkPlayerId` 按钮点击可循环值：`0 → 1 → … → (count-1) → 0`
- 按 Count 减小时自动 clamp ID（如 Count 从 4→3 时 ID=3 → 自动回 1）
- 按钮显示文本随值即时更新（observer 内同时改组件 + Text）
- 修复 `开始联机` 按钮 Query：拆为三个独立 Query，分别读取每个组件

**Non-Goals:**
- UI 布局调整或组件重排
- 大厅 UI （lobby）的玩家信息显示
- 联机模式下地图大小选择（当前固定 Medium）
- relay / transport / simulation 层改动

## Decisions

| 决策 | 选择 | 理由 |
|------|------|------|
| 实现方式 | 内联 observer | 匹配 menu.rs 中 MapSizeBtn、AutoRecordToggle 等 4 个现有模式，无需额外 system |
| Text 更新 | observer 内直接 `get_mut` | 组件值和 Text 在同一次点击中完成更新，避免帧级延迟 |
| 子节点查找 | `children.iter().next()` + `if-let` 链 | 避免 `Entity::PLACEHOLDER` 导致误改其他实体 Text |
| 开始按钮修复 | 拆为 3 个独立 Query | 三组件分布在兄弟实体，无法用 tuple query 一次获取 |

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| [Pre-existing] 开始按钮 Query 不匹配 → 设置的值从未生效 | 本变更一并修复：拆为三个独立 Query |
| [Design] Clamp 需要跨实体访问 ID 按钮 | Count observer 增加 `Query<(&mut NetworkPlayerId, &Children), Without<NetworkPlayerCount>>` |
| [Implementation] `Entity::PLACEHOLDER` 回退可能误改 Text | 改用 `if let Some(child) = children.iter().next()` 模式 |
| [Implementation] Query `get_mut` 需要 `mut q` | observer 闭包参数声明为 `mut q: Query<...>` |
